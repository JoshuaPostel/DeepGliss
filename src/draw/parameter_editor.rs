use std::sync::Arc;

use crate::EditorState;
use crate::state::{ParamConfig, GlissParam};

use egui::{Ui, Rect, Pos2, Response};


pub fn draw_parameter_editor(ui: &mut Ui, state: &Arc<EditorState>, params: Vec<GlissParam>, to_rect: Rect) -> Vec<Response> {
    let mut responses = vec![];
    let configs: Vec<ParamConfig> = params.iter().map(|param| param.get_config()).collect();
    let mut text_max_x_location: f32 = 0.0;
    for (idx, config) in configs.iter().enumerate() {
        let i = idx as f32;
        let location = ui.painter().text(
            Pos2::new(to_rect.min.x, to_rect.min.y + (i * 20.0) + 16.0),
            egui::Align2::LEFT_CENTER,
            config.ui_name,
            egui::TextStyle::Body,
            egui::Color32::GRAY,
        );
        text_max_x_location = text_max_x_location.max(location.max.x);
    }
    for (idx, (config, param)) in configs.iter().zip(params.into_iter()).enumerate() {
        let i = idx as f32;
        let mut val = state.get_ui_parameter(param);
        ui.horizontal(|ui| {
            let edit_rect = Rect::from_two_pos(
                Pos2::new(text_max_x_location + 5.0, to_rect.min.y + (i * 20.0) + 7.5),
                Pos2::new(text_max_x_location  + 55.0, to_rect.min.y + (i * 20.0) + 7.5),
            );
            let pm_rect = Rect::from_two_pos(
                Pos2::new(text_max_x_location + 60.0, to_rect.min.y + (i * 20.0) + 10.0),
                Pos2::new(text_max_x_location + 80.0, to_rect.min.y + (i * 20.0) + 10.0),
            );
            let randomness_rect = Rect::from_two_pos(
                Pos2::new(text_max_x_location + 85.0, to_rect.min.y + (i * 20.0) + 7.5),
                Pos2::new(text_max_x_location + 135.0, to_rect.min.y + (i * 20.0) + 7.5),
            );
            let edit_response = ui.put(
                edit_rect,
                egui::DragValue::new(&mut val)
                    .clamp_range(config.min..=config.max)
                    .speed(config.speed)
                    .fixed_decimals(2),
            );
            if let Some(randomness_param) =  param.get_randomness_param() {
                ui.put(
                    pm_rect,
                    egui::Label::new("+/-"),
                );
                let mut val = state.get_ui_parameter(randomness_param);
                let randomness_config = randomness_param.get_config();
                let edit_randomness_response = ui.put(
                    randomness_rect,
                    egui::DragValue::new(&mut val)
                        .clamp_range(randomness_config.min..=randomness_config.max)
                        .speed(randomness_config.speed)
                        .fixed_decimals(2),
                );
                if edit_randomness_response.changed() {
                    state.set_parameter(randomness_param, val);
                };
                responses.push(edit_randomness_response);
            }
            // TODO add randomness variables
            if edit_response.changed() {
                state.set_parameter(param, val);
            };
            responses.push(edit_response);
        });
    }
    responses
}
