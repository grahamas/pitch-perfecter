use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use crate::audio_controls::{RecordingControl, PlaybackControl};
use eframe::egui;


use super::file_selector_ui::file_selector_ui;
use super::record_controls_ui::record_controls_ui;
use super::playback_controls_ui::playback_controls_ui;
use super::status_ui::status_ui;
use super::waveform_ui::waveform_ui;
use super::spectrogram_ui::spectrogram_ui;

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
        // Try to pick a default recording from the audio directory
        let audio_dir = std::env::current_dir()
            .map(|p| p.join("audio"))
            .unwrap_or_else(|_| std::path::PathBuf::from("audio"));
        let mut default_file = String::new();
        if let Ok(entries) = std::fs::read_dir(&audio_dir) {
            // Pick the first .wav file found (static random choice)
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "wav" {
                        default_file = path.to_string_lossy().to_string();
                        break;
                    }
                }
            }
        }
        let mut app = Self {
            file_path: default_file,
            recording: false,
            playing: false,
            recording_control: None,
            playback_control: None,
            playback_done_rx: None,
            recorded_samples: Arc::new(Mutex::new(Vec::new())),
            playback_start: None,
            cached_sample_rate: None,
            cached_total_samples: None,
            cached_duration_sec: None,
        };
        if !app.file_path.is_empty() {
            app.update_audio_metadata();
        }
        app
    }
}

impl AudioApp {
    pub fn update_audio_metadata(&mut self) {
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
    // UI wrappers
    pub fn file_selector_ui(&mut self, ui: &mut egui::Ui) {
        file_selector_ui(self, ui);
    }
    pub fn record_controls_ui(&mut self, ui: &mut egui::Ui) {
        record_controls_ui(self, ui);
    }
    pub fn playback_controls_ui(&mut self, ui: &mut egui::Ui) {
        playback_controls_ui(self, ui);
    }
    pub fn status_ui(&self, ui: &mut egui::Ui) {
        status_ui(self, ui);
    }
    pub fn waveform_ui(&self, ui: &mut egui::Ui) {
        waveform_ui(self, ui);
    }
    pub fn spectrogram_ui(&mut self, ui: &mut egui::Ui) {
        spectrogram_ui(self, ui);
    }
}

impl eframe::App for AudioApp {
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
                    self.file_selector_ui(ui);
                });
                ui.add_space(20.0);
                self.record_controls_ui(ui);
                ui.add_space(10.0);
                self.playback_controls_ui(ui);
                ui.add_space(20.0);
                self.status_ui(ui);
                ui.add_space(20.0);
                self.waveform_ui(ui);
                ui.add_space(20.0);
                self.spectrogram_ui(ui);
            });
        });
    }
}
