use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

#[derive(Clone)]
pub struct PlaybackControl {
    pub stop_flag: Arc<AtomicBool>,
}

impl PlaybackControl {
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
