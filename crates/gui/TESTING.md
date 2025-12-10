# GUI Testing Guide

This document provides manual testing procedures for the Pitch Perfecter GUI application.

## Prerequisites

- Microphone connected and working
- Audio input permissions granted
- Cargo and Rust installed

## Running the Application

```bash
cargo run -p gui --bin pitch-perfecter-gui
```

## Test Cases

### 1. Pitch Detection Mode

#### Test 1.1: Basic Pitch Detection
1. Launch the application
2. Ensure "Pitch Detection" tab is selected
3. Click "⏺ Start Recording"
4. Grant microphone permissions if prompted
5. Sing or play a sustained note (e.g., A4 = 440 Hz)
6. **Expected**: 
   - Status shows "Recording..."
   - Detected pitch updates in real-time
   - Note name displays correctly (e.g., "A4")
   - Frequency shows accurate Hz value
   - Clarity bar shows percentage
7. Click "⏹ Stop Recording"
8. **Expected**: Status shows "Recording stopped"

#### Test 1.2: Bandpass Filter
1. Start recording
2. Enable "Bandpass Filter (Vocal Range)"
3. Sing a note
4. **Expected**: Pitch detection is more stable with filtered input
5. Disable the filter
6. **Expected**: May detect more harmonics or noise

#### Test 1.3: File Saving
1. Enable "Save to file in real-time"
2. Set filename to "test_recording.wav"
3. Start recording
4. Sing for a few seconds
5. Stop recording
6. **Expected**: 
   - File "test_recording.wav" created in current directory
   - File contains audio data
7. Test with invalid filename (e.g., "test.txt")
8. **Expected**: Warning message about .wav extension

### 2. Learning Mode

#### Test 2.1: Initial State
1. Switch to "Learning" tab
2. **Expected**:
   - Progress section shows 0/12 mastered for both directions
   - All intervals due for review
   - No exercise active message displayed
   - "Start Exercise" button visible

#### Test 2.2: Starting an Exercise
1. Click "Start Exercise"
2. **Expected**:
   - Current Exercise section shows:
     - Direction (Ascending ↑ or Descending ↓)
     - Interval name (e.g., "Perfect Fifth")
     - Base Note (e.g., "A4")
     - Target Note (e.g., "E5")
   - Controls show "🎤 Start Recording" and "⏭ Skip" buttons
   - Feedback message: "Listen to the interval and sing it!"

#### Test 2.3: Skipping Exercise
1. With an exercise active, click "⏭ Skip"
2. **Expected**:
   - New exercise loaded immediately
   - Different interval displayed (may be same type if multiple due)
   - State returns to ShowingExercise

#### Test 2.4: Recording and Checking Answer
1. Start an exercise
2. Note the target note (e.g., E5 = 659.26 Hz)
3. Click "🎤 Start Recording"
4. **Expected**:
   - Feedback message: "Recording... Sing the target note!"
   - Application starts listening
5. Sing the target note (or close to it)
6. **Expected**:
   - Detected pitch shows in real-time
   - Note name and frequency update
   - Clarity bar shows confidence
7. Click "✓ Check Answer"
8. **Expected**:
   - Recording stops automatically
   - Feedback message shows:
     - Rating (Perfect/Good/Hesitant/Difficult/Incorrect/Blackout)
     - Target note and frequency
     - Your sung note and frequency
   - Feedback color matches rating:
     - Perfect: Green
     - Good: Light Green
     - Hesitant: Yellow
     - Difficult: Light Blue
     - Incorrect: Light Red
     - Blackout: Red
   - "Next Exercise" button appears

#### Test 2.5: Performance Ratings

Test each rating level by singing at different accuracies:

1. **Perfect (Green)**: Sing within 10 cents of target
2. **Good (Light Green)**: Sing within 25 cents of target
3. **Hesitant (Yellow)**: Sing within 50 cents of target (half semitone)
4. **Difficult (Light Blue)**: Sing within 1 semitone of target
5. **Incorrect (Light Red)**: Sing within 2 semitones of target
6. **Blackout (Red)**: Sing more than 2 semitones off target

#### Test 2.6: Progress Tracking
1. Complete several exercises with "Perfect" or "Good" ratings
2. **Expected**:
   - Progress section updates
   - "Due for review" count decreases
   - Eventually some intervals show as mastered (after 3+ correct)
3. Complete exercises with poor ratings
4. **Expected**:
   - Those intervals remain in review queue
   - May need more attempts to master

#### Test 2.7: No Pitch Detected
1. Start an exercise and recording
2. Don't make any sound for a few seconds
3. Click "Check Answer"
4. **Expected**: Message "No pitch detected. Please sing louder!"

#### Test 2.8: All Exercises Completed
1. Complete all due exercises
2. **Expected**: Message "Great job! All intervals reviewed for now."
3. Try to start a new exercise
4. **Expected**: No new exercises available (state stays idle)

### 3. Tab Switching

#### Test 3.1: Switch Between Tabs
1. Start recording in Pitch Detection mode
2. Switch to Learning tab
3. **Expected**:
   - Recording continues in background
   - Learning tab displays correctly
4. Switch back to Pitch Detection tab
5. **Expected**:
   - Recording indicator still shows "Recording..."
   - Pitch detection still active
6. Stop recording
7. Switch to Learning tab
8. **Expected**: Can start learning exercises independently

#### Test 3.2: Learning Mode with Active Recording
1. Go to Learning tab
2. Start an exercise
3. Click "🎤 Start Recording"
4. **Expected**:
   - Recording starts automatically
   - Pitch detection works in learning mode
5. Check answer
6. **Expected**: Recording stops automatically

### 4. Edge Cases and Error Handling

#### Test 4.1: No Microphone
1. Disconnect or disable microphone
2. Try to start recording
3. **Expected**: Error message about device not available

#### Test 4.2: Permissions Denied
1. Start application without granting microphone permissions
2. Try to start recording
3. **Expected**: Error message about permissions

#### Test 4.3: Rapid Button Clicking
1. Rapidly click "Start Recording" / "Stop Recording"
2. **Expected**: Application handles gracefully, no crashes

#### Test 4.4: Window Resizing
1. Resize the application window (smaller and larger)
2. **Expected**:
   - UI adapts to different sizes
   - All controls remain accessible
   - Text doesn't overflow

### 5. Integration Tests

#### Test 5.1: Complete Learning Session
1. Start application
2. Switch to Learning tab
3. Complete 5 exercises with varied performance
4. **Expected**:
   - Progress statistics update correctly
   - Different intervals appear based on spaced repetition
   - Mastered intervals shown accurately

#### Test 5.2: Using Bandpass Filter with Learning
1. Go to Pitch Detection tab
2. Enable "Bandpass Filter"
3. Switch to Learning tab
4. Complete an exercise
5. **Expected**: Pitch detection in learning mode also benefits from filter

## Performance Tests

### Test P1: Latency
1. Start recording in Pitch Detection mode
2. Sing a note and observe display update
3. **Expected**: 
   - Display updates within ~50-100ms
   - No noticeable lag between sound and display

### Test P2: Stability
1. Run application for extended period (10+ minutes)
2. Switch between tabs multiple times
3. Complete many learning exercises
4. **Expected**:
   - No memory leaks
   - No performance degradation
   - Application remains responsive

## Regression Tests

After any code changes, run through:
1. Test 1.1 (Basic Pitch Detection)
2. Test 2.2 (Starting an Exercise)
3. Test 2.4 (Recording and Checking Answer)
4. Test 3.1 (Switch Between Tabs)

## Known Limitations

These are expected behaviors, not bugs:

1. Spectral Gating checkbox does nothing (feature not implemented)
2. Note range for learning is fixed at A3-A5
3. No device selection (uses default input device)
4. No sound playback of reference tones
5. Learning progress not persisted between sessions

## Reporting Issues

When reporting issues, please include:
- Operating system and version
- Rust/Cargo version
- Steps to reproduce
- Expected vs actual behavior
- Any error messages from console
- Audio device information (if relevant)
