//! Spectral Gating for Noise Reduction
//!
//! This module provides spectral gating functionality for noise reduction in audio signals.
//! Spectral gating works by:
//! 1. Transforming audio to frequency domain via FFT
//! 2. Comparing each frequency bin to a noise profile
//! 3. Attenuating bins that fall below a threshold relative to the noise floor
//! 4. Transforming back to time domain via inverse FFT
//!
//! The module is designed to support both batch processing and real-time streaming applications.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use audio_cleaning::spectral_gating::{SpectralGate, SpectralGateConfig};
//! use audio_cleaning::Spectrum;
//!
//! // Create a noise profile from a quiet section of audio
//! let noise_samples = vec![0.01, 0.02, -0.01, 0.0]; // noise reference
//! let noise_profile = Spectrum::from_waveform(&noise_samples);
//!
//! // Configure the spectral gate
//! let config = SpectralGateConfig {
//!     noise_threshold_db: 6.0, // Attenuate signals 6 dB below noise floor
//!     smoothing_window: 3,       // Smooth gating decisions across 3 frequency bins
//! };
//!
//! // Create the gate
//! let gate = SpectralGate::new(noise_profile, config);
//!
//! // Process audio
//! let audio_samples = vec![0.1, 0.2, -0.1, 0.05];
//! let cleaned = gate.process(&audio_samples);
//! ```
//!
//! ## Streaming/Real-time Usage
//!
//! For real-time applications, process audio in fixed-size chunks:
//!
//! ```
//! use audio_cleaning::spectral_gating::{SpectralGate, SpectralGateConfig};
//! use audio_cleaning::Spectrum;
//!
//! let noise_profile = Spectrum::from_waveform(&vec![0.01; 1024]);
//! let config = SpectralGateConfig::default();
//! let gate = SpectralGate::new(noise_profile, config);
//!
//! // Process chunks as they arrive
//! let chunk1 = vec![0.1; 1024];
//! let chunk2 = vec![0.2; 1024];
//!
//! let cleaned1 = gate.process(&chunk1);
//! let cleaned2 = gate.process(&chunk2);
//! ```

use rustfft::num_complex::Complex32;
use crate::Spectrum;

/// Configuration for spectral gating
#[derive(Debug, Clone)]
pub struct SpectralGateConfig {
    /// Threshold in dB below noise floor for attenuation.
    /// Signals below (noise_level * threshold_db) will be attenuated.
    /// Default: 6.0 dB (approximately 2x multiplier)
    pub noise_threshold_db: f32,
    
    /// Number of adjacent frequency bins to average for smoothing gating decisions.
    /// Higher values provide smoother transitions but less precise gating.
    /// Default: 1 (no smoothing)
    pub smoothing_window: usize,
}

impl Default for SpectralGateConfig {
    fn default() -> Self {
        Self {
            noise_threshold_db: 6.0,
            smoothing_window: 1,
        }
    }
}

/// Spectral gate for noise reduction
///
/// A spectral gate attenuates frequency components that fall below a threshold
/// relative to a noise profile. This is useful for removing background noise
/// while preserving the desired signal.
pub struct SpectralGate {
    noise_spectrum: Spectrum,
    noise_magnitudes: Vec<f32>,
    config: SpectralGateConfig,
}

impl SpectralGate {
    /// Create a new spectral gate with the given noise profile and configuration
    ///
    /// # Arguments
    /// * `noise_spectrum` - Reference noise spectrum to gate against
    /// * `config` - Configuration parameters for the gate
    ///
    /// # Returns
    /// A new `SpectralGate` instance
    pub fn new(noise_spectrum: Spectrum, config: SpectralGateConfig) -> Self {
        let noise_magnitudes = Self::compute_noise_magnitudes_static(&noise_spectrum, &config);
        Self {
            noise_spectrum,
            noise_magnitudes,
            config,
        }
    }

    /// Create a spectral gate with default configuration
    ///
    /// # Arguments
    /// * `noise_spectrum` - Reference noise spectrum to gate against
    pub fn with_defaults(noise_spectrum: Spectrum) -> Self {
        Self::new(noise_spectrum, SpectralGateConfig::default())
    }

    /// Process audio samples through the spectral gate
    ///
    /// This function:
    /// 1. Transforms input to frequency domain
    /// 2. Applies gating based on noise profile
    /// 3. Returns time-domain result
    ///
    /// # Arguments
    /// * `samples` - Input audio samples to process
    ///
    /// # Returns
    /// Cleaned audio samples with noise reduction applied
    pub fn process(&self, samples: &[f32]) -> Vec<f32> {
        if samples.is_empty() {
            return Vec::new();
        }

        // Convert threshold from dB to linear scale
        let threshold_multiplier = db_to_linear(self.config.noise_threshold_db);

        // Transform to frequency domain
        let mut spectrum = Spectrum::from_waveform(samples);

        // Apply spectral gating to each frequency bin
        self.apply_gate(&mut spectrum, threshold_multiplier);

        // Transform back to time domain and trim to original length
        let output = spectrum.to_time_domain();
        output[..samples.len()].to_vec()
    }

    /// Apply gating to a spectrum in-place
    fn apply_gate(&self, spectrum: &mut Spectrum, threshold_multiplier: f32) {
        for (i, complex_sample) in spectrum.complex.iter_mut().enumerate() {
            let noise_level = self.noise_magnitudes.get(i).copied().unwrap_or(0.0);
            let signal_magnitude = complex_sample.norm();
            
            // Attenuation threshold
            let threshold = noise_level * threshold_multiplier;
            
            if signal_magnitude < threshold {
                // Apply soft gating: gradually reduce gain
                let gain = if noise_level > 0.0 {
                    (signal_magnitude / threshold).max(0.0).min(1.0)
                } else {
                    1.0
                };
                *complex_sample = Complex32::new(
                    complex_sample.re * gain,
                    complex_sample.im * gain,
                );
            }
        }
    }

    /// Compute magnitude spectrum from noise profile with optional smoothing
    fn compute_noise_magnitudes_static(noise_spectrum: &Spectrum, config: &SpectralGateConfig) -> Vec<f32> {
        let magnitudes: Vec<f32> = noise_spectrum.complex
            .iter()
            .map(|c| c.norm())
            .collect();

        if config.smoothing_window <= 1 {
            return magnitudes;
        }

        // Apply moving average smoothing
        Self::smooth_magnitudes(&magnitudes, config.smoothing_window)
    }

    /// Apply moving average smoothing to magnitude spectrum
    fn smooth_magnitudes(magnitudes: &[f32], window_size: usize) -> Vec<f32> {
        let mut smoothed = Vec::with_capacity(magnitudes.len());
        let half_window = window_size / 2;

        for i in 0..magnitudes.len() {
            let start = i.saturating_sub(half_window);
            let end = (i + half_window + 1).min(magnitudes.len());
            let sum: f32 = magnitudes[start..end].iter().sum();
            let count = (end - start) as f32;
            smoothed.push(sum / count);
        }

        smoothed
    }

    /// Update the noise profile
    ///
    /// This allows adapting the gate to changing noise conditions in real-time applications.
    ///
    /// # Arguments
    /// * `noise_spectrum` - New noise profile
    pub fn update_noise_profile(&mut self, noise_spectrum: Spectrum) {
        self.noise_magnitudes = Self::compute_noise_magnitudes_static(&noise_spectrum, &self.config);
        self.noise_spectrum = noise_spectrum;
    }

    /// Get a reference to the current noise profile
    pub fn noise_profile(&self) -> &Spectrum {
        &self.noise_spectrum
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &SpectralGateConfig {
        &self.config
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: SpectralGateConfig) {
        self.noise_magnitudes = Self::compute_noise_magnitudes_static(&self.noise_spectrum, &config);
        self.config = config;
    }
}

/// Convert decibels to linear scale
///
/// Formula: linear = 10^(db/20)
fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectral_gate_empty_input() {
        let noise = Spectrum::from_waveform(&vec![0.01; 4]);
        let gate = SpectralGate::with_defaults(noise);
        let result = gate.process(&[]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_spectral_gate_preserves_length() {
        let noise = Spectrum::from_waveform(&vec![0.01; 8]);
        let gate = SpectralGate::with_defaults(noise);
        let input = vec![0.1, 0.2, -0.1, 0.05];
        let result = gate.process(&input);
        assert_eq!(result.len(), input.len());
    }

    #[test]
    fn test_spectral_gate_attenuates_low_signal() {
        // Create noise profile with moderate amplitude
        let noise_samples = vec![0.1; 16];
        let noise = Spectrum::from_waveform(&noise_samples);
        
        // Create signal much weaker than noise
        let weak_signal = vec![0.01; 16];
        
        let gate = SpectralGate::with_defaults(noise);
        let result = gate.process(&weak_signal);
        
        // The result should have lower energy than input due to attenuation
        let input_energy: f32 = weak_signal.iter().map(|x| x * x).sum();
        let output_energy: f32 = result.iter().map(|x| x * x).sum();
        
        assert!(
            output_energy < input_energy,
            "Weak signal should be attenuated. Input energy: {}, Output energy: {}",
            input_energy,
            output_energy
        );
    }

    #[test]
    fn test_spectral_gate_preserves_strong_signal() {
        // Create noise profile with low amplitude
        let noise_samples = vec![0.01; 16];
        let noise = Spectrum::from_waveform(&noise_samples);
        
        // Create signal much stronger than noise
        let strong_signal = vec![0.5; 16];
        
        let gate = SpectralGate::with_defaults(noise);
        let result = gate.process(&strong_signal);
        
        // The result should preserve most of the energy
        let input_energy: f32 = strong_signal.iter().map(|x| x * x).sum();
        let output_energy: f32 = result.iter().map(|x| x * x).sum();
        
        // Should preserve at least 50% of energy (accounting for FFT/IFFT processing)
        assert!(
            output_energy > 0.5 * input_energy,
            "Strong signal should be preserved. Input energy: {}, Output energy: {}",
            input_energy,
            output_energy
        );
    }

    #[test]
    fn test_db_to_linear_conversion() {
        // 0 dB = 1.0 linear
        assert!((db_to_linear(0.0) - 1.0).abs() < 1e-6);
        
        // 6 dB = ~2.0 linear
        assert!((db_to_linear(6.0) - 2.0).abs() < 0.01);
        
        // -6 dB = ~0.5 linear
        assert!((db_to_linear(-6.0) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_update_noise_profile() {
        let noise1 = Spectrum::from_waveform(&vec![0.01; 4]);
        let noise2 = Spectrum::from_waveform(&vec![0.02; 4]);
        
        let mut gate = SpectralGate::with_defaults(noise1);
        gate.update_noise_profile(noise2);
        
        // Verify that the noise profile was updated
        assert_eq!(gate.noise_profile().complex.len(), 4);
    }

    #[test]
    fn test_update_config() {
        let noise = Spectrum::from_waveform(&vec![0.01; 4]);
        let mut gate = SpectralGate::with_defaults(noise);
        
        let new_config = SpectralGateConfig {
            noise_threshold_db: 12.0,
            smoothing_window: 5,
        };
        
        gate.update_config(new_config);
        
        assert_eq!(gate.config().noise_threshold_db, 12.0);
        assert_eq!(gate.config().smoothing_window, 5);
    }

    #[test]
    fn test_smoothing_window() {
        let noise_samples = vec![0.01; 32];
        
        // Test with different smoothing window sizes
        let config_no_smoothing = SpectralGateConfig {
            noise_threshold_db: 6.0,
            smoothing_window: 1,
        };
        
        let config_with_smoothing = SpectralGateConfig {
            noise_threshold_db: 6.0,
            smoothing_window: 5,
        };
        
        let noise1 = Spectrum::from_waveform(&noise_samples);
        let noise2 = Spectrum::from_waveform(&noise_samples);
        
        let gate_no_smoothing = SpectralGate::new(noise1, config_no_smoothing);
        let gate_with_smoothing = SpectralGate::new(noise2, config_with_smoothing);
        
        // Both should process without error
        let input = vec![0.1; 32];
        let result1 = gate_no_smoothing.process(&input);
        let result2 = gate_with_smoothing.process(&input);
        
        assert_eq!(result1.len(), input.len());
        assert_eq!(result2.len(), input.len());
    }

    #[test]
    fn test_spectral_gate_with_sine_wave() {
        use std::f32::consts::PI;
        
        // Generate a 440 Hz sine wave
        let sample_rate = 8000.0;
        let frequency = 440.0;
        let duration = 0.1;
        let n_samples = (sample_rate * duration) as usize;
        
        let signal: Vec<f32> = (0..n_samples)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * PI * frequency * t).sin()
            })
            .collect();
        
        // Create low-amplitude noise profile
        let noise = Spectrum::from_waveform(&vec![0.01; n_samples]);
        
        let gate = SpectralGate::with_defaults(noise);
        let result = gate.process(&signal);
        
        // High-amplitude sine wave should be mostly preserved
        let input_energy: f32 = signal.iter().map(|x| x * x).sum();
        let output_energy: f32 = result.iter().map(|x| x * x).sum();
        
        assert!(
            output_energy > 0.5 * input_energy,
            "Clean sine wave should be preserved. Input energy: {}, Output energy: {}",
            input_energy,
            output_energy
        );
    }
}
