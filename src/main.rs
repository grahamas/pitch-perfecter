use pitch_perfecter::*;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Pitch Perfecter Audio Recorder",
        options,
        Box::new(|_cc| Ok(Box::new(gui::audio_app::AudioApp::default()))),
    ).unwrap();
}