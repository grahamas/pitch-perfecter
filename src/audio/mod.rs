//! Audio Module
//! 
//! This module provides types and traits for working with audio data in the pitch perfecter application.
//! It is designed for extensibility to support various audio formats (mono, stereo, multi-channel).
//!
//! # Architecture
//! 
//! The module is built around three core traits:
//! - `Audio`: Base trait for all audio types
//! - `MonoAudioSource`: For types that can provide mono audio samples
//! - `IterableAudio`: For types that support windowed iteration
//!
//! # Use Cases
//! 
//! This module supports two primary use cases:
//! 1. **Storing and playing audio**: Load audio from files or memory, process it, and play it back
//! 2. **Streaming and live-editing**: Process audio as it's being recorded in real-time
//!
//! # Examples
//! 
//! ```
//! use pitch_perfecter::audio::{Audio, MonoAudio, MonoAudioSource};
//! 
//! // Create a simple mono audio buffer
//! let samples = vec![0.0, 0.5, 1.0, 0.5, 0.0];
//! let audio = MonoAudio::new(samples, 44100);
//! 
//! // Access audio properties
//! assert_eq!(audio.sample_rate(), 44100);
//! assert_eq!(audio.mono_samples().len(), 5);
//! ```

pub mod types;

pub use types::{MonoAudio, Audio, MonoAudioSource, IterableAudio};
