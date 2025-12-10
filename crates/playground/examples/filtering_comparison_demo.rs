/// Example demonstrating the filtering comparison functionality.
/// 
/// This example shows how to:
/// 1. Generate a noisy voice-like signal
/// 2. Apply different filtering methods
/// 3. Compare the results using FilteringComparison
/// 4. Save before/after audio files
/// 5. Analyze spectral differences
/// 
/// The example creates three comparison scenarios:
/// - Bandpass filtering (default)
/// - Spectral gating with noise profile
/// - No filtering (control)

use audio_utils::MonoAudio;
use audio_cleaning::{
    FilteringComparison, compare_filtering, clean_audio_for_pitch,
    estimate_noise_spectrum,
};
use sound_synth::voice_like_single_pitch;
use rand::Rng;

fn main() {
    println!("=== Filtering Comparison Demo ===\n");

    // Configuration
    let target_freq = 220.0; // A3 note
    let sample_rate = 44100;
    let duration_samples = 44100; // 1 second
    
    // Step 1: Generate a clean voice-like signal
    println!("Step 1: Generating voice-like signal at {} Hz...", target_freq);
    let clean_signal = voice_like_single_pitch(target_freq, 5, sample_rate as f32, duration_samples);
    
    // Step 2: Add noise to simulate realistic conditions
    println!("Step 2: Adding noise to signal...");
    let mut rng = rand::rng();
    let noise_amplitude = 0.3; // Moderate noise level
    let noisy_samples: Vec<f32> = clean_signal
        .iter()
        .map(|&sample| sample + noise_amplitude * (rng.random::<f32>() - 0.5))
        .collect();
    let noisy_audio = MonoAudio::new(noisy_samples, sample_rate);
    
    // Step 3: Create comparisons with different filtering methods
    println!("\nStep 3: Applying different filtering methods...\n");
    
    // Comparison 1: Bandpass filtering (default)
    println!("  a) Bandpass filtering (80-1200 Hz)");
    let mut bandpass_comparison = compare_filtering(&noisy_audio, |audio| {
        clean_audio_for_pitch(audio, None, None)
    });
    
    // Comparison 2: Spectral gating with noise estimation
    println!("  b) Spectral gating with noise estimation");
    let mut spectral_comparison = compare_filtering(&noisy_audio, |audio| {
        let noise_spectrum = estimate_noise_spectrum(audio);
        clean_audio_for_pitch(audio, noise_spectrum, Some(1.5))
    });
    
    // Comparison 3: No filtering (control)
    println!("  c) No filtering (control)");
    let no_filter_comparison = compare_filtering(&noisy_audio, |audio| audio.clone());
    
    // Step 4: Analyze waveforms
    println!("\nStep 4: Analyzing waveforms...");
    let (before_samples, after_bandpass) = bandpass_comparison.get_waveforms();
    let (_, after_spectral) = spectral_comparison.get_waveforms();
    
    // Calculate RMS (Root Mean Square) as a measure of signal energy
    let rms = |samples: &[f32]| -> f32 {
        let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
        (sum_squares / samples.len() as f32).sqrt()
    };
    
    let before_rms = rms(before_samples);
    let after_bandpass_rms = rms(after_bandpass);
    let after_spectral_rms = rms(after_spectral);
    
    println!("  Before filtering RMS: {:.4}", before_rms);
    println!("  After bandpass RMS:   {:.4} ({:.1}% of original)", 
        after_bandpass_rms, (after_bandpass_rms / before_rms) * 100.0);
    println!("  After spectral RMS:   {:.4} ({:.1}% of original)", 
        after_spectral_rms, (after_spectral_rms / before_rms) * 100.0);
    
    // Step 5: Analyze spectra
    println!("\nStep 5: Analyzing frequency spectra...");
    let (before_spectrum, after_bandpass_spectrum) = bandpass_comparison.get_magnitude_spectra();
    let (_, after_spectral_spectrum) = spectral_comparison.get_magnitude_spectra();
    
    // Find peak frequencies
    let find_peak_freq = |spectrum: &[f32], sample_rate: u32| -> (usize, f32) {
        let max_idx = spectrum.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        
        let freq = (max_idx as f32 * sample_rate as f32) / (2.0 * spectrum.len() as f32);
        (max_idx, freq)
    };
    
    let (_, before_peak_freq) = find_peak_freq(&before_spectrum, sample_rate);
    let (_, bandpass_peak_freq) = find_peak_freq(&after_bandpass_spectrum, sample_rate);
    let (_, spectral_peak_freq) = find_peak_freq(&after_spectral_spectrum, sample_rate);
    
    println!("  Before filtering peak: {:.1} Hz", before_peak_freq);
    println!("  After bandpass peak:   {:.1} Hz (target: {} Hz)", bandpass_peak_freq, target_freq);
    println!("  After spectral peak:   {:.1} Hz (target: {} Hz)", spectral_peak_freq, target_freq);
    
    // Step 6: Save audio files for manual review
    println!("\nStep 6: Saving audio files for review...");
    let save_comparison = |comp: &FilteringComparison, name: &str| {
        let before_path = format!("{}_before.wav", name);
        let after_path = format!("{}_after.wav", name);
        match comp.save_audio_pair(&before_path, &after_path) {
            Ok(_) => println!("  Saved {} and {}", before_path, after_path),
            Err(e) => eprintln!("  Error saving {}: {}", name, e),
        }
    };
    
    save_comparison(&bandpass_comparison, "bandpass");
    save_comparison(&spectral_comparison, "spectral");
    save_comparison(&no_filter_comparison, "nofilter");
    
    // Summary
    println!("\n=== Summary ===");
    println!("Generated test files in the current directory:");
    println!("  - bandpass_before.wav / bandpass_after.wav");
    println!("  - spectral_before.wav / spectral_after.wav");
    println!("  - nofilter_before.wav / nofilter_after.wav");
    println!("\nYou can:");
    println!("  1. Listen to the files to compare audio quality");
    println!("  2. Open them in audio editing software (Audacity, etc.) to visualize waveforms");
    println!("  3. View spectrograms to see frequency content differences");
    println!("\nFilteringComparison API provides:");
    println!("  - get_waveforms() for time-domain analysis");
    println!("  - get_magnitude_spectra() for frequency-domain analysis");
    println!("  - save_audio_pair() for manual review in external tools");
}
