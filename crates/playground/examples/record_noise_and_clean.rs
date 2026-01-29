//! Example demonstrating noise recording and spectral gating workflow
//!
//! This example shows how to:
//! 1. Record background noise from the microphone
//! 2. Create a noise profile for spectral gating
//! 3. Record audio and clean it using the noise profile
//!
//! Run with: cargo run --package playground --example record_noise_and_clean
//!
//! This example requires a microphone and is interactive.

use audio_utils::recording::record_noise_from_microphone;
use audio_cleaning::{create_noise_profile, SpectralGate};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("===========================================");
    println!("Noise Recording and Spectral Gating Demo");
    println!("===========================================\n");

    // Step 1: Record background noise
    println!("Step 1: Recording Background Noise");
    println!("-----------------------------------");
    println!("We'll record 2 seconds of background noise.");
    println!("Please remain silent and avoid making sounds.");
    println!();
    print!("Press Enter when ready to record noise...");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    println!("Recording noise NOW... (2 seconds)");
    let noise_duration = 2.0;
    let noise_audio = record_noise_from_microphone(noise_duration)?;
    
    println!("✓ Noise recorded: {} samples at {} Hz", 
        noise_audio.samples.len(), 
        noise_audio.sample_rate
    );
    
    // Calculate noise statistics
    let noise_rms: f32 = (noise_audio.samples.iter().map(|x| x * x).sum::<f32>() 
        / noise_audio.samples.len() as f32).sqrt();
    println!("  Noise RMS level: {:.6}", noise_rms);
    println!();

    // Step 2: Create noise profile
    println!("Step 2: Creating Noise Profile");
    println!("-------------------------------");
    let noise_profile = create_noise_profile(&noise_audio);
    println!("✓ Noise profile created with {} frequency bins", 
        noise_profile.complex.len()
    );
    
    // Show some noise magnitude statistics
    let noise_magnitudes: Vec<f32> = noise_profile.complex
        .iter()
        .map(|c| c.norm())
        .collect();
    let avg_magnitude: f32 = noise_magnitudes.iter().sum::<f32>() / noise_magnitudes.len() as f32;
    let max_magnitude: f32 = noise_magnitudes.iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max);
    
    println!("  Average noise magnitude: {:.6}", avg_magnitude);
    println!("  Max noise magnitude: {:.6}", max_magnitude);
    println!();

    // Step 3: Initialize spectral gate
    println!("Step 3: Initializing Spectral Gate");
    println!("-----------------------------------");
    let gate = SpectralGate::with_defaults(noise_profile);
    println!("✓ Spectral gate initialized with default settings");
    println!("  Noise threshold: {} dB", gate.config().noise_threshold_db);
    println!("  Smoothing window: {} bins", gate.config().smoothing_window);
    println!();

    // Step 4: Process audio through the gate (demonstration with synthetic data)
    println!("Step 4: Demonstrating Noise Reduction");
    println!("--------------------------------------");
    println!("Creating synthetic noisy signal for demonstration...");
    
    // Create a simple test signal with noise
    let test_duration = 1.0;
    let sample_rate = noise_audio.sample_rate as f32;
    let num_samples = (test_duration * sample_rate) as usize;
    
    // Generate a 440 Hz sine wave with added noise
    use std::f32::consts::PI;
    use rand::Rng;
    
    let frequency = 440.0; // A4 note
    let signal_amplitude = 0.3;
    let mut rng = rand::rng();
    
    let test_signal: Vec<f32> = (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate;
            let clean_signal = signal_amplitude * (2.0 * PI * frequency * t).sin();
            // Add noise similar to recorded noise level
            let noise = (noise_rms * 2.0) * (rng.random::<f32>() - 0.5);
            clean_signal + noise
        })
        .collect();
    
    println!("  Test signal: {} Hz sine wave + noise", frequency);
    
    // Calculate input energy
    let input_energy: f32 = test_signal.iter().map(|x| x * x).sum::<f32>() / test_signal.len() as f32;
    
    // Process through spectral gate
    let cleaned_signal = gate.process(&test_signal);
    
    // Calculate output energy
    let output_energy: f32 = cleaned_signal.iter().map(|x| x * x).sum::<f32>() / cleaned_signal.len() as f32;
    
    println!("  Input RMS:  {:.6}", input_energy.sqrt());
    println!("  Output RMS: {:.6}", output_energy.sqrt());
    
    let noise_reduction_db = 10.0 * (input_energy / output_energy.max(1e-10)).log10();
    println!("  Noise reduction: {:.2} dB", noise_reduction_db);
    println!();

    // Summary
    println!("===========================================");
    println!("Summary: How Noise Profiles Work");
    println!("===========================================");
    println!();
    println!("The spectral gate compares incoming audio to the noise profile:");
    println!();
    println!("1. NOISE PROFILE: A frequency-domain representation of background noise");
    println!("   - Created once from recorded silence");
    println!("   - Remains STATIC during processing (not updated automatically)");
    println!("   - Can be manually updated with gate.update_noise_profile()");
    println!();
    println!("2. GATING PROCESS:");
    println!("   - Transform incoming audio to frequency domain (FFT)");
    println!("   - Compare each frequency bin to noise profile");
    println!("   - Attenuate bins below noise_threshold * noise_level");
    println!("   - Transform back to time domain (IFFT)");
    println!();
    println!("3. WHEN TO UPDATE NOISE PROFILE:");
    println!("   - When recording environment changes");
    println!("   - When background noise characteristics change");
    println!("   - NOT updated automatically - stays constant until you change it");
    println!();
    println!("4. BEST PRACTICES:");
    println!("   - Record noise in the same environment as your audio");
    println!("   - Ensure 1-3 seconds of pure silence (no speech/music)");
    println!("   - Reuse the same profile for recordings in same environment");
    println!("   - Record new noise if environment changes significantly");
    println!();
    println!("✓ Demo complete!");

    Ok(())
}
