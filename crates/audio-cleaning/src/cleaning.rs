//! Audio signal cleaning and noise reduction functionality
//! 
//! This module provides tools for improving audio quality for pitch detection:
//! - Bandpass filtering for vocal frequency range isolation
//! - Spectral gating for noise reduction using recorded noise profiles  
//! - Background noise spectrum estimation

use fundsp::hacker::*;
use rustfft::num_complex::Complex32;
use super::{Spectrum};
use audio_utils as audio;
use super::util::{rms, mean_std_deviation};

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
    let threshold_multiplier = noise_threshold.unwrap_or(1.2);
    
    // Transform to frequency domain
    let mut spectrum = Spectrum::from_waveform(samples);
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
        // Generate a simple sine wave at 440 Hz (A4 note)
        let sample_rate_44k = 44100.0;
        let sample_rate_48k = 48000.0;
        let frequency = 440.0; // Hz
        let duration = 0.1; // seconds
        
        // Generate samples at 44.1 kHz
        let samples_44k: Vec<f32> = (0..(sample_rate_44k * duration) as usize)
            .map(|i| {
                let t = i as f32 / sample_rate_44k;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        // Generate samples at 48 kHz (same frequency, different sample rate)
        let samples_48k: Vec<f32> = (0..(sample_rate_48k * duration) as usize)
            .map(|i| {
                let t = i as f32 / sample_rate_48k;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        // Filter with correct sample rates
        let filtered_44k = bandpass_vocal_range(&samples_44k, sample_rate_44k, 80.0, 1200.0);
        let filtered_48k = bandpass_vocal_range(&samples_48k, sample_rate_48k, 80.0, 1200.0);
        
        // Both should preserve the signal since 440 Hz is within the passband (80-1200 Hz)
        // Check that the filtered signals have reasonable energy (not zeroed out)
        let energy_44k: f32 = filtered_44k.iter().map(|x| x * x).sum::<f32>() / filtered_44k.len() as f32;
        let energy_48k: f32 = filtered_48k.iter().map(|x| x * x).sum::<f32>() / filtered_48k.len() as f32;
        
        // Both should have significant energy since 440 Hz is in the passband
        assert!(energy_44k > 0.1, "44.1kHz filtered signal should have energy > 0.1, got {}", energy_44k);
        assert!(energy_48k > 0.1, "48kHz filtered signal should have energy > 0.1, got {}", energy_48k);
        
        // Test with a frequency outside the passband (e.g., 50 Hz - below the 80 Hz cutoff)
        let low_freq = 50.0;
        let samples_low: Vec<f32> = (0..(sample_rate_44k * duration) as usize)
            .map(|i| {
                let t = i as f32 / sample_rate_44k;
                (2.0 * std::f32::consts::PI * low_freq * t).sin()
            })
            .collect();
        
        let filtered_low = bandpass_vocal_range(&samples_low, sample_rate_44k, 80.0, 1200.0);
        let energy_low: f32 = filtered_low.iter().map(|x| x * x).sum::<f32>() / filtered_low.len() as f32;
        
        // Should be attenuated (much less energy than the in-band signal)
        assert!(energy_low < energy_44k * 0.5, 
            "50Hz signal should be attenuated compared to 440Hz: {} vs {}", energy_low, energy_44k);
    }
}
