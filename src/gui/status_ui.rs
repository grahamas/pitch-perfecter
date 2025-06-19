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

    // --- Pitch detection threshold controls ---
    let mut power = app.pitch_power_threshold;
    let mut clarity = app.pitch_clarity_threshold;
    ui.horizontal(|ui| {
        ui.label("Power threshold:");
        if ui.add(egui::DragValue::new(&mut power).speed(0.1).range(0.0..=20.0)).changed() {
            // SAFETY: This is safe because AudioApp is always mutable in the caller
            unsafe { (app as *const _ as *mut crate::gui::audio_app::AudioApp).as_mut().unwrap().pitch_power_threshold = power; }
        }
        ui.label("Clarity threshold:");
        if ui.add(egui::DragValue::new(&mut clarity).speed(0.01).range(0.0..=1.0)).changed() {
            unsafe { (app as *const _ as *mut crate::gui::audio_app::AudioApp).as_mut().unwrap().pitch_clarity_threshold = clarity; }
        }
    });

    // --- Note display ---
    // Only show note if recording or playing
    if app.recording || app.playing {
        // Get the most recent pitch from the buffer (recorded_samples for recording, or from file for playback)
        let (samples, sample_rate) = if app.recording {
            let guard = app.recorded_samples.lock().unwrap();
            if guard.is_empty() { return; }
            (guard.clone(), app.cached_sample_rate.unwrap_or(44100))
        } else if !app.file_path.trim().is_empty() {
            if let Some((s, sr)) = crate::audio_helpers::load_audio_samples_and_rate(&app.file_path) {
                (s, sr)
            } else {
                return;
            }
        } else {
            return;
        };
        // Use a short window at the end for pitch detection
        let window_size = 1024;
        let step_size = 256;
        let threshold = 0.1;
        let len = samples.len();
        if len < window_size { return; }
        let start = len.saturating_sub(window_size);
        let frame = &samples[start..];
        let pitch = crate::pitch_detection::pitch_track_with_thresholds(
            frame,
            sample_rate as f32,
            window_size,
            step_size,
            app.pitch_power_threshold,
            app.pitch_clarity_threshold,
        ).last().copied().unwrap_or(0.0);
        let note = crate::music_notation::hz_to_note_name(pitch);
        ui.label(format!("Detected note: {} ({:.1} Hz)", note, pitch));
    }
}