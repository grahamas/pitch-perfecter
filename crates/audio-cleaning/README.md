# Audio Cleaning

Audio preprocessing and cleaning operations for the Pitch Perfecter project.

## Overview

This crate provides tools to improve audio quality before pitch detection and analysis. It focuses on removing noise and isolating the vocal frequency range to improve the accuracy of downstream audio processing tasks.

## Features

- **Bandpass Filtering**: Isolate vocal frequency range (80-1200 Hz by default)
- **Spectral Gating**: Advanced noise reduction using reference noise profiles
- **Noise Spectrum Estimation**: Automatic background noise detection and profiling
- **FFT/IFFT Processing**: Efficient frequency domain analysis and manipulation
- **Spectrograms**: Time-frequency representation of audio signals

## Usage

### Basic Audio Cleaning

The simplest way to clean audio for pitch detection:

```rust
use audio_cleaning::clean_audio_for_pitch;
use audio_utils::MonoAudio;

// Load or create your audio
let audio = MonoAudio {
    samples: vec![/* your audio samples */],
    sample_rate: 44100,
};

// Clean with default bandpass filter (80-1200 Hz)
let cleaned = clean_audio_for_pitch(&audio, None, None);
```

### Advanced Cleaning with Noise Profile

For better noise reduction when you have a noise reference:

```rust
use audio_cleaning::{clean_audio_for_pitch, estimate_noise_spectrum};
use audio_utils::MonoAudio;

let audio = MonoAudio {
    samples: vec![/* your audio samples */],
    sample_rate: 44100,
};

// Estimate noise from the audio itself (finds quiet sections)
let noise_spectrum = estimate_noise_spectrum(&audio);

// Clean using spectral gating with noise profile
let cleaned = clean_audio_for_pitch(
    &audio, 
    noise_spectrum,
    Some(1.2),  // Noise threshold multiplier
);
```

### Custom Bandpass Filtering

Apply bandpass filter with custom frequency range:

```rust
use audio_cleaning::bandpass_vocal_range;

let samples = vec![/* your audio samples */];
let sample_rate = 44100.0;

// Custom frequency range (e.g., for soprano: 250-1500 Hz)
let filtered = bandpass_vocal_range(
    &samples,
    sample_rate,
    250.0,  // low cutoff
    1500.0, // high cutoff
);
```

### Spectral Analysis

Compute frequency spectrum and spectrogram:

```rust
use audio_cleaning::{Spectrum, Spectrogram, SpectrogramConfig};

// Compute FFT spectrum
let spectrum = Spectrum::from_waveform(&samples);
let magnitudes = spectrum.magnitudes();  // Get magnitude spectrum

// Compute spectrogram
let config = SpectrogramConfig {
    window_size: 1024,
    step_size: 256,
};
let spectrogram = Spectrogram::from_waveform(&samples, config);
```

## Frequency Ranges

Default vocal frequency range (suitable for most voices):
- **Low cutoff**: 80 Hz (removes low-frequency rumble)
- **High cutoff**: 1200 Hz (removes high-frequency noise)

Typical voice ranges for reference:
- **Bass**: 82-330 Hz
- **Tenor**: 130-520 Hz
- **Alto**: 174-700 Hz
- **Soprano**: 247-1046 Hz

You can adjust the bandpass filter cutoffs based on the voice type you're analyzing.

## Noise Reduction Strategies

The crate supports two noise reduction approaches:

### 1. Bandpass Filtering (Default)
- Fast and simple
- Works without noise reference
- Good for general-purpose cleaning
- Removes out-of-band noise

### 2. Spectral Gating (Advanced)
- Requires noise profile
- More sophisticated reduction
- Better preserves signal quality
- Recommended when noise sample available

## API Reference

### Main Functions

- `clean_audio_for_pitch(&MonoAudio, Option<Spectrum>, Option<f32>) -> MonoAudio`
  - High-level cleaning function, handles both filtering strategies

- `bandpass_vocal_range(&[f32], f32, f32, f32) -> Vec<f32>`
  - Apply bandpass filter to isolate frequency range

- `estimate_noise_spectrum(&MonoAudio) -> Option<Spectrum>`
  - Automatically detect and profile background noise

### Types

- `Spectrum`: FFT spectrum with ability to invert back to time domain
- `Spectrogram`: Time-frequency representation of a signal
- `SpectrogramConfig`: Configuration for spectrogram computation

## Performance Considerations

- FFT operations are O(N log N) where N is the number of samples
- Bandpass filtering processes samples sequentially (O(N))
- Spectral gating requires FFT + IFFT (more expensive but higher quality)
- For real-time applications, consider using bandpass filtering only

## Current Limitations

See [PLAN.md](./PLAN.md) for detailed information on known issues and planned improvements:

1. Sample rate parameter in bandpass filter is currently unused
2. Noise window detection assumes noise is in first 200ms-1500ms
3. FFT planner is not cached (performance impact)
4. Peak finding is basic (global maximum only)

## Testing

Run the test suite:

```bash
cargo test -p audio-cleaning
```

Run with ignored tests:

```bash
cargo test -p audio-cleaning -- --ignored
```

## Dependencies

- `audio-utils`: Core audio types and utilities
- `fundsp`: Audio DSP library (for filtering)
- `rustfft`: Fast Fourier Transform implementation

## Future Enhancements

Planned improvements (see [PLAN.md](./PLAN.md) for details):
- Fix sample rate handling in bandpass filter
- Enhanced noise detection algorithms
- FFT planner caching for better performance
- Windowing functions for spectral analysis
- Pre-emphasis filtering
- Real-time processing support

## Examples

See the `playground` crate for complete examples:

```bash
# Run pitch detection with cleaning example
cargo run --package playground --example pitch_detection_with_cleaning
```

## License

<!-- Add license information -->
