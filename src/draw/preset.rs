use std::sync::Arc;

use egui::Ui;

use crate::EditorState;

use anyhow::{Result, Context};

pub fn draw_save_preset(ui: &mut Ui, state: &Arc<EditorState>) -> Result<()> {
    let response = ui.add(egui::widgets::Button::new("Save Preset"));
    if response.clicked() {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H:%M:%S");
        let dir = dirs::home_dir().context("home directory not detected")?;
        let tmp_dir = dir.join("tmp").join(format!("{timestamp}_DeepGliss.preset"));
        let parameter_file = std::fs::File::create(tmp_dir)?;
        state.save_parameters(parameter_file)?;
    }
    Ok(())
}


pub fn draw_load_preset(ui: &mut Ui, state: &Arc<EditorState>) -> Result<()> {

    let f = |ui: &mut egui::Ui| {
        let mut selected = String::new();
        let log_folder = dirs::home_dir().context("home directory not detected")?;
        let log_folder = log_folder.join("tmp");
        let mut paths = vec![];
        for element in std::fs::read_dir(log_folder)? {
            let path = element?.path();
            if let Some(extension) = path.extension() {
                if extension == "preset" {
                    paths.push(path);
                }
            }
        }
        for path in paths {
            ui.selectable_value(&mut selected, path.display().to_string(), path.display());
        }


        Ok(selected)
    };
    if let Some(response) = egui::ComboBox::from_label("")
        .width(20.0)
        .selected_text("Load Preset")
        .show_ui(ui, f)
        .inner {
            match response {
                Ok(response) => {
                    if !response.is_empty() {
                        let file = std::fs::File::open(response)?;
                        state.load_parameters(file)?;
                    }
                    return Ok(());
                },
                Err(e) => return Err(e),
            }
    }
    Ok(())
}
