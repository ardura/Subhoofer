// Ardura 2023 - ui_knob.rs - egui + nih-plug parameter widget with customization
//  this ui_knob.rs is built off a2aaron's knob base as part of nyasynth
// https://github.com/a2aaron/nyasynth/blob/canon/src/ui_knob.rs

use std::{f32::consts::TAU, ops::{Add, Sub, Mul}};

use nih_plug::prelude::{Param, ParamSetter};
use nih_plug_egui::egui::{
    epaint::{PathShape, CircleShape}, pos2, Align2, Color32, FontId, Id, Pos2, Rect, Response, Rgba, Sense,
    Shape, Stroke, Ui, Widget, self, Vec2,
};
use once_cell::sync::Lazy;

static DRAG_AMOUNT_MEMORY_ID: Lazy<Id> = Lazy::new(|| Id::new("drag_amount_memory_id"));

struct SliderRegion<'a, P: Param> {
    param: &'a P,
    param_setter: &'a ParamSetter<'a>,
}

impl<'a, P: Param> SliderRegion<'a, P> {
    fn new(param: &'a P, param_setter: &'a ParamSetter) -> Self {
        SliderRegion {
            param,
            param_setter,
        }
    }

    // Handle the input for a given response. Returns an f32 containing the normalized value of
    // the parameter.
    fn handle_response(&self, ui: &Ui, response: &Response) -> f32 {
        let value = self.param.unmodulated_normalized_value();
        if response.drag_started() {
            self.param_setter.begin_set_parameter(self.param);
            ui.memory().data.insert_temp(*DRAG_AMOUNT_MEMORY_ID, value)
        }

        if response.dragged() {
            // Invert the y axis, since we want dragging up to increase the value and down to
            // decrease it, but drag_delta() has the y-axis increasing downwards.
            let delta = -response.drag_delta().y;
            let mut memory = ui.memory();
            let value = memory.data.get_temp_mut_or(*DRAG_AMOUNT_MEMORY_ID, value);
            *value = (*value + delta / 100.0).clamp(0.0, 1.0);
            self.param_setter
                .set_parameter_normalized(self.param, *value);
        }

        // Reset on doubleclick
        if response.double_clicked() {
            self.param_setter
                .set_parameter_normalized(self.param, self.param.default_normalized_value());
        }

        if response.drag_released() {
            self.param_setter.end_set_parameter(self.param);
        }
        value
    }

    fn get_string(&self) -> String {
        self.param.to_string()
    }
}

pub struct ArcKnob<'a, P: Param> {
    slider_region: SliderRegion<'a, P>,
    radius: f32,
    line_color: Color32,
    fill_color: Color32,
    center_size: f32,
    line_width: f32,
    center_to_line_space: f32,
    hover_text: bool,
    label_text: String,
    outline: bool,
}

#[allow(dead_code)]
pub enum KnobStyle {
    // Knob_line
    SmallTogether,
    MediumThin,
    LargeMedium,
    SmallLarge,
    SmallMedium,
    SmallSmallOutline,
}

#[allow(dead_code)]
impl<'a, P: Param> ArcKnob<'a, P> {
    pub fn for_param(param: &'a P, param_setter: &'a ParamSetter, radius: f32) -> Self {
        ArcKnob {
            slider_region: SliderRegion::new(param, param_setter),
            radius: radius,
            line_color: Color32::BLACK,
            fill_color: Color32::BLACK,
            center_size: 20.0,
            line_width: 2.0,
            center_to_line_space: 0.0,
            hover_text: false,
            label_text: String::new(),
            outline: false,
        }
    }

    // Specify outline drawing
    pub fn use_outline(&mut self, new_bool: bool) {
        self.outline = new_bool;
    }

    // Specify showing value when mouse-over
    pub fn use_hover_text(&mut self, new_bool: bool) {
        self.hover_text = new_bool;
    }

    // Specify knob label
    pub fn set_label(&mut self, new_label: String) {
        self.label_text = new_label;
    }

    // Specify line color for knob outside
    pub fn set_line_color(&mut self, new_color: Color32) {
        self.line_color = new_color;
    }

    // Specify fill color for knob
    pub fn set_fill_color(&mut self, new_color: Color32) {
        self.fill_color = new_color;
    }

    // Specify center knob size
    pub fn set_center_size(&mut self, size: f32) {
        self.center_size = size;
    }

    // Specify line width
    pub fn set_line_width(&mut self, width: f32) {
        self.line_width = width;
    }

    // Specify distance between center and arc
    pub fn set_center_to_line_space(&mut self, new_width: f32) {
        self.center_to_line_space = new_width;
    }

    pub fn preset_style(&mut self, style_id: KnobStyle)
    {
        // These are all calculated off radius to scale better
        match style_id {
            KnobStyle::SmallTogether => {
                self.center_size = self.radius / 4.0;
                self.line_width = self.radius / 2.0;
                self.center_to_line_space = 0.0;
            }
            KnobStyle::MediumThin => {
                self.center_size = self.radius / 2.0;
                self.line_width = self.radius / 8.0;
                self.center_to_line_space = self.radius / 4.0;
            }
            KnobStyle::LargeMedium => {
                self.center_size = self.radius / 1.333;
                self.line_width = self.radius / 4.0;
                self.center_to_line_space = self.radius / 8.0;
            }
            KnobStyle::SmallLarge => {
                self.center_size = self.radius / 8.0;
                self.line_width = self.radius / 1.333;
                self.center_to_line_space = self.radius / 2.0;
            }
            KnobStyle::SmallMedium => {
                self.center_size = self.radius / 4.0;
                self.line_width = self.radius / 2.666;
                self.center_to_line_space = self.radius / 1.666;
            }
            KnobStyle::SmallSmallOutline => {
                self.center_size = self.radius / 4.0;
                self.line_width = self.radius / 4.0;
                self.center_to_line_space = self.radius / 4.0;
                self.outline = true;
            }
        }
    }
}

impl<'a, P: Param> Widget for ArcKnob<'a, P> {
    fn ui(self, ui: &mut Ui) -> Response {
        // Turns into 5 on each side
        let padding = 10.0;

        // Figure out the size to reserve on screen for widget
        let desired_size = egui::vec2(padding + self.radius*2.0,padding + self.radius*2.0);
        let response = ui.allocate_response(desired_size, Sense::click_and_drag());
        let value = self.slider_region.handle_response(&ui, &response);

        ui.vertical(|ui| {
            let painter = ui.painter_at(response.rect);
            let center = response.rect.center();

            // Draw the arc
            let arc_radius = self.center_size + self.center_to_line_space;
            let arc_stroke = Stroke::new(self.line_width, self.line_color);
            let shape = Shape::Path(PathShape {
                points: get_arc_points(center, arc_radius, value, 0.03),
                closed: false,
                fill: Color32::TRANSPARENT,
                stroke: arc_stroke,
            });
            painter.add(shape);

            // Draw the outside ring around the control
            if self.outline {
                let outline_stroke = Stroke::new(1.0, self.fill_color);
                let outline_shape = Shape::Path(PathShape {
                    points: get_arc_points(center, self.radius + 2.0, 1.0, 0.03),
                    closed: false,
                    fill: Color32::TRANSPARENT,
                    stroke: outline_stroke,
                });
                painter.add(outline_shape);
            }

            //reset stroke here so we only have fill
            let line_stroke = Stroke::new(0.0, Color32::TRANSPARENT);

            // Center of Knob
            let circle_shape = Shape::Circle(CircleShape{ center: center, radius: self.center_size, stroke: line_stroke, fill: self.fill_color });
            painter.add(circle_shape);

            // Hover text of value
            if self.hover_text {
                ui.allocate_rect(
                    Rect::from_center_size(center, Vec2::new(self.radius*2.0, self.radius*2.0)), Sense::hover())
                        .on_hover_text(self.slider_region.get_string());
            }

            // Label text from response rect bound
            let label_pos = Pos2::new(response.rect.center_bottom().x,response.rect.center_bottom().y - padding);
            if self.label_text.is_empty() {
                painter.text(label_pos, Align2::CENTER_CENTER, self.slider_region.get_string(), FontId::proportional(12.0), self.line_color);
                //ui.label(self.slider_region.get_string());
            }
            else {
                painter.text(label_pos, Align2::CENTER_CENTER, self.label_text, FontId::proportional(12.0), self.line_color);
                //ui.label(self.label_text);
            }
            
        });
        response
    }
}


fn get_arc_points(center: Pos2, radius: f32, value: f32, max_arc_distance: f32) -> Vec<Pos2> {
    let start_turns: f32 = 0.625;
    let arc_length = lerp(0.0, -0.75, value);
    let end_turns = start_turns + arc_length;

    let points = (arc_length.abs() / max_arc_distance).ceil() as usize;
    let points = points.max(1);
    (0..=points)
        .map(|i| {
            let t = i as f32 / (points - 1) as f32;
            let angle = lerp(start_turns * TAU, end_turns * TAU, t);
            let x = radius * angle.cos();
            let y = -radius * angle.sin();
            pos2(x, y) + center.to_vec2()
        })
        .collect()
}

// Moved lerp to this file to reduce dependencies - Ardura
pub fn lerp<T>(start: T, end: T, t: f32) -> T
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Mul<f32, Output = T> + Copy,
{
    (end - start) * t.clamp(0.0, 1.0) + start
}

pub struct TextSlider<'a, P: Param> {
    slider_region: SliderRegion<'a, P>,
    location: Rect,
}

#[allow(dead_code)]
impl<'a, P: Param> TextSlider<'a, P> {
    pub fn for_param(param: &'a P, param_setter: &'a ParamSetter, location: Rect) -> Self {
        TextSlider {
            slider_region: SliderRegion::new(param, param_setter),
            location,
        }
    }
}

impl<'a, P: Param> Widget for TextSlider<'a, P> {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_rect(self.location, Sense::click_and_drag());
        self.slider_region.handle_response(&ui, &response);

        let painter = ui.painter_at(self.location);
        let center = self.location.center();

        // Draw the text
        let text = self.slider_region.get_string();
        let anchor = Align2::CENTER_CENTER;
        let color = Color32::from(Rgba::WHITE);
        let font = FontId::monospace(16.0);
        painter.text(center, anchor, text, font, color);
        response
    }
}