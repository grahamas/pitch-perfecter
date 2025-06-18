//! Tests for audio_helpers (WAV sample loading)
use pitch_perfecter::audio_helpers;

#[test]
fn test_load_audio_samples_nonexistent() {
    // Should return None for a file that doesn't exist
    assert!(audio_helpers::load_audio_samples("nonexistent.wav").is_none());
}

#[test]
fn test_load_audio_samples_empty() {
    // Should return None for an empty file
    use std::fs::File;
    let _ = File::create("test_empty.wav");
    assert!(audio_helpers::load_audio_samples("test_empty.wav").is_none() ||
            audio_helpers::load_audio_samples("test_empty.wav").unwrap().is_empty());
    let _ = std::fs::remove_file("test_empty.wav");
}

// You can add more tests with actual WAV files in a test_data/ directory for real sample loading
