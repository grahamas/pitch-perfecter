# Pitch Perfecter

A modular Rust workspace for audio processing, pitch detection, and sight-singing learning tools.

## Architecture

This project is organized as a Cargo workspace with multiple crates, each with specific responsibilities:

```
pitch-perfecter/
├── Cargo.toml              # Workspace configuration
└── crates/
    ├── audio-utils/        # Low-level audio types and utilities
    ├── audio-cleaning/     # Audio preprocessing and cleaning
    ├── pitch-detection/    # Pitch detection algorithms
    ├── sound-synth/        # Sound generation for testing
    ├── learning-tools/     # Learning logic and exercises (placeholder)
    ├── gui/               # Graphical user interface (placeholder)
    └── playground/        # Examples, demos, and integration tests
```

## Crates

### audio-utils

Low-level audio types and utilities shared across all modules.

**Features:**
- Core audio data types (`MonoAudio`, `Audio` trait)
- Sample format conversions
- Time ↔ samples conversion helpers
- Audio windowing and iteration

**Dependencies:** None (base crate)

### audio-cleaning

Audio preprocessing and cleaning operations.

**Features:**
- Bandpass filtering for vocal frequency range isolation
- Spectral gating for noise reduction
- Background noise spectrum estimation
- DC offset removal and normalization

**Dependencies:** `audio-utils`, `fundsp`, `rustfft`

### pitch-detection-utils

Pitch detection algorithms and musical note utilities.

**Features:**
- YIN pitch detection algorithm
- Pitch tracking over time
- Frequency to musical note conversion
- Configurable detection parameters

**Dependencies:** `audio-utils`, `pitch-detection` (external crate)

### sound-synth

Sound generation utilities for testing and demonstration.

**Features:**
- Voice-like signal generation with harmonics
- Vibrato sine wave synthesis
- Amplitude envelope application
- Test signal generation

**Dependencies:** `audio-utils`

### learning-tools

Sight-singing learning logic and exercises (placeholder for future development).

**Planned Features:**
- Exercise models (intervals, melodies, rhythm)
- Scoring and grading logic
- Progress tracking
- Practice session management

**Dependencies:** `audio-utils`, `audio-cleaning`, `pitch-detection-utils`

### gui

Graphical user interface (placeholder for future development).

**Planned Features:**
- UI framework integration
- Windows, views, and controls
- Event handling
- Wiring UI to learning-tools APIs

**Dependencies:** `learning-tools`, `audio-utils`, `audio-cleaning`, `pitch-detection-utils`

### playground

Examples, demos, and integration tests.

**Features:**
- Demo applications
- Integration examples
- Binary tools (e.g., `gen_voice_like`)

**Dependencies:** All other workspace crates (including `sound-synth` for test signal generation)

## Building

Build all crates:
```bash
cargo build
```

Build a specific crate:
```bash
cargo build -p audio-utils
cargo build -p pitch-detection-utils
```

## Testing

Run all tests:
```bash
cargo test
```

Test a specific crate:
```bash
cargo test -p audio-cleaning
cargo test -p pitch-detection-utils
```

## Examples

Run the pitch detection with cleaning example:
```bash
cargo run --package playground --example pitch_detection_with_cleaning
```

Generate a voice-like test signal:
```bash
cargo run --package playground --bin gen_voice_like
```

## Dependency Graph

The crates follow a clean dependency hierarchy:

```
audio-utils (base, no dependencies)
    ↑
    ├── audio-cleaning
    ├── pitch-detection-utils
    └── sound-synth
            ↑
            └── learning-tools
                    ↑
                    └── gui

playground (depends on all crates for demos)
```

This structure ensures:
- No circular dependencies
- Clear separation of concerns
- Easy independent development and testing
- Minimal coupling between modules

## Development

When adding new features:

1. **Audio utilities** → Add to `audio-utils` if it's generic and reusable
2. **Cleaning/preprocessing** → Add to `audio-cleaning`
3. **Pitch detection** → Add to `pitch-detection-utils`
4. **Sound generation** → Add to `sound-synth` for test signals and synthesis
5. **Learning features** → Add to `learning-tools`
5. **UI components** → Add to `gui`
6. **Examples/demos** → Add to `playground`

Always maintain the dependency direction shown above to avoid circular dependencies.

## License

<!-- Add your license information here -->
