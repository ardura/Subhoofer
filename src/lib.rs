#![allow(non_snake_case)]
mod CustomWidgets;
use atomic_float::AtomicF32;
use nih_plug::{prelude::*};
use nih_plug_egui::{create_egui_editor, egui::{self, Color32, FontId, Pos2, Rect, RichText, Rounding}, widgets, EguiState};
use CustomWidgets::{db_meter, ui_knob};
use std::{f32::consts::PI, ops::RangeInclusive, sync::Arc};
mod SweetenX;

/***************************************************************************
 * Subhoofer v2.2.2 by Ardura
 * 
 * Build with: cargo xtask bundle Subhoofer --profile <release or profiling>
 * *************************************************************************/

 #[derive(Enum, PartialEq, Eq, Debug, Copy, Clone)]
 pub enum AlgorithmType{
    #[name = "A Bass 3"]
    ABass3,
    #[name = "A Bass 2"]
    ABass2,
    #[name = "8 Harmonic Stack"]
    BBass,
    #[name = "Duro Console"]
    CBass,
    #[name = "TanH Transfer"]
    TanH,
    #[name = "Custom"]
    CustomSliders,
 }

 // GUI Colors
const TEAL: Color32 = Color32::from_rgb(13,62,102);
const NAVY_BLUE: Color32 = Color32::from_rgb(55,50,48);
const BEIGE: Color32 = Color32::from_rgb(239,141,11);
const LIGHT_GREY: Color32 = Color32::from_rgb(204,205,196);

// Plugin sizing
const WIDTH: u32 = 360;
const HEIGHT: u32 = 528;

/// The time it takes for the peak meter to decay by 12 dB after switching to complete silence.
const PEAK_METER_DECAY_MS: f64 = 100.0;

pub struct Subhoofer {
    params: Arc<SubhooferParams>,

    // normalize the peak meter's response based on the sample rate with this
    out_meter_decay_weight: f32,

    // "header" variables from C++ class
    prev_processed_in_r: f32,
    prev_processed_out_r: f32,
    prev_processed_in_l: f32,
    prev_processed_out_l: f32,
    lp: f32,
    iir_sub_bump_a: f32,
    iir_sub_bump_b: f32,
    iir_sub_bump_c: f32,
    iir_drive_sample_a: f32,
    iir_drive_sample_b: f32,
    iir_drive_sample_c: f32,
    iir_drive_sample_d: f32,
    iir_drive_sample_e: f32,
    iir_drive_sample_f: f32,
    osc_gate: f32,
    iir_sample_a: f32,
    iir_sample_b: f32,
    iir_sample_c: f32,
    iir_sample_d: f32,
    iir_sample_e: f32,
    iir_sample_f: f32,
    iir_sample_g: f32,
    iir_sample_h: f32,
    iir_sample_i: f32,
    iir_sample_j: f32,
    iir_sample_k: f32,
    iir_sample_l: f32,
    iir_sample_m: f32,
    iir_sample_n: f32,
    iir_sample_o: f32,
    iir_sample_p: f32,
    iir_sample_q: f32,
    iir_sample_r: f32,
    iir_sample_s: f32,
    iir_sample_t: f32,
    iir_sample_u: f32,
    iir_sample_v: f32,
    iir_sample_w: f32,
    iir_sample_x: f32,
    iir_sample_y: f32,
    iir_sample_z: f32,
    sub_iir: f32,

    // Logic control variables
    sub_octave: bool,
    was_negative: bool,
    bass_flip_counter: i32,

    // The current data for the different meters
    out_meter: Arc<AtomicF32>,
    in_meter: Arc<AtomicF32>,

    // Buffer for SweetenX
    buffer: [f32; 16],
}

// Modified function from Duro Console for different behavior - hoof hardness
fn chebyshev_tape(sample: f32, drive: f32) -> f32 {
    let dry = 1.0 - drive;
    let peak = f32::max(sample.abs(), 1.0);
    let x = sample / peak;
    let x2 = x * x;
    let x3 = x * x2;
    let x5 = x3 * x2;
    let x6 = x3 * x3;
    let y = x
        - 0.166667 * x3
        + 0.00833333 * x5
        - 0.000198413 * x6
        + 0.0000000238 * x6 * drive;
    dry * sample + (1.0 - dry) * y / (1.0 + y.abs())
}

// Modified tape saturation using transfer function from Duro Console
fn tape_saturation(input_signal: f32, drive: f32) -> f32 {
    let idrive = drive;
    // Define the transfer curve for the tape saturation effect
    let transfer = |x: f32| -> f32 {
        // Smoothly transition to linear function as drive approaches 0.0
        let tanh_saturation = (x * idrive).tanh();
        let linear_saturation = x * idrive;
        tanh_saturation + (linear_saturation - tanh_saturation) * 0.5
    };
    // Apply the transfer curve to the input sample
    let output_sample = transfer(input_signal);
    output_sample - input_signal
}

/* One of the other algorithms I was messing around with - not exactly the
    sound I was going for but unique enough to include - Ardura */
fn b_bass_saturation(signal: f32, mut harmonic_strength: f32) -> f32 {
    let num_harmonics: usize = 8;
    let mut summed: f32 = 0.0;

    for j in 1..=num_harmonics {
        if j % 2 == 1 {
            let harmonic_component: f32 = harmonic_strength * 3.0 * (signal * j as f32).cos();
            let harmonic_component2: f32 = harmonic_strength * (signal * j as f32).sin();
            summed += harmonic_component + harmonic_component2;
            continue;
        }
        else if j % 2 == 0 {
            match j {
                4 => harmonic_strength *= 0.6,
                6 => harmonic_strength *= 4.0,
                _ => harmonic_strength *= 1.0
            }
            let harmonic_component: f32 = harmonic_strength * (signal * j as f32).sin();
            summed += harmonic_component;
        }
    }
    summed - signal
}


// Modified "odd_saturation" from Duro Console
fn c_bass_saturation(signal: f32, harmonic_strength: f32) -> f32 {
    let num_harmonics: usize = 7;
    let mut summed: f32 = 0.0;
    for j in 1..=num_harmonics {
        let harmonic_component: f32 = harmonic_strength * 0.3 * (signal * j as f32).sin() - signal;
        let harmonic_component2: f32 = harmonic_strength * (signal * j as f32).cos() - signal;
        summed += harmonic_component + harmonic_component2;
    }
    // Divide this by harmonic addition amount
    summed/7.0
}


fn custom_sincos_saturation(signal: f32, harmonic_strength1: f32, harmonic_strength2: f32, harmonic_strength3: f32, harmonic_strength4: f32) -> f32 {
    let mut summed: f32 = 0.0;

    let harmonic_component: f32 = harmonic_strength1 * (signal * 1.0).cos() - signal;
    summed += harmonic_component;

    let harmonic_component: f32 = harmonic_strength2 * (signal * 2.0).sin() - signal;
    summed += harmonic_component;

    let harmonic_component: f32 = harmonic_strength3 * (signal * 3.0).cos() - signal;
    summed += harmonic_component;

    let harmonic_component2: f32 = harmonic_strength4 * (signal * 4.0).sin() - signal;
    summed += harmonic_component2;

    summed
}

#[derive(Params)]
struct SubhooferParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "free_gain"]
    pub free_gain: FloatParam,

    #[id = "Hoof Hardness"]
    pub hoof_hardness: FloatParam,

    #[id = "Sub Gain"]
    pub sub_gain: FloatParam,

    #[id = "Sub Drive"]
    pub sub_drive: FloatParam,

    #[id = "Harmonics"]
    pub harmonics: FloatParam,

    #[id = "Algorithm"]
    pub h_algorithm: EnumParam<AlgorithmType>,

    #[id = "Custom Strength 1"]
    pub custom_harmonics1: FloatParam,

    #[id = "Custom Strength 2"]
    pub custom_harmonics2: FloatParam,

    #[id = "Custom Strength 3"]
    pub custom_harmonics3: FloatParam,

    #[id = "Custom Strength 4"]
    pub custom_harmonics4: FloatParam,

    #[id = "output_gain"]
    pub output_gain: FloatParam,

    #[id = "dry_wet"]
    pub dry_wet: FloatParam,
}

impl Default for Subhoofer {
    fn default() -> Self {
        Self {
            params: Arc::new(SubhooferParams::default()),
            out_meter_decay_weight: 1.0,
            out_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
            in_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
            osc_gate: 0.0,
            lp: 0.0,
            iir_sub_bump_a: 0.0,
            iir_sub_bump_b: 0.0,
            iir_sub_bump_c: 0.0,
            iir_drive_sample_a: 0.0,
            iir_drive_sample_b: 0.0,
            iir_drive_sample_c: 0.0,
            iir_drive_sample_d: 0.0,
            iir_drive_sample_e: 0.0,
            iir_drive_sample_f: 0.0,
            iir_sample_a: 0.0,
            iir_sample_b: 0.0,
            iir_sample_c: 0.0,
            iir_sample_d: 0.0,
            iir_sample_e: 0.0,
            iir_sample_f: 0.0,
            iir_sample_g: 0.0,
            iir_sample_h: 0.0,
            iir_sample_i: 0.0,
            iir_sample_j: 0.0,
            iir_sample_k: 0.0,
            iir_sample_l: 0.0,
            iir_sample_m: 0.0,
            iir_sample_n: 0.0,
            iir_sample_o: 0.0,
            iir_sample_p: 0.0,
            iir_sample_q: 0.0,
            iir_sample_r: 0.0,
            iir_sample_s: 0.0,
            iir_sample_t: 0.0,
            iir_sample_u: 0.0,
            iir_sample_v: 0.0,
            iir_sample_w: 0.0,
            iir_sample_x: 0.0,
            iir_sample_y: 0.0,
            iir_sample_z: 0.0,
            prev_processed_in_r: 0.0,
            prev_processed_out_r: 0.0,
            prev_processed_in_l: 0.0,
            prev_processed_out_l: 0.0,
            sub_iir: 0.0,
            sub_octave: false,
            was_negative: false,
            bass_flip_counter: 1,
            buffer: [0.0; 16],
        }
    }
}

impl Default for SubhooferParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(WIDTH, HEIGHT),

            // Input gain dB parameter (free as in unrestricted nums)
            free_gain: FloatParam::new(
                "Input Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-12.0),
                    max: util::db_to_gain(12.0),
                    factor: FloatRange::gain_skew_factor(-12.0, 12.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(30.0))
            .with_unit(" In Gain")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(1))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            // Hoof Parameter
            hoof_hardness: FloatParam::new(
                "Hoof Hardness",
                0.0093,
                FloatRange::Linear {
                    min: 0.00,
                    max: 0.30,
                },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Hardness")
            .with_value_to_string(formatters::v2s_f32_percentage(4)),

            // Sub gain dB parameter
            sub_gain: FloatParam::new(
                "Sub Gain",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 24.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" dB Sub Gain")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            // Sub Drive dB parameter
            sub_drive: FloatParam::new(
                "Sub Drive",
                0.0,
                FloatRange::Linear { min: (0.0), max: (1.0) },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit("% Sub Drive")
            .with_value_to_string(formatters::v2s_f32_percentage(2)),

            // Harmonics Parameter
            harmonics: FloatParam::new(
                "Harmonics",
                0.000580,
                FloatRange::Skewed { min: 0.0, max: 1.0, factor: FloatRange::skew_factor(-2.8) }
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Harmonics")
            .with_value_to_string(formatters::v2s_f32_percentage(4)),

            h_algorithm: EnumParam::new("Harmonic Algorithm", AlgorithmType::ABass3),


            // Custom Harmonics Parameter 1
            custom_harmonics1: FloatParam::new(
                "Custom Harmonic 1",
                0.0,
                FloatRange::Skewed { min: 0.0, max: 400.0, factor: FloatRange::skew_factor(-2.0) }
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Custom Harmonic 1"),

            // Custom Harmonics Parameter 2
            custom_harmonics2: FloatParam::new(
                "Custom Harmonic 2",
                0.0,
                FloatRange::Skewed { min: 0.0, max: 400.0, factor: FloatRange::skew_factor(-2.0) }
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Custom Harmonic 2"),

            // Custom Harmonics Parameter 3
            custom_harmonics3: FloatParam::new(
                "Custom Harmonic 3",
                0.0,
                FloatRange::Skewed { min: 0.0, max: 400.0, factor: FloatRange::skew_factor(-2.0) }
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Custom Harmonic 3"),

            // Custom Harmonics Parameter 4
            custom_harmonics4: FloatParam::new(
                "Custom Harmonic 4",
                0.0,
                FloatRange::Skewed { min: 0.0, max: 400.0, factor: FloatRange::skew_factor(-2.0) }
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Custom Harmonic 4"),

            // Output gain parameter
            output_gain: FloatParam::new(
                "Output Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-12.0),
                    max: util::db_to_gain(12.0),
                    factor: FloatRange::gain_skew_factor(-12.0, 12.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" Out Gain")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(1))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            // Dry/Wet parameter
            dry_wet: FloatParam::new(
                "Dry/Wet",
                1.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit("% Wet")
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

impl Plugin for Subhoofer {
    const NAME: &'static str = "Subhoofer";
    const VENDOR: &'static str = "Ardura";
    const URL: &'static str = "https://github.com/ardura";
    const EMAIL: &'static str = "azviscarra@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // This looks like it's flexible for running the plugin in mono or stereo
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {main_input_channels: NonZeroU32::new(2), main_output_channels: NonZeroU32::new(2), ..AudioIOLayout::const_default()},
        AudioIOLayout {main_input_channels: NonZeroU32::new(1), main_output_channels: NonZeroU32::new(1), ..AudioIOLayout::const_default()},
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let in_meter = self.in_meter.clone();
        let out_meter = self.out_meter.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default()
                    .show(egui_ctx, |ui| {
                        // Change colors - there's probably a better way to do this
                        let style_var = ui.style_mut().clone();

                        // Trying to draw background as rect
                        ui.painter().rect_filled(
                            Rect::from_x_y_ranges(
                                RangeInclusive::new(0.0, WIDTH as f32), 
                                RangeInclusive::new(0.0, HEIGHT as f32)), 
                            Rounding::from(16.0), NAVY_BLUE);

                        // Screws for that vintage look
                        let screw_space = 10.0;
                        ui.painter().circle_filled(Pos2::new(screw_space,screw_space), 4.0, Color32::DARK_GRAY);
                        ui.painter().circle_filled(Pos2::new(screw_space,HEIGHT as f32 - screw_space), 4.0, Color32::DARK_GRAY);
                        ui.painter().circle_filled(Pos2::new(WIDTH as f32 - screw_space,screw_space), 4.0, Color32::DARK_GRAY);
                        ui.painter().circle_filled(Pos2::new(WIDTH as f32 - screw_space,HEIGHT as f32 - screw_space), 4.0, Color32::DARK_GRAY);

                        ui.set_style(style_var);

                        // GUI Structure
                        ui.vertical(|ui| {
                            // Spacing :)
                            ui.label(RichText::new("    Subhoofer").font(FontId::proportional(14.0)).color(BEIGE)).on_hover_text("by Ardura!");

                            // Peak Meters
                            let in_meter = util::gain_to_db(in_meter.load(std::sync::atomic::Ordering::Relaxed));
                            let in_meter_text = if in_meter > util::MINUS_INFINITY_DB {
                                format!("{in_meter:.1} dBFS Input")
                            } else {
                                String::from("-inf dBFS Input")
                            };
                            let in_meter_normalized = (in_meter + 60.0) / 60.0;
                            ui.allocate_space(egui::Vec2::splat(2.0));
                            let mut in_meter_obj = db_meter::DBMeter::new(in_meter_normalized).text(in_meter_text);
                            in_meter_obj.set_background_color(TEAL);
                            in_meter_obj.set_bar_color(BEIGE);
                            in_meter_obj.set_border_color(Color32::BLACK);
                            ui.add(in_meter_obj);

                            let out_meter = util::gain_to_db(out_meter.load(std::sync::atomic::Ordering::Relaxed));
                            let out_meter_text = if out_meter > util::MINUS_INFINITY_DB {
                                format!("{out_meter:.1} dBFS Output")
                            } else {
                                String::from("-inf dBFS Output")
                            };
                            let out_meter_normalized = (out_meter + 60.0) / 60.0;
                            ui.allocate_space(egui::Vec2::splat(2.0));
                            let mut out_meter_obj = db_meter::DBMeter::new(out_meter_normalized).text(out_meter_text);
                            out_meter_obj.set_background_color(TEAL);
                            out_meter_obj.set_bar_color(BEIGE);
                            out_meter_obj.set_border_color(Color32::BLACK);
                            ui.add(out_meter_obj);

                            ui.horizontal(|ui| {
                                let knob_size = 42.0;
                                let text_size = 12.0;
                                ui.vertical(|ui| {
                                    let gain_knob = ui_knob::ArcKnob::for_param(
                                        &params.free_gain, 
                                        setter, 
                                        knob_size, 
                                        ui_knob::KnobLayout::Horizonal)
                                            .preset_style(ui_knob::KnobStyle::Preset1)
                                            .set_fill_color(TEAL)
                                            .set_line_color(BEIGE)
                                            .set_text_size(text_size)
                                            .set_hover_text("Input gain into Subhoofer".to_string());
                                    ui.add(gain_knob);

                                    let output_knob = ui_knob::ArcKnob::for_param(
                                        &params.output_gain, 
                                        setter, 
                                        knob_size, 
                                        ui_knob::KnobLayout::Horizonal)
                                            .preset_style(ui_knob::KnobStyle::Preset1)
                                            .set_fill_color(TEAL)
                                            .set_line_color(BEIGE)
                                            .set_text_size(text_size)
                                            .set_hover_text("Output gain from Subhoofer".to_string());
                                    ui.add(output_knob);

                                    let algorithm_knob = ui_knob::ArcKnob::for_param(
                                        &params.h_algorithm, 
                                        setter, 
                                        knob_size, 
                                        ui_knob::KnobLayout::Horizonal)
                                            .preset_style(ui_knob::KnobStyle::Preset1)
                                            .set_fill_color(TEAL)
                                            .set_line_color(BEIGE)
                                            .set_text_size(text_size)
                                            .set_hover_text("The saturation/harmonic algorithm used".to_string());
                                    ui.add(algorithm_knob);
                                
                                    let dry_wet_knob = ui_knob::ArcKnob::for_param(
                                        &params.dry_wet, 
                                        setter, 
                                        knob_size, 
                                        ui_knob::KnobLayout::Horizonal)
                                            .preset_style(ui_knob::KnobStyle::Preset1)
                                            .set_fill_color(TEAL)
                                            .set_line_color(BEIGE)
                                            .set_text_size(text_size)
                                            .set_hover_text("The blend of unprocessed/processed signal".to_string());
                                    ui.add(dry_wet_knob);
                                });

                                ui.vertical(|ui| {
                                    let hardness_knob = ui_knob::ArcKnob::for_param(
                                        &params.hoof_hardness, 
                                        setter, 
                                        knob_size, 
                                        ui_knob::KnobLayout::Horizonal)
                                            .preset_style(ui_knob::KnobStyle::Preset1)
                                            .set_fill_color(TEAL)
                                            .set_line_color(LIGHT_GREY)
                                            .set_text_size(text_size)
                                            .set_hover_text("The amount of saturation Subhoofer uses".to_string());
                                    ui.add(hardness_knob);

                                    let harmonics_knob = ui_knob::ArcKnob::for_param(
                                        &params.harmonics, 
                                        setter, 
                                        knob_size, 
                                        ui_knob::KnobLayout::Horizonal)
                                            .preset_style(ui_knob::KnobStyle::Preset1)
                                            .set_fill_color(TEAL)
                                            .set_line_color(LIGHT_GREY)
                                            .set_text_size(text_size)
                                            .set_hover_text("The strength of harmonics added to signal".to_string());
                                    ui.add(harmonics_knob);

                                    let sub_gain_knob = ui_knob::ArcKnob::for_param(
                                        &params.sub_gain, 
                                        setter, 
                                        knob_size, 
                                        ui_knob::KnobLayout::Horizonal)
                                            .preset_style(ui_knob::KnobStyle::Preset1)
                                            .set_fill_color(TEAL)
                                            .set_line_color(LIGHT_GREY)
                                            .set_text_size(text_size)
                                            .set_hover_text("Gain for the sub layer".to_string());
                                    ui.add(sub_gain_knob);
                                
                                    let sub_drive_knob = ui_knob::ArcKnob::for_param(
                                        &params.sub_drive, 
                                        setter, 
                                        knob_size, 
                                        ui_knob::KnobLayout::Horizonal)
                                            .preset_style(ui_knob::KnobStyle::Preset1)
                                            .set_fill_color(TEAL)
                                            .set_line_color(LIGHT_GREY)
                                            .set_text_size(text_size)
                                            .set_hover_text("Drive into the sub algorithm".to_string());
                                    ui.add(sub_drive_knob);
                                });
                            });
                            //sliders
                            ui.horizontal(|ui|{
                                ui.add_space(16.0);
                                ui.vertical(|ui| {
                                    ui.add(widgets::ParamSlider::for_param(&params.custom_harmonics1, setter).with_width(170.0))
                                        .on_hover_text_at_pointer("Add harmonics when using \"Custom\" Algorithm
Double-click to reset");
                                    ui.add(widgets::ParamSlider::for_param(&params.custom_harmonics2, setter).with_width(170.0))
                                        .on_hover_text_at_pointer("Add harmonics when using \"Custom\" Algorithm
Double-click to reset");
                                    ui.add(widgets::ParamSlider::for_param(&params.custom_harmonics3, setter).with_width(170.0))
                                        .on_hover_text_at_pointer("Add harmonics when using \"Custom\" Algorithm
Double-click to reset");
                                    ui.add(widgets::ParamSlider::for_param(&params.custom_harmonics4, setter).with_width(170.0))
                                        .on_hover_text_at_pointer("Add harmonics when using \"Custom\" Algorithm
Double-click to reset");
                                });
                            });
                        });
                    });
                }
            )
    }

    

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // After `PEAK_METER_DECAY_MS` milliseconds of pure silence, the peak meter's value should
        // have dropped by 12 dB
        self.out_meter_decay_weight = 0.25f64.powf((buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip()) as f32;
        
        nih_dbg!("Plugin started successfully");
        color_backtrace::install();

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for mut channel_samples in buffer.iter_samples() {
            let mut out_amplitude: f32 = 0.0;
            let mut in_amplitude: f32 = 0.0;
            let mut processed_sample_l: f32;
            let mut processed_sample_r: f32;
            let num_samples = channel_samples.len();

            let gain: f32 = util::gain_to_db(self.params.free_gain.smoothed.next());
            let num_gain: f32;
            let hoof_hardness: f32 = self.params.hoof_hardness.smoothed.next();
            let sub_gain: f32 = self.params.sub_gain.smoothed.next();
            let output_gain: f32 = self.params.output_gain.smoothed.next();
            let sub_drive: f32 = self.params.sub_drive.smoothed.next();
            let harmonics: f32 = self.params.harmonics.smoothed.next();
            let custom_harmonics1: f32 = self.params.custom_harmonics1.smoothed.next();
            let custom_harmonics2: f32 = self.params.custom_harmonics2.smoothed.next();
            let custom_harmonics3: f32 = self.params.custom_harmonics3.smoothed.next();
            let custom_harmonics4: f32 = self.params.custom_harmonics4.smoothed.next();
            let h_algorithm: AlgorithmType = self.params.h_algorithm.value();
            let dry_wet: f32 = self.params.dry_wet.value();

            // I picked this
            let mut fake_random: f32 = 0.83;
            let inv_fake_random: f32 = 1.0 - fake_random;
            fake_random /= 2.0;

            // Scale the head bump freqeuncy for Subhoof
            let sample_rate: f32 = context.transport().sample_rate;
            let mut overall_scale: f32 = 1.0;
            overall_scale /= 44100.0;
            overall_scale *= sample_rate;

            // Split left and right same way original subhoofer did
            let mut in_l = *channel_samples.get_mut(0).unwrap();
            let mut in_r = *channel_samples.get_mut(1).unwrap();

            num_gain = gain;
            in_l *= util::db_to_gain(num_gain);
            in_r *= util::db_to_gain(num_gain);
            in_amplitude += in_l + in_r;

            ///////////////////////////////////////////////////////////////////////
            // Perform processing on the sample

            // Normalize really small values
            if in_l.abs() < 1.18e-23 { in_l = 0.1 * 1.18e-17; }
            if in_r.abs() < 1.18e-23 { in_r = 0.1 * 1.18e-17; }

            let mut sub_bump: f32;

            // Sub voicing variables
            let sub_headbump_freq: f32 = (((hoof_hardness) * 0.1) + 0.02) / overall_scale;
            self.sub_iir = sub_headbump_freq / 44.1;

            // Sub drive samples
            // self.lp is our center signal
            self.lp = (in_l + in_r) / 4096.0;
            self.iir_drive_sample_a = (self.iir_drive_sample_a * (1.0 - sub_headbump_freq)) + (self.lp * sub_headbump_freq);
            self.lp = self.iir_drive_sample_a;
            self.iir_drive_sample_b = (self.iir_drive_sample_b * (1.0 - sub_headbump_freq)) + (self.lp * sub_headbump_freq);
            self.lp = self.iir_drive_sample_b;
            // Gate from airwindows
            self.osc_gate += (self.lp * 10.0).abs();
            self.osc_gate -= 0.001;
            if self.osc_gate > 1.0 {self.osc_gate = 1.0;}
            if self.osc_gate < 0.0 {self.osc_gate = 0.0;}
            //got a value that only goes down low when there's silence or near silence on input
            let clamp: f32 = (1.0 - self.osc_gate) * 0.00001;
            // Figure out our zero crossing
            if self.lp > 0.0
            {
                // We are on top of zero crossing
                if self.was_negative
                {
                    self.sub_octave = !self.sub_octave;
                    self.was_negative = false;
                }
            }
            else {
                // On bottom of zero crossing
                self.was_negative = true;
            }
            self.iir_sample_a = (self.iir_sample_a * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_a;
			self.iir_sample_b = (self.iir_sample_b * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_b;
			self.iir_sample_c = (self.iir_sample_c * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_c;
			self.iir_sample_d = (self.iir_sample_d * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_d;
			self.iir_sample_e = (self.iir_sample_e * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_e;
			self.iir_sample_f = (self.iir_sample_f * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_f;
			self.iir_sample_g = (self.iir_sample_g * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_g;
			self.iir_sample_h = (self.iir_sample_h * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_h;
			self.iir_sample_i = (self.iir_sample_i * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_i;
			self.iir_sample_j = (self.iir_sample_j * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_j;
			self.iir_sample_k = (self.iir_sample_k * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_k;
			self.iir_sample_l = (self.iir_sample_l * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_l;
			self.iir_sample_m = (self.iir_sample_m * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_m;
			self.iir_sample_n = (self.iir_sample_n * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_n;
			self.iir_sample_o = (self.iir_sample_o * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_o;
			self.iir_sample_p = (self.iir_sample_p * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_p;
			self.iir_sample_q = (self.iir_sample_q * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_q;
			self.iir_sample_r = (self.iir_sample_r * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_r;
			self.iir_sample_s = (self.iir_sample_s * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_s;
			self.iir_sample_t = (self.iir_sample_t * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_t;
			self.iir_sample_u = (self.iir_sample_u * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_u;
			self.iir_sample_v = (self.iir_sample_v * (1.0 - self.sub_iir)) + (self.lp * self.sub_iir);  self.lp -= self.iir_sample_v;
            let mut head_bump: f32 = self.lp;

            // Regain some volume now that we have sampled
            head_bump = head_bump * 256.0;

            // Calculate drive samples based off the processing so far
            self.iir_sample_w = (self.iir_sample_w * (1.0 - self.sub_iir)) + (head_bump * self.sub_iir);    head_bump -= self.iir_sample_w;
			self.iir_sample_x = (self.iir_sample_x * (1.0 - self.sub_iir)) + (head_bump * self.sub_iir);    head_bump -= self.iir_sample_x;

            // Create SubBump sample from our head bump to modify further
			sub_bump = head_bump;
			self.iir_sample_y = (self.iir_sample_y * (1.0 - self.sub_iir)) + (sub_bump * self.sub_iir);    sub_bump -= self.iir_sample_y;

            // Calculate sub drive samples based off what we've done so far		
            self.iir_drive_sample_c = (self.iir_drive_sample_c * (1.0 - sub_headbump_freq)) + (sub_bump * sub_headbump_freq);   sub_bump = self.iir_drive_sample_c;
            self.iir_drive_sample_d = (self.iir_drive_sample_d * (1.0 - sub_headbump_freq)) + (sub_bump * sub_headbump_freq);   sub_bump = self.iir_drive_sample_d;

            // Flip the bump sample per sub octave for half-freq
            sub_bump = sub_bump.abs();
            sub_bump = if self.sub_octave == false { -sub_bump } else { sub_bump };
            // Note the randD/invrandD is what is flipping from positive to negative here
			// This means bflip = 1 A gets inverted
			// This means bflip = 2 B gets inverted
			// This means bflip = 3 C gets inverted
			// This creates a lower octave using  multiplication depending on sample
            match self.bass_flip_counter
            {
                1 => {
                    self.iir_sub_bump_a += sub_bump * sub_gain;
                    self.iir_sub_bump_a -= self.iir_sub_bump_a * self.iir_sub_bump_a * self.iir_sub_bump_a * sub_headbump_freq;
                    self.iir_sub_bump_a = (inv_fake_random * self.iir_sub_bump_a) + (fake_random * self.iir_sub_bump_b) + (fake_random * self.iir_sub_bump_c);
                    if self.iir_sub_bump_a > 0.0 { self.iir_sub_bump_a -= clamp; }
                    if self.iir_sub_bump_a < 0.0 { self.iir_sub_bump_a += clamp; }
                    sub_bump = self.iir_sub_bump_a;
                }
                2 => {
                    self.iir_sub_bump_b += sub_bump * sub_gain;
                    self.iir_sub_bump_b -= self.iir_sub_bump_b * self.iir_sub_bump_b * self.iir_sub_bump_b * sub_headbump_freq;
                    self.iir_sub_bump_b = (fake_random * self.iir_sub_bump_a) + (inv_fake_random * self.iir_sub_bump_b) + (fake_random * self.iir_sub_bump_c);
                    if self.iir_sub_bump_b > 0.0 { self.iir_sub_bump_b -= clamp; }
                    if self.iir_sub_bump_b < 0.0 { self.iir_sub_bump_b += clamp; }
                    sub_bump = self.iir_sub_bump_b;
                }
                3 => {
                    self.iir_sub_bump_c += sub_bump * sub_gain;
                    self.iir_sub_bump_c -= self.iir_sub_bump_c * self.iir_sub_bump_c * self.iir_sub_bump_c * sub_headbump_freq;
                    self.iir_sub_bump_c = (fake_random * self.iir_sub_bump_a) + (fake_random * self.iir_sub_bump_b) + (inv_fake_random * self.iir_sub_bump_c);
                    if self.iir_sub_bump_c > 0.0 { self.iir_sub_bump_c -= clamp; }
                    if self.iir_sub_bump_c < 0.0 { self.iir_sub_bump_c += clamp; }
                    sub_bump = self.iir_sub_bump_c;
                }
                _ => unreachable!()
            }
            // Resample to reduce the sub bump further
            self.iir_sample_z = (self.iir_sample_z * (1.0 - sub_headbump_freq)) + (sub_bump * sub_headbump_freq);
            sub_bump = self.iir_sample_z;
            self.iir_drive_sample_e = (self.iir_drive_sample_e * (1.0 - self.sub_iir)) + (sub_bump * self.sub_iir);
            sub_bump = self.iir_drive_sample_e;
            self.iir_drive_sample_f = (self.iir_drive_sample_f * (1.0 - self.sub_iir)) + (sub_bump * self.sub_iir);
            sub_bump = self.iir_drive_sample_f;

            // Calculate our final sub drive
            if sub_drive > 0.0
            {
                sub_bump += tape_saturation(sub_bump, sub_drive);
            }
            
            // Add: Original signal + Harmonics + Sub signal
            match h_algorithm {
                AlgorithmType::ABass3 => {
                    let harmonic2_l: f32;
                    let harmonic2_r: f32;
                    let harmonic3_l: f32;
                    let harmonic3_r: f32;
                    let harmonic4_l: f32;
                    let harmonic4_r: f32;
                    let harmonic5_l: f32;
                    let harmonic5_r: f32;
                    let harmonic6_l: f32;
                    let harmonic6_r: f32;
                    let harmonic7_l: f32;
                    let harmonic7_r: f32;
                    let harmonic8_l: f32;
                    let harmonic8_r: f32;
                    let harmonic9_l: f32;
                    let harmonic9_r: f32;
                    let harmonic10_l: f32;
                    let harmonic10_r: f32;
                    let harmonic11_l: f32;
                    let harmonic11_r: f32;
                    let harmonic12_l: f32;
                    let harmonic12_r: f32;
                    let harmonic13_l: f32;
                    let harmonic13_r: f32;
                    let harmonic14_l: f32;
                    let harmonic14_r: f32;

                    (harmonic2_l, harmonic2_r) = SweetenX::process(in_l, in_r, overall_scale, 26.470589, 2, &mut self.buffer);
                    (harmonic3_l, harmonic3_r) = SweetenX::process(in_l, in_r, overall_scale, 8.941176, 3, &mut self.buffer);
                    (harmonic4_l, harmonic4_r) = SweetenX::process(in_l, in_r, overall_scale, 0.1764706, 4, &mut self.buffer);
                    (harmonic5_l, harmonic5_r) = SweetenX::process(in_l, in_r, overall_scale, 0.0, 5, &mut self.buffer);
                    (harmonic6_l, harmonic6_r) = SweetenX::process(in_l, in_r, overall_scale, 0.0, 6, &mut self.buffer);
                    (harmonic7_l, harmonic7_r) = SweetenX::process(in_l, in_r, overall_scale, 0.0, 7, &mut self.buffer);
                    (harmonic8_l, harmonic8_r) = SweetenX::process(in_l, in_r, overall_scale, 0.0, 8, &mut self.buffer);
                    (harmonic9_l, harmonic9_r) = SweetenX::process(in_l, in_r, overall_scale, 171.76471, 9, &mut self.buffer);

                    let octave_l = in_l * in_l * in_l * in_l * in_l * 0.5;
                    let octave_r = in_r * in_r * in_r * in_r * in_r * 0.5;
                    // Start from 5th
                    (harmonic10_l, harmonic10_r) = SweetenX::process(octave_l, octave_r, overall_scale, 0.0, 2, &mut self.buffer);
                    (harmonic11_l, harmonic11_r) = SweetenX::process(octave_l, octave_r, overall_scale, 4000.0, 3, &mut self.buffer);
                    (harmonic12_l, harmonic12_r) = SweetenX::process(octave_l, octave_r, overall_scale, 11764706.0, 4, &mut self.buffer);
                    (harmonic13_l, harmonic13_r) = SweetenX::process(octave_l, octave_r, overall_scale, 5294118000.0, 5, &mut self.buffer);
                    (harmonic14_l, harmonic14_r) = SweetenX::process(octave_l, octave_r, overall_scale, 17647059000.0, 6, &mut self.buffer);

                    processed_sample_l = in_l;
                    processed_sample_r = in_r;

                    // Sum all harmonics into the processed sample
                    //processed_sample_l += harmonic2_l + harmonic3_l + (harmonic7_l - harmonic5_l*2.0 - harmonic3_l*2.0 - harmonic2_l*2.0) + (harmonic9_l - harmonic2_l*2.0) + (sub_bump * sub_gain);
                    processed_sample_l += (harmonic2_l + harmonic3_l + harmonic4_l + harmonic5_l + 
                        harmonic6_l + harmonic7_l + harmonic8_l + harmonic9_l + 
                        harmonic10_l + harmonic11_l + harmonic12_l + harmonic13_l + 
                        harmonic14_l)*(harmonics * 1497.00599) + (sub_bump * sub_gain);
                    //processed_sample_r += harmonic2_r + harmonic3_r + (harmonic7_r - harmonic5_r*2.0 - harmonic3_r*2.0 - harmonic2_r*2.0) + (harmonic9_r - harmonic2_r*2.0) + (sub_bump * sub_gain);
                    processed_sample_r += (harmonic2_r + harmonic3_r + harmonic4_r + harmonic5_r + 
                        harmonic6_r + harmonic7_r + harmonic8_r + harmonic9_r + 
                        harmonic10_r + harmonic11_r + harmonic12_r + harmonic13_r + 
                        harmonic14_r)*(harmonics * 1497.00599) + (sub_bump * sub_gain);

                    // Scaling
                    let scale = util::db_to_gain(-21.2);
                    processed_sample_l *= scale;
                    processed_sample_r *= scale;

                }
                AlgorithmType::ABass2 => {
                    // Ardura's new Algorithm for 2024
                    processed_sample_l = custom_sincos_saturation(
                        in_l, 
                        harmonics * 31.422043, 
                        harmonics * 189.29568, 
                        harmonics * 25.0, 
                        harmonics * 26.197401) + (sub_bump * sub_gain);
                    processed_sample_r = custom_sincos_saturation(
                        in_l, 
                        harmonics * 31.422043, 
                        harmonics * 189.29568, 
                        harmonics * 25.0, 
                        harmonics * 26.197401) + (sub_bump * sub_gain);
                    let h_l = (processed_sample_l * 2.0) - processed_sample_l.powf(2.0);
                    let h_r = (processed_sample_r * 2.0) - processed_sample_r.powf(2.0);
                    processed_sample_l += h_l * 0.0070118904;
                    processed_sample_r += h_r * 0.0070118904;
                    processed_sample_l = util::db_to_gain(-2.4)*processed_sample_l;
                    processed_sample_r = util::db_to_gain(-2.4)*processed_sample_r;
                },
                AlgorithmType::BBass => {
                    // C3 signal in RBass is C3, C4, G4, C5, E5, A#5, D6, F#6
                    processed_sample_l = b_bass_saturation(in_l, harmonics) + (sub_bump * sub_gain);
                    processed_sample_r = b_bass_saturation(in_r, harmonics) + (sub_bump * sub_gain);
                    processed_sample_l = util::db_to_gain(8.7)*processed_sample_l;
                    processed_sample_r = util::db_to_gain(8.7)*processed_sample_r;
                },
                AlgorithmType::CBass => {
                    if harmonics > 0.0 {
                        processed_sample_l = c_bass_saturation(in_l, harmonics) + (sub_bump * sub_gain);
                        processed_sample_r = c_bass_saturation(in_r, harmonics) + (sub_bump * sub_gain);
                    } else {
                        processed_sample_l = sub_bump * sub_gain;
                        processed_sample_r = sub_bump * sub_gain;
                    }
                }
                AlgorithmType::TanH => {
                    // Generate tanh curve harmonics gently
                    processed_sample_l = tape_saturation(in_l, harmonics) + (sub_bump * sub_gain);
                    processed_sample_r = tape_saturation(in_r, harmonics) + (sub_bump * sub_gain);
                    processed_sample_l = util::db_to_gain(8.0)*processed_sample_l;
                    processed_sample_r = util::db_to_gain(8.0)*processed_sample_r;
                },
                AlgorithmType::CustomSliders => {
                    processed_sample_l = custom_sincos_saturation(in_l, harmonics*custom_harmonics1, harmonics*custom_harmonics2, harmonics*custom_harmonics3, harmonics*custom_harmonics4) + (sub_bump * sub_gain);
                    processed_sample_r = custom_sincos_saturation(in_r, harmonics*custom_harmonics1, harmonics*custom_harmonics2, harmonics*custom_harmonics3, harmonics*custom_harmonics4) + (sub_bump * sub_gain);
                    processed_sample_l = util::db_to_gain(-4.2)*processed_sample_l;
                    processed_sample_r = util::db_to_gain(-4.2)*processed_sample_r;
                },
            }

            // Hardness Saturation
            if h_algorithm == AlgorithmType::ABass3 {
                let leaf_wet_l: f32;
                let leaf_wet_r: f32;
                let threshold: f32 = util::db_to_gain(-30.0);
                leaf_wet_l = leaf_saturation(in_l, threshold, 0.5);
                leaf_wet_r = leaf_saturation(in_r, threshold, 0.5);
                let scaler = 0.0016129*hoof_hardness*100.0; //0.0015 default;
                processed_sample_l = scaler*leaf_wet_l + (1.0 - scaler)*processed_sample_l;
                processed_sample_r = scaler*leaf_wet_r + (1.0 - scaler)*processed_sample_r;
            } else {
                processed_sample_l = chebyshev_tape(processed_sample_l, hoof_hardness);
                processed_sample_r = chebyshev_tape(processed_sample_r, hoof_hardness);
            }
            
            
            // Increment/change the bass_flip_counter
            self.bass_flip_counter += 1;
            self.bass_flip_counter = 
                if self.bass_flip_counter < 1 || self.bass_flip_counter > 3 { 1 } 
                else { self.bass_flip_counter };
            
            // Remove DC Offset with single pole HP
            let hp_b0: f32 = 1.0;
            let hp_b1: f32 = -1.0;
            let hp_a1: f32 = -0.995;
        
            // Calculated below by Ardura in advance!
            // double sqrt2 = 1.41421356237;
            // double corner_frequency = 5.0 / sqrt2;
            // double hp_gain = 1 / sqrt(1 + (5.0 / (corner_frequency)) ^ 2);
            //let hp_gain = 0.577350269190468;
            let hp_gain = 1.0;
        
            // Apply the 1 pole HP to left side
            processed_sample_l = hp_gain * processed_sample_l;
            let temp_sample: f32 = hp_b0 * processed_sample_l + hp_b1 * self.prev_processed_in_l - hp_a1 * self.prev_processed_out_l;
            self.prev_processed_in_l = processed_sample_l;
            self.prev_processed_out_l = temp_sample;
            processed_sample_l = temp_sample;

            // Apply the 1 pole HP to right side
            processed_sample_r = hp_gain * processed_sample_r;
            let temp_sample: f32 = hp_b0 * processed_sample_r + hp_b1 * self.prev_processed_in_r - hp_a1 * self.prev_processed_out_r;
            self.prev_processed_in_r = processed_sample_r;
            self.prev_processed_out_r = temp_sample;
            processed_sample_r = temp_sample;
            
            ///////////////////////////////////////////////////////////////////////

            // Calculate dry/wet mix
            let wet_gain: f32 = dry_wet;
            processed_sample_l = in_l + processed_sample_l * wet_gain;
            processed_sample_r = in_r + processed_sample_r * wet_gain;

            // get the output amplitude here
            processed_sample_l = processed_sample_l*output_gain;
            processed_sample_r = processed_sample_r*output_gain;
            out_amplitude += processed_sample_l + processed_sample_r;

            // Assign back so we can output our processed sounds
            *channel_samples.get_mut(0).unwrap() = processed_sample_l;
            *channel_samples.get_mut(1).unwrap() = processed_sample_r;

            // calculations that are only displayed on the GUI while the GUI is open
            if self.params.editor_state.is_open() {
                // Input gain meter
                in_amplitude = (in_amplitude / num_samples as f32).abs();
                let current_in_meter = self.in_meter.load(std::sync::atomic::Ordering::Relaxed);
                let new_in_meter = if in_amplitude > current_in_meter {in_amplitude} else {current_in_meter * self.out_meter_decay_weight + in_amplitude * (1.0 - self.out_meter_decay_weight)};
                self.in_meter.store(new_in_meter, std::sync::atomic::Ordering::Relaxed);

                // Output gain meter
                out_amplitude = (out_amplitude / num_samples as f32).abs();
                let current_out_meter = self.out_meter.load(std::sync::atomic::Ordering::Relaxed);
                let new_out_meter = if out_amplitude > current_out_meter {out_amplitude} else {current_out_meter * self.out_meter_decay_weight + out_amplitude * (1.0 - self.out_meter_decay_weight)};
                self.out_meter.store(new_out_meter, std::sync::atomic::Ordering::Relaxed);
            }
        }

        ProcessStatus::Normal
    }

    const MIDI_INPUT: MidiConfig = MidiConfig::None;

    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const HARD_REALTIME_ONLY: bool = false;

    fn task_executor(&mut self) -> TaskExecutor<Self> {
        // In the default implementation we can simply ignore the value
        Box::new(|_| ())
    }

    fn filter_state(_state: &mut PluginState) {}

    fn reset(&mut self) {
        nih_dbg!("Plugin resetting...");
    }

    fn deactivate(&mut self) {
        nih_dbg!("Plugin deactivating...");
    }
}

impl ClapPlugin for Subhoofer {
    const CLAP_ID: &'static str = "com.ardura.subhoofer";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Harmonic and Subharmonic Bass Enhancement");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Distortion
    ];
}

impl Vst3Plugin for Subhoofer {
    const VST3_CLASS_ID: [u8; 16] = *b"SubhooferArduraA";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nih_export_clap!(Subhoofer);
nih_export_vst3!(Subhoofer);

// "Leaf" Saturation designed by Ardura
fn leaf_saturation(input_signal: f32, threshold: f32, drive: f32) -> f32 {
    let range = 6.0;
    let min_value = 1.0;
    let drive_db = min_value + drive * range;
    let signal_holder = input_signal * util::db_to_gain(drive_db);
    
    let curve = (signal_holder / 999.0).powf(2.0);

    let mut y = signal_holder / threshold;
    y = (2.0 / PI) * y.atan();
    (threshold + (1.0 - threshold) * curve) * y
}