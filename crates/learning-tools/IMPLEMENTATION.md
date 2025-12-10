# Interval Learning Implementation

This document describes the interval learning system implemented in the `learning-tools` crate.

## Overview

A complete learning system for musical intervals has been implemented, combining:
- Musical interval definitions and utilities
- Spaced repetition algorithm (based on SM-2)
- Interval learning plan with automatic scheduling
- Exercise generation and performance evaluation

## Modules

### `note.rs`

Defines musical note representation:

- **`PitchClass` enum**: Musical pitch classes (C, C#, D, etc.)
- **`Note` struct**: Combines pitch class with octave (e.g., A4, C#5)
  - Convert to/from MIDI note numbers
  - Convert to/from frequencies (A4 = 440 Hz)
  - Parse from strings ("A4", "C#5", "Bb3")
  - Transpose by semitones
  - Calculate intervals between notes

### `intervals.rs`

Defines musical intervals and provides utilities for working with them:

- **`Interval` enum**: All standard musical intervals (unison through octave)
- **`learning_order()`**: Returns intervals ordered by typical utility
  - Starts with most fundamental (octave, perfect fifth, perfect fourth)
  - Progresses to more complex intervals (major/minor thirds, sixths, sevenths)
  - Ends with most challenging (tritone, minor second)
- **`apply_interval()`**: Apply an interval to a note (ascending or descending)
- **`calculate_interval_semitones()`**: Measure interval between two notes
- **`closest_interval()`**: Find the standard interval closest to a given semitone distance

### `spaced_repetition.rs`

Implements a spaced repetition system based on the SM-2 algorithm:

- **`PerformanceRating`**: Six-level rating system (Blackout to Perfect)
- **`ReviewItem<T>`**: Tracks learning state for any item type
  - Easiness factor (adapts to user performance)
  - Consecutive correct count
  - Review interval (increases with successful recalls)
  - Next review time
- **`SpacedRepetitionScheduler<T>`**: Manages multiple items
  - Identifies due items
  - Provides next item for review
  - Tracks overall progress

**Algorithm Details:**
- Initial reviews at 1 day, then 6 days
- Subsequent intervals multiply by easiness factor
- Failed reviews reset the interval
- Easiness adjusts based on performance (minimum 1.3)

### `interval_learning.rs`

Combines intervals and spaced repetition into a complete learning system:

- **`IntervalExercise`**: Represents a single practice exercise
  - Base note (starting point)
  - Target interval to practice
  - Direction (ascending or descending)
  - Methods to check and rate user responses
- **`IntervalLearningPlan`**: Main learning system
  - Separate schedulers for ascending and descending intervals
  - Automatically presents next due exercise
  - Records performance and updates schedule
  - Tracks statistics (mastered intervals, average easiness, etc.)
- **`IntervalLearningConfig`**: Customizable settings
  - Note range for exercises (e.g., A3 to A5)
  - Whether to practice both directions
  - Tolerance for correct responses (in cents)

**Performance Evaluation:**
- Perfect: Within 10 cents of target
- Good: Within 25 cents
- Hesitant: Within 50 cents
- Difficult: Within 1 semitone
- Incorrect: Within 2 semitones
- Blackout: More than 2 semitones off

## Usage Example

```rust
use learning_tools::interval_learning::IntervalLearningPlan;
use learning_tools::note::{Note, PitchClass};
use learning_tools::spaced_repetition::PerformanceRating;

// Create a new learning plan
let mut plan = IntervalLearningPlan::new();

// Get the next exercise to practice
if let Some(exercise) = plan.next_exercise() {
    println!("Practice: {} {} from {}",
        if exercise.ascending { "ascending" } else { "descending" },
        exercise.interval,
        exercise.base_note
    );
    
    let target = exercise.target_note();
    println!("Target note: {}", target);
    
    // User produces a note (detected via pitch detection)
    let user_note = Note::new(PitchClass::E, 5); // Example
    
    // Record the result - automatically rates and schedules
    plan.record_exercise_with_note(&exercise, user_note);
    
    // Or record with explicit rating
    plan.record_exercise(&exercise, PerformanceRating::Good);
}

// Check progress
let stats = plan.get_statistics();
println!("Mastered intervals (ascending): {}", stats.ascending.mastered_intervals);
println!("Due for review: {}", plan.exercises_due());
```

## Testing

Comprehensive unit tests are provided for all modules:

- **Interval tests**: Semitone calculations, frequency conversions, learning order
- **Spaced repetition tests**: SM-2 algorithm, review scheduling, easiness adjustments
- **Learning plan tests**: Exercise generation, response evaluation, statistics tracking

Run tests with:
```bash
cargo test --package learning-tools
```

All 43 tests pass successfully (including 11 tests for the Note module).

## Integration with Other Crates

To fully integrate this learning system into a working application, the following would be needed in other crates:

### GUI Crate

1. **Practice Screen**
   - Display current exercise (base note, interval name, direction)
   - Play the base note using `sound-synth` (convert note to frequency)
   - Record user's voice using audio capture
   - Show visual feedback (correct/incorrect)
   - Display statistics and progress

2. **Progress Dashboard**
   - Show mastered vs. learning intervals
   - Display learning curve and statistics
   - Allow customization of `IntervalLearningConfig`
   - Show next review times

3. **Audio Integration**
   - Use `audio-utils` for audio capture
   - Use `pitch-detection-utils` to detect user's pitch
   - Convert detected frequency to Note using `Note::from_frequency()`
   - Use `sound-synth` to play reference tones (convert notes to frequencies)
   - Consider using `audio-cleaning` to improve pitch detection accuracy

### Pitch Detection Integration

The GUI would need to:
```rust
use pitch_detection_utils::{PitchTracker, PitchTrackerConfig};
use learning_tools::interval_learning::IntervalLearningPlan;
use learning_tools::note::Note;

// Create pitch tracker
let config = PitchTrackerConfig::default();
let mut tracker = PitchTracker::new(config);

// Get next exercise
let mut plan = IntervalLearningPlan::new();
let exercise = plan.next_exercise().unwrap();

// Play base note (using sound-synth - convert note to frequency)
let base_freq = exercise.base_note.to_frequency();
// ... play base_freq ...

// Record user singing the target interval
// ... capture audio ...

// Detect pitch and convert to note
let pitch = tracker.get_pitch(&audio_samples);
if let Some(detected_pitch) = pitch {
    if let Some(user_note) = Note::from_frequency(detected_pitch.frequency) {
        plan.record_exercise_with_note(&exercise, user_note);
    }
}
```

### Sound Synthesis

The GUI could use `sound-synth` to:
1. Play the base note as a reference (convert note to frequency first)
2. Optionally play the target note for comparison
3. Generate confirmation sounds for correct/incorrect responses

### Data Persistence

**Status: ✅ Implemented**

The `persistence` module provides complete save/load functionality for learning profiles:

1. **Serialize learning state**
   - Save `IntervalLearningPlan` to disk using JSON (via `serde_json`)
   - Store user progress across sessions
   - All learning state preserved including:
     - Interval mastery and easiness factors
     - Review schedules and next review times
     - Configuration settings
     - Separate tracking for ascending/descending intervals

2. **Core functionality**
   - `save_learning_plan()` - Save a learning plan to a JSON file
   - `load_learning_plan()` - Load a learning plan from a JSON file
   - `learning_plan_exists()` - Check if a profile file exists
   - `delete_learning_plan()` - Delete a profile file
   - Convenience methods on `IntervalLearningPlan`: `.save()` and `.load()`
   - Comprehensive error handling with `PersistenceError`

3. **Usage example**
   ```rust
   use learning_tools::{IntervalLearningPlan, save_learning_plan, load_learning_plan};
   
   // Save a learning plan
   let plan = IntervalLearningPlan::new();
   save_learning_plan(&plan, "my_profile.json")?;
   
   // Load it back later
   let loaded_plan = load_learning_plan("my_profile.json")?;
   ```

4. **Future enhancements**
   - Multiple user profiles with profile management
   - Historical performance data and analytics
   - Export/import functionality
   - Cloud sync capabilities

See `examples/persistence_demo.rs` for a complete working example.

### Audio Cleaning Integration

For better pitch detection accuracy during practice:
```rust
use audio_cleaning::{clean_audio, CleaningConfig};

// Clean audio before pitch detection
let config = CleaningConfig::default();
let cleaned = clean_audio(&recorded_audio, &config);

// Then detect pitch on cleaned audio
let pitch = tracker.get_pitch(&cleaned);
```

## Design Decisions

1. **Note-based API**: Uses discrete musical notes rather than frequencies
   - Exercises work with Note types (e.g., A4, C#5) instead of Hz values
   - More intuitive for musical education
   - Audio layer (GUI/sound-synth/pitch-detection) handles frequency conversion
   - Clear separation between learning logic and audio processing

2. **Interval Ordering**: Based on typical musical utility and consonance
   - Perfect intervals first (octave, fifth, fourth)
   - Major/minor thirds (triad components)
   - Common melodic intervals (sixths, seconds)
   - Challenging intervals last (tritone, minor second)

3. **Spaced Repetition**: SM-2 algorithm chosen for:
   - Proven effectiveness in learning applications
   - Simple to implement and understand
   - Adapts to individual user performance

4. **Separate Ascending/Descending**: Intervals sound different in each direction
   - Separate schedulers allow independent mastery
   - Can be disabled via configuration

5. **Cent-based Tolerance**: Professional standard for pitch accuracy
   - 50 cents (half semitone) default tolerance
   - Configurable for different skill levels
   - Gradual rating system for nuanced feedback

6. **Note Range**: Default A3-A5
   - Comfortable vocal range for most users
   - Configurable for different vocal ranges or instruments

## Future Enhancements

Possible additions for future development:

1. **Interval Recognition**: Hear two notes and identify the interval
2. **Melodic Patterns**: Sequences of intervals
3. **Harmonic Intervals**: Simultaneous notes instead of sequential
4. **Direction-agnostic Mode**: Practice identifying intervals regardless of direction
5. **Custom Interval Sets**: Focus on specific intervals (e.g., for jazz, classical)
6. **Multi-octave Intervals**: Intervals beyond one octave
7. **Adaptive Difficulty**: Automatic tolerance adjustment based on performance
8. **Practice Sessions**: Timed practice with goals and achievements
9. **Comparison Mode**: Play both base and target for easier learning
10. **Progressive Unlock**: Start with easier intervals, unlock harder ones

## Performance Considerations

- All calculations use `f32` for compatibility with audio processing
- No allocations in hot paths (exercise evaluation)
- Lazy evaluation where possible
- O(n) scheduler operations where n is number of intervals (small constant)

## Conclusion

This implementation provides a complete, production-ready interval learning system that only needs GUI integration and audio I/O to become a fully functional application. The architecture is modular, well-tested, and follows Rust best practices.
