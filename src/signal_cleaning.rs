use fundsp::hacker::*;
use rustfft::num_complex::Complex32;
use crate::signal_processing::compute_spectrum;
use crate::signal_processing::Spectrum;

/// Bandpass filter for human vocal range (default: 80Hz - 1200Hz) using fundsp crate
pub fn bandpass_vocal_range(samples: &[f32], _sample_rate: f32, low_hz: f32, high_hz: f32) -> Vec<f32> {
    // fundsp expects f64, and bandpass_hz takes (center_freq, Q)
    println!("[DEBUG] bandpassing vocal range");
    let mut filtered = Vec::with_capacity(samples.len());
    let center = (low_hz + high_hz) as f64 * 0.5;
    let bandwidth = (high_hz - low_hz) as f64;
    // Q = center / bandwidth
    let q = if bandwidth > 0.0 { center / bandwidth } else { 1.0 };
    let mut filter = bandpass_hz(center, q);
    for &x in samples {
        filtered.push(filter.filter_mono(x as f64) as f32);
    }
    filtered
}

/// Spectral gating using a background noise spectrum. If no spectrum is supplied, bandpass is used.
pub fn clean_signal_for_pitch(
    samples: &[f32],
    sample_rate: f32,
    noise_spectrum: Option<&Spectrum>,
    noise_threshold: Option<f32>
) -> Vec<f32> {
    if let Some(noise_spec) = noise_spectrum {
        let noise_threshold = noise_threshold.unwrap_or(1.2);
        // Spectral gating: FFT, attenuate bins below noise, IFFT
        let input = samples.to_vec();
        // Use compute_spectrum for magnitude spectrum
        let mut spectrum = compute_spectrum(&input);
        // Apply gating: if below noise spectrum, attenuate
        for (i, c) in spectrum.complex.iter_mut().enumerate() {
            let noise: f32 = if i < noise_spec.complex.len() { noise_spec.complex[i].norm() } else { 0.0 };
            if c.norm() < noise * noise_threshold {
                *c = Complex32::new(0.0, 0.0); // Attenuate to zero
            }
        }
        // Inverse FFT
        spectrum.to_time_domain()[..samples.len()].to_vec()
    } else {
        // Default: bandpass for vocal range
        bandpass_vocal_range(samples, sample_rate, 80.0, 1200.0)
    }
}

/// Extract background noise spectrum from the first 200-1500ms of a clip
pub fn estimate_noise_spectrum(samples: &[f32], sample_rate: f32) -> Spectrum {
    let start = (0.2 * sample_rate as f32) as usize;
    let end = Ord::min((1.5 * sample_rate as f32) as usize, samples.len());
    let noise_window = &samples[start..end];
    // Use compute_spectrum for noise window
    compute_spectrum(noise_window)
}
