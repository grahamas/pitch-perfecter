//! Sound Synthesis
//! 
//! This crate provides sound generation utilities for testing and demonstration.
//! It includes functions for generating voice-like signals with harmonics,
//! vibrato, and amplitude envelopes.
//!
//! All functions depend only on `audio-utils` for audio types.

pub mod voice_synth;

pub use voice_synth::{vibrato_sine_wave, voice_like_signal, voice_like_single_pitch};
