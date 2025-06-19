use super::audio_app::AudioApp;
use crate::RecordingControl;
use egui;
use std::sync::Arc;
use chrono;

/// Record controls UI: Record and Stop Recording buttons
pub fn record_controls_ui(app: &mut AudioApp, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Record button
            if ui.add_enabled(!app.recording, egui::Button::new(egui::RichText::new("⏺ Record").size(18.0))).clicked() {
                app.recording = true;
                // If no file is selected, create a unique file in audio/
                if app.file_path.trim().is_empty() {
                    let audio_dir = std::env::current_dir()
                        .map(|p| p.join("audio"))
                        .unwrap_or_else(|_| std::path::PathBuf::from("audio"));
                    let _ = std::fs::create_dir_all(&audio_dir);
                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                    let unique_name = format!("recording_{}.wav", timestamp);
                    let unique_path = audio_dir.join(unique_name);
                    app.file_path = unique_path.to_string_lossy().to_string();
                }
                let path = app.file_path.clone();
                let control = RecordingControl::new();
                app.recording_control = Some(control.clone());
                app.recorded_samples.lock().unwrap().clear();
                let live_buffer = app.recorded_samples.clone();
                std::thread::spawn(move || {
                    crate::audio::record_audio_with_control_and_buffer(&path, control, live_buffer);
                });
            }
            // Stop Recording button
            if ui.add_enabled(app.recording, egui::Button::new(egui::RichText::new("⏹ Stop Recording").size(18.0))).clicked() {
                if let Some(control) = &app.recording_control {
                    control.stop();
                }
                app.recording = false;
                app.recording_control = None;
            }
        });
    }