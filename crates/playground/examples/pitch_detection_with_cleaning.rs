use audio_utils::MonoAudio;
use pitch_detection_utils::{ExternalYinDetector, MonoPitchDetector};
use sound_synth::voice_like_single_pitch;
use audio_cleaning::clean_audio_for_pitch;
use rand::Rng;

/// Example demonstrating the effectiveness of signal cleaning for pitch detection.
/// 
/// This example creates a noisy signal where pitch detection fails, then applies signal cleaning
/// to recover the ability to detect the correct pitch. It shows the integration between:
/// - Voice synthesis (creating test signals)
/// - YIN pitch detection algorithm
/// - Signal cleaning functionality
/// 
/// Flow:
/// 1. Generate a clean voice-like signal at a target frequency (220 Hz)
/// 2. Add significant white noise to make pitch detection fail
/// 3. Verify that YIN detector fails on the noisy signal
/// 4. Apply signal cleaning to the noisy signal
/// 5. Verify that YIN detector succeeds on the cleaned signal
fn main() {
    // Test configuration
    let target_freq = 220.0; // A3 note
    let sample_rate = 8000;
    let window_size = 2048;
    let duration_samples = window_size; // Make sure we have enough samples for the window

    // Generate a clean voice-like signal at target frequency
    let clean_signal = voice_like_single_pitch(target_freq, 3, sample_rate as f32, duration_samples);
    
    // Add significant white noise to make pitch detection fail
    let mut rng = rand::rng();
    let noise_amplitude = 2.0; // Very high noise level to ensure detection failure
    let noisy_signal: Vec<f32> = clean_signal
        .iter()
        .map(|&sample| sample + noise_amplitude * (rng.random::<f32>() - 0.5))
        .collect();

    let noisy_audio = MonoAudio::new(noisy_signal, sample_rate);

    // Test 1: Pitch detection on noisy signal should fail or be inaccurate
    let mut yin_detector = ExternalYinDetector::new(0.1, 0.7, window_size, window_size / 2);
    let noisy_pitch_result = yin_detector.get_mono_pitch(noisy_audio.clone());
    
    // Check if detection was accurate by examining the frequency if present
    let noisy_detection_accurate = match &noisy_pitch_result {
        Some(pitch) => (pitch.frequency - target_freq).abs() < 20.0,
        None => false,
    };
    
    println!("Noisy signal pitch detection: frequency = {:?}, accurate = {}", 
        noisy_pitch_result.as_ref().map(|p| p.frequency), noisy_detection_accurate);
    
    if noisy_detection_accurate {
        println!("WARNING: Noisy signal was detected accurately (expected to fail)");
    }
    
    // Step 2: Apply signal cleaning and try again
    let cleaned_audio = clean_audio_for_pitch(&noisy_audio, None, None);
    
    // Create a new detector for the cleaned signal test
    let cleaned_pitch_result = yin_detector.get_mono_pitch(cleaned_audio);

    // Should detect accurately after cleaning
    let cleaned_detection_accurate = match &cleaned_pitch_result {
        Some(pitch) => (pitch.frequency - target_freq).abs() < 20.0,
        None => false,
    };
    
    println!("Cleaned signal pitch detection: frequency = {:?}, accurate = {}", 
        cleaned_pitch_result.as_ref().map(|p| p.frequency), cleaned_detection_accurate);
    
    // Result: cleaning should enable accurate detection when noisy signal fails
    if cleaned_detection_accurate {
        println!("✓ SUCCESS: Signal cleaning enabled accurate pitch detection!");
        println!("  Target: {:.1} Hz, Detected: {:.1} Hz", 
            target_freq, cleaned_pitch_result.as_ref().map(|p| p.frequency).unwrap());
    } else {
        println!("✗ FAILED: Signal cleaning did not enable accurate pitch detection.");
        println!("  Target: {:.1} Hz, Cleaned frequency: {:?}", 
            target_freq, cleaned_pitch_result.as_ref().map(|p| p.frequency));
    }
}