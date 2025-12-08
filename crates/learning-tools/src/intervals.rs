//! Musical intervals and related utilities
//!
//! This module defines musical intervals, their semitone distances,
//! and utilities for working with musical notes.

use std::fmt;
use crate::note::Note;

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

/// Apply an interval to a note
///
/// # Arguments
/// * `base_note` - The starting note
/// * `interval` - The interval to add
/// * `ascending` - Whether to go up (true) or down (false)
///
/// # Returns
/// The target note after applying the interval
pub fn apply_interval(base_note: Note, interval: Interval, ascending: bool) -> Note {
    let semitones = if ascending {
        interval.semitones()
    } else {
        -interval.semitones()
    };
    base_note.transpose(semitones)
}

/// Calculate the interval between two notes
///
/// # Arguments
/// * `note1` - The first note
/// * `note2` - The second note
///
/// # Returns
/// The interval in semitones (positive if note2 > note1, negative otherwise)
pub fn calculate_interval_semitones(note1: Note, note2: Note) -> i32 {
    note1.interval_to(&note2)
}

/// Find the closest standard interval to a given number of semitones
///
/// # Arguments
/// * `semitones` - The number of semitones
///
/// # Returns
/// The closest standard interval
pub fn closest_interval(semitones: i32) -> Interval {
    let intervals = Interval::all();
    intervals
        .into_iter()
        .min_by_key(|interval| {
            (interval.semitones() - semitones).abs()
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
        use crate::note::{Note, PitchClass};
        
        let a4 = Note::new(PitchClass::A, 4);
        
        // A4 + octave ascending = A5
        let a5 = apply_interval(a4, Interval::Octave, true);
        assert_eq!(a5.pitch_class, PitchClass::A);
        assert_eq!(a5.octave, 5);
        
        // A4 + perfect fifth ascending = E5
        let e5 = apply_interval(a4, Interval::PerfectFifth, true);
        assert_eq!(e5.pitch_class, PitchClass::E);
        assert_eq!(e5.octave, 5);
        
        // A4 + major third ascending = C#5
        let cs5 = apply_interval(a4, Interval::MajorThird, true);
        assert_eq!(cs5.pitch_class, PitchClass::CSharp);
        assert_eq!(cs5.octave, 5);
        
        // A4 + octave descending = A3
        let a3 = apply_interval(a4, Interval::Octave, false);
        assert_eq!(a3.pitch_class, PitchClass::A);
        assert_eq!(a3.octave, 3);
    }

    #[test]
    fn test_calculate_interval_semitones() {
        use crate::note::{Note, PitchClass};
        
        let a4 = Note::new(PitchClass::A, 4);
        let a5 = Note::new(PitchClass::A, 5);
        
        // Octave = 12 semitones
        let semitones = calculate_interval_semitones(a4, a5);
        assert_eq!(semitones, 12);
        
        // Perfect fifth = 7 semitones
        let e5 = Note::new(PitchClass::E, 5);
        let semitones = calculate_interval_semitones(a4, e5);
        assert_eq!(semitones, 7);
    }

    #[test]
    fn test_closest_interval() {
        assert_eq!(closest_interval(0), Interval::Unison);
        assert_eq!(closest_interval(4), Interval::MajorThird);
        assert_eq!(closest_interval(7), Interval::PerfectFifth);
        assert_eq!(closest_interval(12), Interval::Octave);
    }

    #[test]
    fn test_learning_order() {
        let order = Interval::learning_order();
        assert_eq!(order[0], Interval::Octave);
        assert_eq!(order[1], Interval::PerfectFifth);
        assert!(order.len() == 13);
    }
}
