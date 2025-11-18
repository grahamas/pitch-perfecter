//! Playground
//! 
//! This crate provides examples, demos, and integration tests for the pitch perfecter
//! functionality. It includes:
//! - Demo applications showcasing audio processing pipeline
//! - Integration examples combining multiple crates
//!
//! This is useful for development, testing, and demonstrating the capabilities
//! of the pitch perfecter system.
//!
//! Note: Voice synthesis utilities have been moved to the `sound-synth` crate.

// Re-export sound synthesis functions for convenience
pub use sound_synth::{vibrato_sine_wave, voice_like_signal, voice_like_single_pitch};
