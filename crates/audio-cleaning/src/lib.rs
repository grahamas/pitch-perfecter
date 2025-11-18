//! Audio Cleaning
//!
//! This crate provides audio preprocessing and cleaning operations including:
//! - Bandpass filtering for vocal frequency range isolation
//! - Spectral gating for noise reduction
//! - Background noise spectrum estimation
//!
//! These operations are designed to improve audio quality for pitch detection
//! and other audio analysis tasks.

pub mod cleaning;
pub mod processing;
pub mod types;
mod util;

pub use cleaning::{
    bandpass_vocal_range, clean_audio_for_pitch, clean_signal_for_pitch, estimate_noise_spectrum,
    DEFAULT_VOCAL_HIGH_HZ, DEFAULT_VOCAL_LOW_HZ,
};
pub use processing::find_peak;
pub use types::{Spectrogram, SpectrogramConfig, Spectrum};
