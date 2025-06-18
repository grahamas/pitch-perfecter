use eframe::egui;
use crate::{RecordingControl, PlaybackControl};
use std::sync::mpsc;
use rfd::FileDialog;
use std::env;
use egui_plot::{Plot, Line, PlotPoints};
use crate::audio_helpers;
use crate::signal_processing;
use std::sync::{Arc, Mutex};

/// Main application state for Pitch Perfecter
pub struct AudioApp {
    pub file_path: String,
    pub recording: bool,
    pub playing: bool,
    pub recording_control: Option<RecordingControl>,
    pub playback_control: Option<PlaybackControl>,
    pub playback_done_rx: Option<mpsc::Receiver<()>>,
    /// In-memory buffer for live waveform display while recording
    pub recorded_samples: Arc<Mutex<Vec<f32>>>,
    pub playback_start: Option<std::time::Instant>, // Track playback start time
    // Cached audio metadata
    pub cached_sample_rate: Option<u32>,
    pub cached_total_samples: Option<u32>,
    pub cached_duration_sec: Option<f32>,
}

impl Default for AudioApp {
    fn default() -> Self {
        Self {
            file_path: String::new(), // No pre-selected file
            recording: false,
            playing: false,
            recording_control: None,
            playback_control: None,
            playback_done_rx: None,
            recorded_samples: Arc::new(Mutex::new(Vec::new())),
            playback_start: None, // Initialize playback_start
            cached_sample_rate: None,
            cached_total_samples: None,
            cached_duration_sec: None,
        }
    }
}

impl AudioApp {
    /// File selection UI: open/save dialogs and file path entry
    fn file_selector_ui(&mut self, ui: &mut egui::Ui) {
        let audio_dir = env::current_dir()
            .map(|p| p.join("audio"))
            .unwrap_or_else(|_| std::path::PathBuf::from("audio"));
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("File path:").size(18.0));
            ui.add_sized([350.0, 28.0], egui::TextEdit::singleline(&mut self.file_path));
            // Open file dialog
            if ui.button("📂").on_hover_text("Select file to open...").clicked() {
                if let Some(path) = FileDialog::new().set_directory(&audio_dir).pick_file() {
                    if let Some(path_str) = path.to_str() {
                        self.file_path = path_str.to_owned();
                        self.update_audio_metadata(); // Update cache on file select
                    }
                }
            }
            // Save as file dialog
            if ui.button("💾").on_hover_text("Select file to save as...").clicked() {
                if let Some(path) = FileDialog::new().set_directory(&audio_dir).set_file_name(&self.file_path).save_file() {
                    if let Some(path_str) = path.to_str() {
                        self.file_path = path_str.to_owned();
                        self.update_audio_metadata(); // Update cache on file save
                    }
                }
            }
        });
    }

    /// Record controls UI: Record and Stop Recording buttons
    fn record_controls_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Record button
            if ui.add_enabled(!self.recording, egui::Button::new(egui::RichText::new("⏺ Record").size(18.0))).clicked() {
                self.recording = true;
                // If no file is selected, create a unique file in audio/
                if self.file_path.trim().is_empty() {
                    let audio_dir = std::env::current_dir()
                        .map(|p| p.join("audio"))
                        .unwrap_or_else(|_| std::path::PathBuf::from("audio"));
                    let _ = std::fs::create_dir_all(&audio_dir);
                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                    let unique_name = format!("recording_{}.wav", timestamp);
                    let unique_path = audio_dir.join(unique_name);
                    self.file_path = unique_path.to_string_lossy().to_string();
                }
                let path = self.file_path.clone();
                let control = RecordingControl::new();
                self.recording_control = Some(control.clone());
                self.recorded_samples.lock().unwrap().clear();
                let live_buffer = self.recorded_samples.clone();
                std::thread::spawn(move || {
                    crate::audio::record_audio_with_control_and_buffer(&path, control, live_buffer);
                });
            }
            // Stop Recording button
            if ui.add_enabled(self.recording, egui::Button::new(egui::RichText::new("⏹ Stop Recording").size(18.0))).clicked() {
                if let Some(control) = &self.recording_control {
                    control.stop();
                }
                self.recording = false;
                self.recording_control = None;
            }
        });
    }

    /// Playback controls UI: Play and Stop Playback buttons
    fn playback_controls_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Play button
            if ui.add_enabled(!self.playing, egui::Button::new(egui::RichText::new("▶ Play").size(18.0))).clicked() {
                self.playing = true;
                self.playback_start = Some(std::time::Instant::now()); // Set playback_start only once here
                let path = self.file_path.clone();
                let control = PlaybackControl::new();
                self.playback_control = Some(control.clone());
                let (tx, rx) = mpsc::channel();
                self.playback_done_rx = Some(rx);
                std::thread::spawn(move || {
                    crate::audio::play_audio_with_control_and_notify(&path, control, tx);
                });
            }
            // Stop Playback button
            if ui.add_enabled(self.playing, egui::Button::new(egui::RichText::new("⏹ Stop Playback").size(18.0))).clicked() {
                if let Some(control) = &self.playback_control {
                    control.stop();
                }
                self.playing = false;
                self.playback_control = None;
                self.playback_done_rx = None;
                self.playback_start = None; // Reset playback_start on stop
            }
        });
    }

    /// Status display UI: shows current state (Recording, Playing, Idle)
    fn status_ui(&self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new(
            if self.recording {
                "Recording..."
            } else if self.playing {
                "Playing..."
            } else {
                "Idle"
            }
        ).size(16.0).italics());
    }

    /// Waveform plot UI: shows the waveform of the loaded audio file or live recording
    fn waveform_ui(&self, ui: &mut egui::Ui) {
        let sample_rate = 44100.0;
        let five_sec_samples = (sample_rate * 5.0) as usize;
        let samples = if self.recording {
            let guard = self.recorded_samples.lock().unwrap();
            if guard.is_empty() { None } else { Some(guard.clone()) }
        } else if self.file_path.trim().is_empty() {
            None
        } else {
            audio_helpers::load_audio_samples(&self.file_path)
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

    /// Spectrogram plot UI: shows the log-magnitude spectrogram of the loaded audio file with a viridis-like color map and moving peak overlay
    fn spectrogram_ui(&mut self, ui: &mut egui::Ui) {
        // Only show for loaded files, not live recording
        if self.recording || self.file_path.trim().is_empty() {
            return;
        }
        if let Some((samples, sample_rate)) = audio_helpers::load_audio_samples_and_rate(&self.file_path) {
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
                let y = 1.0 - (freq_bin as f32 / n_freq as f32); // normalized freq (flip y for image coordinates)
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
                    let mapped = egui::pos2(rect.left() + p.x * rect.width(), rect.top() + (1.0 - p.y) * rect.height());
                    painter.add(Shape::circle_filled(mapped, 3.0, Color32::YELLOW));
                }
            }
            // Draw a vertical red line to indicate playback time
            if self.playing {
                // Align red line with spectrogram time bins (not raw samples), using cached metadata
                if let (Some(sample_rate), Some(duration_sec)) = (self.cached_sample_rate, self.cached_duration_sec) {
                    let mut elapsed_sec = 0.0;
                    if let Some(start) = self.playback_start {
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

    /// Update cached audio metadata (sample rate, total samples, duration)
    fn update_audio_metadata(&mut self) {
        if let Ok(reader) = hound::WavReader::open(&self.file_path) {
            let spec = reader.spec();
            let total_samples = reader.duration();
            let duration_sec = total_samples as f32 / spec.sample_rate as f32;
            self.cached_sample_rate = Some(spec.sample_rate);
            self.cached_total_samples = Some(total_samples as u32);
            self.cached_duration_sec = Some(duration_sec);
        } else {
            self.cached_sample_rate = None;
            self.cached_total_samples = None;
            self.cached_duration_sec = None;
        }
    }
}

impl eframe::App for AudioApp {
    /// Main update function: builds the GUI each frame
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if playback finished and reset state if so
        if let Some(rx) = &self.playback_done_rx {
            if rx.try_recv().is_ok() {
                self.playing = false;
                self.playback_control = None;
                self.playback_done_rx = None;
            }
        }
        // Force GUI to repaint frequently while recording or playing for smooth updates
        if self.recording {
            ctx.request_repaint();
        }
        if self.playing {
            ctx.request_repaint_after(std::time::Duration::from_millis(16)); // ~60 FPS
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new("Pitch Perfecter Audio Recorder").size(28.0).strong());
                ui.add_space(20.0);
                ui.group(|ui| {
                    // File selection controls
                    self.file_selector_ui(ui);
                });
                ui.add_space(20.0);
                // Record controls
                self.record_controls_ui(ui);
                ui.add_space(10.0);
                // Playback controls
                self.playback_controls_ui(ui);
                ui.add_space(20.0);
                // Status display
                self.status_ui(ui);
                ui.add_space(20.0);
                // Waveform plot
                self.waveform_ui(ui);
                ui.add_space(20.0);
                // Spectrogram plot
                self.spectrogram_ui(ui);
            });
        });
    }
}
