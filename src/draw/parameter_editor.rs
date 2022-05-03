use std::sync::Arc;

use crate::EditorState;
use crate::state::{ParamConfig, GlissParam};

use egui::{Ui, Rect, Pos2, Response};


pub fn draw_parameter_editor(ui: &mut Ui, state: &Arc<EditorState>, params: Vec<GlissParam>, to_rect: Rect) -> Vec<Response> {
    let mut responses = vec![];
    let configs: Vec<ParamConfig> = params.iter().map(|param| param.get_config()).collect();
    for (idx, (config, param)) in configs.iter().zip(params.into_iter()).enumerate() {
        let i = idx as f32;
        let mut val = state.get_ui_parameter(param);
        ui.horizontal(|ui| {
            let text_rect = Rect::from_two_pos(
                Pos2::new(to_rect.min.x, to_rect.min.y + (i * 20.0) + 10.0),
                Pos2::new(to_rect.min.x + 100.0, to_rect.min.y + (i * 20.0) + 10.0),
            );
            let edit_rect = Rect::from_two_pos(
                Pos2::new(to_rect.min.x + 105.0, to_rect.min.y + (i * 20.0) + 7.5),
                Pos2::new(to_rect.min.x + 150.0, to_rect.min.y + (i * 20.0) + 7.5),
            );
            let pm_rect = Rect::from_two_pos(
                Pos2::new(to_rect.min.x + 155.0, to_rect.min.y + (i * 20.0) + 10.0),
                Pos2::new(to_rect.min.x + 175.0, to_rect.min.y + (i * 20.0) + 10.0),
            );
            let randomness_rect = Rect::from_two_pos(
                Pos2::new(to_rect.min.x + 230.0, to_rect.min.y + (i * 20.0) + 7.5),
                Pos2::new(to_rect.min.x + 180.0, to_rect.min.y + (i * 20.0) + 7.5),
            );
//            ui.put(
//                text_rect,
//                egui::Label::new(format!("{} = ", config.ui_display)),
//            );
            let edit_response = ui.put(
                edit_rect,
                egui::DragValue::new(&mut val)
                    .clamp_range(config.min..=config.max)
                    .speed(config.speed)
                    .fixed_decimals(2),
            );
            ui.put(
                pm_rect,
                egui::Label::new("+/-"),
            );
            ui.painter().text(
                Pos2::new(to_rect.min.x + 100.0, to_rect.min.y + (i * 20.0) + 15.0),
                egui::Align2::RIGHT_CENTER,
                format!("{} = ", config.ui_display),
                egui::TextStyle::Body,
                egui::Color32::GRAY,
                );
            // TODO add randomness variables
            let _edit_randomness_response = ui.put(
                randomness_rect,
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
    responses
}
