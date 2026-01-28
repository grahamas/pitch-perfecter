use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig, Sample};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use audio_utils::LatencyMetrics;
use crate::pitch_processor::{PitchProcessor, PitchResult};
use pitch_detection_utils::ThreadSafeYinDetector;

/// Delay in milliseconds to wait after pausing a stream before dropping it.
/// This gives ALSA time to process the pause command and transition to a stable state.
const ALSA_PAUSE_DELAY_MS: u64 = 10;

pub struct AudioRecorder {
    stream: Option<Stream>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            stream: None,
        }
    }
    
    pub fn start(
        &mut self,
        pitch_sender: Sender<PitchResult>,
        power_threshold: f32,
        clarity_threshold: f32,
        window_size: usize,
        hop_size: usize,
        enable_bandpass: bool,
        enable_spectral_gating: bool,
        save_to_file: bool,
        save_path: String,
    ) -> Result<(), String> {
        if self.stream.is_some() {
            return Err("Already recording".to_string());
        }
        
        // Get the default host and input device
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device available")?;
        
        // Get the default input config
        let config = device.default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;
        
        // Create the stream based on sample format
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.build_stream::<f32>(
                &device,
                &config.into(),
                pitch_sender,
                power_threshold,
                clarity_threshold,
                window_size,
                hop_size,
                enable_bandpass,
                enable_spectral_gating,
                save_to_file,
                save_path,
            )?,
            cpal::SampleFormat::I16 => self.build_stream::<i16>(
                &device,
                &config.into(),
                pitch_sender,
                power_threshold,
                clarity_threshold,
                window_size,
                hop_size,
                enable_bandpass,
                enable_spectral_gating,
                save_to_file,
                save_path,
            )?,
            cpal::SampleFormat::U16 => self.build_stream::<u16>(
                &device,
                &config.into(),
                pitch_sender,
                power_threshold,
                clarity_threshold,
                window_size,
                hop_size,
                enable_bandpass,
                enable_spectral_gating,
                save_to_file,
                save_path,
            )?,
            sample_format => return Err(format!("Unsupported sample format: {:?}", sample_format)),
        };
        
        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;
        
        self.stream = Some(stream);
        
        Ok(())
    }
    
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(stream) = self.stream.take() {
            Self::cleanup_stream(stream);
        }
        Ok(())
    }
    
    /// Helper method to safely cleanup a stream by pausing it and waiting before dropping.
    /// This prevents ALSA panics by giving the backend time to process the pause command.
    fn cleanup_stream(stream: Stream) {
        let _ = stream.pause();
        // Give ALSA time to process the pause command
        std::thread::sleep(Duration::from_millis(ALSA_PAUSE_DELAY_MS));
        drop(stream);
    }
    
    fn build_stream<T>(
        &self,
        device: &Device,
        config: &StreamConfig,
        pitch_sender: Sender<PitchResult>,
        power_threshold: f32,
        clarity_threshold: f32,
        window_size: usize,
        hop_size: usize,
        enable_bandpass: bool,
        enable_spectral_gating: bool,
        save_to_file: bool,
        save_path: String,
    ) -> Result<Stream, String>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let sample_rate = config.sample_rate.0;
        let channels = config.channels as usize;
        
        // Create circular buffer for audio samples
        let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        
        // Setup file writer if saving is enabled
        let wav_writer = if save_to_file {
            let spec = hound::WavSpec {
                channels: 1, // We convert to mono
                sample_rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            };
            
            match hound::WavWriter::create(&save_path, spec) {
                Ok(writer) => Some(Arc::new(Mutex::new(writer))),
                Err(e) => {
                    eprintln!("Failed to create WAV file: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        let buffer_clone = Arc::clone(&audio_buffer);
        
        // Process audio in chunks
        let err_fn = |err| eprintln!("Stream error: {}", err);
        
        let stream = device.build_input_stream(
            config,
            move |data: &[T], _callback_info: &cpal::InputCallbackInfo| {
                // Create latency metrics and capture callback timestamp
                let latency = LatencyMetrics::with_callback_timestamp(Instant::now());
                
                // Create detector locally in the audio thread
                // This avoids Send issues with Rc in the detector
                thread_local! {
                    static DETECTOR: std::cell::RefCell<Option<ThreadSafeYinDetector>> = std::cell::RefCell::new(None);
                }
                
                DETECTOR.with(|detector_cell| {
                    let mut detector = detector_cell.borrow_mut();
                    if detector.is_none() {
                        *detector = Some(ThreadSafeYinDetector::new(
                            power_threshold,
                            clarity_threshold,
                            window_size,
                            hop_size,
                        ));
                    }
                    let detector = detector.as_mut().unwrap();
                // Convert samples to f32 and mix to mono
                let mono_samples: Vec<f32> = if channels == 1 {
                    data.iter()
                        .map(|&s| f32::from_sample(s))
                        .collect()
                } else {
                    // Mix stereo to mono by averaging channels
                    data.chunks_exact(channels)
                        .map(|frame| {
                            let sum: f32 = frame.iter()
                                .map(|&s| f32::from_sample(s))
                                .sum();
                            sum / channels as f32
                        })
                        .collect()
                };
                
                // Save to file if enabled
                if let Some(ref writer) = wav_writer {
                    if let Ok(mut w) = writer.lock() {
                        for &sample in &mono_samples {
                            let _ = w.write_sample(sample);
                        }
                    }
                }
                
                    // Add to buffer
                    if let Ok(mut buffer) = buffer_clone.lock() {
                        buffer.extend_from_slice(&mono_samples);
                        
                        // Process when we have enough samples for pitch detection
                        // Use window_size instead of BUFFER_SIZE to match detector expectations
                        while buffer.len() >= window_size {
                            // Take exactly window_size samples for processing
                            let samples_to_process: Vec<f32> = buffer.drain(..window_size).collect();
                            
                            // Process pitch detection directly on audio thread
                            // Clone latency metrics for this chunk
                            if let Some(pitch_result) = PitchProcessor::process_audio_chunk(
                                detector,
                                samples_to_process,
                                sample_rate,
                                enable_bandpass,
                                enable_spectral_gating,
                                latency.clone(),
                            ) {
                                // Send result to main thread
                                let _ = pitch_sender.send(pitch_result);
                            }
                        }
                    }
                });
            },
            err_fn,
            None,
        ).map_err(|e| format!("Failed to build input stream: {}", e))?;
        
        Ok(stream)
    }
}

impl Drop for AudioRecorder {
    fn drop(&mut self) {
        // Ensure stream is properly cleaned up even if stop() wasn't called
        if let Some(stream) = self.stream.take() {
            Self::cleanup_stream(stream);
        }
    }
}
