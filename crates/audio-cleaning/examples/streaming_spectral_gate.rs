//! Example demonstrating real-time streaming spectral gating
//!
//! This example shows how to use the SpectralGate for processing audio
//! in fixed-size chunks, simulating a real-time microphone input scenario.
//!
//! Run with: cargo run --package audio-cleaning --example streaming_spectral_gate

use audio_cleaning::{SpectralGate, SpectralGateConfig, Spectrum};
use std::f32::consts::PI;

fn main() {
    println!("Streaming Spectral Gate Example");
    println!("================================\n");

    // Simulation parameters
    let sample_rate = 44100.0;
    let chunk_size = 1024; // Process in 1024-sample chunks (about 23ms at 44.1kHz)
    let frequency = 440.0; // A4 note
    let num_chunks = 10;

    // Step 1: Create a noise profile from background noise
    println!("Step 1: Creating noise profile from background noise...");
    let noise_samples: Vec<f32> = (0..chunk_size)
        .map(|_| (rand::random::<f32>() - 0.5) * 0.05) // Low-amplitude noise
        .collect();
    let noise_profile = Spectrum::from_waveform(&noise_samples);
    println!("  Noise profile created with {} samples\n", noise_samples.len());

    // Step 2: Configure the spectral gate
    println!("Step 2: Configuring spectral gate...");
    let threshold_db = 6.0;
    let config = SpectralGateConfig {
        noise_threshold_db: threshold_db,   // Attenuate signals 6 dB below noise floor
        smoothing_window: 3,                 // Smooth across 3 frequency bins
    };
    let gate = SpectralGate::new(noise_profile, config);
    println!("  Gate configured with {} dB threshold\n", threshold_db);

    // Step 3: Simulate streaming audio chunks
    println!("Step 3: Processing {} audio chunks...", num_chunks);
    
    let mut total_input_energy = 0.0;
    let mut total_output_energy = 0.0;

    for chunk_idx in 0..num_chunks {
        // Generate a chunk with signal + noise
        let chunk: Vec<f32> = (0..chunk_size)
            .map(|i| {
                let t = (chunk_idx * chunk_size + i) as f32 / sample_rate;
                let signal = 0.5 * (2.0 * PI * frequency * t).sin(); // Clean sine wave
                let noise = (rand::random::<f32>() - 0.5) * 0.02; // Low noise
                signal + noise
            })
            .collect();

        // Process the chunk through the gate
        let cleaned = gate.process(&chunk);

        // Calculate energy for this chunk
        let input_energy: f32 = chunk.iter().map(|x| x * x).sum();
        let output_energy: f32 = cleaned.iter().map(|x| x * x).sum();
        
        total_input_energy += input_energy;
        total_output_energy += output_energy;

        let noise_reduction_db = 10.0 * (input_energy / output_energy.max(1e-10)).log10();
        
        println!(
            "  Chunk {:2}: Input energy: {:.4}, Output energy: {:.4}, Reduction: {:.2} dB",
            chunk_idx + 1, input_energy, output_energy, noise_reduction_db
        );
    }

    println!("\nStep 4: Summary");
    println!("  Total input energy:  {:.4}", total_input_energy);
    println!("  Total output energy: {:.4}", total_output_energy);
    let total_reduction = 10.0 * (total_input_energy / total_output_energy.max(1e-10)).log10();
    println!("  Overall noise reduction: {:.2} dB", total_reduction);
    println!("  Energy preserved: {:.1}%", (total_output_energy / total_input_energy) * 100.0);

    println!("\n✓ Streaming processing complete!");
    println!("\nKey takeaways:");
    println!("  • Each chunk is processed independently (no inter-chunk state)");
    println!("  • The gate preserves the signal while reducing noise");
    println!("  • This approach is suitable for real-time microphone input");
    println!("  • You can update the noise profile dynamically with update_noise_profile()");
}

// Simple random number generator for demonstration
mod rand {
    use std::cell::Cell;
    use std::num::Wrapping;

    thread_local! {
        static SEED: Cell<Wrapping<u32>> = Cell::new(Wrapping(12345));
    }

    pub fn random<T>() -> T
    where
        T: From<f32>,
    {
        SEED.with(|seed| {
            let mut s = seed.get();
            s = Wrapping(s.0.wrapping_mul(1103515245).wrapping_add(12345));
            seed.set(s);
            let value = ((s.0 / 65536) % 32768) as f32 / 32768.0;
            T::from(value)
        })
    }
}
