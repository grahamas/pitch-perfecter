/// Example demonstrating audio file loading and saving.
/// 
/// This example shows how to:
/// 1. Generate audio programmatically
/// 2. Save it to a WAV file
/// 3. Load it back from the file
/// 4. Verify the loaded audio matches the original
use audio_utils::{MonoAudio, io::{save_wav, load_wav}};
use sound_synth::voice_like_single_pitch;

fn main() {
    println!("=== Audio File I/O Demo ===\n");
    
    // Step 1: Generate a voice-like audio signal
    println!("1. Generating voice-like audio signal...");
    let sample_rate = 44100;
    let duration_sec = 2.0;
    let len = (sample_rate as f32 * duration_sec) as usize;
    let freq = 440.0; // A4 note
    let harmonics = 6;
    
    let signal = voice_like_single_pitch(freq, harmonics, sample_rate as f32, len);
    let original_audio = MonoAudio::new(signal, sample_rate);
    
    println!("   Generated {} samples at {} Hz", original_audio.samples.len(), original_audio.sample_rate);
    println!("   Duration: {:.2} seconds", original_audio.samples.len() as f32 / original_audio.sample_rate as f32);
    println!("   Frequency: {} Hz (A4 note)", freq);
    
    // Step 2: Save the audio to a file
    println!("\n2. Saving audio to 'demo_audio.wav'...");
    let file_path = "demo_audio.wav";
    save_wav(file_path, &original_audio).expect("Failed to save audio file");
    println!("   ✓ Audio saved successfully");
    
    // Step 3: Load the audio back from the file
    println!("\n3. Loading audio from 'demo_audio.wav'...");
    let loaded_audio = load_wav(file_path).expect("Failed to load audio file");
    println!("   ✓ Audio loaded successfully");
    println!("   Loaded {} samples at {} Hz", loaded_audio.samples.len(), loaded_audio.sample_rate);
    
    // Step 4: Verify the loaded audio matches the original
    println!("\n4. Verifying loaded audio matches original...");
    assert_eq!(loaded_audio.sample_rate, original_audio.sample_rate, 
               "Sample rate mismatch!");
    assert_eq!(loaded_audio.samples.len(), original_audio.samples.len(), 
               "Sample count mismatch!");
    
    // Check that samples are nearly identical (allow for small floating point errors)
    let max_diff = loaded_audio.samples.iter()
        .zip(original_audio.samples.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    
    println!("   Maximum sample difference: {:.10}", max_diff);
    assert!(max_diff < 1e-6, "Samples differ too much!");
    
    println!("   ✓ Loaded audio matches original");
    
    println!("\n=== Demo Complete ===");
    println!("The file 'demo_audio.wav' has been created and verified.");
    println!("You can play it with any audio player or import it into audio editing software.");
}
