//! Audio File I/O Module
//!
//! This module provides functions for loading and saving audio files.
//! Currently supports WAV format through the hound library.

use crate::audio::MonoAudio;
use std::path::Path;

/// Error type for audio file I/O operations
#[derive(Debug)]
pub enum AudioIoError {
    /// Error reading from file
    ReadError(String),
    /// Error writing to file
    WriteError(String),
    /// Unsupported format
    UnsupportedFormat(String),
}

impl std::fmt::Display for AudioIoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioIoError::ReadError(msg) => write!(f, "Read error: {}", msg),
            AudioIoError::WriteError(msg) => write!(f, "Write error: {}", msg),
            AudioIoError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
        }
    }
}

impl std::error::Error for AudioIoError {}

/// Load mono audio from a WAV file
///
/// This function reads a WAV file containing mono audio.
/// If the file contains stereo or multi-channel audio, an error will be returned.
///
/// # Arguments
/// * `path` - Path to the WAV file to load
///
/// # Returns
/// * `Ok(MonoAudio)` - Successfully loaded audio
/// * `Err(AudioIoError)` - Error reading the file or if the file is not mono
///
/// # Examples
/// ```no_run
/// use audio_utils::io::load_wav;
///
/// let audio = load_wav("input.wav").expect("Failed to load audio");
/// println!("Loaded {} samples at {} Hz", audio.samples.len(), audio.sample_rate);
/// ```
pub fn load_wav<P: AsRef<Path>>(path: P) -> Result<MonoAudio, AudioIoError> {
    let path_ref = path.as_ref();
    
    let reader = hound::WavReader::open(path_ref)
        .map_err(|e| AudioIoError::ReadError(format!("Failed to open file: {}", e)))?;
    
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    let channels = spec.channels as usize;
    
    // Only mono audio is supported
    if channels != 1 {
        return Err(AudioIoError::UnsupportedFormat(
            format!("Only mono audio is supported, found {} channels", channels)
        ));
    }
    
    // Read all samples based on the sample format
    let samples: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
        (hound::SampleFormat::Float, 32) => {
            reader.into_samples::<f32>()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AudioIoError::ReadError(format!("Failed to read samples: {}", e)))?
        },
        (hound::SampleFormat::Int, 16) => {
            let int_samples: Vec<i16> = reader.into_samples::<i16>()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AudioIoError::ReadError(format!("Failed to read samples: {}", e)))?;
            int_samples.iter().map(|&s| s as f32 / i16::MAX as f32).collect()
        },
        (hound::SampleFormat::Int, 24) => {
            let int_samples: Vec<i32> = reader.into_samples::<i32>()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AudioIoError::ReadError(format!("Failed to read samples: {}", e)))?;
            int_samples.iter().map(|&s| s as f32 / 8388608.0).collect() // 2^23
        },
        (hound::SampleFormat::Int, 32) => {
            let int_samples: Vec<i32> = reader.into_samples::<i32>()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AudioIoError::ReadError(format!("Failed to read samples: {}", e)))?;
            int_samples.iter().map(|&s| s as f32 / i32::MAX as f32).collect()
        },
        _ => {
            return Err(AudioIoError::UnsupportedFormat(
                format!("Unsupported sample format: {:?} with {} bits", spec.sample_format, spec.bits_per_sample)
            ));
        }
    };
    
    Ok(MonoAudio::new(samples, sample_rate))
}

/// Save mono audio to a WAV file
///
/// This function writes mono audio data to a WAV file in 32-bit float format.
///
/// # Arguments
/// * `path` - Path where the WAV file should be written
/// * `audio` - The mono audio data to save
///
/// # Returns
/// * `Ok(())` - Successfully saved audio
/// * `Err(AudioIoError)` - Error writing the file
///
/// # Examples
/// ```no_run
/// use audio_utils::{MonoAudio, io::save_wav};
///
/// let audio = MonoAudio::new(vec![0.0, 0.5, 1.0, 0.5, 0.0], 44100);
/// save_wav("output.wav", &audio).expect("Failed to save audio");
/// ```
pub fn save_wav<P: AsRef<Path>>(path: P, audio: &MonoAudio) -> Result<(), AudioIoError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: audio.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| AudioIoError::WriteError(format!("Failed to create file: {}", e)))?;
    
    for &sample in &audio.samples {
        writer.write_sample(sample)
            .map_err(|e| AudioIoError::WriteError(format!("Failed to write sample: {}", e)))?;
    }
    
    writer.finalize()
        .map_err(|e| AudioIoError::WriteError(format!("Failed to finalize file: {}", e)))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_save_and_load_wav() {
        // Create temporary test file path
        let test_path = "/tmp/test_audio_io.wav";
        
        // Create test audio
        let original_samples = vec![0.0, 0.25, 0.5, 0.75, 1.0, 0.75, 0.5, 0.25, 0.0];
        let original_audio = MonoAudio::new(original_samples.clone(), 44100);
        
        // Save the audio
        save_wav(test_path, &original_audio).expect("Failed to save audio");
        
        // Load the audio back
        let loaded_audio = load_wav(test_path).expect("Failed to load audio");
        
        // Verify the audio matches
        assert_eq!(loaded_audio.sample_rate, original_audio.sample_rate);
        assert_eq!(loaded_audio.samples.len(), original_audio.samples.len());
        
        // Compare samples (allow for small floating point errors)
        for (loaded, original) in loaded_audio.samples.iter().zip(original_samples.iter()) {
            assert!((loaded - original).abs() < 1e-6, 
                    "Sample mismatch: loaded = {}, original = {}", loaded, original);
        }
        
        // Clean up
        fs::remove_file(test_path).ok();
    }
    
    #[test]
    fn test_save_wav_creates_valid_file() {
        let test_path = "/tmp/test_save_wav.wav";
        
        // Create simple sine-like wave
        let samples: Vec<f32> = (0..100)
            .map(|i| (i as f32 * 0.1).sin() * 0.5)
            .collect();
        let audio = MonoAudio::new(samples, 48000);
        
        // Save
        save_wav(test_path, &audio).expect("Failed to save audio");
        
        // Verify file exists and has content
        let metadata = fs::metadata(test_path).expect("File was not created");
        assert!(metadata.len() > 0, "File is empty");
        
        // Clean up
        fs::remove_file(test_path).ok();
    }
    
    #[test]
    fn test_load_nonexistent_file() {
        let result = load_wav("/tmp/nonexistent_audio_file_12345.wav");
        assert!(result.is_err(), "Should fail to load nonexistent file");
        
        match result {
            Err(AudioIoError::ReadError(_)) => {}, // Expected
            _ => panic!("Expected ReadError"),
        }
    }
    
    #[test]
    fn test_round_trip_preserves_sample_rate() {
        let test_path = "/tmp/test_sample_rate.wav";
        
        let rates = vec![8000, 16000, 22050, 44100, 48000];
        
        for rate in rates {
            let audio = MonoAudio::new(vec![0.1, 0.2, 0.3], rate);
            save_wav(test_path, &audio).expect("Failed to save");
            let loaded = load_wav(test_path).expect("Failed to load");
            
            assert_eq!(loaded.sample_rate, rate, "Sample rate not preserved for {} Hz", rate);
        }
        
        fs::remove_file(test_path).ok();
    }
    
    #[test]
    fn test_load_stereo_file_returns_error() {
        let test_path = "/tmp/test_stereo.wav";
        
        // Create a stereo WAV file manually using hound
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        
        let mut writer = hound::WavWriter::create(test_path, spec).unwrap();
        // Write some stereo samples (left, right, left, right, ...)
        for i in 0..10 {
            writer.write_sample((i as f32) * 0.1).unwrap(); // Left channel
            writer.write_sample((i as f32) * -0.1).unwrap(); // Right channel
        }
        writer.finalize().unwrap();
        
        // Try to load the stereo file
        let result = load_wav(test_path);
        
        // Should return an error
        assert!(result.is_err(), "Should fail to load stereo file");
        
        match result {
            Err(AudioIoError::UnsupportedFormat(msg)) => {
                assert!(msg.contains("mono"), "Error message should mention mono");
                assert!(msg.contains("2"), "Error message should mention 2 channels");
            },
            _ => panic!("Expected UnsupportedFormat error"),
        }
        
        // Clean up
        fs::remove_file(test_path).ok();
    }
}
