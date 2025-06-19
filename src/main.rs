use pitch_perfecter::*;

fn main() {
    let mut options = eframe::NativeOptions::default();
    options.viewport = eframe::egui::ViewportBuilder::default().with_inner_size([900.0, 900.0]);
    eframe::run_native(
        "Pitch Perfecter Audio Recorder",
        options,
        Box::new(|_cc| Ok(Box::new(gui::audio_app::AudioApp::default()))),
    ).unwrap();
}