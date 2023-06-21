mod ui_knob;
mod db_meter;
use atomic_float::AtomicF32;
use nih_plug::{prelude::*};
use nih_plug_egui::{create_egui_editor, egui::{self, Color32, Rect, Rounding, RichText, FontId, Pos2}, EguiState};
use std::{sync::{Arc}, ops::RangeInclusive};

/**************************************************
 * Duro Console by Ardura
 * 
 * Build with: cargo xtask bundle duro_console
 * ************************************************/

// GUI Colors
const A_TEAL: Color32 = Color32::from_rgb(60, 110, 113);
const A_DARK_GRAY: Color32 = Color32::from_rgb(53, 53, 53);
const A_PLATINUM: Color32 = Color32::from_rgb(217, 217, 217);
const A_BLUE: Color32 = Color32::from_rgb(40, 75, 99);
const A_WHITE: Color32 = Color32::WHITE;

// Plugin sizing
const WIDTH: u32 = 400;
const HEIGHT: u32 = 500;

/// The time it takes for the peak meter to decay by 12 dB after switching to complete silence.
const PEAK_METER_DECAY_MS: f64 = 100.0;

pub struct Gain {
    params: Arc<GainParams>,

    // normalize the peak meter's response based on the sample rate with this
    out_meter_decay_weight: f32,

    // "header" variables from C++ class
    lp: f32,
    iirDriveSampleA: f32,
    iirDriveSampleB: f32,
    oscGate: f32,

    // The current data for the different meters
    out_meter: Arc<AtomicF32>,
    in_meter: Arc<AtomicF32>,
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

    #[id = "Sub Algorithm"]
    pub sub_algorithm: IntParam,

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
            oscGate: 0.0,
            lp: 0.0,
            iirDriveSampleA: 0.0,
            iirDriveSampleB: 0.0,
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
            .with_unit(" Input Gain")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            // Hoof Parameter
            hoof_hardness: FloatParam::new(
                "Hoof Hardness",
                0.5,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" Hardness"),

            // Sub gain dB parameter
            sub_gain: FloatParam::new(
                "Sub Gain",
                util::db_to_gain(3.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(0.0),
                    max: util::db_to_gain(24.0),
                    factor: FloatRange::gain_skew_factor(0.0, 24.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" dB Sub Gain")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            // Sub Drive dB parameter
            sub_drive: FloatParam::new(
                "Sub Drive",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(0.0),
                    max: util::db_to_gain(24.0),
                    factor: FloatRange::gain_skew_factor(0.0, 24.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" dB Sub Drive")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            // Harmonics Parameter
            harmonics: FloatParam::new(
                "Harmonics",
                util::db_to_gain(-24.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-24.0),
                    max: util::db_to_gain(24.0),
                    factor: FloatRange::gain_skew_factor(-24.0, 24.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(30.0))
            .with_unit(" dB Sub Drive")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            // Sub algorithm Parameter
            sub_algorithm: IntParam::new(
                "Sub Algorithm",
                1,
                IntRange::Linear { min: 1, max: 3 },
            )
            .with_smoother(SmoothingStyle::Linear(30.0)),

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
            .with_unit(" dB Output Gain")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
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

    fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
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
                        //style_var.visuals.widgets.inactive.bg_fill = A_DARK_GRAY;

                        // Assign default colors if user colors not set
                        /*
                        style_var.visuals.widgets.inactive.fg_stroke.color = A_TEAL;
                        style_var.visuals.widgets.noninteractive.fg_stroke.color = Color32::WHITE;
                        style_var.visuals.widgets.inactive.bg_stroke.color = A_PLATINUM;
                        style_var.visuals.widgets.active.fg_stroke.color = Color32::LIGHT_RED;
                        style_var.visuals.widgets.active.bg_stroke.color = A_WHITE;
                        style_var.visuals.widgets.open.fg_stroke.color = A_BLUE;
                        // Param fill
                        style_var.visuals.selection.bg_fill = A_WHITE;

                        style_var.visuals.widgets.noninteractive.bg_stroke.color = Color32::LIGHT_YELLOW;
                        style_var.visuals.widgets.noninteractive.bg_fill = Color32::RED;
                        */
                        // Trying to draw background as rect
                        ui.painter().rect_filled(
                            Rect::from_x_y_ranges(
                                RangeInclusive::new(0.0, WIDTH as f32), 
                                RangeInclusive::new(0.0, HEIGHT as f32)), 
                            Rounding::from(16.0), A_DARK_GRAY);

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
                            ui.label(RichText::new("    Subhoofer").font(FontId::proportional(14.0)).color(A_TEAL)).on_hover_text("by Ardura!");

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
                            in_meter_obj.set_background_color(A_DARK_GRAY);
                            in_meter_obj.set_bar_color(A_BLUE);
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
                            out_meter_obj.set_background_color(A_DARK_GRAY);
                            out_meter_obj.set_bar_color(A_BLUE);
                            out_meter_obj.set_border_color(Color32::BLACK);
                            ui.add(out_meter_obj);

                            ui.horizontal(|ui| {
                                let knob_size = 38.0;
                                ui.vertical_centered(|ui| {
                                    let mut gain_knob = ui_knob::ArcKnob::for_param(&params.free_gain, setter, knob_size);
                                    gain_knob.preset_style(ui_knob::KnobStyle::SmallTogether);
                                    gain_knob.set_fill_color(A_WHITE);
                                    gain_knob.set_line_color(A_TEAL);
                                    ui.add(gain_knob);

                                    let mut sat_type_knob = ui_knob::ArcKnob::for_param(&params.hoof_hardness, setter, knob_size);
                                    sat_type_knob.preset_style(ui_knob::KnobStyle::SmallSmallOutline);
                                    sat_type_knob.set_fill_color(A_BLUE);
                                    sat_type_knob.set_line_color(A_PLATINUM);
                                    ui.add(sat_type_knob);
                                });

                                ui.vertical_centered(|ui| {
                                    let mut threshold_knob = ui_knob::ArcKnob::for_param(&params.sub_gain, setter, knob_size + 8.0);
                                    threshold_knob.preset_style(ui_knob::KnobStyle::SmallMedium);
                                    threshold_knob.set_fill_color(A_BLUE);
                                    threshold_knob.set_line_color(A_PLATINUM);
                                    ui.add(threshold_knob);
                                
                                    let mut drive_knob = ui_knob::ArcKnob::for_param(&params.sub_drive, setter, knob_size + 8.0);
                                    drive_knob.preset_style(ui_knob::KnobStyle::SmallMedium);
                                    drive_knob.set_fill_color(A_BLUE);
                                    drive_knob.set_line_color(A_PLATINUM);
                                    ui.add(drive_knob);

                                    let mut console_knob = ui_knob::ArcKnob::for_param(&params.harmonics, setter, knob_size + 8.0);
                                    console_knob.preset_style(ui_knob::KnobStyle::SmallMedium);
                                    console_knob.set_fill_color(A_BLUE);
                                    console_knob.set_line_color(A_PLATINUM);
                                    ui.add(console_knob);
                                });

                                ui.vertical_centered(|ui| {
                                    let mut output_knob = ui_knob::ArcKnob::for_param(&params.output_gain, setter, knob_size);
                                    output_knob.preset_style(ui_knob::KnobStyle::SmallTogether);
                                    output_knob.set_fill_color(A_WHITE);
                                    output_knob.set_line_color(A_TEAL);
                                    ui.add(output_knob);
                                
                                    let mut dry_wet_knob = ui_knob::ArcKnob::for_param(&params.dry_wet, setter, knob_size);
                                    dry_wet_knob.preset_style(ui_knob::KnobStyle::SmallTogether);
                                    dry_wet_knob.set_fill_color(A_WHITE);
                                    dry_wet_knob.set_line_color(A_TEAL);
                                    ui.add(dry_wet_knob);
                                
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

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {

        //widgets::ParamEvent
        // Buffer level
        for channel_samples in buffer.iter_samples() {
            let mut out_amplitude: f32 = 0.0;
            let mut in_amplitude: f32 = 0.0;
            let mut processed_sample: f32;
            let num_samples = channel_samples.len();

            let gain: f32 = util::gain_to_db(self.params.free_gain.smoothed.next());
            let mut num_gain: f32;
            let hoof_hardness: f32 = self.params.hoof_hardness.smoothed.next();
            let sub_gain: f32 = self.params.sub_gain.smoothed.next();
            let output_gain: f32 = self.params.output_gain.smoothed.next();
            let sub_drive: f32 = self.params.sub_drive.smoothed.next();
            let harmonics: f32 = self.params.harmonics.smoothed.next();
            let sub_algorithm: i32 = self.params.sub_algorithm.smoothed.next();
            let dry_wet: f32 = self.params.dry_wet.value();

            let fake_random: f32 = 0.83;
            let inv_fake_random: f32 = 1.0 - fake_random;

            // Scale the head bump freqeuncy for Subhoof
            let sample_rate: f32 = _context.transport().sample_rate;
            let mut overall_scale: f32 = 1.0;
            overall_scale /= 44100.0;
            overall_scale *= sample_rate;

            for sample in channel_samples {
                num_gain = gain;
                *sample *= util::db_to_gain(num_gain);
                in_amplitude += *sample;

                ///////////////////////////////////////////////////////////////////////
                // Perform processing on the sample
                
                // Normalize really small values
                if sample.abs() < 1.18e-23 { *sample = 0.1 * 1.18e-17; }

                // Sub voicing variables
                let sub_headbump_freq: f32 = (((hoof_hardness) * 0.1) + 0.02) / overall_scale;
                let sub_iir: f32 = sub_headbump_freq / 44.1;
                // BassGain = sub_drive

                // Sub drive samples
                self.lp = *sample / 2048.0;
                self.iirDriveSampleA = (self.iirDriveSampleA * (1.0 - sub_headbump_freq)) + (self.lp * sub_headbump_freq);
                self.lp = self.iirDriveSampleA;

                if sub_algorithm == 1
                {
                    // Gate from airwindows
                    self.oscGate += (self.lp * 10.0).abs();
                    self.oscGate -= 0.001;
                    if self.oscGate > 1.0 {self.oscGate = 1.0;}
                    if self.oscGate < 0.0 {self.oscGate = 0.0;}
                    //got a value that only goes down low when there's silence or near silence on input
                    let clamp = (1.0 - self.oscGate) * 0.00001;

                    // Figure out our zero crossing
                }
                else if sub_algorithm == 2
                {
                    // TODO: Pitch shift
                }
                else if sub_algorithm == 3
                {
                    // TODO: FFT Find lowest Freq
                }
                

                processed_sample = 0.0;
                ///////////////////////////////////////////////////////////////////////

                // Calculate dry/wet mix (no compression but saturation possible)
                let wet_gain = dry_wet;
                let dry_gain = 1.0 - dry_wet;
                processed_sample = *sample * dry_gain + processed_sample * wet_gain;

                // get the output amplitude here
                processed_sample = processed_sample*output_gain;
                *sample = processed_sample;
                out_amplitude += processed_sample;
            }

            // To save resources, a plugin can (and probably should!) only perform expensive
            // calculations that are only displayed on the GUI while the GUI is open
            if self.params.editor_state.is_open() {
                // Input gain meter
                in_amplitude = (in_amplitude / num_samples as f32).abs();
                let current_in_meter = self.in_meter.load(std::sync::atomic::Ordering::Relaxed);
                let new_in_meter = if in_amplitude > current_in_meter {in_amplitude}                                else {current_in_meter * self.out_meter_decay_weight + in_amplitude * (1.0 - self.out_meter_decay_weight)};
                self.in_meter.store(new_in_meter, std::sync::atomic::Ordering::Relaxed);

                // Output gain meter
                out_amplitude = (out_amplitude / num_samples as f32).abs();
                let current_out_meter = self.out_meter.load(std::sync::atomic::Ordering::Relaxed);
                let new_out_meter = if out_amplitude > current_out_meter {out_amplitude}                            else {current_out_meter * self.out_meter_decay_weight + out_amplitude * (1.0 - self.out_meter_decay_weight)};
                self.out_meter.store(new_out_meter, std::sync::atomic::Ordering::Relaxed);
            }
        }

        ProcessStatus::Normal
    }

    const MIDI_INPUT: MidiConfig = MidiConfig::None;

    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const HARD_REALTIME_ONLY: bool = false;

    fn task_executor(&self) -> TaskExecutor<Self> {
        // In the default implementation we can simply ignore the value
        Box::new(|_| ())
    }

    fn filter_state(_state: &mut PluginState) {}

    fn reset(&mut self) {}

    fn deactivate(&mut self) {}
}

impl ClapPlugin for Gain {
    const CLAP_ID: &'static str = "com.ardura.duro.console";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A console with a combination of saturation algorithms");
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
    const VST3_CLASS_ID: [u8; 16] = *b"SubhooferAAAAAAA";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nih_export_clap!(Gain);
nih_export_vst3!(Gain);
