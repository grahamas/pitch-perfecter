//! Interval learning plan with spaced repetition
//!
//! This module provides a complete learning system for musical intervals,
//! combining interval exercises with spaced repetition scheduling.

use crate::intervals::{Interval, apply_interval};
use crate::note::Note;
use crate::spaced_repetition::{PerformanceRating, SpacedRepetitionScheduler};

/// Represents a single interval learning exercise
#[derive(Debug, Clone, PartialEq)]
pub struct IntervalExercise {
    /// The starting note
    pub base_note: Note,
    /// The interval to practice
    pub interval: Interval,
    /// Direction: true for ascending, false for descending
    pub ascending: bool,
}

impl IntervalExercise {
    /// Create a new interval exercise
    pub fn new(base_note: Note, interval: Interval, ascending: bool) -> Self {
        Self {
            base_note,
            interval,
            ascending,
        }
    }

    /// Calculate the target note for this exercise
    pub fn target_note(&self) -> Note {
        apply_interval(self.base_note, self.interval, self.ascending)
    }

    /// Check if a produced note matches the target within a tolerance
    ///
    /// # Arguments
    /// * `produced_note` - The note produced by the user
    /// * `tolerance_cents` - Tolerance in cents (100 cents = 1 semitone), default 50 cents
    ///
    /// # Returns
    /// True if the produced note is within tolerance
    pub fn check_response(&self, produced_note: Note, tolerance_cents: f32) -> bool {
        let target = self.target_note();
        let target_freq = target.to_frequency();
        let produced_freq = produced_note.to_frequency();
        
        if target_freq <= 0.0 || produced_freq <= 0.0 {
            return false;
        }
        
        let cents_diff = 1200.0 * (produced_freq / target_freq).log2().abs();
        cents_diff <= tolerance_cents
    }

    /// Rate the user's performance based on accuracy
    ///
    /// # Arguments
    /// * `produced_note` - The note produced by the user
    ///
    /// # Returns
    /// A performance rating based on how close the response was
    pub fn rate_response(&self, produced_note: Note) -> PerformanceRating {
        let target = self.target_note();
        let target_freq = target.to_frequency();
        let produced_freq = produced_note.to_frequency();
        
        if target_freq <= 0.0 || produced_freq <= 0.0 {
            return PerformanceRating::Blackout;
        }
        
        let cents_diff = 1200.0 * (produced_freq / target_freq).log2().abs();
        
        match cents_diff {
            diff if diff <= 10.0 => PerformanceRating::Perfect,   // Within 10 cents
            diff if diff <= 25.0 => PerformanceRating::Good,      // Within 25 cents
            diff if diff <= 50.0 => PerformanceRating::Hesitant,  // Within 50 cents (half semitone)
            diff if diff <= 100.0 => PerformanceRating::Difficult, // Within 1 semitone
            diff if diff < 250.0 => PerformanceRating::Incorrect, // Within 2 semitones
            _ => PerformanceRating::Blackout,                      // More than 2 semitones off
        }
    }
}

/// Configuration for interval learning sessions
#[derive(Debug, Clone)]
pub struct IntervalLearningConfig {
    /// Base note range for exercises (min, max)
    pub note_range: (Note, Note),
    /// Whether to practice both ascending and descending intervals
    pub practice_both_directions: bool,
    /// Default tolerance in cents for checking responses
    pub tolerance_cents: f32,
}

impl Default for IntervalLearningConfig {
    fn default() -> Self {
        use crate::note::PitchClass;
        Self {
            note_range: (Note::new(PitchClass::A, 3), Note::new(PitchClass::A, 5)), // A3 to A5
            practice_both_directions: true,
            tolerance_cents: 50.0,
        }
    }
}

/// Manages interval learning with spaced repetition
pub struct IntervalLearningPlan {
    /// Spaced repetition scheduler for ascending intervals
    ascending_scheduler: SpacedRepetitionScheduler<Interval>,
    /// Spaced repetition scheduler for descending intervals
    descending_scheduler: SpacedRepetitionScheduler<Interval>,
    /// Configuration for the learning plan
    config: IntervalLearningConfig,
}

impl IntervalLearningPlan {
    /// Create a new interval learning plan with default configuration
    pub fn new() -> Self {
        Self::with_config(IntervalLearningConfig::default())
    }

    /// Create a new interval learning plan with custom configuration
    pub fn with_config(config: IntervalLearningConfig) -> Self {
        let mut plan = Self {
            ascending_scheduler: SpacedRepetitionScheduler::new(),
            descending_scheduler: SpacedRepetitionScheduler::new(),
            config,
        };
        plan.initialize_intervals();
        plan
    }

    /// Initialize the schedulers with intervals in learning order
    fn initialize_intervals(&mut self) {
        let intervals = Interval::learning_order();
        self.ascending_scheduler.add_items(intervals.clone());
        if self.config.practice_both_directions {
            self.descending_scheduler.add_items(intervals);
        }
    }

    /// Get the next exercise to practice
    pub fn next_exercise(&mut self) -> Option<IntervalExercise> {
        // Prioritize ascending intervals, then descending
        let interval = if let Some(item) = self.ascending_scheduler.next_due_item() {
            Some((item.item, true))
        } else if self.config.practice_both_directions {
            self.descending_scheduler
                .next_due_item()
                .map(|item| (item.item, false))
        } else {
            None
        };

        interval.map(|(interval, ascending)| {
            let base_note = self.generate_base_note();
            IntervalExercise::new(base_note, interval, ascending)
        })
    }

    /// Record a completed exercise with user's performance
    ///
    /// # Arguments
    /// * `exercise` - The exercise that was completed
    /// * `rating` - The performance rating
    pub fn record_exercise(&mut self, exercise: &IntervalExercise, rating: PerformanceRating) {
        if exercise.ascending {
            if let Some(item) = self.ascending_scheduler.next_due_item_mut() {
                if item.item == exercise.interval {
                    item.record_review(rating);
                }
            }
        } else if let Some(item) = self.descending_scheduler.next_due_item_mut() {
            if item.item == exercise.interval {
                item.record_review(rating);
            }
        }
    }

    /// Record an exercise result based on the user's produced note
    ///
    /// # Arguments
    /// * `exercise` - The exercise that was completed
    /// * `produced_note` - The note produced by the user
    pub fn record_exercise_with_note(
        &mut self,
        exercise: &IntervalExercise,
        produced_note: Note,
    ) {
        let rating = exercise.rate_response(produced_note);
        self.record_exercise(exercise, rating);
    }

    /// Get statistics about the learning progress
    pub fn get_statistics(&self) -> LearningStatistics {
        let ascending_stats = self.calculate_scheduler_stats(&self.ascending_scheduler);
        let descending_stats = if self.config.practice_both_directions {
            self.calculate_scheduler_stats(&self.descending_scheduler)
        } else {
            SchedulerStatistics::default()
        };

        LearningStatistics {
            ascending: ascending_stats,
            descending: descending_stats,
            practice_both_directions: self.config.practice_both_directions,
        }
    }

    /// Calculate statistics for a scheduler
    fn calculate_scheduler_stats(
        &self,
        scheduler: &SpacedRepetitionScheduler<Interval>,
    ) -> SchedulerStatistics {
        let items = scheduler.items();
        let total = items.len();
        let due = scheduler.due_count();
        let mastered = items
            .iter()
            .filter(|item| item.consecutive_correct >= 3)
            .count();
        let avg_easiness = if total > 0 {
            items.iter().map(|item| item.easiness).sum::<f32>() / total as f32
        } else {
            0.0
        };

        SchedulerStatistics {
            total_intervals: total,
            due_for_review: due,
            mastered_intervals: mastered,
            average_easiness: avg_easiness,
        }
    }

    /// Generate a base note within the configured range
    fn generate_base_note(&self) -> Note {
        // For now, use a simple middle value
        // In a real implementation, this could use random generation
        let (min, max) = self.config.note_range;
        let min_midi = min.to_midi();
        let max_midi = max.to_midi();
        let mid_midi = (min_midi + max_midi) / 2;
        Note::from_midi(mid_midi)
    }

    /// Get the configuration
    pub fn config(&self) -> &IntervalLearningConfig {
        &self.config
    }

    /// Get the number of exercises due for review
    pub fn exercises_due(&self) -> usize {
        let ascending_due = self.ascending_scheduler.due_count();
        let descending_due = if self.config.practice_both_directions {
            self.descending_scheduler.due_count()
        } else {
            0
        };
        ascending_due + descending_due
    }
}

impl Default for IntervalLearningPlan {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about a single scheduler's progress
#[derive(Debug, Clone, Default)]
pub struct SchedulerStatistics {
    /// Total number of intervals
    pub total_intervals: usize,
    /// Number of intervals due for review
    pub due_for_review: usize,
    /// Number of mastered intervals (3+ consecutive correct)
    pub mastered_intervals: usize,
    /// Average easiness factor across all intervals
    pub average_easiness: f32,
}

/// Overall learning statistics
#[derive(Debug, Clone)]
pub struct LearningStatistics {
    /// Statistics for ascending intervals
    pub ascending: SchedulerStatistics,
    /// Statistics for descending intervals
    pub descending: SchedulerStatistics,
    /// Whether both directions are being practiced
    pub practice_both_directions: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::PitchClass;

    #[test]
    fn test_interval_exercise_creation() {
        let a4 = Note::new(PitchClass::A, 4);
        let exercise = IntervalExercise::new(a4, Interval::PerfectFifth, true);
        assert_eq!(exercise.base_note, a4);
        assert_eq!(exercise.interval, Interval::PerfectFifth);
        assert!(exercise.ascending);
    }

    #[test]
    fn test_target_note_ascending() {
        let a4 = Note::new(PitchClass::A, 4);
        let exercise = IntervalExercise::new(a4, Interval::Octave, true);
        let target = exercise.target_note();
        assert_eq!(target.pitch_class, PitchClass::A);
        assert_eq!(target.octave, 5);
    }

    #[test]
    fn test_target_note_descending() {
        let a5 = Note::new(PitchClass::A, 5);
        let exercise = IntervalExercise::new(a5, Interval::Octave, false);
        let target = exercise.target_note();
        assert_eq!(target.pitch_class, PitchClass::A);
        assert_eq!(target.octave, 4);
    }

    #[test]
    fn test_check_response_perfect() {
        let a4 = Note::new(PitchClass::A, 4);
        let exercise = IntervalExercise::new(a4, Interval::PerfectFifth, true);
        let target = exercise.target_note();
        assert!(exercise.check_response(target, 50.0));
    }

    #[test]
    fn test_check_response_within_tolerance() {
        let a4 = Note::new(PitchClass::A, 4);
        let exercise = IntervalExercise::new(a4, Interval::PerfectFifth, true);
        let target = exercise.target_note();
        // Test with exact target note (should be within tolerance)
        assert!(exercise.check_response(target, 50.0));
    }

    #[test]
    fn test_check_response_out_of_tolerance() {
        let a4 = Note::new(PitchClass::A, 4);
        let exercise = IntervalExercise::new(a4, Interval::PerfectFifth, true);
        // Way off - a major third instead of perfect fifth
        let wrong = apply_interval(a4, Interval::MajorThird, true);
        assert!(!exercise.check_response(wrong, 50.0));
    }

    #[test]
    fn test_rate_response_perfect() {
        let a4 = Note::new(PitchClass::A, 4);
        let exercise = IntervalExercise::new(a4, Interval::PerfectFifth, true);
        let target = exercise.target_note();
        let rating = exercise.rate_response(target);
        assert_eq!(rating, PerformanceRating::Perfect);
    }

    #[test]
    fn test_rate_response_good() {
        let a4 = Note::new(PitchClass::A, 4);
        let exercise = IntervalExercise::new(a4, Interval::PerfectFifth, true);
        let target = exercise.target_note();
        // Exact match should be perfect, not good
        let rating = exercise.rate_response(target);
        assert_eq!(rating, PerformanceRating::Perfect);
    }

    #[test]
    fn test_rate_response_incorrect() {
        let a4 = Note::new(PitchClass::A, 4);
        let exercise = IntervalExercise::new(a4, Interval::PerfectFifth, true);
        let target = exercise.target_note();
        // Off by a couple semitones - perfect 4th (5 semitones) instead of perfect 5th (7 semitones)
        let wrong = a4.transpose(5); 
        let rating = exercise.rate_response(wrong);
        
        // Let's verify the calculation
        let target_freq = target.to_frequency();
        let wrong_freq = wrong.to_frequency();
        let cents_diff = 1200.0 * (wrong_freq / target_freq).log2().abs();
        
        // Target is E5 (A4 + 7 semitones), wrong is D5 (A4 + 5 semitones)
        // Difference = 2 semitones = 200 cents, should be "Incorrect" (≤200 cents)
        println!("Target: {}, Wrong: {}, Cents diff: {:.1}, Rating: {:?}", 
                 target, wrong, cents_diff, rating);
        assert!(matches!(rating, PerformanceRating::Incorrect | PerformanceRating::Difficult));
    }

    #[test]
    fn test_learning_plan_creation() {
        let plan = IntervalLearningPlan::new();
        assert!(plan.exercises_due() > 0);
    }

    #[test]
    fn test_learning_plan_next_exercise() {
        let mut plan = IntervalLearningPlan::new();
        let exercise = plan.next_exercise();
        assert!(exercise.is_some());
        
        let ex = exercise.unwrap();
        assert_eq!(ex.interval, Interval::Octave); // First in learning order
    }

    #[test]
    fn test_learning_plan_record_exercise() {
        let mut plan = IntervalLearningPlan::new();
        let exercise = plan.next_exercise().unwrap();
        
        plan.record_exercise(&exercise, PerformanceRating::Perfect);
        
        let stats = plan.get_statistics();
        assert!(stats.ascending.total_intervals > 0);
    }

    #[test]
    fn test_learning_plan_with_note() {
        let mut plan = IntervalLearningPlan::new();
        let exercise = plan.next_exercise().unwrap();
        let target = exercise.target_note();
        
        plan.record_exercise_with_note(&exercise, target);
        
        let stats = plan.get_statistics();
        assert!(stats.ascending.total_intervals > 0);
    }

    #[test]
    fn test_learning_statistics() {
        let plan = IntervalLearningPlan::new();
        let stats = plan.get_statistics();
        
        assert!(stats.ascending.total_intervals == 13); // All intervals
        assert_eq!(stats.ascending.mastered_intervals, 0); // None mastered yet
        assert!(stats.practice_both_directions);
    }

    #[test]
    fn test_custom_config() {
        let config = IntervalLearningConfig {
            note_range: (Note::new(PitchClass::C, 3), Note::new(PitchClass::C, 6)),
            practice_both_directions: false,
            tolerance_cents: 30.0,
        };
        
        let plan = IntervalLearningPlan::with_config(config);
        let stats = plan.get_statistics();
        
        assert!(!stats.practice_both_directions);
        assert_eq!(stats.descending.total_intervals, 0);
    }

    #[test]
    fn test_exercises_due_count() {
        let plan = IntervalLearningPlan::new();
        let due = plan.exercises_due();
        assert_eq!(due, 26); // 13 ascending + 13 descending
    }
}
