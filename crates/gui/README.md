# Pitch Perfecter GUI

A real-time pitch detection GUI application with low-latency audio processing.

## Quick Start

```bash
cargo run -p gui --bin pitch-perfecter-gui
```

**Usage**: Grant microphone access when prompted, click "⏺ Start Recording", and sing or play an instrument. The detected pitch will be displayed in real-time.

## Features

- **Real-time Pitch Detection**: Displays pitch frequency, musical note name, and confidence
- **Low Latency**: ~50ms end-to-end (capture → processing → display)
- **Audio Cleaning**: Configurable bandpass filter for vocal range (80-800 Hz)
- **WAV Recording**: Optional real-time file saving
- **Cross-platform**: Works on Linux, macOS, and Windows

## Architecture

### Threading Model

```
Microphone → Audio Thread (cpal callback)
                  ↓
            Pitch Detection (YIN algorithm, thread-local)
                  ↓
            Channel (pitch results)
                  ↓
            Main Thread (GUI display)
```

**Key Design**: Pitch detection runs directly on the audio callback thread using thread-local storage. This avoids Send/Sync issues with the external `pitch-detection` crate's use of `Rc<RefCell<>>` while achieving minimal latency.

### Components

- **main.rs**: egui application managing UI state and display
- **audio_recorder.rs**: cpal-based audio capture with thread-local pitch processing
- **pitch_processor.rs**: Audio cleaning and pitch detection utilities

### Performance

- **Buffer size**: 4096 samples (~93ms at 44.1kHz)
- **Detection window**: 2048 samples
- **Total latency**: ~50ms (audio capture + processing + display)
- **Processing location**: Audio callback thread (not main thread)

## UI Layout

```
┌────────────────────────────────┐
│     Pitch Perfecter            │
├────────────────────────────────┤
│ Recording                      │
│  [⏺ Start Recording]           │
│  Status: Ready                 │
├────────────────────────────────┤
│ Cleaning Options               │
│  ☑ Bandpass Filter             │
│  ☐ Spectral Gating             │
├────────────────────────────────┤
│ Detected Pitch                 │
│  Note: A4                      │
│  Frequency: 440.00 Hz          │
│  Clarity: ████████░░ 80%       │
├────────────────────────────────┤
│ Save Recording                 │
│  ☐ Save to file in real-time  │
│  Filename: [recording.wav]     │
└────────────────────────────────┘
```

### Controls

**Recording**
- Start/Stop button with status display
- Automatically uses default system input device

**Cleaning Options**
- **Bandpass Filter**: Removes frequencies outside 80-800 Hz (recommended for vocals)
- **Spectral Gating**: Placeholder for noise reduction (requires noise profile)

**Pitch Display**
- **Note**: Musical note in scientific notation (e.g., A4, C#5)
- **Frequency**: Hz value with 2 decimal precision
- **Clarity**: Confidence metric (0-100%)
  - 90-100%: Excellent, stable
  - 70-89%: Good, reliable
  - 50-69%: Fair, may be unstable
  - <50%: Poor, unreliable

**File Saving**
- Real-time WAV recording (32-bit float, mono)
- Saved to current working directory
- Warning if filename doesn't end with `.wav`

## Dependencies

```toml
eframe = "0.30"      # GUI framework (egui)
cpal = "0.15"        # Cross-platform audio I/O
hound = "3.5"        # WAV file I/O
audio-utils          # Audio data structures
audio-cleaning       # Bandpass filtering
pitch-detection-utils # YIN algorithm with ThreadSafeYinDetector
```

## API Requirements

This crate depends on:
- `audio_utils::MonoAudio` - Audio data representation
- `audio_cleaning::clean_audio_for_pitch` - Signal preprocessing
- `pitch_detection_utils::ThreadSafeYinDetector` - Thread-safe pitch detection wrapper (added)
- `pitch_detection_utils::hz_to_note_name` - Frequency to note conversion

**Note**: `ThreadSafeYinDetector` was added to `pitch-detection-utils` to enable thread-safe pitch detection. It wraps the external crate's `YINDetector` (which uses `Rc<RefCell<>>`) with `Arc<Mutex<>>`.

## Troubleshooting

### No Pitch Detected
- **Cause**: Input too quiet, noisy environment, or non-pitched sound
- **Solution**: Increase microphone gain, enable bandpass filter, sing/play louder

### Error Starting Recording
- **Cause**: No input device, permissions denied, or device in use
- **Solution**: Check microphone connection, grant permissions, close other audio apps

### Unstable/Jumping Pitch
- **Cause**: Inconsistent input, background noise, or vibrato
- **Solution**: Reduce noise, enable bandpass filter, practice steady tone

### File Won't Save
- **Cause**: Invalid filename, permission issues, or disk full
- **Solution**: Ensure filename ends with `.wav`, check permissions and disk space

## Limitations

1. **Mono only**: Stereo inputs are mixed to mono
2. **Default device**: No device selection UI
3. **Fixed buffer**: 4096 samples, not configurable
4. **Spectral gating**: Not functional (requires noise profile)

## Implementation Notes

### Design Decisions

**Why thread-local storage?**
The external `pitch-detection` crate uses `Rc<RefCell<>>` for buffer pooling, making it non-`Send`. We use `thread_local!` to create detector instances per-thread, avoiding the need to send the detector across thread boundaries while still processing on the audio thread.

**Why egui?**
Immediate mode GUI is simple to reason about and well-suited for real-time updates. Pure Rust with minimal dependencies and excellent cross-platform support.

**Why fixed buffer size?**
4096 samples at 44.1kHz (~93ms) balances latency and stability. Smaller buffers risk audio glitches; larger buffers increase latency. This is optimal for real-time pitch detection.

### Security

- No unsafe code
- Bounded memory allocations
- File paths not validated (relies on OS, low risk)
- Mutex unwrap acceptable (poisoned mutex = bug)

### Future Enhancements

**High Priority**
- Audio device selection UI
- Configurable buffer size
- Real-time noise profile estimation

**Medium Priority**
- Waveform visualization
- Pitch history graph
- Keyboard shortcuts

**Low Priority**
- MIDI output
- Alternative tuning references (A=432 Hz)
- Dark/light theme toggle

## Building

Development build:
```bash
cargo build -p gui
```

Optimized release:
```bash
cargo build -p gui --release
```

Run tests:
```bash
cargo test -p gui
```

## License

See repository root for license information.
