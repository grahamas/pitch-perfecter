use egui_plot::{Plot, Line, PlotPoints};
use super::audio_app::AudioApp;
use crate::audio_helpers;

/// Waveform plot UI: shows the waveform of the loaded audio file or live recording
pub fn waveform_ui(app: &AudioApp, ui: &mut egui::Ui) {
    let sample_rate = 44100.0;
    let five_sec_samples = (sample_rate * 5.0) as usize;
    let samples = if app.recording {
        let guard = app.recorded_samples.lock().unwrap();
        if guard.is_empty() { None } else { Some(guard.clone()) }
    } else if app.file_path.trim().is_empty() {
        None
    } else {
        audio_helpers::load_audio_samples(&app.file_path)
    };
    let mut padded_samples = vec![0.0; five_sec_samples];
    if let Some(s) = samples {
        let len = s.len().min(five_sec_samples);
        padded_samples[(five_sec_samples - len)..].copy_from_slice(&s[s.len().saturating_sub(five_sec_samples)..]);
    }
    // X axis: just use sample index (no label, no custom range, no ticks)
    let points: PlotPoints = padded_samples.iter().enumerate().map(|(i, &s)| [i as f64, s as f64]).collect();
    let line = Line::new(points);
    // Y axis: always include -1 and 1, but expand if needed
    let min_y = padded_samples.iter().cloned().fold(0.0/0.0, f32::min).min(-1.0);
    let max_y = padded_samples.iter().cloned().fold(0.0/0.0, f32::max).max(1.0);
    Plot::new("waveform")
        .height(150.0)
        .include_y(min_y as f64)
        .include_y(max_y as f64)
        .show_axes(false)
        .show_grid(false)
        .show(ui, |plot_ui| {
            plot_ui.line(line);
        });
}