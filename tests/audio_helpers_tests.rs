use pitch_perfecter::audio_helpers::*;

#[test]
fn test_load_audio_samples_nonexistent() {
    // Should return None for a file that doesn't exist
    assert!(load_audio_samples("nonexistent.wav").is_none());
}

#[test]
fn test_load_audio_samples_empty() {
    // Should return None for an empty file
    use std::fs::File;
    let _ = File::create("test_empty.wav");
    assert!(load_audio_samples("test_empty.wav").is_none() ||
            load_audio_samples("test_empty.wav").unwrap().is_empty());
    let _ = std::fs::remove_file("test_empty.wav");
}
// More tests can be added for real WAV files in a test_data/ directory
