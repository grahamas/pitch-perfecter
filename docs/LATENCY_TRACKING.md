# Latency Tracking Feature

## Overview

The latency tracking feature provides real-time visibility into the audio processing pipeline's performance, helping developers and users identify potential bottlenecks in the pitch detection system.

## Features

The latency tracking system measures and displays four key metrics:

### 1. Input Device Latency
Currently not available in the implementation due to cpal 0.15 API limitations. This would measure the time from audio capture by the hardware to the callback invocation.

### 2. Processing Latency
Measures the time taken for:
- Audio cleaning (bandpass filtering, spectral gating)
- Pitch detection using the YIN algorithm

**Color coding:**
- Green: < 20ms (excellent)
- Orange: 20-50ms (warning)

### 3. Callback to Output Latency
Measures the total time from when the audio callback is invoked to when the pitch result is ready to be displayed.

**Color coding:**
- Green: < 30ms (excellent)
- Orange: 30-50ms (acceptable)
- Red: > 50ms (high latency)

### 4. End-to-End Latency
The total latency including input device latency (if available) and processing time.

**Color coding:**
- Green: < 50ms (excellent)
- Orange: 50-70ms (acceptable)
- Red: > 70ms (high latency)

## Implementation Details

### Core Components

1. **LatencyMetrics struct** (`crates/audio-utils/src/latency.rs`)
   - Stores timestamps for callback, processing start, and processing end
   - Provides methods to calculate various latency metrics
   - Includes comprehensive unit tests

2. **PitchResult enhancement** (`crates/gui/src/pitch_processor.rs`)
   - Extended to include `LatencyMetrics`
   - Latency is tracked throughout the processing pipeline

3. **Audio Recorder integration** (`crates/gui/src/audio_recorder.rs`)
   - Captures callback timestamp using `Instant::now()`
   - Passes latency metrics through the processing chain

4. **GUI Display** (`crates/gui/src/main.rs`)
   - New "Latency Metrics" section in the UI
   - Color-coded display based on performance thresholds
   - Warning messages for high latency situations

### Usage

When the application is running and recording audio:
1. Start recording by clicking "Start Recording"
2. The "Latency Metrics" section will display real-time measurements
3. Color indicators help identify performance issues
4. If latency exceeds 70ms, a warning suggests disabling audio cleaning

### Performance Tips

If you observe high latency:
1. Disable "Spectral Gating" as it's computationally intensive
2. Consider disabling "Bandpass Filter" if still experiencing issues
3. Reduce the window size or hop size in the pitch detector settings
4. Check system load and ensure adequate CPU resources

## Testing

The latency module includes comprehensive unit tests:
- Metrics creation and initialization
- Processing duration calculation
- Total latency calculation
- End-to-end latency with device latency

Run tests with:
```bash
cargo test -p audio-utils
```

## Future Enhancements

Potential improvements:
1. Extract actual input device latency from cpal when upgrading to a newer version
2. Add latency history visualization (graph over time)
3. Export latency statistics for performance analysis
4. Add configurable latency thresholds
5. Implement adaptive quality settings based on latency
