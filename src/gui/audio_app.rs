use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use crate::audio_controls::{RecordingControl, PlaybackControl};
use crate::audio::LoadedAudio;
use eframe::egui;

use crate::track_pitch;
use crate::signal_processing::{Spectrogram, SpectrogramConfig};

use super::file_selector_ui::file_selector_ui;
use super::record_controls_ui::record_controls_ui;
use super::playback_controls_ui::playback_controls_ui;
use super::status_ui::status_ui;
use super::waveform_ui::waveform_ui;
use super::spectrogram_ui::spectrogram_ui;
use super::note_display_ui::note_display_ui;

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
    pub show_peak_overlay: bool, // Toggle for peak overlay
    pub track_pitch_config: track_pitch::TrackPitchConfig,
    pub show_signal_cleaning: bool, // Toggle for signal cleaning
    pub clean_playback_signal: bool, // Toggle for playback and spectrogram cleaning
    pub loaded_audio: Option<LoadedAudio>,
    pub spectrogram_config: SpectrogramConfig,
    pub loaded_spectrogram: Option<Spectrogram>,
    pub recording_sample_rate: Option<u32>, // Sample rate of the current recording device
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

        let loaded_audio = LoadedAudio::from_file(&default_file);
        let _spectrogram = if let Some(audio) = loaded_audio.clone() {
            // Generate spectrogram from waveform if available
            Some(
                Spectrogram::from_waveform(
                    audio.samples(),
                    SpectrogramConfig::default(),
                ))
        } else {
            None
        };

        let app = Self {
            file_path: default_file,
            recording: false,
            playing: false,
            recording_control: None,
            playback_control: None,
            playback_done_rx: None,
            recorded_samples: Arc::new(Mutex::new(Vec::new())),
            playback_start: None,
            show_peak_overlay: true,
            track_pitch_config: track_pitch::TrackPitchConfig::default(),
            show_signal_cleaning: true, // Default: enabled
            clean_playback_signal: false, // Default: off
            loaded_audio: loaded_audio,
            recording_sample_rate: None,
            loaded_spectrogram: None,
            spectrogram_config: SpectrogramConfig::default(),
        };
        app
    }
}

impl AudioApp {
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
        // Use SidePanel for the right note display
        egui::SidePanel::right("note_display_panel").min_width(220.0).show(ctx, |ui| {
            ui.add_space(32.0);
            ui.vertical_centered(|ui| {
            ui.group(|ui| {
                note_display_ui(self, ui);
            });});
            ui.add_space(18.0);
            crate::gui::note_display_ui::signal_cleaning_toggle_ui(self, ui);
            crate::gui::note_display_ui::pitch_tracker_controls_ui(self, ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.set_min_width(400.0);
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
