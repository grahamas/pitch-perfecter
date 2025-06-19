use super::audio_app::AudioApp;
use crate::audio_helpers;
use crate::signal_processing;
use egui;

/// Spectrogram plot UI: shows the log-magnitude spectrogram of the loaded audio file with a viridis-like color map and moving peak overlay
pub fn spectrogram_ui(app: &mut AudioApp, ui: &mut egui::Ui) {
    // Only show for loaded files, not live recording
    if app.recording || app.file_path.trim().is_empty() {
        return;
    }
    if let Some((samples, sample_rate)) = audio_helpers::load_audio_samples_and_rate(&app.file_path) {
        let window_size = 1024;
        let step_size = 256;
        let window_sec = window_size as f32 / sample_rate as f32;
        let step_sec = step_size as f32 / sample_rate as f32;
        let mut spectrogram = signal_processing::compute_log_spectrogram(&samples, window_size, step_size);
        if spectrogram.is_empty() { return; }
        // Drop the upper (unreal) half of the spectrum (keep only positive frequencies)
        let n_freq = spectrogram[0].len() / 2;
        let n_time = spectrogram.len();
        for spec in &mut spectrogram {
            spec.truncate(n_freq);
        }
        // Find min/max for normalization
        let min_val = spectrogram.iter().flatten().cloned().fold(f32::INFINITY, f32::min);
        let max_val = spectrogram.iter().flatten().cloned().fold(f32::NEG_INFINITY, f32::max);
        // Viridis colormap (simple approximation)
        fn viridis(t: f32) -> [u8; 3] {
            let t = t.clamp(0.0, 1.0);
            let r = (34.0 + 222.0 * t + 0.0 * t * t) as u8;
            let g = (39.0 + 201.0 * t - 39.0 * t * t) as u8;
            let b = (99.0 + 55.0 * t + 101.0 * t * t) as u8;
            [r, g, b]
        }
        // Convert to a flat Vec<u8> for egui::ColorImage
        let mut pixels = Vec::with_capacity(n_freq * n_time * 4);
        for freq_bin in 0..n_freq {
            for t in 0..n_time {
                let v = spectrogram[t][freq_bin];
                let norm = if max_val > min_val {
                    (v - min_val) / (max_val - min_val)
                } else {
                    0.0
                };
                let [r, g, b] = viridis(norm);
                pixels.extend_from_slice(&[r, g, b, 255]);
            }
        }
        // Draw the spectrogram image
        let image = egui::ColorImage::from_rgba_unmultiplied([
            n_time, n_freq
        ], &pixels);
        let texture = ui.ctx().load_texture(
            "spectrogram",
            image,
            egui::TextureOptions::NEAREST,
        );
        let time_label = format!("Time (s), step {:.3}s", step_sec);
        let freq_label = format!("Frequency (Hz), window {:.3}s", window_sec);
        ui.label(&time_label);
        // Overlay: moving peak trace (yellow line)
        use egui::epaint::{Color32, Shape, Stroke};
        let peak_indices = signal_processing::detect_moving_peak(&spectrogram);
        let mut peak_points = Vec::with_capacity(n_time);
        for (t, &freq_bin) in peak_indices.iter().enumerate() {
            let x = t as f32 / n_time as f32; // normalized time
            let y = freq_bin as f32 / n_freq as f32; // normalized freq (flip y for image coordinates)
            peak_points.push(egui::pos2(x, y));
        }
        // Draw image and overlay in a custom painter
        let (response, painter) = ui.allocate_painter(egui::vec2(600.0, 400.0), egui::Sense::hover());
        let rect = response.rect;
        // Draw the spectrogram image
        painter.image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        // Draw the moving peak overlay as unconnected yellow points (circles)
        if peak_points.len() > 1 {
            for p in &peak_points {
                // Map normalized coordinates to painter rect
                let mapped = egui::pos2(rect.left() + p.x * rect.width(), rect.top() + (p.y) * rect.height());
                painter.add(Shape::circle_filled(mapped, 3.0, Color32::YELLOW));
            }
        }
        // Draw a vertical red line to indicate playback time
        if app.playing {
            // Align red line with spectrogram time bins (not raw samples), using cached metadata
            if let (Some(_sample_rate), Some(duration_sec)) = (app.cached_sample_rate, app.cached_duration_sec) {
                let mut elapsed_sec = 0.0;
                if let Some(start) = app.playback_start {
                    elapsed_sec = std::time::Instant::now().duration_since(start).as_secs_f32();
                }
                // Clamp to total duration
                let clamped_time = elapsed_sec.min(duration_sec);
                // Map elapsed time to normalized spectrogram X (time bin)
                let progress = (clamped_time / duration_sec).min(1.0).max(0.0);
                let x = rect.left() + progress * rect.width();
                painter.add(Shape::line_segment([
                    egui::pos2(x, rect.top()),
                    egui::pos2(x, rect.bottom())
                ], Stroke::new(2.0, Color32::RED)));
            }
        }
        ui.label(&freq_label);
    }
}