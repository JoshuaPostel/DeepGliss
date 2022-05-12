use std::ops::{Range, RangeInclusive};
use std::time::Duration;

use std::sync::Arc;

use crate::midi::Note;
use crate::state::GlissParam::{BendDuration, HoldDuration};
use crate::EditorState;

use egui::{Color32, Id, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};

pub struct Timeline {
    to_screen: emath::RectTransform,
    pub midi_notes: Range<u8>,
    line_spacing: f32,
    line_spacing_absolute: f32,
    //present_x_coordinate: f32,
    pub total_duration: Duration,
    // TODO infer this from EditorState?
    pub bend_duration: Duration,
    pub history_duration: Duration,
}

impl Timeline {
    pub fn new(
        rect: Rect,
        octave_range: RangeInclusive<u8>,
        total_duration: Duration,
        bend_duration: Duration,
    ) -> Self {
        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0), rect);

        let midi_notes = ((octave_range.start() + 1) * 12)..((octave_range.end() + 2) * 12);
        let n_notes = octave_range.len() * 12;
        let line_spacing = 1.0 / n_notes as f32;
        let p1 = to_screen * Pos2::new(0.0, line_spacing);
        let p2 = to_screen * Pos2::new(0.0, line_spacing * 2.0);
        let line_spacing_absolute = p2.y - p1.y;

        Timeline {
            to_screen,
            midi_notes,
            line_spacing,
            line_spacing_absolute,
            total_duration,
            bend_duration,
            history_duration: total_duration - bend_duration,
        }
    }

    pub fn draw(&self, now: Duration, recent_notes: Vec<Note>) -> Vec<Shape> {
        log::debug!("drawing {} notes", recent_notes.len());
        let mut shapes = self.draw_hlines();
        shapes.push(self.draw_vline());

        for note in recent_notes {
            //if (self.midi_notes.start-1..=self.midi_notes.end+1).contains(&note.midi_number) {
            if self.midi_notes.contains(&note.midi_number) {
                if let Some(whole_note) = self.draw_whole_note(now, note) {
                    shapes.push(whole_note);
                }
                // TODO implement else case
            }
        }
        shapes
    }

    fn draw_vline(&self) -> Shape {
        let x = 1.0 - self.bend_duration.div_duration_f32(self.total_duration);
        let p1 = self.to_screen * Pos2::new(x, 0.0);
        let p2 = self.to_screen * Pos2::new(x, 1.0);
        let stroke = Stroke::new(0.5, Color32::from_additive_luminance(50));
        egui::Shape::line_segment([p1, p2], stroke)
    }

    fn draw_hlines(&self) -> Vec<Shape> {
        let mut hlines = vec![];
        for i in 0..=self.midi_notes.len() {
            let p1 = self.to_screen * Pos2::new(0.0, self.line_spacing * i as f32);
            let p2 = self.to_screen * Pos2::new(1.0, self.line_spacing * i as f32);
            let stroke = Stroke::new(0.5, Color32::from_additive_luminance(50));
            let hline = egui::Shape::line_segment([p1, p2], stroke);
            hlines.push(hline);
        }
        hlines
    }

    fn draw_whole_note(&self, now: Duration, note: Note) -> Option<Shape> {
        let relative_note = self.midi_notes.end - note.midi_number - 1;
        let stroke = Stroke::new(self.line_spacing_absolute / 7.5, Color32::WHITE);
        if let Some(end_time) = now.checked_sub(self.history_duration) {
            let draw_time: Duration = if note.new_note_on {
                note.ui_time
            } else {
                note.ui_time + Duration::from_nanos(note.bend_duration as u64)
            };
            log::debug!("time debug - note.ui_time after adj: {:?}", note.ui_time);
            if let Some(relative_time) = draw_time.checked_sub(end_time) {
                let x = relative_time.div_duration_f32(self.total_duration);
                let p1 = self.to_screen
                    * Pos2::new(
                        x,
                        (relative_note as f32 * self.line_spacing) + (self.line_spacing / 2.0),
                    );
                let radius = (self.line_spacing_absolute / 2.0) - stroke.width;
                let whole_note = Shape::circle_stroke(p1, radius, stroke);
                Some(whole_note)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn draw_control_pin(&self, state: &Arc<EditorState>, ui: &mut Ui) {
        let config = BendDuration.get_config();
        let mut bend_duration_param = state.get_ui_parameter(BendDuration);
        let x = 1.0 - self.bend_duration.div_duration_f32(self.total_duration);

        let mut p1 = self.to_screen * Pos2::new(x, 0.0);
        let radius: f32 = 7.0;
        let box_diag = 2.0_f32.sqrt();
        let color = Color32::WHITE;
        p1 += Vec2::new(0.0, -radius * box_diag);
        let mut pin_rect = Rect::from_center_size(p1, Vec2::new(radius * 2.0, radius * 2.0));
        let pin_circle_shape = Shape::circle_filled(p1, radius, color);
        let circle_corner = 2.0_f32.sqrt() / 2.0;
        let p2 = p1 + Vec2::new(radius * circle_corner, radius * circle_corner);
        let p3 = p1 + Vec2::new(-radius * circle_corner, radius * circle_corner);
        let p4 = p1 + Vec2::new(0.0, radius * box_diag);
        let stroke = Stroke::new(0.0, color);
        //let pin_cone_shape = Shape::convex_polygon(vec![p2, p3, p4], color, stroke);
        let pin_cone_shape = Shape::convex_polygon(vec![p4, p3, p2], color, stroke);
        pin_rect.extend_with(p4);

        let pin_id = Id::new(42);
        let pin_response = ui.interact(pin_rect, pin_id, Sense::drag());
        let to_rect = self.to_screen.to();
        let to_rect_width = to_rect.max.x - to_rect.min.x;
        let relative_drag = pin_response.drag_delta().x / to_rect_width;
        if relative_drag != 0.0 {
            log::debug!("to_rect_width: {to_rect_width}");
            log::debug!("relative_drag: {relative_drag}");
            log::debug!("self.total_duration: {:?}", self.total_duration);
            log::debug!("bend_duration_param before: {bend_duration_param}");
            let delta = self.total_duration.as_secs_f64() * relative_drag.abs() as f64;
            log::debug!("delta: {delta}");
            if relative_drag > 0.0 {
                bend_duration_param -= delta;
            } else {
                bend_duration_param += delta;
            }
            log::debug!("bend_duration_param after: {bend_duration_param}");
            state.set_parameter(
                BendDuration,
                bend_duration_param.max(config.min).min(config.max),
            );
        }
        ui.painter().add(pin_cone_shape);
        ui.painter().add(pin_circle_shape);
        if pin_response.dragged() {
            let mut editor_params = state.editor_params.lock().unwrap();
            *editor_params = vec![BendDuration, HoldDuration];
        }
        if pin_response.double_clicked() {
            state.set_parameter_to_default(BendDuration)
        }
    }

    // TODO rename some variables and refactor to stay DRY
    pub fn draw_hold_pin(&self, state: &Arc<EditorState>, ui: &mut Ui) {
        let radius: f32 = 7.0;
        let hold_radius: f32 = 5.0;
        let box_diag = 2.0_f32.sqrt();
        let color = Color32::GRAY;

        let config = HoldDuration.get_config();
        let mut hold_duration_param = state.get_ui_parameter(HoldDuration);
        let bend_x = 1.0 - self.bend_duration.div_duration_f32(self.total_duration);
        let x = bend_x
            + Duration::from_nanos((1_000_000_000.0 * hold_duration_param) as u64)
                .div_duration_f32(self.total_duration);

        let bend_pin_center =
            self.to_screen * Pos2::new(bend_x, 0.0) + Vec2::new(0.0, -radius * box_diag);
        let mut p1 = self.to_screen * Pos2::new(x, 0.0);
        p1 += Vec2::new(0.0, -radius * box_diag);
        let pin_rect = Rect::from_center_size(p1, Vec2::new(hold_radius * 2.0, hold_radius * 2.0));
        let pin_circle_shape = Shape::circle_filled(p1, hold_radius, color);
        let stroke = Stroke::new(2.0, color);
        let connecting_line =
            Shape::line_segment([bend_pin_center + Vec2::new(radius, 0.0), p1], stroke);

        let pin_id = Id::new(43);
        let pin_response = ui.interact(pin_rect, pin_id, Sense::drag());
        let to_rect = self.to_screen.to();
        let to_rect_width = to_rect.max.x - to_rect.min.x;
        let relative_drag = pin_response.drag_delta().x / to_rect_width;
        if relative_drag != 0.0 {
            let delta = self.total_duration.as_secs_f64() * relative_drag.abs() as f64;
            log::debug!("delta: {delta}");
            if relative_drag > 0.0 {
                hold_duration_param += delta;
            } else {
                hold_duration_param -= delta;
            }
            state.set_parameter(
                HoldDuration,
                hold_duration_param.max(config.min).min(config.max),
            );
        }
        ui.painter().add(connecting_line);
        ui.painter().add(pin_circle_shape);
        if pin_response.dragged() {
            let mut editor_params = state.editor_params.lock().unwrap();
            *editor_params = vec![BendDuration, HoldDuration];
        }
        if pin_response.double_clicked() {
            state.set_parameter_to_default(HoldDuration)
        }
    }
}
