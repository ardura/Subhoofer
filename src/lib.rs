#![allow(non_snake_case)]
mod ui_knob;
mod db_meter;
use atomic_float::AtomicF32;
use nih_plug::{prelude::*};
use nih_plug_egui::{create_egui_editor, egui::{self, Color32, Rect, Rounding, RichText, FontId, Pos2}, EguiState, widgets};
use std::{sync::{Arc}, ops::RangeInclusive};

/***************************************************************************
 * Subhoofer v2 by Ardura
 * 
 * Build with: cargo xtask bundle Subhoofer --profile <release or profiling>
 * *************************************************************************/

 #[derive(Enum, PartialEq, Eq, Debug, Copy, Clone)]
 pub enum AlgorithmType{
     #[name = "A Rennaisance Type Bass"]
     ABass,
     #[name = "8 Harmonic Stack"]
     BBass,
     #[name = "Non Octave Duro Console"]
     CBass,
     #[name = "TanH Transfer"]
     TanH,
     #[name = "Rennaisance Inspired 2"]
     ABass2,
     #[name = "User Control Sliders"]
     CustomSliders,
 }

 // GUI Colors
const A_KNOB_OUTSIDE_COLOR: Color32 = Color32::from_rgb(112,141,129);
const A_BACKGROUND_COLOR: Color32 = Color32::from_rgb(0,20,39);
const A_KNOB_INSIDE_COLOR: Color32 = Color32::from_rgb(244,213,141);
const A_KNOB_OUTSIDE_COLOR2: Color32 = Color32::from_rgb(242,100,25);

// Plugin sizing
const WIDTH: u32 = 360;
const HEIGHT: u32 = 528;

/// The time it takes for the peak meter to decay by 12 dB after switching to complete silence.
const PEAK_METER_DECAY_MS: f64 = 100.0;

pub struct Gain {
    params: Arc<GainParams>,

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
    if drive == 0.0 { return 0.0; }
    let idrive = drive;
    // Define the transfer curve for the tape saturation effect
    let transfer = |x: f32| -> f32 {
        (x * idrive).tanh()
    };
    // Apply the transfer curve to the input sample
    let output_sample = transfer(input_signal);
    output_sample - input_signal
}

/* This is my "close enough" to a certain letter bass plugin algorithm. The
    original one probably works in the frequency domain. This one is in the
    time domain since I have little to no knowledge of the FFT at this time.
    Maybe in the future this can become more efficient. - Ardura */
fn a_bass_saturation(signal: f32, harmonic_strength: f32) -> f32 {
    let num_harmonics: usize = 4;
    let mut summed: f32 = 0.0;

    for j in 1..=num_harmonics {
        match j {
            1 => {
                let harmonic_component: f32 = harmonic_strength * 170.0 * (signal * j as f32).cos() - signal;
                summed += harmonic_component;
            },
            2 => {
                let harmonic_component: f32 = harmonic_strength * 25.0 * (signal * j as f32).sin() - signal;
                summed += harmonic_component;
            },
            3 => {
                let harmonic_component: f32 = harmonic_strength * 150.0 * (signal * j as f32).cos() - signal;
                summed += harmonic_component;
            },
            4 => {
                let harmonic_component2: f32 = harmonic_strength * 80.0 * (signal * j as f32).sin() - signal;
                summed += harmonic_component2;
            },
            _ => unreachable!()
        }
    }
    if harmonic_strength > 0.0
    {
        summed
    }
    else {
        0.0
    }
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
    if harmonic_strength > 0.0
    {
        summed - signal
    }
    else {
        0.0
    }
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
    let num_harmonics: usize = 4;
    let mut summed: f32 = 0.0;

    for j in 1..=num_harmonics {
        match j {
            1 => {
                let harmonic_component: f32 = harmonic_strength1 * (signal * j as f32).cos() - signal;
                summed += harmonic_component;
            },
            2 => {
                let harmonic_component: f32 = harmonic_strength2 * (signal * j as f32).sin() - signal;
                summed += harmonic_component;
            },
            3 => {
                let harmonic_component: f32 = harmonic_strength3 * (signal * j as f32).cos() - signal;
                summed += harmonic_component;
            },
            4 => {
                let harmonic_component2: f32 = harmonic_strength4 * (signal * j as f32).sin() - signal;
                summed += harmonic_component2;
            },
            _ => unreachable!()
        }
    }
    summed
}

#[derive(Params)]
struct GainParams {
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

    #[id = "Harmonic Algorithm"]
    //pub h_algorithm: IntParam,
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

impl Default for Gain {
    fn default() -> Self {
        Self {
            params: Arc::new(GainParams::default()),
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
        }
    }
}

impl Default for GainParams {
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
                0.04,
                FloatRange::Linear {
                    min: 0.00,
                    max: 0.30,
                },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Hardness")
            .with_value_to_string(formatters::v2s_f32_percentage(2)),

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
                0.0011,
                FloatRange::Skewed { min: 0.0, max: 1.0, factor: FloatRange::skew_factor(-2.8) }
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Harmonics")
            .with_value_to_string(formatters::v2s_f32_percentage(2)),

            /*
            // Sub algorithm Parameter
            h_algorithm: IntParam::new(
                "Harmonic Algorithm",
                1,
                IntRange::Linear { min: 1, max: 5 },
            )
            .with_smoother(SmoothingStyle::Linear(30.0)),
            */

            h_algorithm: EnumParam::new("Harmonic Algorithm", AlgorithmType::ABass),


            // Custom Harmonics Parameter 1
            custom_harmonics1: FloatParam::new(
                "Custom Harmonic 1",
                0.0,
                FloatRange::Skewed { min: 0.0, max: 200.0, factor: FloatRange::skew_factor(-2.0) }
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Custom Harmonic 1"),

            // Custom Harmonics Parameter 2
            custom_harmonics2: FloatParam::new(
                "Custom Harmonic 2",
                0.0,
                FloatRange::Skewed { min: 0.0, max: 200.0, factor: FloatRange::skew_factor(-2.0) }
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Custom Harmonic 2"),

            // Custom Harmonics Parameter 3
            custom_harmonics3: FloatParam::new(
                "Custom Harmonic 3",
                0.0,
                FloatRange::Skewed { min: 0.0, max: 200.0, factor: FloatRange::skew_factor(-2.0) }
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Custom Harmonic 3"),

            // Custom Harmonics Parameter 4
            custom_harmonics4: FloatParam::new(
                "Custom Harmonic 4",
                0.0,
                FloatRange::Skewed { min: 0.0, max: 200.0, factor: FloatRange::skew_factor(-2.0) }
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Custom Harmonic 4"),

            // Output gain parameter
            output_gain: FloatParam::new(
                "Output Gain",
                util::db_to_gain(-2.8),
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

impl Plugin for Gain {
    const NAME: &'static str = "Duro Console";
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

    fn editor(self: &Gain, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
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
                            Rounding::from(16.0), A_BACKGROUND_COLOR);

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
                            ui.label(RichText::new("    Subhoofer").font(FontId::proportional(14.0)).color(A_KNOB_OUTSIDE_COLOR)).on_hover_text("by Ardura!");

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
                            in_meter_obj.set_background_color(A_KNOB_OUTSIDE_COLOR);
                            in_meter_obj.set_bar_color(A_KNOB_INSIDE_COLOR);
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
                            out_meter_obj.set_background_color(A_KNOB_OUTSIDE_COLOR);
                            out_meter_obj.set_bar_color(A_KNOB_INSIDE_COLOR);
                            out_meter_obj.set_border_color(Color32::BLACK);
                            ui.add(out_meter_obj);

                            ui.horizontal(|ui| {
                                let knob_size = 40.0;
                                ui.vertical(|ui| {
                                    let mut gain_knob = ui_knob::ArcKnob::for_param(&params.free_gain, setter, knob_size);
                                    gain_knob.preset_style(ui_knob::KnobStyle::SmallTogether);
                                    gain_knob.set_fill_color(A_KNOB_OUTSIDE_COLOR2);
                                    gain_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                    ui.add(gain_knob);

                                    let mut output_knob = ui_knob::ArcKnob::for_param(&params.output_gain, setter, knob_size);
                                    output_knob.preset_style(ui_knob::KnobStyle::SmallTogether);
                                    output_knob.set_fill_color(A_KNOB_OUTSIDE_COLOR2);
                                    output_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                    ui.add(output_knob);
                                
                                    let mut dry_wet_knob = ui_knob::ArcKnob::for_param(&params.dry_wet, setter, knob_size);
                                    dry_wet_knob.preset_style(ui_knob::KnobStyle::SmallTogether);
                                    dry_wet_knob.set_fill_color(A_KNOB_OUTSIDE_COLOR2);
                                    dry_wet_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                    ui.add(dry_wet_knob);
                                });

                                ui.vertical(|ui| {
                                    let mut hardness_knob = ui_knob::ArcKnob::for_param(&params.hoof_hardness, setter, knob_size + 16.0);
                                    hardness_knob.preset_style(ui_knob::KnobStyle::MediumThin);
                                    hardness_knob.set_fill_color(A_KNOB_INSIDE_COLOR);
                                    hardness_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                    ui.add(hardness_knob);

                                    let mut harmonics_knob = ui_knob::ArcKnob::for_param(&params.harmonics, setter, knob_size + 16.0);
                                    harmonics_knob.preset_style(ui_knob::KnobStyle::SmallMedium);
                                    harmonics_knob.set_fill_color(A_KNOB_INSIDE_COLOR);
                                    harmonics_knob.set_line_color(A_KNOB_OUTSIDE_COLOR);
                                    ui.add(harmonics_knob);
                                });

                                ui.vertical(|ui| {
                                    let mut sub_gain_knob = ui_knob::ArcKnob::for_param(&params.sub_gain, setter, knob_size);
                                    sub_gain_knob.preset_style(ui_knob::KnobStyle::LargeMedium);
                                    sub_gain_knob.set_fill_color(A_KNOB_INSIDE_COLOR);
                                    sub_gain_knob.set_line_color(A_KNOB_OUTSIDE_COLOR2);
                                    ui.add(sub_gain_knob);
                                
                                    let mut sub_drive_knob = ui_knob::ArcKnob::for_param(&params.sub_drive, setter, knob_size);
                                    sub_drive_knob.preset_style(ui_knob::KnobStyle::LargeMedium);
                                    sub_drive_knob.set_fill_color(A_KNOB_INSIDE_COLOR);
                                    sub_drive_knob.set_line_color(A_KNOB_OUTSIDE_COLOR2);
                                    ui.add(sub_drive_knob);

                                    // Deer
                                    ui.label(RichText::new(r"  ((        ))
   \\      //
 _| \\____// |__
\~~/ ~    ~\/~~~/
 -(|    _/o  ~.-
   /  /     ,|
  (~~~)__.-\ |
   ``-     | |
    |      | |
    |        |
").font(FontId::monospace(10.0)).color(A_KNOB_OUTSIDE_COLOR2));
                                });
                            });
                            //sliders
                            ui.vertical(|ui| {
                                ui.label("Harmonic Algorithm");
                                ui.add(widgets::ParamSlider::for_param(&params.h_algorithm, setter).with_width(100.0));
                                ui.add(widgets::ParamSlider::for_param(&params.custom_harmonics1, setter).with_width(200.0));
                                ui.add(widgets::ParamSlider::for_param(&params.custom_harmonics2, setter).with_width(200.0));
                                ui.add(widgets::ParamSlider::for_param(&params.custom_harmonics3, setter).with_width(200.0));
                                ui.add(widgets::ParamSlider::for_param(&params.custom_harmonics4, setter).with_width(200.0));
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

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
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
            let sample_rate: f32 = _context.transport().sample_rate;
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
			let mut sub_bump: f32 = head_bump;
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
                AlgorithmType::ABass => {
                    processed_sample_l = a_bass_saturation(in_l, harmonics) + (sub_bump * sub_gain);
                    processed_sample_r = a_bass_saturation(in_r, harmonics) + (sub_bump * sub_gain);
                },
                AlgorithmType::BBass => {
                    // C3 signal in RBass is C3, C4, G4, C5, E5, A#5, D6, F#6
                    processed_sample_l = b_bass_saturation(in_l, harmonics) + (sub_bump * sub_gain);
                    processed_sample_r = b_bass_saturation(in_r, harmonics) + (sub_bump * sub_gain);
                },
                AlgorithmType::CBass => {
                    processed_sample_l = c_bass_saturation(in_l, harmonics) + (sub_bump * sub_gain);
                    processed_sample_r = c_bass_saturation(in_r, harmonics) + (sub_bump * sub_gain);
                }
                AlgorithmType::TanH => {
                    // Generate tanh curve harmonics gently
                    processed_sample_l = tape_saturation(in_l, harmonics);
                    processed_sample_r = tape_saturation(in_r, harmonics);
                },
                AlgorithmType::ABass2 => {
                    // RBass inspired but skipping first and second harmonic multiplication
                    processed_sample_l = custom_sincos_saturation(in_l, harmonics*0.0, harmonics*0.0, harmonics*77.29513, harmonics*112.16742) + (sub_bump * sub_gain);
                    processed_sample_r = custom_sincos_saturation(in_l, harmonics*0.0, harmonics*0.0, harmonics*77.29513, harmonics*112.16742) + (sub_bump * sub_gain);
                },
                AlgorithmType::CustomSliders => {
                    processed_sample_l = custom_sincos_saturation(in_l, harmonics*custom_harmonics1, harmonics*custom_harmonics2, harmonics*custom_harmonics3, harmonics*custom_harmonics4) + (sub_bump * sub_gain);
                    processed_sample_r = custom_sincos_saturation(in_l, harmonics*custom_harmonics1, harmonics*custom_harmonics2, harmonics*custom_harmonics3, harmonics*custom_harmonics4) + (sub_bump * sub_gain);
                },
            }

            // Hardness Saturation
            processed_sample_l = chebyshev_tape(processed_sample_l, hoof_hardness);
            processed_sample_r = chebyshev_tape(processed_sample_r, hoof_hardness);
            
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

    fn task_executor(self: &Gain) -> TaskExecutor<Self> {
        // In the default implementation we can simply ignore the value
        Box::new(|_| ())
    }

    fn filter_state(_state: &mut PluginState) {}

    fn reset(&mut self) {}

    fn deactivate(&mut self) {}
}

impl ClapPlugin for Gain {
    const CLAP_ID: &'static str = "com.ardura.subhoofer";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Harmonic and Subharmonic Bass Enhancement");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for Gain {
    const VST3_CLASS_ID: [u8; 16] = *b"SubhooferArduraA";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nih_export_clap!(Gain);
nih_export_vst3!(Gain);
