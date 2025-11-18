/// Convert a frequency in Hz to the nearest musical note name (e.g., "A4", "C#5")
pub fn hz_to_note_name(hz: f32) -> String {
    if hz <= 0.0 {
        return "N/A".to_string();
    }
    // A4 = 440 Hz, MIDI note 69
    let midi = (69.0 + 12.0 * (hz / 440.0).log2()).round() as i32;
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let note = note_names[(midi.rem_euclid(12)) as usize];
    let octave = (midi / 12) - 1;
    format!("{}{}", note, octave)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hz_to_note_name_standard_notes() {
        assert_eq!(hz_to_note_name(440.0), "A4"); // A4
        assert_eq!(hz_to_note_name(261.63), "C4"); // C4 (middle C)
        assert_eq!(hz_to_note_name(329.63), "E4"); // E4
        assert_eq!(hz_to_note_name(0.0), "N/A"); // Invalid
        assert_eq!(hz_to_note_name(-10.0), "N/A"); // Invalid
    }

    #[test]
    fn test_hz_to_note_name_octaves() {
        assert_eq!(hz_to_note_name(880.0), "A5"); // A5
        assert_eq!(hz_to_note_name(110.0), "A2"); // A2
        assert_eq!(hz_to_note_name(27.5), "A0"); // A0
    }

    #[test]
    fn test_hz_to_note_name_accidentals() {
        assert_eq!(hz_to_note_name(277.18), "C#4"); // C#4
        assert_eq!(hz_to_note_name(311.13), "D#4"); // D#4
        assert_eq!(hz_to_note_name(466.16), "A#4"); // A#4
    }
}