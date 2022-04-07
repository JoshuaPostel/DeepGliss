use crate::draw::theme::GLISS_THEME;
use crate::midi::mapper::ChordMap;
use crate::midi::paths::Path;
use crate::state::GlissParam::{
    BendMapping, BendPath, BendPathAmplitude, BendPathPeriods, BendPathSCurveSharpness,
};
use crate::state::{EditorState, GlissParam, ParamConfig};

use std::sync::Arc;

use egui::{Color32, Pos2, Rect, SelectableLabel, Shape, Stroke, Ui};

pub fn draw_linesegments(
    ui: &mut Ui,
    lines: Vec<[Pos2; 2]>,
    //points: [[Pos2; 2]; 3],
    from_rect: Rect,
    to_rect: Rect,
    selected: bool,
) {
    for (i, line) in lines.into_iter().enumerate() {
        draw_linesegment(
            ui,
            line,
            from_rect,
            to_rect,
            selected,
            GLISS_THEME.channel_colors[i % 3],
        );
    }
}

pub fn draw_linesegment(
    ui: &mut Ui,
    points: [Pos2; 2],
    from_rect: Rect,
    to_rect: Rect,
    selected: bool,
    color: Color32,
) {
    let to_screen = emath::RectTransform::from_to(from_rect, to_rect);
    let color = if selected { color } else { Color32::GRAY };
    let p1 = to_screen * points[0];
    let p2 = to_screen * points[1];
    let stroke = Stroke::new(2.0, color);
    let line = Shape::line_segment([p1, p2], stroke);
    ui.painter().add(line);
}

pub fn draw_line(
    ui: &mut Ui,
    mut points: Vec<Pos2>,
    from_rect: Rect,
    to_rect: Rect,
    selected: bool,
    color: Color32,
) {
    let to_screen = emath::RectTransform::from_to(from_rect, to_rect);
    let color = if selected { color } else { Color32::GRAY };
    let stroke = Stroke::new(2.0, color);
    let line = Shape::line(
        points.iter_mut().map(|pos| to_screen * *pos).collect(),
        stroke,
    );
    ui.painter().add(line);
}

pub fn draw_notes(ui: &mut Ui, points: Vec<&Pos2>, from_rect: Rect, to_rect: Rect, selected: bool) {
    let to_screen = emath::RectTransform::from_to(from_rect, to_rect);
    let radius = 2.5;
    let color = if selected {
        Color32::WHITE
    } else {
        Color32::GRAY
    };
    let note_stroke = Stroke::new(1.0, color);
    for point in points {
        ui.painter().add(Shape::circle_stroke(
            to_screen * *point,
            radius,
            note_stroke,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_map_button(
    ui: &mut Ui,
    text: &str,
    lines: Vec<[Pos2; 2]>,
    notes: Vec<&Pos2>,
    from_rect: Rect,
    to_rect: Rect,
    state: &Arc<EditorState>,
    map_variant: ChordMap,
    selected: bool,
) {
    let label = SelectableLabel::new(false, text);
    if ui.put(to_rect, label).clicked() {
        state.set_parameter(BendMapping, map_variant.as_f64());
    }
    draw_linesegments(ui, lines, from_rect, to_rect, selected);
    draw_notes(ui, notes, from_rect, to_rect, selected);
}

#[allow(clippy::too_many_arguments)]
pub fn draw_path_button(
    ui: &mut Ui,
    state: &Arc<EditorState>,
    from_rect: Rect,
    to_rect: Rect,
    curve: Vec<Pos2>,
    notes: Vec<&Pos2>,
    selected: bool,
    path_variant: Path,
    params: Vec<GlissParam>,
    keyboard_focus: &mut Option<Path>,
) {
    //let n_points = 50;
    let configs: Vec<ParamConfig> = params.iter().map(|param| param.get_config()).collect();
    let button = egui::Button::new("")
        .fill(Color32::TRANSPARENT)
        .sense(egui::Sense::drag());
    let response = ui.put(to_rect, button);
    if response.dragged() {
        state.set_parameter(BendPath, path_variant.as_f64());
        for (config, param) in configs.iter().zip(params.iter()) {
            let dragged = match param {
                BendPathAmplitude => config.speed * response.drag_delta().y as f64,
                BendPathPeriods => config.speed * response.drag_delta().x as f64,
                BendPathSCurveSharpness => config.speed * response.drag_delta().y as f64,
                _ => unimplemented!(),
            };
            let val = state.get_ui_parameter(*param);
            let new_val = (val + dragged).min(config.max).max(config.min);
            state.set_parameter(*param, new_val);
        }
        // TODO figure out how to "was_dragged_recently"
        // if dragged != 0.0 {
        //    *keyboard_focus = Some(path_variant)
        // } else {
        //    *keyboard_focus = None
        // }
    }
    if response.double_clicked() {
        for (config, param) in configs.iter().zip(params.iter()) {
            state.set_parameter(*param, config.default)
        }
    }
    if response.secondary_clicked() {
        match *keyboard_focus {
            Some(x) if x == path_variant => *keyboard_focus = None,
            _ => *keyboard_focus = Some(path_variant),
        }
    }
    if *keyboard_focus == Some(path_variant) {
        let mut responses = vec![response];
        for (idx, (config, param)) in configs.iter().zip(params.into_iter()).enumerate() {
            let i = idx as f32;
            let mut val = state.get_ui_parameter(param);
            // TODO design
            // which looks better?
            // * prefixed DragValue
            // * text then DragValue
            ui.horizontal(|ui| {
                // * prefixed DragValue
                // let rect = Rect::from_two_pos(
                //     Pos2::new(to_rect.min.x, to_rect.max.y + (i * 25.0) + 10.0),
                //     Pos2::new(to_rect.max.x, to_rect.max.y + (i * 25.0) + 10.0),
                // );
                // let edit_response = ui.put(
                //     rect,
                //     egui::DragValue::new(&mut val)
                //         .clamp_range(config.min..=config.max)
                //         .speed(config.speed)
                //         .prefix(format!("{}: ", config.ui_display))
                //         .fixed_decimals(2),
                // );

                // * text then DragValue
                let text_rect = Rect::from_two_pos(
                    Pos2::new(to_rect.min.x, to_rect.max.y + (i * 25.0) + 10.0),
                    Pos2::new(to_rect.min.x + 20.0, to_rect.max.y + (i * 25.0) + 10.0),
                );
                let edit_rect = Rect::from_two_pos(
                    Pos2::new(to_rect.min.x + 25.0, to_rect.max.y + (i * 25.0) + 7.5),
                    Pos2::new(to_rect.max.x, to_rect.max.y + (i * 25.0) + 7.5),
                );
                ui.put(
                    text_rect,
                    egui::Label::new(format!("{}: ", config.ui_display)),
                );
                let edit_response = ui.put(
                    edit_rect,
                    egui::DragValue::new(&mut val)
                        .clamp_range(config.min..=config.max)
                        .speed(config.speed)
                        .fixed_decimals(2),
                );
                if edit_response.changed() {
                    state.set_parameter(param, val);
                };
                responses.push(edit_response);
            });
        }
        if responses
            .iter()
            .all(|response| response.clicked_elsewhere())
        {
            *keyboard_focus = None;
        }
    }
    draw_line(
        ui,
        curve,
        from_rect,
        to_rect,
        selected,
        GLISS_THEME.channel_colors[0],
    );
    draw_notes(ui, notes, from_rect, to_rect, selected);
}
