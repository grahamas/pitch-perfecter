# Expectations for the `sound_synth` Package API and Functionality

## Overview

The `sound_synth` package is expected to provide a flexible and efficient API for generating synthetic audio signals, particularly for use in audio analysis, pitch detection, and testing. Its primary focus should be on producing voice-like, musical, and test signals with configurable parameters.

## Required Functionality

1. **Signal Generation**
   - Ability to generate basic waveforms: sine, square, triangle, sawtooth.
   - Support for generating voice-like signals with harmonics, vibrato, and amplitude envelopes.
   - Configurable sample rate, duration, and amplitude.

2. **Voice-Like Synthesis**
   - Functions to synthesize signals that mimic human voice characteristics:
     - Control over fundamental frequency (pitch).
     - Control over number and strength of harmonics.
     - Optional vibrato (frequency modulation) and tremolo (amplitude modulation).
     - Envelope shaping (attack, decay, sustain, release).

3. **Batch and Real-Time Generation**
   - Ability to generate signals as a complete buffer (Vec<f32> or similar).
   - Optionally, support for streaming or iterator-based generation for real-time applications.

4. **Noise Generation**
   - Functions to generate white, pink, and brown noise for testing and simulation.

5. **Parameterization**
   - All synthesis functions should accept parameters for sample rate, length/duration, amplitude, and other relevant controls.

## Example API

