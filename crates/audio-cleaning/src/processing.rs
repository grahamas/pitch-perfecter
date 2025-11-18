//! # Signal Processing
//! This module provides signal processing functionalities for audio data.
//! It includes functionalities for cleaning audio signals, computing spectra, and spectrograms.
//! 


/// Find the index and value of the peak in a signal
/// FIXME There must be a better way to do this (library function, or more robust)
pub fn find_peak(signal: &[f32]) -> Option<(usize, f32)> {
    signal
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, &v)| (i, v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Spectrum;
    use std::f32::consts::PI;

    // Helper: generate a sine wave
    fn sine_wave(freq: f32, sample_rate: f32, len: usize) -> Vec<f32> {
        (0..len)
            .map(|i| (2.0 * PI * freq * i as f32 / sample_rate).sin())
            .collect()
    }

    #[test]
    fn test_spectrum_detects_sine() {
        let sample_rate = 1024.0;
        let freq = 128.0;
        let len = 1024;
        let signal = sine_wave(freq, sample_rate, len);
        let spectrum = Spectrum::from_waveform(&signal);
        // The peak should be at bin k = freq * N / sample_rate
        let k = (freq * len as f32 / sample_rate).round() as usize;
        let max_bin = spectrum.magnitudes().iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
        assert_eq!(k, max_bin);
    }

    #[test]
    fn test_spectrogram_detects_two_tones() {
        let sample_rate = 1000.0;
        let len = 1000;
        let mut signal = sine_wave(100.0, sample_rate, len);
        let signal2 = sine_wave(200.0, sample_rate, len);
        for (a, b) in signal.iter_mut().zip(signal2.iter()) {
            *a += b;
        }
        let spec = Spectrum::from_waveform(&signal);
        let k1 = (100.0 * len as f32 / sample_rate).round() as usize;
        let k2 = (200.0 * len as f32 / sample_rate).round() as usize;
        let max1 = spec.magnitudes()[k1];
        let max2 = spec.magnitudes()[k2];
        // Both peaks should be prominent
        assert!(max1 > 0.5 * spec.magnitudes().iter().cloned().fold(0.0, f32::max));
        assert!(max2 > 0.5 * spec.magnitudes().iter().cloned().fold(0.0, f32::max));
    }

    #[test]
    fn test_find_peak() {
        let waveform = vec![0.1, 0.5, 2.0, 0.3, 0.2];
        let peak = find_peak(&waveform);
        assert_eq!(peak, Some((2, 2.0)));
    }
}
