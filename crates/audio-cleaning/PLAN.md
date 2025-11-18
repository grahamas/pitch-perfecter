# Audio Cleaning Improvement Plan

## Overview

This document outlines improvements and enhancements for the audio-cleaning crate. The crate currently provides audio preprocessing and cleaning operations including bandpass filtering, spectral gating, and noise reduction. This plan addresses existing limitations (marked as FIXME in the code) and proposes enhancements to improve robustness and performance.

## Current Status

The audio-cleaning crate provides:
- ✅ Bandpass filtering for vocal frequency range isolation
- ✅ Spectral gating for noise reduction
- ✅ Background noise spectrum estimation
- ✅ DC offset removal and normalization (via bandpass)
- ✅ FFT/IFFT operations for frequency domain processing

## Identified Issues and Improvements

### 1. Bandpass Filter Sample Rate Issue

**Location:** `cleaning.rs:24-25`

**Current Issue:**
```rust
pub fn bandpass_vocal_range(samples: &[f32], _sample_rate: f32, low_hz: f32, high_hz: f32)
```
The function accepts a `sample_rate` parameter but doesn't use it. The fundsp library's `bandpass_hz` operates in an implicit sample rate context.

**Impact:**
- Potential incorrect filtering if fundsp's assumed sample rate differs from actual audio sample rate
- API confusion - parameter suggests it's used but isn't

**Proposed Solutions:**

**Option A: Use fundsp's rate conversion (Recommended)**
- Set up fundsp's audio graph with explicit sample rate using `An::set_sample_rate()`
- Ensures filter operates at correct frequencies for any input sample rate
- More robust and correct implementation

**Option B: Document and simplify**
- If fundsp always assumes a standard rate (e.g., 44100 Hz), document this limitation
- Consider removing the unused parameter or add runtime check
- Add tests to verify behavior at different sample rates

**Priority:** HIGH - Correctness issue that could affect pitch detection accuracy

### 2. Noise Window Detection

**Location:** `cleaning.rs:144-146`

**Current Issues:**
```rust
// FIXME: Assumes noise is in first 200ms-1500ms
// FIXME: Uses simple RMS and Z-score with only 1 STD deviation threshold
```

**Impact:**
- Fails for audio where noise isn't in the beginning
- May not detect noise in recordings with immediate vocal onset
- Single threshold may not work for all audio types

**Proposed Solutions:**

**Phase 1: Enhanced Detection Algorithm**
- Implement multiple detection strategies:
  1. **Start-silence detection** (current approach, but improved)
  2. **End-silence detection** - check last N seconds
  3. **Inter-phrase gaps** - detect quiet periods between speech/singing
  4. **Spectral stability** - find regions with minimal spectral variation

**Phase 2: Adaptive Thresholding**
- Make Z-score threshold configurable (currently hardcoded to -1.0)
- Implement adaptive threshold based on audio characteristics:
  - Higher threshold for clean recordings
  - Lower threshold for noisy environments
- Add validation metrics to assess noise window quality

**Phase 3: Configuration API**
```rust
pub struct NoiseDetectionConfig {
    pub search_regions: Vec<NoiseSearchRegion>,
    pub z_score_threshold: f32,
    pub min_window_duration_ms: f32,
    pub max_window_duration_ms: f32,
}

pub enum NoiseSearchRegion {
    Start { from_ms: f32, to_ms: f32 },
    End { from_ms: f32, to_ms: f32 },
    InterPhraseGaps { energy_threshold: f32 },
    Manual { start_sample: usize, end_sample: usize },
}
```

**Priority:** MEDIUM-HIGH - Affects noise reduction quality but has fallback

### 3. FFT Planner Reuse

**Location:** `types.rs:14`

**Current Issue:**
```rust
pub fn from_waveform(signal: &[f32]) -> Self {
    // FIXME save the planner for reuse
    let n_fft = signal.len();
    let spectrum = compute_spectrum(signal, n_fft);
    ...
}
```

**Impact:**
- Performance: Creating new FFT planner on every call is expensive
- Particularly problematic for real-time or batch processing

**Proposed Solutions:**

**Option A: Thread-local Planner Cache (Recommended)**
```rust
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static FFT_PLANNER_CACHE: RefCell<FftPlanner<f32>> = 
        RefCell::new(FftPlanner::new());
}

impl Spectrum {
    pub fn from_waveform(signal: &[f32]) -> Self {
        FFT_PLANNER_CACHE.with(|cache| {
            let mut planner = cache.borrow_mut();
            // Use cached planner
            ...
        })
    }
}
```

**Option B: Explicit Planner Parameter**
- Add optional planner parameter to functions
- Allows caller to manage planner lifecycle
- Good for batch processing

**Option C: Lazy Static Global Planner**
- Simple but less flexible
- May have contention issues in multi-threaded scenarios

**Priority:** MEDIUM - Performance optimization, doesn't affect correctness

### 4. Peak Finding Robustness

**Location:** `processing.rs:8`

**Current Issue:**
```rust
// FIXME There must be a better way to do this (library function, or more robust)
pub fn find_peak(signal: &[f32]) -> Option<(usize, f32)> {
    signal.iter().enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, &v)| (i, v))
}
```

**Issues:**
- Only finds global maximum
- No peak prominence/isolation checking
- Doesn't handle NaN values safely (unwrap on partial_cmp)
- No option for finding multiple peaks

**Proposed Solutions:**

**Phase 1: Safer Implementation**
```rust
pub fn find_peak(signal: &[f32]) -> Option<(usize, f32)> {
    signal.iter().enumerate()
        .filter(|(_, &v)| v.is_finite())
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, &v)| (i, v))
}
```

**Phase 2: Advanced Peak Detection**
- Implement prominence-based peak finding
- Support for finding N highest peaks
- Minimum peak separation parameter
- Optional interpolation for sub-bin accuracy

**Phase 3: Consider External Library**
- Evaluate libraries like `findpeaks` or implement proven algorithms
- Parabolic interpolation for frequency precision
- Peak width and prominence metrics

**Priority:** LOW-MEDIUM - Current implementation works for basic use cases

## Additional Enhancements

### 5. Windowing for Spectral Analysis

**Current State:** No windowing applied before FFT

**Proposed Enhancement:**
- Add optional windowing functions (Hamming, Hann, Blackman)
- Reduces spectral leakage in FFT analysis
- Important for accurate frequency analysis

**Implementation:**
```rust
pub enum WindowFunction {
    None,
    Hann,
    Hamming,
    Blackman,
}

impl Spectrum {
    pub fn from_waveform_windowed(
        signal: &[f32], 
        window: WindowFunction
    ) -> Self {
        let windowed = apply_window(signal, window);
        Self::from_waveform(&windowed)
    }
}
```

**Priority:** MEDIUM - Improves spectral analysis quality

### 6. Pre-emphasis Filtering

**Proposed Feature:**
Pre-emphasis filter to boost high frequencies before pitch detection, compensating for the natural -6dB/octave slope in speech signals.

**Benefits:**
- Improves pitch detection for higher harmonics
- Better SNR for upper vocal range

**Implementation:**
```rust
pub fn apply_preemphasis(samples: &[f32], alpha: f32) -> Vec<f32> {
    let mut result = vec![samples[0]];
    for i in 1..samples.len() {
        result.push(samples[i] - alpha * samples[i-1]);
    }
    result
}
```

**Priority:** LOW - Nice to have for improved analysis

### 7. Real-time Processing Support

**Proposed Enhancement:**
Support for streaming/chunked audio processing

**Features:**
- Stateful filters that maintain history across chunks
- Overlap-add for spectral processing
- Latency-aware processing modes

**Priority:** LOW - Future feature for real-time applications

### 8. Additional Cleaning Algorithms

**Proposed Additions:**

1. **Adaptive Noise Reduction**
   - Wiener filtering
   - Spectral subtraction with oversubtraction factor

2. **Click/Pop Removal**
   - Detect and interpolate transient artifacts

3. **Automatic Gain Control (AGC)**
   - Normalize volume variations in recordings

4. **De-essing**
   - Reduce harsh sibilant sounds (6-8 kHz range)

**Priority:** LOW - Expansion features

## Testing Strategy

### Current Test Coverage
- ✅ Basic bandpass filter tests
- ✅ Spectrum and spectrogram tests
- ✅ Utility function tests (RMS, mean, std deviation)
- ⚠️ Limited noise estimation tests (one ignored test)

### Proposed Test Improvements

1. **Parametric Testing**
   - Test filters at multiple sample rates (8kHz, 16kHz, 44.1kHz, 48kHz)
   - Verify frequency response matches specifications
   - Test with various audio lengths and edge cases

2. **Noise Reduction Quality Metrics**
   - Signal-to-Noise Ratio (SNR) improvement measurements
   - Spectral distortion metrics
   - A/B comparison tests with known signals

3. **Performance Benchmarks**
   - Measure processing time for typical audio lengths
   - Memory usage profiling
   - Compare with/without FFT planner caching

4. **Integration Tests**
   - End-to-end cleaning pipeline tests
   - Test with real-world audio samples
   - Validate with pitch detection downstream

## Implementation Priorities

### Phase 1: Critical Fixes (Sprint 1)
1. Fix bandpass filter sample rate issue (#1) - HIGH
2. Improve noise window detection (#2) - HIGH
3. Add comprehensive tests for sample rate handling

### Phase 2: Performance & Robustness (Sprint 2)
1. Implement FFT planner caching (#3) - MEDIUM
2. Enhanced peak finding with safety (#4) - MEDIUM
3. Add windowing functions (#5) - MEDIUM

### Phase 3: Feature Expansion (Sprint 3)
1. Configurable noise detection (#2 Phase 2-3)
2. Pre-emphasis filtering (#6)
3. Additional cleaning algorithms (#8)

### Phase 4: Production Ready (Sprint 4)
1. Real-time processing support (#7)
2. Comprehensive documentation
3. Performance optimization and profiling
4. Production-quality error handling

## Success Metrics

1. **Correctness:** All filters operate correctly across sample rates (8kHz-48kHz)
2. **Quality:** Noise reduction improves SNR by 10-20dB without signal degradation
3. **Performance:** Process 1 second of 44.1kHz audio in < 10ms on average hardware
4. **Robustness:** Handle edge cases (silence, clipping, short clips) gracefully
5. **API:** Clean, well-documented public API with sensible defaults

## Documentation Improvements

1. Add comprehensive rustdoc examples for each public function
2. Create tutorial/guide for common use cases:
   - Basic pitch cleaning pipeline
   - Custom noise profile workflow
   - Performance optimization tips
3. Document audio quality expectations and limitations
4. Add troubleshooting guide for common issues

## Breaking Changes

Potential API changes to consider:
- Making sample_rate parameter actually used in bandpass filter
- Changing noise detection to return Result instead of Option
- Adding builder pattern for cleaning configuration

All breaking changes should be:
1. Clearly documented in CHANGELOG
2. Accompanied by migration guide
3. Implemented with deprecation warnings where possible

## Conclusion

This plan provides a roadmap for improving the audio-cleaning crate from its current functional state to a production-ready, robust audio preprocessing library. The phased approach allows for incremental improvements while maintaining backward compatibility where possible.

Key focus areas:
1. **Correctness** - Fix sample rate handling
2. **Robustness** - Better noise detection
3. **Performance** - FFT planner caching
4. **Quality** - Enhanced algorithms and testing

The improvements will benefit the downstream pitch-detection and learning-tools crates by providing higher quality preprocessed audio.
