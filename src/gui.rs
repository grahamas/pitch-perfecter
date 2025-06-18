use eframe::egui;
use crate::{RecordingControl, PlaybackControl};
use std::sync::mpsc;
use rfd::FileDialog;
use std::env;
use egui_plot::{Plot, Line, PlotPoints};
use crate::audio_helpers;
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
                    }
                }
            }
            // Save as file dialog
            if ui.button("💾").on_hover_text("Select file to save as...").clicked() {
                if let Some(path) = FileDialog::new().set_directory(&audio_dir).set_file_name(&self.file_path).save_file() {
                    if let Some(path_str) = path.to_str() {
                        self.file_path = path_str.to_owned();
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
        // Force GUI to repaint frequently while recording for smooth waveform updates
        if self.recording {
            ctx.request_repaint();
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
            });
        });
    }
}
