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