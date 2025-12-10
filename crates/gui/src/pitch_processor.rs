use audio_utils::MonoAudio;
use audio_cleaning::clean_audio_for_pitch;
use pitch_detection_utils::{ThreadSafeYinDetector, MonoPitchDetector, hz_to_note_name};

const WINDOW_SIZE: usize = 2048;

#[derive(Debug, Clone)]
pub struct PitchResult {
    pub frequency: f32,
    pub note_name: String,
    pub clarity: f32,
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
        noise_profile: Option<&audio_cleaning::Spectrum>,
    ) -> Option<PitchResult> {
        // Only process if we have enough samples
        if samples.len() < WINDOW_SIZE {
            return None;
        }
        
        // Create audio object
        let audio = MonoAudio::new(samples, sample_rate);
        
        // Apply cleaning if enabled
        let processed_audio = if enable_bandpass || enable_spectral_gating {
            // Use noise profile only if spectral gating is enabled AND profile is available
            let noise_spectrum = if enable_spectral_gating && noise_profile.is_some() {
                noise_profile.cloned()
            } else {
                None
            };
            
            // Note: This uses defaults for bandpass (80-800 Hz for vocals)
            // If noise_spectrum is None, clean_audio_for_pitch falls back to bandpass filtering
            clean_audio_for_pitch(&audio, noise_spectrum, None)
        } else {
            audio
        };
        
        // Detect pitch
        if let Some(pitch) = detector.get_mono_pitch(processed_audio) {
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
