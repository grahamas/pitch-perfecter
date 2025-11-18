use audio_utils::MonoAudio;
use audio_cleaning::clean_audio_for_pitch;
use pitch_detection_utils::{ExternalYinDetector, MonoPitchDetector, hz_to_note_name};

const WINDOW_SIZE: usize = 2048;
const HOP_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct PitchResult {
    pub frequency: f32,
    pub note_name: String,
    pub clarity: f32,
}

#[derive(Debug, Clone)]
pub struct AudioChunk {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

pub struct PitchProcessor {
    detector: ExternalYinDetector,
}

impl PitchProcessor {
    pub fn new() -> Self {
        // Create YIN detector with reasonable parameters
        // threshold: 0.1 (lower is more strict)
        // confidence_threshold: 0.7 (0-1, higher means more confident)
        let detector = ExternalYinDetector::new(0.1, 0.7, WINDOW_SIZE, HOP_SIZE);
        
        Self {
            detector,
        }
    }
    
    pub fn process_audio_chunk(
        &mut self,
        samples: Vec<f32>,
        sample_rate: u32,
        enable_bandpass: bool,
        enable_spectral_gating: bool,
    ) -> Option<PitchResult> {
        // Only process if we have enough samples
        if samples.len() < WINDOW_SIZE {
            return None;
        }
        
        // Create audio object
        let audio = MonoAudio::new(samples, sample_rate);
        
        // Apply cleaning if enabled
        let processed_audio = if enable_bandpass || enable_spectral_gating {
            // Note: This uses defaults for bandpass (80-800 Hz for vocals)
            // For real-time processing, we don't use noise spectrum estimation
            // as it requires a pre-recorded noise profile
            clean_audio_for_pitch(&audio, None, None)
        } else {
            audio
        };
        
        // Detect pitch
        if let Some(pitch) = self.detector.get_mono_pitch(processed_audio) {
            let note_name = hz_to_note_name(pitch.frequency);
            
            Some(PitchResult {
                frequency: pitch.frequency,
                note_name,
                clarity: pitch.clarity,
            })
        } else {
            None
        }
    }
}
