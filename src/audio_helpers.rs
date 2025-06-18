//! Helper functions for audio file operations (testable, non-GUI)
use hound::WavReader;

/// Load audio samples from a WAV file (f32 or i16 PCM)
pub fn load_audio_samples(path: &str) -> Option<Vec<f32>> {
    if let Ok(mut reader) = WavReader::open(path) {
        let spec = reader.spec();
        let samples: Vec<f32> = if spec.sample_format == hound::SampleFormat::Float {
            reader.samples::<f32>().filter_map(Result::ok).collect()
        } else {
            reader.samples::<i16>().filter_map(Result::ok).map(|s| s as f32 / i16::MAX as f32).collect()
        };
        Some(samples)
    } else {
        None
    }
}

/// Load audio samples and sample rate from a WAV file
pub fn load_audio_samples_and_rate(path: &str) -> Option<(Vec<f32>, u32)> {
    if let Ok(mut reader) = WavReader::open(path) {
        let spec = reader.spec();
        let samples: Vec<f32> = if spec.sample_format == hound::SampleFormat::Float {
            reader.samples::<f32>().filter_map(Result::ok).collect()
        } else {
            reader.samples::<i16>().filter_map(Result::ok).map(|s| s as f32 / i16::MAX as f32).collect()
        };
        Some((samples, spec.sample_rate))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_audio_samples_nonexistent() {
        // Should return None for a file that doesn't exist
        assert!(load_audio_samples("nonexistent.wav").is_none());
    }
    #[test]
    fn test_load_audio_samples_empty() {
        // Should return None for an empty file
        use std::fs::File;
        let _ = File::create("test_empty.wav");
        assert!(load_audio_samples("test_empty.wav").is_none() ||
                load_audio_samples("test_empty.wav").unwrap().is_empty());
        let _ = std::fs::remove_file("test_empty.wav");
    }
    // More tests can be added for real WAV files in a test_data/ directory
}
