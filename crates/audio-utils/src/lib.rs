//! Audio Utilities
//! 
//! This crate provides low-level audio types and utilities for audio processing.
//! It contains core audio data structures and helpers that are reusable across
//! different audio processing modules.

pub mod audio;
pub mod io;
pub mod recording;

pub use audio::{Audio, MonoAudio, MonoAudioSource, IterableAudio};
