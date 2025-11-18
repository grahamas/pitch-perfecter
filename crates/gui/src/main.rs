use eframe::egui;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver};

mod audio_recorder;
mod pitch_processor;

use audio_recorder::AudioRecorder;
use pitch_processor::{PitchProcessor, PitchResult, AudioChunk};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_resizable(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "Pitch Perfecter",
        options,
        Box::new(|cc| Ok(Box::new(PitchPerfecterApp::new(cc)))),
    )
}

struct PitchPerfecterApp {
    // Audio recording
    audio_recorder: Arc<Mutex<AudioRecorder>>,
    
    // Pitch processing (runs on main thread)
    pitch_processor: PitchProcessor,
    audio_receiver: Receiver<AudioChunk>,
    
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
}

impl PitchPerfecterApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (_audio_tx, audio_rx) = channel();
        
        let audio_recorder = Arc::new(Mutex::new(AudioRecorder::new()));
        let pitch_processor = PitchProcessor::new();
        
        Self {
            audio_recorder,
            pitch_processor,
            audio_receiver: audio_rx,
            is_recording: false,
            current_pitch: None,
            enable_bandpass: true,
            enable_spectral_gating: false,
            save_to_file: false,
            save_path: "recording.wav".to_string(),
            status_message: "Ready".to_string(),
        }
    }
    
    fn start_recording(&mut self) {
        let save_to_file = self.save_to_file;
        let save_path = self.save_path.clone();
        
        // Get the sender from the receiver
        let (audio_tx, audio_rx) = channel();
        self.audio_receiver = audio_rx;
        
        let result = self.audio_recorder.lock().unwrap().start(
            audio_tx,
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
        // Process audio chunks from the recording thread
        while let Ok(audio_chunk) = self.audio_receiver.try_recv() {
            let pitch_result = self.pitch_processor.process_audio_chunk(
                audio_chunk.samples,
                audio_chunk.sample_rate,
                self.enable_bandpass,
                self.enable_spectral_gating,
            );
            if let Some(result) = pitch_result {
                self.current_pitch = Some(result);
            }
        }
        
        // Request continuous repaint for real-time updates
        ctx.request_repaint();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pitch Perfecter");
            ui.add_space(10.0);
            
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
        });
    }
}
