//! Playground
//! 
//! This crate provides examples, demos, and integration tests for the pitch perfecter
//! functionality. It includes:
//! - Voice synthesis utilities for testing
//! - Demo applications showcasing audio processing pipeline
//! - Integration examples combining multiple crates
//!
//! This is useful for development, testing, and demonstrating the capabilities
//! of the pitch perfecter system.

pub mod voice_synth;

pub use voice_synth::{vibrato_sine_wave, voice_like_signal, voice_like_single_pitch};
