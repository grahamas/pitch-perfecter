use egui_plot::{Plot, Line, PlotPoints};
use super::audio_app::AudioApp;

const MAX_POINTS: usize = 2048; // Maximum points to display in waveform

/// Main waveform plot UI: dispatches to live or loaded waveform plotting
pub fn waveform_ui(app: &AudioApp, ui: &mut egui::Ui) {
    // FIXME: should just compute the waveform to plot in the subfunctions
    //        the plotting is the same either way so should be done here
    if app.recording {
        plot_live_waveform(app, ui);
    } else if !app.file_path.trim().is_empty() {
        plot_loaded_waveform(app, ui);
    } else {
        // No data to show
        ui.label("No audio loaded or recording.");
    }
}

/// Plot the waveform for a live recording (shows only the most recent 5 seconds)
pub fn plot_live_waveform(app: &AudioApp, ui: &mut egui::Ui) {
    // Maximum number of points to display (for performance)
    let max_points = 2048;
    // Get the current live buffer and sample rate
    let guard = app.recorded_samples.lock().unwrap();
    let sample_rate = app.recording_sample_rate.unwrap_or(44100);
    if guard.is_empty() {
        ui.label("No live audio data.");
        return;
    }
    let samples = guard.clone();
    // Number of samples to show (last 5 seconds)
    let five_sec_samples = (sample_rate as usize * 5).max(1);
    // Take the last 5 seconds of samples (or all if shorter)
    let window = if samples.len() > five_sec_samples {
        &samples[samples.len() - five_sec_samples..]
    } else {
        &samples[..]
    };
    // Downsample the window to max_points using min/max snapshotting
    let downsampled = downsample_for_waveform(window, max_points);
    
    // Convert to PlotPoints for egui_plot with time-based X-axis
    // Map the downsampled data to a 5-second time window
    let window_duration = window.len() as f64 / sample_rate as f64;
    let points: PlotPoints = downsampled.iter().enumerate().map(|(i, &s)| {
        // Map sample index to time within the actual window duration
        let time_in_window = (i as f64 / downsampled.len() as f64) * window_duration;
        // Offset so the window always appears at the end of the 5-second display
        let time_sec = 5.0 - window_duration + time_in_window;
        [time_sec.max(0.0), s as f64]
    }).collect();
    let line = Line::new(points);
    // Y axis: always include -1 and 1, but expand if needed
    let min_y = downsampled.iter().cloned().fold(0.0/0.0, f32::min).min(-1.0);
    let max_y = downsampled.iter().cloned().fold(0.0/0.0, f32::max).max(1.0);
    Plot::new("waveform_live")
        .height(200.0)  // Increased height for better visibility
        .include_y(min_y as f64)
        .include_y(max_y as f64)
        .include_x(0.0)
        .include_x(5.0) // Always show 5 seconds
        .show_axes(true)
        .show_grid(true)
        .allow_zoom(false)
        .allow_scroll(false)
        .show(ui, |plot_ui| {
            plot_ui.line(line);
        });
}

/// Plot the waveform for a loaded audio file (shows the entire file)
pub fn plot_loaded_waveform(app: &AudioApp, ui: &mut egui::Ui) {
    // Maximum number of points to display (for performance)
    let (samples, sample_rate) = if let Some(audio) = &app.loaded_audio {
        (audio.samples().to_owned(), audio.sample_rate())
    } else {
        (vec![], 44100)
    };
    if samples.is_empty() {
        ui.label("No audio data in file.");
        return;
    }
    // Downsample the entire file to max_points
    let downsampled = downsample_for_waveform(&samples, MAX_POINTS);
    // Convert to PlotPoints for egui_plot with time-based X-axis
    let duration = samples.len() as f64 / sample_rate as f64;
    let points: PlotPoints = downsampled.iter().enumerate().map(|(i, &s)| {
        let time_sec = (i as f64 / downsampled.len() as f64) * duration;
        [time_sec, s as f64]
    }).collect();
    let line = Line::new(points);
    // Y axis: always include -1 and 1, but expand if needed
    let min_y = downsampled.iter().cloned().fold(0.0/0.0, f32::min).min(-1.0);
    let max_y = downsampled.iter().cloned().fold(0.0/0.0, f32::max).max(1.0);
    Plot::new("waveform_loaded")
        .height(200.0)  // Increased height for better visibility
        .include_y(min_y as f64)
        .include_y(max_y as f64)
        .include_x(0.0)
        .include_x(duration)
        .show_axes(true)
        .show_grid(true)
        .allow_zoom(true)
        .allow_scroll(true)
        .show(ui, |plot_ui| {
            plot_ui.line(line);
        });
}

/// Downsample a slice of samples to at most max_points using min/max snapshotting
fn downsample_for_waveform(samples: &[f32], max_points: usize) -> Vec<f32> {
    if samples.len() > max_points {
        let chunk = (samples.len() as f32 / max_points as f32).ceil() as usize;
        let mut down = Vec::with_capacity(max_points * 2);
        for chunked in samples.chunks(chunk) {
            // For each chunk, find min and max (preserves peaks)
            let min = chunked.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = chunked.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            down.push(min);
            down.push(max);
        }
        down
    } else {
        samples.to_vec()
    }
}