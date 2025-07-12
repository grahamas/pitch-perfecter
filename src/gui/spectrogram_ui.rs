use super::audio_app::AudioApp;
use crate::signal_processing::SpectrogramConfig;
use colorous::VIRIDIS;
use egui;

const MAX_TEXTURE: usize = 8192; // Maximum texture size for egui

/// Spectrogram plot UI: shows the log-magnitude spectrogram of the loaded audio file with a viridis-like color map and moving peak overlay
pub fn spectrogram_ui(app: &mut AudioApp, ui: &mut egui::Ui) {
    // Only show for loaded files, not live recording
    if app.recording || app.file_path.trim().is_empty() {
        return;
    }

    // Get loaded spectrogram if it exists
    let Some(spectrogram) = &app.loaded_spectrogram else {
        return;
    };
    let SpectrogramConfig { window_size, step_size } = app.spectrogram_config;


    // Downsample if necessary
    let n_time = spectrogram.n_time_steps();
    let n_freq = spectrogram.n_freq_bins();
    let (n_time, n_freq, _downsample_factor, spectra) = if n_time > MAX_TEXTURE {
        downsample_spectrogram_if_needed(&spectrogram.spectra, MAX_TEXTURE)
    } else {
        (n_time, n_freq, 1, spectrogram.spectra.clone())
    };

    // Find min/max for normalization
    let min_val = spectra.iter().flatten().cloned().fold(f32::INFINITY, f32::min);
    let max_val = spectra.iter().flatten().cloned().fold(f32::NEG_INFINITY, f32::max);

    // Convert to a flat Vec<u8> for egui::ColorImage
    let pixels = spectrogram_to_pixels(&spectrogram.spectra, n_time, n_freq, min_val, max_val);

    // Draw the spectrogram image
    let image = egui::ColorImage::from_rgba_unmultiplied([
        n_time, n_freq
    ], &pixels);
    let texture = ui.ctx().load_texture(
        "spectrogram",
        image,
        egui::TextureOptions::NEAREST,
    );
    let Some(audio) = &app.loaded_audio else {
        ui.label("No audio loaded");
        return;
    };
    let sample_rate = audio.sample_rate() as usize;
    let step_sec = step_size as f32 / sample_rate as f32;
    let window_sec = window_size as f32 / sample_rate as f32;
    let time_label = format!("Time (s), step {:.3}s", step_sec);
    let freq_label = format!("Frequency (Hz), window {:.3}s", window_sec);
    ui.label(&time_label);
    // Button to toggle peak overlay
    if ui.button(if app.show_peak_overlay { "Hide Peak Overlay" } else { "Show Peak Overlay" }).clicked() {
        app.show_peak_overlay = !app.show_peak_overlay;
    }
    // Toggle for playback signal cleaning above spectrogram
    let label = if app.clean_playback_signal { "Disable Signal Cleaning for Playback & Spectrogram" } else { "Enable Signal Cleaning for Playback & Spectrogram" };
    if ui.button(label).clicked() {
        app.clean_playback_signal = !app.clean_playback_signal;
    }
    // Overlay: moving peak trace (yellow line)
    use egui::epaint::{Color32, Shape, Stroke};
    // Calculate pitch-based overlay (linear frequency mapping, no vertical flip)
    let pitch_overlay_indices = crate::gui::peak_overlay::get_peak_indices(audio.samples(), app.track_pitch_config, sample_rate, n_freq-1);
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
    if app.show_peak_overlay {
        crate::gui::peak_overlay::draw_peak_overlay(&pitch_overlay_indices, rect, &painter, n_freq, n_time);
    }
    // Draw a vertical red line to indicate playback time
    if app.playing {
        let duration_sec = audio.duration();
        // Align red line with spectrogram time bins (not raw samples), using cached metadata
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
    ui.label(&freq_label);
}

/// Downsample spectrogram in time if needed for texture size limits
/// FIXME might also need to downsample in frequency if n_freq is too large
/// Returns (n_time, downsample_factor, downsampled_spectrogram)
fn downsample_spectrogram_if_needed(
    spectrogram: &[Vec<f32>],
    max_texture: usize
) -> (usize, usize, usize, Vec<Vec<f32>>) {
    let n_time = spectrogram.len();
    let n_freq = spectrogram.first().map_or(0, |s| s.len());
    let factor = (n_time as f32 / max_texture as f32).ceil() as usize;
    let mut downsampled = Vec::with_capacity(max_texture);
    for chunk in spectrogram.chunks(factor) {
        // Average each chunk
        let mut avg = vec![0.0; n_freq];
        for spec in chunk {
            for (i, &v) in spec.iter().enumerate() {
                avg[i] += v;
            }
        }
        let len = chunk.len() as f32;
        for v in &mut avg {
            *v /= len;
        }
        downsampled.push(avg);
    }
    (downsampled.len(), n_freq, factor, downsampled)
}

/// Convert a 2D spectrogram to a flat Vec<u8> (RGBA) using the VIRIDIS colormap
fn spectrogram_to_pixels(
    spectrogram: &[Vec<f32>],
    n_time: usize,
    n_freq: usize,
    min_val: f32,
    max_val: f32,
) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(n_freq * n_time * 4);
    for freq_bin in (0..n_freq).rev() {
        for t in 0..n_time {
            let v = spectrogram[t][freq_bin];
            let norm = if max_val > min_val {
                (v - min_val) / (max_val - min_val)
            } else {
                0.0
            };
            let color = VIRIDIS.eval_continuous(norm as f64);
            pixels.extend_from_slice(&[color.r, color.g, color.b, 255]);
        }
    }
    pixels
}