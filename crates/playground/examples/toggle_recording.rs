use audio_utils::recording::MicrophoneRecorder;
use audio_utils::io::save_wav;
use std::io::{self, Write};

/// Example demonstrating toggle-based microphone recording.
///
/// This example shows how to use the `MicrophoneRecorder` API for interactive
/// recording where the user controls when to start and stop recording.
///
/// This is the recommended approach for applications where recording is triggered
/// by user input (keyboard, button, etc.) rather than a fixed duration.
///
/// Usage:
/// ```bash
/// cargo run --package playground --example toggle_recording
/// ```
fn main() {
    println!("=== Toggle-Based Microphone Recording ===\n");
    
    // Create the recorder
    println!("Initializing microphone recorder...");
    let mut recorder = match MicrophoneRecorder::new() {
        Ok(r) => {
            println!("✓ Recorder initialized");
            r
        }
        Err(e) => {
            println!("✗ Failed to create recorder: {}", e);
            println!("\nMake sure you have a microphone connected.");
            return;
        }
    };
    
    // Wait for user to press Enter to start
    println!("\nPress Enter to START recording...");
    wait_for_enter();
    
    // Start recording
    match recorder.start() {
        Ok(_) => {
            println!("🎙️  Recording... Press Enter to STOP");
        }
        Err(e) => {
            println!("✗ Failed to start recording: {}", e);
            return;
        }
    }
    
    // Wait for user to press Enter to stop
    wait_for_enter();
    
    // Stop recording and get the audio
    println!("⏹️  Stopping...");
    let audio = match recorder.stop() {
        Ok(audio) => {
            println!("✓ Recording stopped");
            println!("  Recorded {} samples at {} Hz", audio.samples.len(), audio.sample_rate);
            println!("  Duration: {:.2} seconds", audio.samples.len() as f32 / audio.sample_rate as f32);
            audio
        }
        Err(e) => {
            println!("✗ Failed to stop recording: {}", e);
            return;
        }
    };
    
    // Save the recorded audio
    let output_path = "/tmp/toggle_recorded_audio.wav";
    match save_wav(output_path, &audio) {
        Ok(_) => {
            println!("\n✓ Saved recording to {}", output_path);
        }
        Err(e) => {
            println!("\n✗ Failed to save recording: {}", e);
        }
    }
    
    println!("\n=== Recording Complete ===");
}

/// Helper function to wait for user to press Enter
fn wait_for_enter() {
    let mut buffer = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut buffer).unwrap();
}
