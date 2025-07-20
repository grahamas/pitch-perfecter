/// Audio analysis functionality for pitch detection and note identification
use crate::signal_cleaning;
use crate::track_pitch::PitchTrackerConfig;

/// Extract the most recent audio frame for analysis
pub fn extract_analysis_frame(samples: &[f32], config: PitchTrackerConfig) -> Option<Vec<f32>> {
    let window_size = config.window_size;
    if samples.len() < window_size {
        return None;
    }
    let start = samples.len().saturating_sub(window_size);
    Some(samples[start..].to_vec())
}

/// Detect pitch from processed audio frame
pub fn detect_pitch(frame: &[f32], config: PitchTrackerConfig, sample_rate: u32) -> Option<f32> {
    let pitches = crate::track_pitch::track_pitch(
        frame,
        config,
        sample_rate as usize,
    );
    
    pitches.last()
        .copied()
        .filter(|&pitch| pitch > 0.0) // Filter out invalid pitches
        .map(|pitch| pitch as f32)
}
