use eframe::egui;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver};

mod audio_recorder;
mod pitch_processor;
mod learning_pane;

use audio_recorder::AudioRecorder;
use pitch_processor::PitchResult;
use learning_pane::LearningPane;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "Pitch Perfecter",
        options,
        Box::new(|cc| Ok(Box::new(PitchPerfecterApp::new(cc)))),
    )
}

/// Which tab is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveTab {
    PitchDetection,
    Learning,
}

struct PitchPerfecterApp {
    // Active tab
    active_tab: ActiveTab,
    
    // Audio recording
    audio_recorder: Arc<Mutex<AudioRecorder>>,
    
    // Pitch results receiver (processing now runs on audio thread)
    pitch_receiver: Receiver<PitchResult>,
    
    // UI state
    is_recording: bool,
    current_pitch: Option<PitchResult>,
    
    // Cleaning options
    enable_bandpass: bool,
    enable_spectral_gating: bool,
    
    // File saving
    save_to_file: bool,
    save_path: String,
    
    // Status messages
    status_message: String,
    
    // Learning pane
    learning_pane: LearningPane,
}

impl PitchPerfecterApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (_pitch_tx, pitch_rx) = channel();
        
        let audio_recorder = Arc::new(Mutex::new(AudioRecorder::new()));
        
        Self {
            active_tab: ActiveTab::PitchDetection,
            audio_recorder,
            pitch_receiver: pitch_rx,
            is_recording: false,
            current_pitch: None,
            enable_bandpass: true,
            enable_spectral_gating: false,
            save_to_file: false,
            save_path: "recording.wav".to_string(),
            status_message: "Ready".to_string(),
            learning_pane: LearningPane::new(),
        }
    }
    
    fn start_recording(&mut self) {
        let save_to_file = self.save_to_file;
        let save_path = self.save_path.clone();
        let enable_bandpass = self.enable_bandpass;
        let enable_spectral_gating = self.enable_spectral_gating;
        
        // Create a new channel for this recording session
        let (pitch_tx, pitch_rx) = channel();
        self.pitch_receiver = pitch_rx;
        
        // Detector parameters - same as in PitchProcessor
        const WINDOW_SIZE: usize = 2048;
        const HOP_SIZE: usize = 1024;
        const POWER_THRESHOLD: f32 = 0.1;
        const CLARITY_THRESHOLD: f32 = 0.7;
        
        let result = self.audio_recorder.lock().unwrap().start(
            pitch_tx,
            POWER_THRESHOLD,
            CLARITY_THRESHOLD,
            WINDOW_SIZE,
            HOP_SIZE,
            enable_bandpass,
            enable_spectral_gating,
            save_to_file,
            save_path,
        );
        
        match result {
            Ok(_) => {
                self.is_recording = true;
                self.status_message = "Recording...".to_string();
            }
            Err(e) => {
                self.status_message = format!("Error starting recording: {}", e);
            }
        }
    }
    
    fn stop_recording(&mut self) {
        match self.audio_recorder.lock().unwrap().stop() {
            Ok(_) => {
                self.is_recording = false;
                self.status_message = "Recording stopped".to_string();
            }
            Err(e) => {
                self.status_message = format!("Error stopping recording: {}", e);
            }
        }
    }
}

impl eframe::App for PitchPerfecterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Receive pitch results from the audio thread
        // Pitch detection now happens directly on the audio callback thread
        // Store the latest pitch result and share it with both views
        let mut latest_pitch = None;
        while let Ok(pitch_result) = self.pitch_receiver.try_recv() {
            latest_pitch = Some(pitch_result);
        }
        
        // Update current pitch for pitch detection tab
        if let Some(pitch) = latest_pitch {
            self.current_pitch = Some(pitch.clone());
            // Also update learning pane with the latest pitch
            self.learning_pane.update_pitch_direct(pitch);
        }
        
        // Request continuous repaint for real-time updates
        ctx.request_repaint();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pitch Perfecter");
            ui.add_space(10.0);
            
            // Tab selection
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, ActiveTab::PitchDetection, "Pitch Detection");
                ui.selectable_value(&mut self.active_tab, ActiveTab::Learning, "Learning");
            });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Render the appropriate tab content
            match self.active_tab {
                ActiveTab::PitchDetection => self.render_pitch_detection_tab(ui),
                ActiveTab::Learning => self.render_learning_tab(ui),
            }
        });
    }
}

impl PitchPerfecterApp {
    fn render_pitch_detection_tab(&mut self, ui: &mut egui::Ui) {
        // Recording control
        ui.group(|ui| {
                ui.heading("Recording");
                ui.add_space(5.0);
                
                if self.is_recording {
                    if ui.button("⏹ Stop Recording").clicked() {
                        self.stop_recording();
                    }
                } else {
                    if ui.button("⏺ Start Recording").clicked() {
                        self.start_recording();
                    }
                }
                
                ui.add_space(5.0);
                ui.label(&self.status_message);
            });
            
            ui.add_space(10.0);
            
            // Cleaning options
            ui.group(|ui| {
                ui.heading("Cleaning Options");
                ui.add_space(5.0);
                
                ui.checkbox(&mut self.enable_bandpass, "Bandpass Filter (Vocal Range)")
                    .on_hover_text("Filter frequencies outside typical vocal range (80-800 Hz)");
                
                ui.checkbox(&mut self.enable_spectral_gating, "Spectral Gating (Noise Reduction)")
                    .on_hover_text("Reduce background noise using spectral gating");
            });
            
            ui.add_space(10.0);
            
            // Pitch display
            ui.group(|ui| {
                ui.heading("Detected Pitch");
                ui.add_space(5.0);
                
                if let Some(ref pitch) = self.current_pitch {
                    ui.horizontal(|ui| {
                        ui.label("Note:");
                        ui.heading(&pitch.note_name);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Frequency:");
                        ui.heading(format!("{:.2} Hz", pitch.frequency));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Clarity:");
                        ui.add(egui::ProgressBar::new(pitch.clarity).show_percentage());
                    });
                } else {
                    ui.label("No pitch detected");
                }
            });
            
            ui.add_space(10.0);
            
            // File saving options
            ui.group(|ui| {
                ui.heading("Save Recording");
                ui.add_space(5.0);
                
                ui.checkbox(&mut self.save_to_file, "Save to file in real-time")
                    .on_hover_text("Save audio to a WAV file while recording");
                
                ui.horizontal(|ui| {
                    ui.label("Filename:");
                    ui.text_edit_singleline(&mut self.save_path);
                });
                
                if !self.save_path.ends_with(".wav") {
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Filename should end with .wav");
                }
            });
    }
    
    fn render_learning_tab(&mut self, ui: &mut egui::Ui) {
        let should_start_recording = self.learning_pane.render(ui);
        
        // Handle recording state synchronization
        let should_be_recording = self.learning_pane.should_be_recording();
        
        if should_start_recording && !self.is_recording {
            self.start_recording();
        } else if !should_be_recording && self.is_recording {
            self.stop_recording();
        }
    }
}
