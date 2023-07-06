// db_meter.rs - Ardura 2023
// A decibel meter akin to Vizia's nice one in nih-plug

use nih_plug_egui::egui::{Ui, Widget, Stroke, Color32, WidgetText, Response, vec2, Sense, NumExt, Rect, lerp, Shape, Pos2, Vec2, TextStyle};

// TODO - let percentage work?
#[allow(dead_code)]
enum DBMeterText {
    Custom(WidgetText),
    Percentage,
}

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct DBMeter {
    level: f32,
    desired_width: Option<f32>,
    text: Option<DBMeterText>,
    animate: bool,
    border_color: Color32,
    bar_color: Color32,
    background_color: Color32,
}

#[allow(dead_code)]
impl DBMeter {
    /// Progress in the `[0, 1]` range, where `1` means "completed".
    pub fn new(level: f32) -> Self {
        Self {
            level: level.clamp(0.0, 1.0),
            desired_width: None,
            text: None,
            animate: false,
            border_color: Color32::BLACK,
            bar_color: Color32::GREEN,
            background_color: Color32::GRAY,
        }
    }

    /// The desired width of the bar. Will use all horizontal space if not set.
    pub fn desired_width(mut self, desired_width: f32) -> Self {
        self.desired_width = Some(desired_width);
        self
    }

    /// A custom text to display on the progress bar.
    pub fn text(mut self, text: impl Into<WidgetText>) -> Self {
        self.text = Some(DBMeterText::Custom(text.into()));
        self
    }

    /// Set the color of the outline and text
    pub fn set_border_color(&mut self, new_color: Color32) {
        self.border_color = new_color;
    }

    /// Set the bar color for the meter
    pub fn set_bar_color(&mut self, new_color: Color32) {
        self.bar_color = new_color;
    }

    /// Set the background color
    pub fn set_background_color(&mut self, new_color: Color32) {
        self.background_color = new_color;
    }
}

impl Widget for DBMeter {
    #[allow(unused_variables)]
    fn ui(self, ui: &mut Ui) -> Response {
        let DBMeter {
            level,
            desired_width,
            text,
            animate, 
            border_color, 
            bar_color, 
            background_color } = self;

        let animate = animate && level < 1.0;

        let desired_width =
            desired_width.unwrap_or_else(|| ui.available_size_before_wrap().x.at_least(96.0));
        let height = ui.spacing().interact_size.y;
        let (outer_rect, response) =
            ui.allocate_exact_size(vec2(desired_width, height), Sense::hover());

        if ui.is_rect_visible(response.rect) {
            if animate {
                ui.ctx().request_repaint();
            }

            let visuals = ui.style().visuals.clone();
            //let rounding = outer_rect.height() / 2.0;
            // Removed rounding
            let rounding = 0.0;
            ui.painter().rect(
                outer_rect,
                rounding,
                self.background_color,
                Stroke::new(1.0,self.border_color),
            );
            let inner_rect = Rect::from_min_size(
                outer_rect.min,
                vec2(
                    (outer_rect.width() * level).at_least(outer_rect.height()),
                    outer_rect.height(),
                ),
            );

            ui.painter().rect(
                inner_rect,
                rounding,
                if self.level < 1.0 {self.bar_color} else {Color32::RED},
                Stroke::none(),
            );

            if animate {
                let n_points = 20;
                let start_angle = ui.input().time * std::f64::consts::TAU;
                let end_angle = start_angle + 240f64.to_radians() * ui.input().time.sin();
                let circle_radius = rounding - 2.0;
                let points: Vec<Pos2> = (0..n_points)
                    .map(|i| {
                        let angle = lerp(start_angle..=end_angle, i as f64 / n_points as f64);
                        let (sin, cos) = angle.sin_cos();
                        inner_rect.right_center()
                            + circle_radius * vec2(cos as f32, sin as f32)
                            + vec2(-rounding, 0.0)
                    })
                    .collect();
                ui.painter().add(Shape::line(
                    points,
                    Stroke::new(2.0, self.border_color),
                ));
            }

            // Markers
            let marker_spacing = outer_rect.width()/12.0;
            let points_x = (
                outer_rect.left_bottom().x as i32..=outer_rect.right_bottom().x as i32).step_by(marker_spacing as usize);

            for x in points_x
            {
                let points: Vec<Pos2> = vec![Pos2::new(x as f32,outer_rect.left_bottom().y),Pos2::new(x as f32,outer_rect.left_bottom().y-10.0)];
                ui.painter().add(Shape::line(points,Stroke::new(1.0, self.border_color),));
            }

            if let Some(text_kind) = text {
                let text = match text_kind {
                    DBMeterText::Custom(text) => text,
                    DBMeterText::Percentage => {
                        format!("{}%", (level * 100.0) as usize).into()
                    }
                };
                let galley = text.into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);
                let text_pos = outer_rect.left_center() - Vec2::new(0.0, galley.size().y / 2.0)
                    + vec2(ui.spacing().item_spacing.x, 0.0);
                let text_color = visuals
                    .override_text_color
                    .unwrap_or(self.border_color);
                galley.paint_with_fallback_color(
                    &ui.painter().with_clip_rect(outer_rect),
                    text_pos,
                    text_color,
                );
            }
        }

        response
    }
}