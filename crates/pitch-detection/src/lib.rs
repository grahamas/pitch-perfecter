//! Pitch Detection
//! 
//! This crate provides pitch detection algorithms and utilities for musical
//! note frequency analysis. It includes:
//! - YIN pitch detection algorithm
//! - Pitch tracking over time
//! - Frequency to musical note conversion

pub mod pitch_tracking;
pub mod music_notation;

pub use pitch_tracking::{
    detection::{MonoPitchDetector, Pitch},
    detection_algorithms::yin::ExternalYinDetector,
    tracking::{PitchTracker, PitchTrackerConfig},
};
pub use music_notation::hz_to_note_name;
