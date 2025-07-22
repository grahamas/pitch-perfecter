//! # Pitch Detection Traits
//! This module defines traits for pitch detection, including a generic `PitchDetector`
//! and a `MonoPitchDetector` for mono audio sources.

use crate::audio::{MonoAudioSource};
use pitch_detection;

pub type Pitch = pitch_detection::Pitch<f32>;

/// Trait for pitch detection on mono audio sources
pub trait MonoPitchDetector {
    fn get_mono_pitch<T: MonoAudioSource>(&mut self, mono_audio: T) -> Option<Pitch>;
    fn get_pitch<T: MonoAudioSource>(&mut self, audio: T) -> Option<Pitch> {
        self.get_mono_pitch(audio)
    }
}