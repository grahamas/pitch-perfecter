use super::audio_app::AudioApp;
use egui;

/// Returns (note_name, pitch_hz) if a note is detected, otherwise None
pub fn get_detected_note(app: &AudioApp) -> Option<(String, f32)> {
    if app.recording || app.playing {
        let (samples, sample_rate) = if app.recording {
            let guard = app.recorded_samples.lock().unwrap();
            if guard.is_empty() { return None; }
            (guard.clone(), app.recording_sample_rate.unwrap_or(44100))
        } else if !app.file_path.trim().is_empty() {
            if let Ok((s, sr)) = crate::audio::load_audio_samples_and_rate(&app.file_path) {
                (s, sr)
            } else {
                return None;
            }
        } else {
            return None;
        };
        let config = app.track_pitch_config;
        let len = samples.len();
        if len < config.window_size { return None; }
        let start = len.saturating_sub(config.window_size);
        let frame = &samples[start..];
        let pitches = crate::track_pitch::track_pitch(
            frame,
            config,
            sample_rate as usize,
        );
        let pitch = pitches.last().copied().unwrap_or(0.0) as f32;
        let note = crate::music_notation::hz_to_note_name(pitch);
        Some((note, pitch))
    } else {
        None
    }
}

/// Status display UI: shows current state (Recording, Playing, Idle)
pub fn status_ui(app: &AudioApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(
            if app.recording {
                "Recording..."
            } else if app.playing {
                "Playing..."
            } else {
                "Idle"
            }
        ).size(16.0).italics());
    });
}