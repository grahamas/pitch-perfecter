use super::audio_app::AudioApp;
use egui::{self, Ui};
use rfd::FileDialog;
use std::env;

/// File selection UI: open/save dialogs and file path entry
pub fn file_selector_ui(app: &mut AudioApp, ui: &mut Ui) {
    let audio_dir = env::current_dir()
        .map(|p| p.join("audio"))
        .unwrap_or_else(|_| std::path::PathBuf::from("audio"));
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("File path:").size(18.0));
        ui.add_sized([350.0, 28.0], egui::TextEdit::singleline(&mut app.file_path));
        // Open file dialog
        if ui.button("📂").on_hover_text("Select file to open...").clicked() {
            if let Some(path) = FileDialog::new().set_directory(&audio_dir).pick_file() {
                if let Some(path_str) = path.to_str() {
                    app.file_path = path_str.to_owned();
                    app.update_audio_metadata(); // Update cache on file select
                }
            }
        }
        // Save as file dialog
        if ui.button("💾").on_hover_text("Select file to save as...").clicked() {
            if let Some(path) = FileDialog::new().set_directory(&audio_dir).set_file_name(&app.file_path).save_file() {
                if let Some(path_str) = path.to_str() {
                    app.file_path = path_str.to_owned();
                    app.update_audio_metadata(); // Update cache on file save
                }
            }
        }
    });
}
