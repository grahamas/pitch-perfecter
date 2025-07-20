# Pitch Perfecter - AI Coding Agent Instructions

## Architecture Overview

This is a **real-time audio pitch detection application** built with Rust, egui, and cpal. The app records audio, analyzes pitch in real-time, and provides visual feedback through waveforms and spectrograms.

### Core Components
- **`AudioApp`** (`src/gui/audio_app.rs`) - Central state management for the entire application
- **Audio Pipeline** - `cpal` for I/O → `signal_cleaning` → `track_pitch` → UI display
- **Threading Model** - Audio I/O runs in background threads, GUI updates via shared `Arc<Mutex<Vec<f32>>>` buffers
- **UI Architecture** - Modular UI components in `src/gui/` with shared utilities in `ui_utils.rs`

## Critical Patterns

### Audio Data Flow
1. **Recording**: `cpal` stream → `recorded_samples: Arc<Mutex<Vec<f32>>>` → live waveform display
2. **Analysis**: Latest audio frame → `signal_cleaning` → `track_pitch` (YIN algorithm) → note detection
3. **Playback**: File → optional cleaning → `cpal` output stream with position tracking

### State Management Pattern
```rust
// AudioApp holds ALL application state - avoid splitting state across components
pub struct AudioApp {
    pub recording: bool,
    pub playing: bool,
    pub recorded_samples: Arc<Mutex<Vec<f32>>>,  // Live audio buffer
    pub loaded_audio: Option<LoadedAudio>,       // File audio with embedded sample rate
    pub noise_spectrum: Option<Spectrum>,        // For spectral gating
    // ... etc
}
```

### UI Component Pattern
```rust
// All UI functions take (&AudioApp, &mut egui::Ui) - never split state
pub fn component_ui(app: &AudioApp, ui: &mut egui::Ui) {
    // Use ui_utils::colors, ui_utils::buttons for consistency
    if ui_utils::buttons::play_button(ui, !app.playing).clicked() {
        // Modify app state directly
    }
}
```

## Essential Development Workflows

### Build & Run
```bash
cargo run --bin pitch_perfecter  # Main GUI app
cargo run --bin gen_voice_like   # Voice synthesis utility
cargo check                      # Fast compilation check
```

### Audio Testing
- Test files auto-populate from `./audio/` directory on startup
- Recording creates timestamped files: `recording_20241213_142755.wav`
- Live audio buffers are automatically limited to 10 seconds for performance

### Signal Processing Chain
1. **Raw Audio** → `signal_cleaning::clean_signal_for_pitch()` (optional)
2. **Cleaned Audio** → `track_pitch::track_pitch()` with YIN detector
3. **Pitch Data** → `music_notation::hz_to_note_name()` for display

## Key Integration Points

### Audio Controls Pattern
Use `RecordingControl`/`PlaybackControl` with `Arc<AtomicBool>` for thread-safe stop signaling:
```rust
let control = RecordingControl::new();
app.recording_control = Some(control.clone());
std::thread::spawn(move || {
    audio::record_audio_with_control_and_buffer(&path, control, live_buffer);
});
```

### Sample Rate Coupling
**CRITICAL**: Always pair samples with sample rates. The `LoadedAudio` struct embeds sample rate:
```rust
pub struct LoadedAudio {
    samples: Vec<f32>,
    sample_rate: u32,  // Always stored together
    filepath: Option<String>,
}
```

### Waveform Display Architecture
- **Live**: 5-second rolling window, no zoom/scroll
- **Loaded**: Full file duration, zoom/scroll enabled  
- **Playing**: Full file + red position indicator
- All waveforms use rectified dB scale: `20.0 * sample.abs().log10()`

## Signal Processing Conventions

### Frequency Ranges
- **Vocal bandpass**: 80Hz - 1200Hz (`signal_cleaning::DEFAULT_VOCAL_*`)
- **Noise estimation**: 200ms - 1500ms of audio file
- **dB floor**: -120dB for very quiet signals

### Pitch Detection Config
```rust
PitchTrackerConfig {
    window_size: 1024,    // YIN analysis window
    step_size: 256,       // Overlap for continuous analysis
    power_threshold: 5.0, // Minimum signal power
    clarity_threshold: 0.1, // YIN clarity threshold
}
```

## Common Gotchas

1. **Sample Rate Handling**: Never hardcode 44100 - always use actual sample rate from audio data
2. **Thread Safety**: Audio buffers use `Arc<Mutex<>>` - always check lock() success
3. **UI Updates**: Call `ctx.request_repaint()` for smooth live audio visualization
4. **Buffer Limits**: Live buffers auto-trim to prevent memory bloat (10 seconds max)
5. **Error Handling**: Audio I/O can fail - use `expect()` with descriptive messages

## External Dependencies

- **cpal**: Cross-platform audio I/O - handles device enumeration and streaming
- **hound**: WAV file I/O with automatic format conversion to f32
- **egui/eframe**: Immediate mode GUI with plotting via egui_plot
- **rustfft**: FFT operations for spectrograms and spectral gating
- **pitch-detection**: YIN algorithm implementation for fundamental frequency
- **fundsp**: DSP library for bandpass filtering (uses f64 internally)
