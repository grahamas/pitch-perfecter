use audio_utils::recording::{list_input_devices, record_from_microphone};
use audio_utils::io::save_wav;
use audio_cleaning::clean_audio_for_pitch;
use pitch_detection_utils::{ExternalYinDetector, MonoPitchDetector};

/// Convert frequency to note name (e.g., 440.0 Hz -> "A4")
fn frequency_to_note_name(frequency: f32) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    
    // A4 = 440 Hz is our reference (MIDI note 69)
    let a4_freq = 440.0;
    let midi_note = 69.0 + 12.0 * (frequency / a4_freq).log2();
    let note_index = (midi_note.round() as i32) % 12;
    let octave = (midi_note.round() as i32) / 12 - 1;
    
    format!("{}{}", note_names[note_index as usize], octave)
}

/// Example demonstrating microphone recording and pitch detection.
///
/// This example shows how to:
/// 1. List available input devices
/// 2. Record audio from the microphone
/// 3. Save the recorded audio to a file
/// 4. Clean the audio for better pitch detection
/// 5. Detect the pitch from the recorded audio
///
/// Usage:
/// ```bash
/// cargo run --package playground --example record_from_microphone
/// ```
fn main() {
    println!("=== Microphone Recording Demo ===\n");
    
    // Step 1: List available input devices
    println!("Available input devices:");
    match list_input_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("  No input devices found!");
                println!("  Make sure you have a microphone connected.");
                return;
            }
            for device in devices {
                let marker = if device.is_default { " (default)" } else { "" };
                println!("  - {}{}", device.name, marker);
            }
        }
        Err(e) => {
            println!("  Error listing devices: {}", e);
            return;
        }
    }
    
    println!();
    
    // Step 2: Record audio from the microphone
    println!("Recording 3 seconds from the default microphone...");
    println!("Speak or make a sound!");
    
    let audio = match record_from_microphone(3.0) {
        Ok(audio) => {
            println!("✓ Recorded {} samples at {} Hz", audio.samples.len(), audio.sample_rate);
            audio
        }
        Err(e) => {
            println!("✗ Failed to record: {}", e);
            return;
        }
    };
    
    // Step 3: Save the recorded audio
    let output_path = "/tmp/recorded_audio.wav";
    match save_wav(output_path, &audio) {
        Ok(_) => {
            println!("✓ Saved recording to {}", output_path);
        }
        Err(e) => {
            println!("✗ Failed to save: {}", e);
        }
    }
    
    // Step 4: Clean the audio for pitch detection
    println!("\nCleaning audio for pitch detection...");
    let cleaned_audio = clean_audio_for_pitch(&audio, None, None);
    
    // Save cleaned audio too
    let cleaned_path = "/tmp/recorded_audio_cleaned.wav";
    match save_wav(cleaned_path, &cleaned_audio) {
        Ok(_) => {
            println!("✓ Saved cleaned audio to {}", cleaned_path);
        }
        Err(e) => {
            println!("✗ Failed to save cleaned audio: {}", e);
        }
    }
    
    // Step 5: Detect pitch from the cleaned audio
    println!("\nDetecting pitch from cleaned audio...");
    let window_size = 2048;
    let mut yin_detector = ExternalYinDetector::new(0.1, 0.7, window_size, window_size / 2);
    
    match yin_detector.get_mono_pitch(cleaned_audio) {
        Some(pitch) => {
            println!("✓ Detected pitch:");
            println!("  Frequency: {:.1} Hz", pitch.frequency);
            println!("  Confidence: {:.2}%", pitch.clarity * 100.0);
            
            // Convert frequency to note name
            let note_name = frequency_to_note_name(pitch.frequency);
            println!("  Nearest note: {}", note_name);
        }
        None => {
            println!("✗ No clear pitch detected");
            println!("  Try speaking or singing a sustained note");
        }
    }
    
    println!("\n=== Demo Complete ===");
}
