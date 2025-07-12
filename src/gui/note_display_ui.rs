use super::audio_app::AudioApp;
use crate::signal_cleaning;
use egui;

/// Returns (note_name, pitch_hz) if a note is detected, otherwise None
pub fn get_detected_note(app: &AudioApp) -> Option<(String, f32)> {
    // Get the most recent frame from the appropriate source
    let (samples, sample_rate) = if app.recording {
        let guard = app.recorded_samples.lock().unwrap();
        if guard.is_empty() {
            return None;
        }
        (guard.clone(), app.recording_sample_rate.unwrap_or(44100))
    } else if let Some(audio) = &app.loaded_audio {
        (audio.samples().to_vec(), audio.sample_rate())
    } else {
        return None;
    };
    let config = app.track_pitch_config;
    let window_size = config.window_size;
    let len = samples.len();
    if len < window_size {
        return None;
    }
    let start = len.saturating_sub(window_size);
    let frame = &samples[start..];
    // Optionally clean the signal
    let frame = if app.show_signal_cleaning {
        signal_cleaning::clean_signal_for_pitch(
            frame,
            sample_rate as f32,
            None,
            None,
        )
    } else {
        frame.to_vec()
    };
    // Run pitch detection
    let pitches = crate::track_pitch::track_pitch(
        &frame,
        config,
        sample_rate as usize,
    );
    let pitch = pitches.last().copied().unwrap_or(0.0) as f32;
    let note = crate::music_notation::hz_to_note_name(pitch);
    Some((note, pitch))
}

/// Prominent detected note display UI element
pub fn note_display_ui(app: &AudioApp, ui: &mut egui::Ui) {
    use egui::{Color32, FontId, Align2, Pos2, Stroke, vec2, Layout};
    let box_bg = Color32::from_rgb(70, 110, 150); // steely blue
    let box_border = Color32::from_rgb(40, 60, 90); // darker steely blue
    let box_size = vec2(140.0, 110.0);
    let corner_radius = 14.0;
    ui.vertical_centered(|ui| {
        ui.allocate_ui_with_layout(box_size, Layout::top_down_justified(egui::Align::Center), |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();
            painter.rect_filled(rect, corner_radius, box_bg);
            painter.rect_stroke(rect, corner_radius, Stroke::new(3.0, box_border), egui::StrokeKind::Outside);
            let center = rect.center();
            if let Some((note, pitch)) = get_detected_note(app) {
                let note_text = note;
                let freq_text = format!("{:.1} Hz", pitch);
                let note_font = FontId::proportional(56.0);
                painter.text(
                    center,
                    Align2::CENTER_CENTER,
                    &note_text,
                    note_font,
                    Color32::BLACK,
                );
                let freq_font = FontId::proportional(22.0);
                let freq_offset = 32.0;
                let freq_pos = Pos2::new(center.x, center.y + freq_offset);
                painter.text(
                    freq_pos,
                    Align2::CENTER_TOP,
                    &freq_text,
                    freq_font,
                    Color32::BLACK,
                );
            }
        });
        // Force vertical space after the note box
        ui.add_space(box_size.y);
    });
}

/// Pitch tracker controls UI element (window size, step size, power, clarity)
pub fn pitch_tracker_controls_ui(app: &mut crate::gui::audio_app::AudioApp, ui: &mut egui::Ui) {
    let mut window_size = app.track_pitch_config.window_size as u32;
    let mut step_size = app.track_pitch_config.step_size as u32;
    let mut power = app.track_pitch_config.power_threshold;
    let mut clarity = app.track_pitch_config.clarity_threshold;
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label("Window size:");
            if ui.add(egui::DragValue::new(&mut window_size).speed(64).range(128..=8192)).changed() {
                app.track_pitch_config.window_size = window_size as usize;
            }
            ui.label("Step size:");
            if ui.add(egui::DragValue::new(&mut step_size).speed(16).range(16..=4096)).changed() {
                app.track_pitch_config.step_size = step_size as usize;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Power threshold:");
            if ui.add(egui::DragValue::new(&mut power).speed(0.1).range(0.0..=20.0)).changed() {
                app.track_pitch_config.power_threshold = power;
            }
            ui.label("Clarity threshold:");
            if ui.add(egui::DragValue::new(&mut clarity).speed(0.01).range(0.0..=1.0)).changed() {
                app.track_pitch_config.clarity_threshold = clarity;
            }
        });
    });
}

/// Signal cleaning toggle UI element
pub fn signal_cleaning_toggle_ui(app: &mut crate::gui::audio_app::AudioApp, ui: &mut egui::Ui) {
    let label = if app.show_signal_cleaning { "Disable Signal Cleaning" } else { "Enable Signal Cleaning" };
    if ui.button(label).clicked() {
        app.show_signal_cleaning = !app.show_signal_cleaning;
    }
}
