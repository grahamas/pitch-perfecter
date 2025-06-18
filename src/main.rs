mod audio;
mod audio_helpers;
mod signal_processing;
pub use crate::recording_control::RecordingControl;
pub use crate::playback_control::PlaybackControl;
mod gui;
mod recording_control;
mod playback_control;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Pitch Perfecter Audio Recorder",
        options,
        Box::new(|_cc| Ok(Box::new(gui::AudioApp::default()))),
    ).unwrap();
}