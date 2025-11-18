# Pitch Perfecter GUI - Usage Guide

## Quick Start

1. **Build and Run**:
   ```bash
   cargo run -p gui --bin pitch-perfecter-gui
   ```

2. **Allow Microphone Access**: Your operating system may prompt you to allow microphone access. Grant permission.

3. **Start Recording**: Click the "⏺ Start Recording" button

4. **Sing or Play**: The GUI will display the detected pitch in real-time

5. **Stop Recording**: Click "⏹ Stop Recording" when done

## Detailed Usage

### Recording Audio

#### Starting
1. Ensure your microphone is connected and set as the default input device
2. Click "⏺ Start Recording"
3. Speak, sing, or play an instrument into your microphone
4. The detected pitch will appear in the "Detected Pitch" section

#### During Recording
- The pitch display updates continuously in real-time
- You can toggle cleaning options while recording (changes apply to new audio)
- If file saving is enabled, audio is being written to disk

#### Stopping
- Click "⏹ Stop Recording" to end the session
- The last detected pitch remains visible
- If saving to file, the WAV file is finalized

### Audio Cleaning Options

#### Bandpass Filter (Vocal Range)
- **What it does**: Filters out frequencies outside the typical vocal range (80-800 Hz)
- **When to use**: Enabled by default, good for most vocal/singing applications
- **Benefits**: Removes low-frequency rumble and high-frequency noise
- **Effect on pitch detection**: Usually improves accuracy by removing irrelevant frequencies

#### Spectral Gating (Noise Reduction)
- **Current status**: Placeholder - not fully functional in real-time mode
- **Future functionality**: Will reduce background noise using spectral subtraction
- **Note**: Requires pre-recorded noise profile for best results

**Recommendation**: Keep Bandpass Filter enabled for singing/voice applications. Disable if detecting pitch from instruments outside the vocal range.

### Understanding the Pitch Display

#### Note Name
- Shows the nearest musical note (e.g., A4, C#5, Bb3)
- Uses standard scientific pitch notation
- Octave numbering: C4 is middle C
- Sharps (#) are used for accidentals

#### Frequency
- Displays the detected frequency in Hertz (Hz)
- Shown with 2 decimal places for precision
- Standard tuning: A4 = 440.00 Hz
- If no pitch detected: Section shows "No pitch detected"

#### Clarity
- Visual progress bar showing detection confidence (0-100%)
- Higher percentage = more confident/stable detection
- Low clarity may indicate:
  - Noisy environment
  - Multiple pitches present
  - Very quiet input
  - Non-pitched sounds (percussion, speech, etc.)

**Interpreting Clarity Values**:
- 90-100%: Excellent, very stable pitch
- 70-89%: Good, reliable detection
- 50-69%: Fair, may be unstable
- Below 50%: Poor, results may be unreliable

### File Saving

#### Enabling
1. Check "Save to file in real-time"
2. Enter desired filename (must end with .wav)
3. Start recording

#### File Format
- Format: WAV (uncompressed)
- Channels: Mono (mixed down from stereo if necessary)
- Sample format: 32-bit float
- Sample rate: Matches input device sample rate (typically 44100 or 48000 Hz)

#### File Location
- Files are saved to the current working directory
- If running from cargo: Repository root directory
- To specify different location: Include path in filename (e.g., "recordings/my_recording.wav")

#### Notes
- File is written in real-time as audio is captured
- Stopping recording finalizes and closes the file
- If recording fails, partial files may be created
- Warning shown if filename doesn't end with .wav

## Tips for Best Results

### Microphone Setup
1. **Position**: 6-12 inches from mouth/instrument
2. **Environment**: Quiet room with minimal echo/reverb
3. **Levels**: Adjust input gain to avoid clipping (distortion)
4. **Type**: Dynamic or condenser microphone recommended

### Pitch Detection
1. **Sustain notes**: Hold pitches steady for at least 0.5 seconds
2. **Avoid vibrato**: Wide vibrato may confuse detection
3. **Single pitches**: One note at a time (no chords)
4. **Volume**: Moderate to loud input gives best results

### Performance
1. **Buffer size**: Fixed at 4096 samples (good for most systems)
2. **Latency**: Expect ~100ms delay from sound to display
3. **CPU usage**: Moderate - YIN algorithm is computationally intensive
4. **Background apps**: Close other audio applications to avoid conflicts

## Troubleshooting

### No Pitch Detected
**Possible causes**:
- Input too quiet: Increase microphone gain
- Noisy environment: Enable bandpass filter, move to quieter location
- Non-pitched sound: Pitch detection works on sustained tones, not percussion or speech
- Input device issue: Check microphone connection and default device settings

**Solutions**:
1. Verify microphone is working (test in other apps)
2. Increase input volume in system settings
3. Sing/play louder
4. Enable bandpass filter if not already on

### Error Starting Recording
**Possible causes**:
- No input device available
- Microphone access denied
- Device in use by another application

**Solutions**:
1. Check microphone is connected
2. Grant microphone permissions when prompted
3. Close other applications using microphone
4. Restart the application

### Unstable/Jumping Pitch
**Possible causes**:
- Inconsistent singing/playing
- Background noise
- Vibrato too wide
- Multiple sound sources

**Solutions**:
1. Sing/play more steadily
2. Reduce background noise
3. Move closer to microphone
4. Enable bandpass filter
5. Practice consistent tone production

### File Won't Save
**Possible causes**:
- Invalid filename
- No write permissions in directory
- Disk full

**Solutions**:
1. Ensure filename ends with .wav
2. Check directory permissions
3. Try saving to different location
4. Verify sufficient disk space

### High CPU Usage
**Expected behavior**: Pitch detection is computationally intensive

**If excessive**:
1. Close other applications
2. Check for other background processes
3. Note: Real-time audio processing requires continuous CPU

## System Requirements

### Minimum
- **OS**: Linux, macOS, or Windows (with ALSA on Linux)
- **CPU**: Dual-core processor, 2 GHz
- **RAM**: 512 MB available
- **Audio**: Any microphone or line input device
- **Display**: 400x500 minimum window size

### Recommended
- **OS**: Modern Linux distribution with ALSA, macOS 10.15+, Windows 10+
- **CPU**: Quad-core processor, 2.5 GHz or better
- **RAM**: 2 GB available
- **Audio**: USB microphone or audio interface
- **Display**: 1920x1080 or higher

## Known Limitations

1. **Mono Only**: Stereo inputs are mixed to mono
2. **Default Device**: No device selection UI (uses system default)
3. **Buffer Size**: Fixed, not user-configurable
4. **Spectral Gating**: Not functional in real-time mode
5. **No Visualization**: No waveform or spectrum display
6. **No History**: Previous pitches not stored or displayed

## Keyboard Shortcuts

Currently, no keyboard shortcuts are implemented. Future versions may include:
- Space: Toggle recording
- Esc: Stop recording and close
- S: Toggle file saving
- B: Toggle bandpass filter

## Getting Help

If you encounter issues:
1. Check this usage guide
2. Review the README.md for architecture details
3. Check the issue tracker on GitHub
4. File a new issue with:
   - Operating system and version
   - Microphone/audio device details
   - Error messages or unexpected behavior
   - Steps to reproduce the issue

## Advanced Usage

### Custom Build Flags
Build with optimizations for better performance:
```bash
cargo build -p gui --bin pitch-perfecter-gui --release
```

### Running from Binary
After building, run directly:
```bash
./target/release/pitch-perfecter-gui
```

### Custom Working Directory
To save files to specific location:
```bash
cd /path/to/recordings
/path/to/pitch-perfecter-gui
```

## Further Reading

- **YIN Algorithm**: Research paper on pitch detection method used
- **egui Documentation**: Learn about the GUI framework
- **cpal Documentation**: Understand audio I/O in Rust
- **Project README**: High-level architecture and design decisions
