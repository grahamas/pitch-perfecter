use audio_utils::{MonoAudio, LatencyMetrics};
use audio_cleaning::clean_audio_for_pitch;
use pitch_detection_utils::{ThreadSafeYinDetector, MonoPitchDetector, hz_to_note_name};

const WINDOW_SIZE: usize = 2048;

#[derive(Debug, Clone)]
pub struct PitchResult {
    pub frequency: f32,
    pub note_name: String,
    pub clarity: f32,
    pub latency: LatencyMetrics,
}

pub struct PitchProcessor;

impl PitchProcessor {
    /// Process an audio chunk and return pitch detection result.
    /// This is a static method that can be called from any thread.
    pub fn process_audio_chunk(
        detector: &mut ThreadSafeYinDetector,
        samples: Vec<f32>,
        sample_rate: u32,
        enable_bandpass: bool,
        enable_spectral_gating: bool,
        mut latency: LatencyMetrics,
    ) -> Option<PitchResult> {
        // Mark the start of processing
        latency.mark_processing_start();
        
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
        let result = if let Some(pitch) = detector.get_mono_pitch(processed_audio) {
            let note_name = hz_to_note_name(pitch.frequency);
            
            // Mark the end of processing
            latency.mark_processing_end();
            
            Some(PitchResult {
                frequency: pitch.frequency,
                note_name,
                clarity: pitch.clarity,
                latency,
            })
        } else {
            None
        };
        
        result
    }
}
