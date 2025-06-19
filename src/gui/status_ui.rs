use super::audio_app::AudioApp;
use egui;

/// Status display UI: shows current state (Recording, Playing, Idle)
pub fn status_ui(app: &AudioApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new(
        if app.recording {
            "Recording..."
        } else if app.playing {
            "Playing..."
        } else {
            "Idle"
        }
    ).size(16.0).italics());
}