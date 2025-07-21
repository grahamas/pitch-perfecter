/// FIXME file needs refinement after AI

//! Audio signal cleaning and noise reduction functionality
//! 
//! This module provides tools for improving audio quality for pitch detection:
//! - Bandpass filtering for vocal frequency range isolation
//! - Spectral gating for noise reduction using recorded noise profiles  
//! - Background noise spectrum estimation

use fundsp::hacker::*;
use rustfft::num_complex::Complex32;
use crate::signal_processing::{compute_spectrum, Spectrum};

/// Default vocal frequency range for bandpass filtering
pub const DEFAULT_VOCAL_LOW_HZ: f32 = 80.0;
pub const DEFAULT_VOCAL_HIGH_HZ: f32 = 1200.0;

/// Applies bandpass filter optimized for human vocal range (80Hz - 1200Hz)
/// 
/// Uses fundsp's bandpass filter with center frequency and Q factor calculation.
/// This removes both low-frequency rumble and high-frequency noise that can
/// interfere with pitch detection.
/// 
/// FIXME This function currently does not use the sample rate parameter,
/// Obviously it needs the sample rate to properly calculate the filter coefficients.
/// 
/// # Arguments
/// * `samples` - Input audio samples
/// * `_sample_rate` - Sample rate (currently unused but kept for API consistency)
/// * `low_hz` - Low cutoff frequency in Hz
/// * `high_hz` - High cutoff frequency in Hz
/// 
/// # Returns
/// Filtered audio samples with the same length as input
pub fn bandpass_vocal_range(samples: &[f32], _sample_rate: f32, low_hz: f32, high_hz: f32) -> Vec<f32> {
    let mut filtered = Vec::with_capacity(samples.len());
    let center = (low_hz + high_hz) as f64 * 0.5;
    let bandwidth = (high_hz - low_hz) as f64;
    let q = if bandwidth > 0.0 { center / bandwidth } else { 1.0 };
    
    let mut filter = bandpass_hz(center, q);
    
    for &sample in samples {
        filtered.push(filter.filter_mono(sample as f64) as f32);
    }
    
    filtered
}

/// Cleans audio signal for improved pitch detection using spectral gating or bandpass filtering
/// 
/// This is the main entry point for signal cleaning. If a noise spectrum is provided,
/// it uses spectral gating to attenuate frequency bins that are below the noise floor.
/// Otherwise, it falls back to bandpass filtering for the vocal range.
/// 
/// # Arguments
/// * `samples` - Input audio samples to clean
/// * `sample_rate` - Sample rate of the audio
/// * `noise_spectrum` - Optional recorded noise spectrum for spectral gating
/// * `noise_threshold` - Multiplier for noise floor (default: 1.2)
/// 
/// # Returns
/// Cleaned audio samples suitable for pitch detection
pub fn clean_signal_for_pitch(
    samples: &[f32],
    sample_rate: f32,
    noise_spectrum: Option<&Spectrum>,
    noise_threshold: Option<f32>
) -> Vec<f32> {
    match noise_spectrum {
        Some(noise_spec) => apply_spectral_gating(samples, noise_spec, noise_threshold),
        None => bandpass_vocal_range(samples, sample_rate, DEFAULT_VOCAL_LOW_HZ, DEFAULT_VOCAL_HIGH_HZ),
    }
}
pub fn clean_audio_for_pitch(
    audio: &audio::MonoAudio,
    noise_spectrum: Option<&Spectrum>,
    noise_threshold: Option<f32>
) -> audio::MonoAudio {
    let cleaned_samples = clean_signal_for_pitch(
        &audio.samples,
        audio.sample_rate as f32,
        noise_spectrum,
        noise_threshold
    );
    
    audio::MonoAudio {
        samples: cleaned_samples,
        sample_rate: audio.sample_rate,
    }
}

/// Applies spectral gating using a recorded noise spectrum
/// 
/// This advanced noise reduction technique:
/// 1. Transforms audio to frequency domain via FFT
/// 2. Compares each frequency bin to the noise spectrum  
/// 3. Attenuates bins that fall below noise_threshold * noise_level
/// 4. Transforms back to time domain via inverse FFT
/// 
/// # Arguments
/// * `samples` - Input audio samples
/// * `noise_spec` - Reference noise spectrum to gate against
/// * `noise_threshold` - Multiplier for noise floor (default: 1.2)
/// 
/// # Returns
/// Noise-gated audio samples
fn apply_spectral_gating(
    samples: &[f32], 
    noise_spec: &Spectrum, 
    noise_threshold: Option<f32>
) -> Vec<f32> {
    let threshold_multiplier = noise_threshold.unwrap_or(1.2);
    
    // Transform to frequency domain
    let mut spectrum = compute_spectrum(samples);
    
    // Apply spectral gating to each frequency bin
    for (i, complex_sample) in spectrum.complex.iter_mut().enumerate() {
        let noise_level = noise_spec.complex
            .get(i)
            .map(|c| c.norm())
            .unwrap_or(0.0);
            
        if complex_sample.norm() < noise_level * threshold_multiplier {
            *complex_sample = Complex32::new(0.0, 0.0);
        }
    }
    
    // Transform back to time domain and trim to original length
    spectrum.to_time_domain()[..samples.len()].to_vec()
}

/// Estimates background noise spectrum from a quiet section of audio
/// 
/// Analyzes audio between 200ms and 1500ms to capture background noise
/// characteristics without including speech content that typically starts
/// after the first few hundred milliseconds.
/// 
/// # Arguments
/// * `samples` - Audio samples containing background noise
/// * `sample_rate` - Sample rate of the audio
/// 
/// # Returns
/// Frequency spectrum representing the background noise profile
pub fn _estimate_noise_spectrum(samples: &[f32], sample_rate: f32) -> Spectrum {
    let start_idx = (0.2 * sample_rate) as usize;
    let end_idx = (1.5 * sample_rate).min(samples.len() as f32) as usize;
    
    let noise_window = &samples[start_idx..end_idx];
    compute_spectrum(noise_window)
}

pub fn estimate_noise_spectrum(audio: &audio::MonoAudio) -> Option<Spectrum> {
    if audio.samples.is_empty() {
        return None;
    }
    
    Some(_estimate_noise_spectrum(&audio.samples, audio.sample_rate as f32))
}
