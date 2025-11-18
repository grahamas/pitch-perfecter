# GUI Crate

This crate provides a graphical user interface for the Pitch Perfecter application using the egui framework.

## Features

- **Real-time Audio Recording**: Captures audio from your default input device (microphone)
- **Live Pitch Detection**: Displays detected pitch frequency and musical note name in real-time
- **Audio Cleaning Options**:
  - Bandpass Filter: Isolates vocal frequency range (80-800 Hz)
  - Spectral Gating: Reduces background noise (placeholder for future implementation)
- **File Saving**: Optional real-time WAV file recording
- **Responsive UI**: Designed for real-time performance with continuous updates

## Running the Application

```bash
cargo run -p gui --bin pitch-perfecter-gui
```

Or from the repository root:

```bash
cd crates/gui
cargo run --bin pitch-perfecter-gui
```

## Architecture

### Components

1. **Main App (`main.rs`)**: The egui application that manages UI state and coordinates between recording and processing
2. **Audio Recorder (`audio_recorder.rs`)**: Handles audio input using cpal, manages the recording stream
3. **Pitch Processor (`pitch_processor.rs`)**: Processes audio chunks and detects pitch using YIN algorithm

### Threading Model

- **Audio Recording Thread**: Runs in cpal's audio callback, captures samples and sends chunks to main thread
- **Main Thread**: Runs the GUI, receives audio chunks, performs pitch detection, and updates the display

This design ensures:
- Low latency audio capture
- Real-time pitch detection without blocking the audio thread
- Responsive UI updates

### Data Flow

```
Microphone → Audio Recording Thread → Audio Chunks (channel) → Main Thread
                                                                     ↓
                                                              Pitch Detection
                                                                     ↓
                                                               GUI Display
```

## Dependencies

- **eframe/egui**: Immediate mode GUI framework
- **cpal**: Cross-platform audio I/O library
- **hound**: WAV file I/O for recording
- **audio-utils**: Audio data structures
- **audio-cleaning**: Audio preprocessing
- **pitch-detection-utils**: Pitch detection algorithms

## Required APIs from Other Crates

The GUI implementation uses the following existing APIs:

- `audio_utils::MonoAudio`: Audio data representation with sample rate
- `audio_cleaning::clean_audio_for_pitch`: Bandpass filtering for vocal range
- `pitch_detection_utils::ExternalYinDetector`: YIN pitch detection algorithm
- `pitch_detection_utils::hz_to_note_name`: Converts frequency to note name

No modifications to other crates are required.

## Future Enhancements

### Recommended API Additions for Other Crates

While the current GUI works with existing APIs, the following additions would improve functionality:

#### audio-cleaning crate

1. **Real-time Noise Profile Estimation**:
   ```rust
   pub fn estimate_noise_from_audio_chunk(audio: &MonoAudio) -> Spectrum
   ```
   - Currently, spectral gating requires pre-recorded noise profile
   - A real-time version could estimate noise from quiet portions of the signal

2. **Configurable Bandpass Parameters**:
   ```rust
   pub fn bandpass_filter(audio: &MonoAudio, low_hz: f32, high_hz: f32) -> MonoAudio
   ```
   - Allow users to customize the frequency range (currently hardcoded to 80-800 Hz)

#### pitch-detection crate

1. **Confidence Metrics**:
   - The current clarity metric could be enhanced with more detailed confidence information
   - Add methods to adjust detection sensitivity in real-time

2. **Thread-Safe Detector**:
   - Consider providing a `Send` version of the pitch detector for multi-threaded use
   - Current implementation uses `Rc` which is not thread-safe

## Known Limitations

1. **Spectral Gating**: Currently disabled as it requires a pre-recorded noise profile
2. **Audio Device Selection**: Uses default input device; no device picker UI
3. **Buffer Size**: Fixed at 4096 samples; could be made configurable for latency tuning
4. **Mono Only**: Only processes mono audio; stereo inputs are mixed down

## Performance Considerations

- Buffer size of 4096 samples provides good balance between latency and stability
- YIN detector window size of 2048 samples suitable for real-time use
- GUI requests continuous repaint for responsive updates
- Audio processing happens on main thread to avoid Send/Sync issues with detector
