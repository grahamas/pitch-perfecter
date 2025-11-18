//! Audio Recording Module
//!
//! This module provides functions for recording audio from a microphone using the `cpal` library.
//! It supports listing available input devices and recording mono audio that can be used with
//! the pitch detection and audio processing modules.

use crate::audio::MonoAudio;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

/// Error type for audio recording operations
#[derive(Debug)]
pub enum RecordingError {
    /// No input device available
    NoInputDevice(String),
    /// Failed to get device configuration
    DeviceConfigError(String),
    /// Failed to build audio stream
    StreamError(String),
    /// Failed to record audio
    RecordError(String),
    /// Unsupported configuration
    UnsupportedConfig(String),
}

impl std::fmt::Display for RecordingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordingError::NoInputDevice(msg) => write!(f, "No input device: {}", msg),
            RecordingError::DeviceConfigError(msg) => write!(f, "Device config error: {}", msg),
            RecordingError::StreamError(msg) => write!(f, "Stream error: {}", msg),
            RecordingError::RecordError(msg) => write!(f, "Record error: {}", msg),
            RecordingError::UnsupportedConfig(msg) => write!(f, "Unsupported config: {}", msg),
        }
    }
}

impl std::error::Error for RecordingError {}

/// Information about an audio input device
#[derive(Debug, Clone)]
pub struct InputDevice {
    /// Name of the device
    pub name: String,
    /// Whether this is the default input device
    pub is_default: bool,
}

/// List all available audio input devices
///
/// # Returns
/// * `Ok(Vec<InputDevice>)` - List of available input devices
/// * `Err(RecordingError)` - Error getting device list
///
/// # Examples
/// ```no_run
/// use audio_utils::recording::list_input_devices;
///
/// let devices = list_input_devices().expect("Failed to get input devices");
/// for device in devices {
///     println!("Device: {} (default: {})", device.name, device.is_default);
/// }
/// ```
pub fn list_input_devices() -> Result<Vec<InputDevice>, RecordingError> {
    let host = cpal::default_host();
    
    let default_device_name = host.default_input_device()
        .and_then(|d| d.name().ok());
    
    let devices = host.input_devices()
        .map_err(|e| RecordingError::NoInputDevice(format!("Failed to enumerate devices: {}", e)))?;
    
    let mut result = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            let is_default = default_device_name.as_ref() == Some(&name);
            result.push(InputDevice {
                name,
                is_default,
            });
        }
    }
    
    Ok(result)
}

/// Record audio from the default input device for a specified duration
///
/// This function records mono audio from the system's default microphone.
/// The audio is automatically converted to mono if the input is stereo or multi-channel.
///
/// # Arguments
/// * `duration_secs` - Duration to record in seconds
///
/// # Returns
/// * `Ok(MonoAudio)` - Successfully recorded audio
/// * `Err(RecordingError)` - Error during recording
///
/// # Examples
/// ```no_run
/// use audio_utils::recording::record_from_microphone;
///
/// // Record 3 seconds of audio
/// let audio = record_from_microphone(3.0).expect("Failed to record");
/// println!("Recorded {} samples at {} Hz", audio.samples.len(), audio.sample_rate);
/// ```
pub fn record_from_microphone(duration_secs: f32) -> Result<MonoAudio, RecordingError> {
    let host = cpal::default_host();
    
    let device = host.default_input_device()
        .ok_or_else(|| RecordingError::NoInputDevice("No default input device found".to_string()))?;
    
    let config = device.default_input_config()
        .map_err(|e| RecordingError::DeviceConfigError(format!("Failed to get default config: {}", e)))?;
    
    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    
    // Calculate total samples needed
    let total_samples = (sample_rate as f32 * duration_secs) as usize;
    
    // Buffer to store recorded samples (will be converted to mono)
    let samples = Arc::new(Mutex::new(Vec::with_capacity(total_samples)));
    let samples_clone = Arc::clone(&samples);
    
    // Build the input stream based on the sample format
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            build_input_stream_f32(&device, &config.into(), samples_clone, channels)
        },
        cpal::SampleFormat::I16 => {
            build_input_stream_i16(&device, &config.into(), samples_clone, channels)
        },
        cpal::SampleFormat::U16 => {
            build_input_stream_u16(&device, &config.into(), samples_clone, channels)
        },
        sample_format => {
            return Err(RecordingError::UnsupportedConfig(
                format!("Unsupported sample format: {:?}", sample_format)
            ));
        }
    }?;
    
    // Start recording
    stream.play()
        .map_err(|e| RecordingError::StreamError(format!("Failed to start stream: {}", e)))?;
    
    // Wait for the specified duration
    std::thread::sleep(std::time::Duration::from_secs_f32(duration_secs));
    
    // Stop recording
    drop(stream);
    
    // Extract samples
    let recorded_samples = samples.lock()
        .map_err(|e| RecordingError::RecordError(format!("Failed to lock samples: {}", e)))?
        .clone();
    
    if recorded_samples.is_empty() {
        return Err(RecordingError::RecordError("No samples recorded".to_string()));
    }
    
    Ok(MonoAudio::new(recorded_samples, sample_rate))
}

/// Helper function to build an input stream for f32 samples
fn build_input_stream_f32(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
    channels: usize,
) -> Result<cpal::Stream, RecordingError> {
    let err_fn = |err| {
        eprintln!("Error in audio stream: {}", err);
    };
    
    let stream = device.build_input_stream(
        config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut samples_lock = samples.lock().unwrap();
            
            // Mix down to mono if needed
            for frame in data.chunks(channels) {
                // Average all channels to create mono
                let mono_sample: f32 = frame.iter().sum::<f32>() / channels as f32;
                samples_lock.push(mono_sample);
            }
        },
        err_fn,
        None,
    )
    .map_err(|e| RecordingError::StreamError(format!("Failed to build stream: {}", e)))?;
    
    Ok(stream)
}

/// Helper function to build an input stream for i16 samples
fn build_input_stream_i16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
    channels: usize,
) -> Result<cpal::Stream, RecordingError> {
    let err_fn = |err| {
        eprintln!("Error in audio stream: {}", err);
    };
    
    let stream = device.build_input_stream(
        config,
        move |data: &[i16], _: &cpal::InputCallbackInfo| {
            let mut samples_lock = samples.lock().unwrap();
            
            // Convert to f32 and mix down to mono if needed
            for frame in data.chunks(channels) {
                // Average all channels to create mono
                let mono_sample: f32 = frame.iter()
                    .map(|&s| s as f32 / i16::MAX as f32)
                    .sum::<f32>() / channels as f32;
                samples_lock.push(mono_sample);
            }
        },
        err_fn,
        None,
    )
    .map_err(|e| RecordingError::StreamError(format!("Failed to build stream: {}", e)))?;
    
    Ok(stream)
}

/// Helper function to build an input stream for u16 samples
fn build_input_stream_u16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
    channels: usize,
) -> Result<cpal::Stream, RecordingError> {
    let err_fn = |err| {
        eprintln!("Error in audio stream: {}", err);
    };
    
    let stream = device.build_input_stream(
        config,
        move |data: &[u16], _: &cpal::InputCallbackInfo| {
            let mut samples_lock = samples.lock().unwrap();
            
            // Convert to f32 and mix down to mono if needed
            for frame in data.chunks(channels) {
                // Average all channels to create mono
                let mono_sample: f32 = frame.iter()
                    .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                    .sum::<f32>() / channels as f32;
                samples_lock.push(mono_sample);
            }
        },
        err_fn,
        None,
    )
    .map_err(|e| RecordingError::StreamError(format!("Failed to build stream: {}", e)))?;
    
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_list_input_devices() {
        // This test may fail in CI environments without audio devices
        match list_input_devices() {
            Ok(devices) => {
                println!("Found {} input devices", devices.len());
                for device in devices {
                    println!("  - {} (default: {})", device.name, device.is_default);
                }
            },
            Err(e) => {
                println!("No input devices available (expected in CI): {}", e);
            }
        }
    }
    
    #[test]
    #[ignore] // Ignore by default as it requires a microphone
    fn test_record_from_microphone() {
        // Record 1 second of audio
        let result = record_from_microphone(1.0);
        
        match result {
            Ok(audio) => {
                println!("Recorded {} samples at {} Hz", audio.samples.len(), audio.sample_rate);
                assert!(!audio.samples.is_empty(), "Should have recorded samples");
                assert!(audio.sample_rate > 0, "Sample rate should be positive");
                
                // Check that we got roughly the right number of samples
                let expected_samples = audio.sample_rate as usize;
                let tolerance = expected_samples / 10; // 10% tolerance
                assert!(
                    audio.samples.len() >= expected_samples - tolerance &&
                    audio.samples.len() <= expected_samples + tolerance,
                    "Sample count {} should be close to {}",
                    audio.samples.len(),
                    expected_samples
                );
            },
            Err(e) => {
                panic!("Failed to record: {}", e);
            }
        }
    }
}
