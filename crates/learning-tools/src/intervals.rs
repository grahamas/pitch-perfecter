//! Musical intervals and related utilities
//!
//! This module defines musical intervals, their semitone distances,
//! and utilities for working with musical notes and frequencies.

use std::fmt;

/// Standard musical intervals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Interval {
    /// Perfect unison (0 semitones)
    Unison,
    /// Minor second (1 semitone)
    MinorSecond,
    /// Major second (2 semitones)
    MajorSecond,
    /// Minor third (3 semitones)
    MinorThird,
    /// Major third (4 semitones)
    MajorThird,
    /// Perfect fourth (5 semitones)
    PerfectFourth,
    /// Tritone / Augmented fourth / Diminished fifth (6 semitones)
    Tritone,
    /// Perfect fifth (7 semitones)
    PerfectFifth,
    /// Minor sixth (8 semitones)
    MinorSixth,
    /// Major sixth (9 semitones)
    MajorSixth,
    /// Minor seventh (10 semitones)
    MinorSeventh,
    /// Major seventh (11 semitones)
    MajorSeventh,
    /// Perfect octave (12 semitones)
    Octave,
}

impl Interval {
    /// Returns the distance of the interval in semitones
    pub fn semitones(&self) -> i32 {
        match self {
            Interval::Unison => 0,
            Interval::MinorSecond => 1,
            Interval::MajorSecond => 2,
            Interval::MinorThird => 3,
            Interval::MajorThird => 4,
            Interval::PerfectFourth => 5,
            Interval::Tritone => 6,
            Interval::PerfectFifth => 7,
            Interval::MinorSixth => 8,
            Interval::MajorSixth => 9,
            Interval::MinorSeventh => 10,
            Interval::MajorSeventh => 11,
            Interval::Octave => 12,
        }
    }

    /// Returns a human-readable name for the interval
    pub fn name(&self) -> &'static str {
        match self {
            Interval::Unison => "Unison",
            Interval::MinorSecond => "Minor 2nd",
            Interval::MajorSecond => "Major 2nd",
            Interval::MinorThird => "Minor 3rd",
            Interval::MajorThird => "Major 3rd",
            Interval::PerfectFourth => "Perfect 4th",
            Interval::Tritone => "Tritone",
            Interval::PerfectFifth => "Perfect 5th",
            Interval::MinorSixth => "Minor 6th",
            Interval::MajorSixth => "Major 6th",
            Interval::MinorSeventh => "Minor 7th",
            Interval::MajorSeventh => "Major 7th",
            Interval::Octave => "Octave",
        }
    }

    /// Returns intervals ordered by typical utility for learning
    /// (more common/useful intervals first)
    pub fn learning_order() -> Vec<Interval> {
        vec![
            Interval::Octave,        // Most fundamental
            Interval::PerfectFifth,  // Very common, consonant
            Interval::PerfectFourth, // Very common, consonant
            Interval::MajorThird,    // Major triad component
            Interval::MinorThird,    // Minor triad component
            Interval::MajorSecond,   // Scale step
            Interval::MajorSixth,    // Common, consonant
            Interval::MinorSixth,    // Common in minor keys
            Interval::MinorSeventh,  // Seventh chords
            Interval::MajorSeventh,  // Major seventh chords
            Interval::Tritone,       // Dissonant but important
            Interval::MinorSecond,   // Most dissonant, challenging
            Interval::Unison,        // For completeness
        ]
    }

    /// Get all intervals as a vector
    pub fn all() -> Vec<Interval> {
        vec![
            Interval::Unison,
            Interval::MinorSecond,
            Interval::MajorSecond,
            Interval::MinorThird,
            Interval::MajorThird,
            Interval::PerfectFourth,
            Interval::Tritone,
            Interval::PerfectFifth,
            Interval::MinorSixth,
            Interval::MajorSixth,
            Interval::MinorSeventh,
            Interval::MajorSeventh,
            Interval::Octave,
        ]
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Calculate the frequency of a note given a starting frequency and an interval
///
/// # Arguments
/// * `base_freq` - The starting frequency in Hz
/// * `interval` - The interval to add
///
/// # Returns
/// The frequency of the target note in Hz
pub fn apply_interval(base_freq: f32, interval: Interval) -> f32 {
    base_freq * 2.0_f32.powf(interval.semitones() as f32 / 12.0)
}

/// Calculate the interval between two frequencies
///
/// # Arguments
/// * `freq1` - The first frequency in Hz
/// * `freq2` - The second frequency in Hz
///
/// # Returns
/// The interval in semitones (positive if freq2 > freq1, negative otherwise)
pub fn calculate_interval_semitones(freq1: f32, freq2: f32) -> f32 {
    if freq1 <= 0.0 || freq2 <= 0.0 {
        return 0.0;
    }
    12.0 * (freq2 / freq1).log2()
}

/// Find the closest standard interval to a given number of semitones
///
/// # Arguments
/// * `semitones` - The number of semitones (can be fractional)
///
/// # Returns
/// The closest standard interval
pub fn closest_interval(semitones: f32) -> Interval {
    let intervals = Interval::all();
    intervals
        .into_iter()
        .min_by_key(|interval| {
            ((interval.semitones() as f32 - semitones).abs() * 100.0) as i32
        })
        .unwrap_or(Interval::Unison)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_semitones() {
        assert_eq!(Interval::Unison.semitones(), 0);
        assert_eq!(Interval::MajorThird.semitones(), 4);
        assert_eq!(Interval::PerfectFifth.semitones(), 7);
        assert_eq!(Interval::Octave.semitones(), 12);
    }

    #[test]
    fn test_interval_names() {
        assert_eq!(Interval::MajorThird.name(), "Major 3rd");
        assert_eq!(Interval::PerfectFifth.name(), "Perfect 5th");
    }

    #[test]
    fn test_apply_interval() {
        // A4 = 440 Hz
        let a4 = 440.0;
        
        // A4 + octave = A5 = 880 Hz
        let a5 = apply_interval(a4, Interval::Octave);
        assert!((a5 - 880.0).abs() < 0.1);
        
        // A4 + perfect fifth = E5 ≈ 659.26 Hz
        let e5 = apply_interval(a4, Interval::PerfectFifth);
        assert!((e5 - 659.26).abs() < 0.1);
        
        // A4 + major third = C#5 ≈ 554.37 Hz
        let cs5 = apply_interval(a4, Interval::MajorThird);
        assert!((cs5 - 554.37).abs() < 0.1);
    }

    #[test]
    fn test_calculate_interval_semitones() {
        let a4 = 440.0;
        let a5 = 880.0;
        
        // Octave = 12 semitones
        let semitones = calculate_interval_semitones(a4, a5);
        assert!((semitones - 12.0).abs() < 0.01);
        
        // Perfect fifth ≈ 7 semitones
        let e5 = 659.26;
        let semitones = calculate_interval_semitones(a4, e5);
        assert!((semitones - 7.0).abs() < 0.1);
    }

    #[test]
    fn test_closest_interval() {
        assert_eq!(closest_interval(0.0), Interval::Unison);
        assert_eq!(closest_interval(4.1), Interval::MajorThird);
        assert_eq!(closest_interval(6.8), Interval::PerfectFifth);
        assert_eq!(closest_interval(11.9), Interval::Octave);
    }

    #[test]
    fn test_learning_order() {
        let order = Interval::learning_order();
        assert_eq!(order[0], Interval::Octave);
        assert_eq!(order[1], Interval::PerfectFifth);
        assert!(order.len() == 13);
    }
}
