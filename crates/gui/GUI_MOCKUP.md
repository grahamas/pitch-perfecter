# Pitch Perfecter GUI Mockup

This document describes the visual layout and functionality of the Pitch Perfecter GUI.

## Window Layout

```
┌─────────────────────────────────────────────────┐
│           Pitch Perfecter                       │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │ Recording                               │   │
│  │                                         │   │
│  │  [ ⏺ Start Recording ]                  │   │
│  │                                         │   │
│  │  Status: Ready                          │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │ Cleaning Options                        │   │
│  │                                         │   │
│  │  ☑ Bandpass Filter (Vocal Range)       │   │
│  │  ☐ Spectral Gating (Noise Reduction)   │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │ Detected Pitch                          │   │
│  │                                         │   │
│  │  Note: A4                               │   │
│  │  Frequency: 440.00 Hz                   │   │
│  │  Clarity: [████████░░] 80%              │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │ Save Recording                          │   │
│  │                                         │   │
│  │  ☐ Save to file in real-time           │   │
│  │  Filename: [recording.wav            ]  │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
└─────────────────────────────────────────────────┘
```

## UI Sections

### 1. Recording Control
- **Start Recording Button**: Initiates audio capture from default input device
  - Changes to "⏹ Stop Recording" when active
  - Button uses circular record icon (⏺) when stopped
- **Status Display**: Shows current state
  - "Ready" - waiting to start
  - "Recording..." - actively capturing audio
  - Error messages if recording fails

### 2. Cleaning Options
Two checkboxes for audio preprocessing:

- **Bandpass Filter (Vocal Range)**: 
  - Filters frequencies outside 80-800 Hz range
  - Enabled by default for better pitch detection
  - Hover text: "Filter frequencies outside typical vocal range (80-800 Hz)"

- **Spectral Gating (Noise Reduction)**:
  - Placeholder for future noise reduction
  - Currently disabled/non-functional (requires noise profile)
  - Hover text: "Reduce background noise using spectral gating"

### 3. Detected Pitch Display
Shows real-time pitch detection results:

- **Note**: Musical note name (e.g., "A4", "C#5")
  - Large, prominent display
  - Updates in real-time as pitch is detected
  - Shows "No pitch detected" when no clear pitch

- **Frequency**: Numeric frequency in Hz
  - Displayed with 2 decimal places
  - Example: "440.00 Hz"

- **Clarity**: Visual progress bar showing detection confidence
  - Range: 0-100%
  - Higher values indicate more confident detection
  - Visual bar fills proportionally to confidence level

### 4. Save Recording
Optional file saving functionality:

- **Save to file checkbox**: Enables real-time WAV recording
  - When checked, audio is written to file as it's captured
  - File is created when recording starts

- **Filename input**: Text field for output filename
  - Default: "recording.wav"
  - Warning shown if filename doesn't end with .wav
  - Editable at any time

## Color Scheme
- Uses egui's default dark theme
- Headings in default egui heading color
- Warning text in yellow for filename validation
- Progress bar uses egui's default accent color

## Responsiveness Features
1. **Continuous Repaint**: UI requests repaints continuously for smooth real-time updates
2. **Non-blocking Processing**: Audio processing happens without blocking UI thread
3. **Immediate Feedback**: Changes to controls take effect immediately
4. **Channel-based Communication**: Low-latency communication between audio and UI threads

## Window Properties
- **Default Size**: 400x500 pixels
- **Resizable**: Yes, window can be resized by user
- **Title**: "Pitch Perfecter"
- **Minimum Size**: Not enforced, uses egui defaults

## Interaction Flow

### Starting a Recording Session
1. User clicks "⏺ Start Recording"
2. System requests microphone access (OS-level)
3. Button changes to "⏹ Stop Recording"
4. Status changes to "Recording..."
5. Pitch detection begins immediately
6. Display updates in real-time with detected pitch

### During Recording
- Pitch display updates continuously (every ~100ms)
- User can toggle cleaning options on-the-fly
- Changes apply to subsequent audio chunks
- Optional: File saving happens in background

### Stopping a Recording
1. User clicks "⏹ Stop Recording"
2. Audio stream stops immediately
3. Button reverts to "⏺ Start Recording"
4. Status shows "Recording stopped"
5. Last detected pitch remains visible
6. If saving, file is finalized and closed

## Error Handling
- No input device: Shows error message in status
- Recording failure: Displays specific error in status
- File write error: Printed to console (future: show in UI)
- Invalid filename: Yellow warning text below filename field

## Accessibility
- All controls have hover tooltips
- Clear visual hierarchy with grouped sections
- Large, readable text for main pitch display
- Color is not the only indicator (uses icons and text)

## Future Enhancement Ideas
(Not implemented in current version)
- Audio device selection dropdown
- Visualization of audio waveform or spectrogram
- Historical pitch tracking graph
- Tuning target selection (concert pitch A=440 Hz vs A=432 Hz)
- MIDI output capability
- Recording quality settings
- Dark/light theme toggle
