//! GUI
//! 
//! This crate provides the graphical user interface for the pitch perfecter application.
//! 
//! ## Features
//! - Real-time audio recording from microphone
//! - Live pitch detection and display
//! - Configurable audio cleaning options (bandpass filter, spectral gating)
//! - Musical note display with frequency
//! - Optional real-time WAV file saving
//! 
//! ## Running the GUI
//! ```bash
//! cargo run -p gui --bin pitch-perfecter-gui
//! ```
//! 
//! ## Architecture
//! - Uses `egui` for the GUI framework (immediate mode, responsive)
//! - Uses `cpal` for cross-platform audio input
//! - Processes audio in chunks for real-time pitch detection
//! - Multi-threaded design: audio recording runs on separate thread from GUI
//! 
//! ## Required APIs from Other Crates
//! This implementation requires the following APIs which already exist:
//! - `audio_utils::MonoAudio` - for audio data representation
//! - `audio_cleaning::clean_audio_for_pitch` - for audio preprocessing
//! - `pitch_detection_utils::ExternalYinDetector` - for pitch detection
//! - `pitch_detection_utils::hz_to_note_name` - for note name conversion

// Re-export main modules for library use
pub mod audio_recorder;
pub mod pitch_processor;
pub mod learning_pane;
