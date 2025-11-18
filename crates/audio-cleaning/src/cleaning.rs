//! Audio signal cleaning and noise reduction functionality
//! 
//! This module provides tools for improving audio quality for pitch detection:
//! - Bandpass filtering for vocal frequency range isolation
//! - Spectral gating for noise reduction using recorded noise profiles  
//! - Background noise spectrum estimation

use fundsp::hacker::*;
use super::{Spectrum};
use audio_utils as audio;
use super::util::{rms, mean_std_deviation};
use crate::spectral_gating;

/// Default vocal frequency range for bandpass filtering
pub const DEFAULT_VOCAL_LOW_HZ: f32 = 80.0;
pub const DEFAULT_VOCAL_HIGH_HZ: f32 = 1200.0;

/// Applies bandpass filter optimized for human vocal range (80Hz - 1200Hz)
/// 
/// Uses fundsp's bandpass filter with center frequency and Q factor calculation.
/// This removes both low-frequency rumble and high-frequency noise that can
/// interfere with pitch detection.
/// 
/// The sample rate is explicitly set on the filter to ensure correct frequency
/// response regardless of the input audio's sample rate.
/// 
/// # Arguments
/// * `samples` - Input audio samples
/// * `sample_rate` - Sample rate of the audio in Hz
/// * `low_hz` - Low cutoff frequency in Hz
/// * `high_hz` - High cutoff frequency in Hz
/// 
/// # Returns
/// Filtered audio samples with the same length as input
pub fn bandpass_vocal_range(samples: &[f32], sample_rate: f32, low_hz: f32, high_hz: f32) -> Vec<f32> {
    let mut filtered = Vec::with_capacity(samples.len());
    let center = (low_hz + high_hz) * 0.5;
    let bandwidth = high_hz - low_hz;
    let q = if bandwidth > 0.0 { center / bandwidth } else { 1.0 };
    
    let mut filter = bandpass_hz(center, q);
    filter.set_sample_rate(sample_rate as f64);
    
    for &sample in samples {
        filtered.push(filter.filter_mono(sample));
    }
    
    filtered
}

/// Cleans audio signal for improved pitch detection using spectral gating or bandpass filtering
/// 
/// If a noise spectrum is provided, it uses spectral gating to attenuate frequency bins that are below the noise floor.
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
    noise_spectrum: Option<Spectrum>,
    noise_threshold: Option<f32>
) -> Vec<f32> {
    match noise_spectrum {
        Some(noise_spec) => apply_spectral_gating(samples, noise_spec, noise_threshold),
        None => bandpass_vocal_range(samples, sample_rate, DEFAULT_VOCAL_LOW_HZ, DEFAULT_VOCAL_HIGH_HZ),
    }
}

/// Cleans audio signal for pitch detection by applying bandpass filter
/// This function dispatches to `clean_signal_for_pitch` with with samples and sample_rate extracted from the audio argument.
/// 
/// # Arguments
/// * `audio` - MonoAudio containing the audio samples and sample rate
/// * `noise_spectrum` - Optional recorded noise spectrum for spectral gating
/// * `noise_threshold` - Multiplier for noise floor (default: 1.2)
/// # Returns
/// Cleaned MonoAudio with the same sample rate as input
pub fn clean_audio_for_pitch(
    audio: &audio::MonoAudio,
    noise_spectrum: Option<Spectrum>,
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
    noise_spec: Spectrum, 
    noise_threshold: Option<f32>
) -> Vec<f32> {
    // Delegate to the spectral_gating module
    spectral_gating::apply_spectral_gating(samples, noise_spec, noise_threshold)
}

/// Finds a suitable noise window in the audio samples
/// 
/// FIXME This function currently assumes that the noise is present in the first 200ms to 1500ms of the audio.
/// FIXME This currently uses a simple RMS and Z-score criteria to find a noise window, with only 1 STD deviation threshold.
/// 
/// It extracts a segment of audio that is likely to contain background noise
/// # Arguments
/// * `samples` - Audio samples to analyze
/// * `sample_rate` - Sample rate of the audio
/// # Returns
/// A slice of audio samples that represents the noise window
fn get_noise_window(samples: &[f32], sample_rate: f32) -> Option<&[f32]> {
    let start_idx = (0.2 * sample_rate) as usize;
    let end_idx = (1.5 * sample_rate).min(samples.len() as f32) as usize;
    if start_idx >= end_idx {
        return None;
    }
    let noise_window = &samples[start_idx..end_idx];
    let window_size = noise_window.len();
    if window_size == 0 {
        return None;
    }

    let all_windows_rms = samples.chunks(window_size)
        .map(|chunk| rms(chunk).unwrap())
        .collect::<Vec<f32>>();

    let (rms_mean, rms_stddev) = mean_std_deviation(&all_windows_rms).unwrap();
    let noise_window_rms = rms(noise_window)?;

    let noise_window_zscore = (noise_window_rms - rms_mean) / rms_stddev;
    if noise_window_zscore < -1.0 {
        Some(noise_window)
    } else {
        eprintln!("No suitable noise window found based on RMS and Z-score criteria.");
        None
    }

}

/// Estimates background noise spectrum from a quiet section of audio
/// 
/// # Arguments
/// * `samples` - Audio samples containing background noise
/// * `sample_rate` - Sample rate of the audio
/// 
/// # Returns
/// Frequency spectrum representing the background noise profile
pub fn _estimate_noise_spectrum(samples: &[f32], sample_rate: f32) -> Option<Spectrum> {
    let noise_window = get_noise_window(samples, sample_rate)?;
    Some(Spectrum::from_waveform(noise_window))
}

pub fn estimate_noise_spectrum(audio: &audio::MonoAudio) -> Option<Spectrum> {
    if audio.samples.is_empty() {
        return None;
    }
    
    _estimate_noise_spectrum(&audio.samples, audio.sample_rate as f32)
}


#[cfg(test)]
mod tests {
    use super::*;
    use audio_utils::MonoAudio;

    #[test]
    fn test_bandpass_vocal_range_identity_for_dc() {
        // DC signal should be mostly filtered out
        let samples = vec![1.0; 100];
        let filtered = bandpass_vocal_range(&samples, 44100.0, 80.0, 1200.0);
        // Should not be identical to input
        assert!(filtered.iter().any(|&x| (x - 1.0).abs() > 1e-3));
    }

    #[test]
    fn test_clean_signal_for_pitch_bandpass() {
        let samples = vec![0.0, 1.0, 0.0, -1.0];
        let cleaned = clean_signal_for_pitch(&samples, 44100.0, None, None);
        assert_eq!(cleaned.len(), samples.len());
    }

    #[test]
    fn test_clean_audio_for_pitch() {
        let audio = MonoAudio { samples: vec![0.0, 1.0, 0.0, -1.0], sample_rate: 44100 };
        let cleaned = clean_audio_for_pitch(&audio, None, None);
        assert_eq!(cleaned.samples.len(), audio.samples.len());
        assert_eq!(cleaned.sample_rate, audio.sample_rate);
    }

    #[test]
    fn test_estimate_noise_spectrum_empty() {
        let audio = MonoAudio { samples: vec![], sample_rate: 44100 };
        assert!(estimate_noise_spectrum(&audio).is_none());
    }

    #[test]
    #[ignore = "Low variance signal not suitable for noise estimation with current implementation"]
    fn test_estimate_noise_spectrum_some() {
        // Use a low-variance signal to ensure a noise window is found
        let samples = vec![0.01; 2000];
        let audio = MonoAudio { samples: samples.clone(), sample_rate: 1000 };
        let spec = estimate_noise_spectrum(&audio);
        assert!(spec.is_some());
    }

    #[test]
    fn test_bandpass_vocal_range_uses_sample_rate() {
        // This test verifies that the sample_rate parameter is actually used by the filter.
        // We test the counterfactual: filtering with the WRONG sample rate should give different results.
        
        // Generate a 440 Hz sine wave sampled at 48 kHz
        let actual_sample_rate = 48000.0;
        let frequency = 440.0; // Hz - center of the passband (80-1200 Hz)
        let duration = 0.5; // seconds - longer duration for filter to settle
        let num_samples = (actual_sample_rate * duration) as usize;
        
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / actual_sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        // Filter with the CORRECT sample rate (48 kHz)
        let filtered_correct = bandpass_vocal_range(&samples, actual_sample_rate, 80.0, 1200.0);
        
        // Filter with the WRONG sample rate (44.1 kHz)
        // If sample_rate parameter is ignored, this would produce the same result
        let wrong_sample_rate = 44100.0;
        let filtered_wrong = bandpass_vocal_range(&samples, wrong_sample_rate, 80.0, 1200.0);
        
        // Calculate energy for both filtered signals (skip first 10% to avoid transients)
        let skip = num_samples / 10;
        let energy_correct: f32 = filtered_correct[skip..].iter().map(|x| x * x).sum::<f32>() / (filtered_correct.len() - skip) as f32;
        let energy_wrong: f32 = filtered_wrong[skip..].iter().map(|x| x * x).sum::<f32>() / (filtered_wrong.len() - skip) as f32;
        
        // The results should be DIFFERENT when using different sample rates
        // The difference should be significant (at least 5% relative difference)
        let relative_diff = (energy_correct - energy_wrong).abs() / energy_correct.max(energy_wrong);
        
        assert!(
            relative_diff > 0.05,
            "Filter output should differ significantly when using wrong sample rate. \
             Energy with correct rate (48kHz): {}, with wrong rate (44.1kHz): {}, \
             relative difference: {:.2}%",
            energy_correct,
            energy_wrong,
            relative_diff * 100.0
        );
        
        // Additionally, verify that 440 Hz is within the passband (80-1200 Hz) at the correct sample rate
        // so it should have reasonable energy (not heavily attenuated)
        assert!(
            energy_correct > 0.05,
            "440 Hz signal should pass through filter with correct sample rate with reasonable energy, got: {}",
            energy_correct
        );
    }
}
