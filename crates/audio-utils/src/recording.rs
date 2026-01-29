//! Audio Recording Module
//!
//! This module provides functions for recording audio from a microphone using the `cpal` library.
//! It supports listing available input devices and recording mono audio that can be used with
//! the pitch detection and audio processing modules.

use crate::audio::MonoAudio;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Delay in milliseconds to wait after successfully pausing a stream before dropping it.
/// 
/// ALSA processes pause commands asynchronously. This delay gives ALSA time to complete
/// the state transition before the stream is dropped, preventing panics.
/// 
/// Note: cpal does not provide a way to query stream state, so we cannot verify that the
/// pause has completed. This delay is based on typical ALSA behavior.
const ALSA_PAUSE_DELAY_MS: u64 = 10;

/// Attempt to pause a stream and wait for the pause to complete.
/// 
/// Since cpal doesn't expose stream state, we cannot actually verify that ALSA has
/// completed processing the pause command. Instead, we:
/// 1. Check that pause() succeeds (command was accepted)
/// 2. Wait a conservative amount of time for ALSA to process it
/// 
/// Returns true if the pause command was successful and we waited appropriately.
fn pause_and_await_completion(stream: &cpal::Stream) -> bool {
    match stream.pause() {
        Ok(()) => {
            // Pause command was accepted by cpal, now wait for ALSA to process it
            std::thread::sleep(Duration::from_millis(ALSA_PAUSE_DELAY_MS));
            true
        }
        Err(e) => {
            // Pause failed - log and indicate we didn't wait
            eprintln!("Failed to pause stream: {}. Stream will be dropped immediately.", e);
            false
        }
    }
}

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

/// A microphone recorder that can be started and stopped on demand.
///
/// This is the preferred API for interactive recording scenarios where the user
/// controls when to start and stop recording (e.g., with keyboard shortcuts or buttons).
///
/// # Examples
/// ```no_run
/// use audio_utils::recording::MicrophoneRecorder;
///
/// // Create and start recording
/// let mut recorder = MicrophoneRecorder::new().expect("Failed to create recorder");
/// recorder.start().expect("Failed to start recording");
///
/// // ... user interaction (e.g., wait for key press) ...
///
/// // Stop and get the recorded audio
/// let audio = recorder.stop().expect("Failed to stop recording");
/// println!("Recorded {} samples at {} Hz", audio.samples.len(), audio.sample_rate);
/// ```
pub struct MicrophoneRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    stream: Option<cpal::Stream>,
}

impl MicrophoneRecorder {
    /// Create a new microphone recorder using the default input device.
    ///
    /// The recorder is created in a stopped state. Call `start()` to begin recording.
    ///
    /// # Returns
    /// * `Ok(MicrophoneRecorder)` - Successfully created recorder
    /// * `Err(RecordingError)` - Error setting up the recorder
    pub fn new() -> Result<Self, RecordingError> {
        let host = cpal::default_host();
        
        let device = host.default_input_device()
            .ok_or_else(|| RecordingError::NoInputDevice("No default input device found".to_string()))?;
        
        let config = device.default_input_config()
            .map_err(|e| RecordingError::DeviceConfigError(format!("Failed to get default config: {}", e)))?;
        
        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        
        let samples = Arc::new(Mutex::new(Vec::new()));
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
        
        Ok(MicrophoneRecorder {
            samples,
            sample_rate,
            stream: Some(stream),
        })
    }
    
    /// Start recording audio from the microphone.
    ///
    /// If recording is already in progress, this does nothing.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully started recording
    /// * `Err(RecordingError)` - Error starting the stream
    pub fn start(&mut self) -> Result<(), RecordingError> {
        if let Some(stream) = &self.stream {
            stream.play()
                .map_err(|e| RecordingError::StreamError(format!("Failed to start stream: {}", e)))?;
        }
        Ok(())
    }
    
    /// Stop recording and return the recorded audio.
    ///
    /// This consumes the recorder and returns all audio recorded since `start()` was called.
    /// The stream is automatically stopped and cleaned up.
    ///
    /// # Returns
    /// * `Ok(MonoAudio)` - Successfully recorded audio
    /// * `Err(RecordingError)` - Error stopping or retrieving the audio
    pub fn stop(mut self) -> Result<MonoAudio, RecordingError> {
        // Cleanup stream safely to avoid ALSA panic
        if let Some(stream) = self.stream.take() {
            Self::cleanup_stream(stream);
        }
        
        // Extract samples
        let recorded_samples = self.samples.lock()
            .map_err(|e| RecordingError::RecordError(format!("Failed to lock samples: {}", e)))?
            .clone();
        
        if recorded_samples.is_empty() {
            return Err(RecordingError::RecordError("No samples recorded".to_string()));
        }
        
        Ok(MonoAudio::new(recorded_samples, self.sample_rate))
    }
    
    /// Pause recording without stopping the stream.
    ///
    /// Audio data will not be captured while paused. Call `start()` to resume.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully paused recording
    /// * `Err(RecordingError)` - Error pausing the stream
    pub fn pause(&mut self) -> Result<(), RecordingError> {
        if let Some(stream) = &self.stream {
            stream.pause()
                .map_err(|e| RecordingError::StreamError(format!("Failed to pause stream: {}", e)))?;
        }
        Ok(())
    }
    
    /// Check if the recorder is currently recording.
    ///
    /// Note: This returns `true` if the stream exists and was started, but may not
    /// perfectly reflect the actual hardware state.
    pub fn is_recording(&self) -> bool {
        self.stream.is_some()
    }
    
    /// Helper method to safely cleanup a stream by pausing it and waiting before dropping.
    /// This prevents ALSA panics by giving the backend time to process the pause command.
    fn cleanup_stream(stream: cpal::Stream) {
        // Attempt to pause and wait for ALSA to process the command
        pause_and_await_completion(&stream);
        
        // Now it's safe to drop the stream
        drop(stream);
    }
}

impl Drop for MicrophoneRecorder {
    fn drop(&mut self) {
        // Ensure stream is properly cleaned up even if stop() wasn't called
        if let Some(stream) = self.stream.take() {
            Self::cleanup_stream(stream);
        }
    }
}

/// Record audio from the default input device for a specified duration
///
/// This is a convenience function for simple use cases where you want to record
/// a fixed duration of audio. For interactive recording (start/stop on demand),
/// use `MicrophoneRecorder` instead.
///
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
    
    // Pause the stream before dropping to avoid ALSA panic
    pause_and_await_completion(&stream);
    
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
    
    #[test]
    #[ignore] // Ignore by default as it requires a microphone
    fn test_microphone_recorder_toggle() {
        // Create recorder
        let mut recorder = MicrophoneRecorder::new().expect("Failed to create recorder");
        
        // Start recording
        recorder.start().expect("Failed to start recording");
        
        // Record for 1 second
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        // Stop and get audio
        let audio = recorder.stop().expect("Failed to stop recording");
        
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
    }
}
