use super::audio_app::AudioApp;
use crate::PlaybackControl;
use egui;
use std::sync::mpsc;

/// Playback controls UI: Play and Stop Playback buttons
pub fn playback_controls_ui(app: &mut AudioApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        // Play button
        if ui.add_enabled(!app.playing, egui::Button::new(egui::RichText::new("▶ Play").size(18.0))).clicked() {
            app.playing = true;
            app.playback_start = Some(std::time::Instant::now()); // Set playback_start only once here
            let path = app.file_path.clone();
            let control = PlaybackControl::new();
            app.playback_control = Some(control.clone());
            let (tx, rx) = mpsc::channel();
            app.playback_done_rx = Some(rx);
            std::thread::spawn(move || {
                crate::audio::play_audio_with_control_and_notify(&path, control, tx);
            });
        }
        // Stop Playback button
        if ui.add_enabled(app.playing, egui::Button::new(egui::RichText::new("⏹ Stop Playback").size(18.0))).clicked() {
            if let Some(control) = &app.playback_control {
                control.stop();
            }
            app.playing = false;
            app.playback_control = None;
            app.playback_done_rx = None;
            app.playback_start = None; // Reset playback_start on stop
        }
    });
}