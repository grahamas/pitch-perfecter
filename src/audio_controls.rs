use std::sync::{Arc, atomic::{AtomicBool, Ordering, AtomicUsize}};

#[derive(Clone)]
pub struct PlaybackControl {
    pub stop_flag: Arc<AtomicBool>,
    pub sample_index: Arc<AtomicUsize>, // Track playback sample index
}

impl PlaybackControl {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
            sample_index: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }
    pub fn should_stop(&self) -> bool {
        self.stop_flag.load(Ordering::SeqCst)
    }
    pub fn sample_index(&self) -> usize {
        self.sample_index.load(Ordering::SeqCst)
    }
}

#[derive(Clone)]
pub struct RecordingControl {
    pub stop_flag: Arc<AtomicBool>,
}

impl RecordingControl {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }
    pub fn should_stop(&self) -> bool {
        self.stop_flag.load(Ordering::SeqCst)
    }
}
