//! Musical note representation
//!
//! This module defines a note type for representing musical pitches
//! as discrete pitch classes with octaves (e.g., A4, C#5, Bb3).

use std::fmt;
use serde::{Deserialize, Serialize};

/// Represents a musical pitch class (C, C#, D, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PitchClass {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl PitchClass {
    /// Get the semitone offset from C (C=0, C#=1, D=2, etc.)
    pub fn semitone_offset(&self) -> i32 {
        match self {
            PitchClass::C => 0,
            PitchClass::CSharp => 1,
            PitchClass::D => 2,
            PitchClass::DSharp => 3,
            PitchClass::E => 4,
            PitchClass::F => 5,
            PitchClass::FSharp => 6,
            PitchClass::G => 7,
            PitchClass::GSharp => 8,
            PitchClass::A => 9,
            PitchClass::ASharp => 10,
            PitchClass::B => 11,
        }
    }

    /// Get the pitch class from a semitone offset (0-11)
    pub fn from_semitone_offset(offset: i32) -> Self {
        match offset.rem_euclid(12) {
            0 => PitchClass::C,
            1 => PitchClass::CSharp,
            2 => PitchClass::D,
            3 => PitchClass::DSharp,
            4 => PitchClass::E,
            5 => PitchClass::F,
            6 => PitchClass::FSharp,
            7 => PitchClass::G,
            8 => PitchClass::GSharp,
            9 => PitchClass::A,
            10 => PitchClass::ASharp,
            11 => PitchClass::B,
            _ => unreachable!(),
        }
    }

    /// Get the name of the pitch class
    pub fn name(&self) -> &'static str {
        match self {
            PitchClass::C => "C",
            PitchClass::CSharp => "C#",
            PitchClass::D => "D",
            PitchClass::DSharp => "D#",
            PitchClass::E => "E",
            PitchClass::F => "F",
            PitchClass::FSharp => "F#",
            PitchClass::G => "G",
            PitchClass::GSharp => "G#",
            PitchClass::A => "A",
            PitchClass::ASharp => "A#",
            PitchClass::B => "B",
        }
    }

    /// Parse a pitch class from a string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "C" => Some(PitchClass::C),
            "C#" | "Db" => Some(PitchClass::CSharp),
            "D" => Some(PitchClass::D),
            "D#" | "Eb" => Some(PitchClass::DSharp),
            "E" => Some(PitchClass::E),
            "F" => Some(PitchClass::F),
            "F#" | "Gb" => Some(PitchClass::FSharp),
            "G" => Some(PitchClass::G),
            "G#" | "Ab" => Some(PitchClass::GSharp),
            "A" => Some(PitchClass::A),
            "A#" | "Bb" => Some(PitchClass::ASharp),
            "B" => Some(PitchClass::B),
            _ => None,
        }
    }
}

impl fmt::Display for PitchClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Represents a musical note with pitch class and octave
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Note {
    pub pitch_class: PitchClass,
    pub octave: i32,
}

impl Note {
    /// Create a new note
    pub fn new(pitch_class: PitchClass, octave: i32) -> Self {
        Self {
            pitch_class,
            octave,
        }
    }

    /// Get the MIDI note number (A4 = 69, middle C = 60)
    pub fn to_midi(&self) -> i32 {
        (self.octave + 1) * 12 + self.pitch_class.semitone_offset()
    }

    /// Create a note from a MIDI note number
    pub fn from_midi(midi: i32) -> Self {
        let octave = (midi / 12) - 1;
        let pitch_class = PitchClass::from_semitone_offset(midi % 12);
        Self::new(pitch_class, octave)
    }

    /// Parse a note from a string (e.g., "A4", "C#5", "Bb3")
    pub fn parse(s: &str) -> Option<Self> {
        if s.len() < 2 {
            return None;
        }

        // Try to parse 2-char pitch class first (e.g., "C#")
        if s.len() >= 3 {
            if let Some(pitch_class) = PitchClass::parse(&s[..2]) {
                if let Ok(octave) = s[2..].parse::<i32>() {
                    return Some(Self::new(pitch_class, octave));
                }
            }
        }

        // Try 1-char pitch class (e.g., "C")
        if let Some(pitch_class) = PitchClass::parse(&s[..1]) {
            if let Ok(octave) = s[1..].parse::<i32>() {
                return Some(Self::new(pitch_class, octave));
            }
        }

        None
    }

    /// Convert note to frequency in Hz (using equal temperament, A4 = 440 Hz)
    pub fn to_frequency(&self) -> f32 {
        let midi = self.to_midi();
        440.0 * 2.0_f32.powf((midi - 69) as f32 / 12.0)
    }

    /// Create a note from a frequency (rounds to nearest note)
    pub fn from_frequency(hz: f32) -> Option<Self> {
        if hz <= 0.0 {
            return None;
        }
        let midi = (69.0 + 12.0 * (hz / 440.0).log2()).round() as i32;
        Some(Self::from_midi(midi))
    }

    /// Apply an interval to this note
    ///
    /// # Arguments
    /// * `semitones` - Number of semitones to add (positive for ascending, negative for descending)
    pub fn transpose(&self, semitones: i32) -> Self {
        let midi = self.to_midi() + semitones;
        Self::from_midi(midi)
    }

    /// Calculate the interval between two notes in semitones
    pub fn interval_to(&self, other: &Note) -> i32 {
        other.to_midi() - self.to_midi()
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.pitch_class, self.octave)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_class_semitone_offset() {
        assert_eq!(PitchClass::C.semitone_offset(), 0);
        assert_eq!(PitchClass::CSharp.semitone_offset(), 1);
        assert_eq!(PitchClass::A.semitone_offset(), 9);
        assert_eq!(PitchClass::B.semitone_offset(), 11);
    }

    #[test]
    fn test_pitch_class_from_semitone_offset() {
        assert_eq!(PitchClass::from_semitone_offset(0), PitchClass::C);
        assert_eq!(PitchClass::from_semitone_offset(9), PitchClass::A);
        assert_eq!(PitchClass::from_semitone_offset(12), PitchClass::C);
        assert_eq!(PitchClass::from_semitone_offset(21), PitchClass::A);
    }

    #[test]
    fn test_pitch_class_parse() {
        assert_eq!(PitchClass::parse("A"), Some(PitchClass::A));
        assert_eq!(PitchClass::parse("C#"), Some(PitchClass::CSharp));
        assert_eq!(PitchClass::parse("Bb"), Some(PitchClass::ASharp));
        assert_eq!(PitchClass::parse("X"), None);
    }

    #[test]
    fn test_note_to_midi() {
        let a4 = Note::new(PitchClass::A, 4);
        assert_eq!(a4.to_midi(), 69);

        let c4 = Note::new(PitchClass::C, 4);
        assert_eq!(c4.to_midi(), 60);
    }

    #[test]
    fn test_note_from_midi() {
        let note = Note::from_midi(69);
        assert_eq!(note.pitch_class, PitchClass::A);
        assert_eq!(note.octave, 4);

        let note = Note::from_midi(60);
        assert_eq!(note.pitch_class, PitchClass::C);
        assert_eq!(note.octave, 4);
    }

    #[test]
    fn test_note_parse() {
        let note = Note::parse("A4").unwrap();
        assert_eq!(note.pitch_class, PitchClass::A);
        assert_eq!(note.octave, 4);

        let note = Note::parse("C#5").unwrap();
        assert_eq!(note.pitch_class, PitchClass::CSharp);
        assert_eq!(note.octave, 5);

        let note = Note::parse("Bb3").unwrap();
        assert_eq!(note.pitch_class, PitchClass::ASharp);
        assert_eq!(note.octave, 3);

        assert!(Note::parse("X4").is_none());
    }

    #[test]
    fn test_note_to_frequency() {
        let a4 = Note::new(PitchClass::A, 4);
        assert!((a4.to_frequency() - 440.0).abs() < 0.1);

        let a5 = Note::new(PitchClass::A, 5);
        assert!((a5.to_frequency() - 880.0).abs() < 0.1);
    }

    #[test]
    fn test_note_from_frequency() {
        let note = Note::from_frequency(440.0).unwrap();
        assert_eq!(note.pitch_class, PitchClass::A);
        assert_eq!(note.octave, 4);

        let note = Note::from_frequency(261.63).unwrap();
        assert_eq!(note.pitch_class, PitchClass::C);
        assert_eq!(note.octave, 4);
    }

    #[test]
    fn test_note_transpose() {
        let a4 = Note::new(PitchClass::A, 4);
        
        let a5 = a4.transpose(12); // Octave up
        assert_eq!(a5.pitch_class, PitchClass::A);
        assert_eq!(a5.octave, 5);

        let e5 = a4.transpose(7); // Perfect fifth up
        assert_eq!(e5.pitch_class, PitchClass::E);
        assert_eq!(e5.octave, 5);

        let a3 = a4.transpose(-12); // Octave down
        assert_eq!(a3.octave, 3);
    }

    #[test]
    fn test_note_interval_to() {
        let a4 = Note::new(PitchClass::A, 4);
        let a5 = Note::new(PitchClass::A, 5);
        
        assert_eq!(a4.interval_to(&a5), 12);
        assert_eq!(a5.interval_to(&a4), -12);

        let c4 = Note::new(PitchClass::C, 4);
        assert_eq!(c4.interval_to(&a4), 9);
    }

    #[test]
    fn test_note_display() {
        let note = Note::new(PitchClass::A, 4);
        assert_eq!(format!("{}", note), "A4");

        let note = Note::new(PitchClass::CSharp, 5);
        assert_eq!(format!("{}", note), "C#5");
    }
}
