mod audio;
mod audio_helpers;
mod signal_processing;
pub use crate::recording_control::RecordingControl;
pub use crate::playback_control::PlaybackControl;
mod gui;
mod recording_control;
mod playback_control;
// pub mod voice_synth; // This module is now provided by the sound_synth crate
// mod yin; // All YIN and pYIN code is now in the pitch submodule

// If you need YIN or pYIN, use crate::pitch::yin or crate::pitch::pyin

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Pitch Perfecter Audio Recorder",
        options,
        Box::new(|_cc| Ok(Box::new(gui::AudioApp::default()))),
    ).unwrap();
}