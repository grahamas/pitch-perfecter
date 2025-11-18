//! Interval learning plan with spaced repetition
//!
//! This module provides a complete learning system for musical intervals,
//! combining interval exercises with spaced repetition scheduling.

use crate::intervals::{Interval, apply_interval};
use crate::spaced_repetition::{PerformanceRating, SpacedRepetitionScheduler};

/// Represents a single interval learning exercise
#[derive(Debug, Clone, PartialEq)]
pub struct IntervalExercise {
    /// The starting frequency in Hz
    pub base_frequency: f32,
    /// The interval to practice
    pub interval: Interval,
    /// Direction: true for ascending, false for descending
    pub ascending: bool,
}

impl IntervalExercise {
    /// Create a new interval exercise
    pub fn new(base_frequency: f32, interval: Interval, ascending: bool) -> Self {
        Self {
            base_frequency,
            interval,
            ascending,
        }
    }

    /// Calculate the target frequency for this exercise
    pub fn target_frequency(&self) -> f32 {
        if self.ascending {
            apply_interval(self.base_frequency, self.interval)
        } else {
            self.base_frequency / 2.0_f32.powf(self.interval.semitones() as f32 / 12.0)
        }
    }

    /// Check if a produced frequency matches the target within a tolerance
    ///
    /// # Arguments
    /// * `produced_freq` - The frequency produced by the user in Hz
    /// * `tolerance_cents` - Tolerance in cents (100 cents = 1 semitone), default 50 cents
    ///
    /// # Returns
    /// True if the produced frequency is within tolerance
    pub fn check_response(&self, produced_freq: f32, tolerance_cents: f32) -> bool {
        let target = self.target_frequency();
        if target <= 0.0 || produced_freq <= 0.0 {
            return false;
        }
        
        let cents_diff = 1200.0 * (produced_freq / target).log2().abs();
        cents_diff <= tolerance_cents
    }

    /// Rate the user's performance based on accuracy
    ///
    /// # Arguments
    /// * `produced_freq` - The frequency produced by the user in Hz
    ///
    /// # Returns
    /// A performance rating based on how close the response was
    pub fn rate_response(&self, produced_freq: f32) -> PerformanceRating {
        let target = self.target_frequency();
        if target <= 0.0 || produced_freq <= 0.0 {
            return PerformanceRating::Blackout;
        }
        
        let cents_diff = 1200.0 * (produced_freq / target).log2().abs();
        
        match cents_diff {
            diff if diff <= 10.0 => PerformanceRating::Perfect,   // Within 10 cents
            diff if diff <= 25.0 => PerformanceRating::Good,      // Within 25 cents
            diff if diff <= 50.0 => PerformanceRating::Hesitant,  // Within 50 cents (half semitone)
            diff if diff <= 100.0 => PerformanceRating::Difficult, // Within 1 semitone
            diff if diff <= 200.0 => PerformanceRating::Incorrect, // Within 2 semitones
            _ => PerformanceRating::Blackout,                      // More than 2 semitones off
        }
    }
}

/// Configuration for interval learning sessions
#[derive(Debug, Clone)]
pub struct IntervalLearningConfig {
    /// Base frequency range for exercises (min, max) in Hz
    pub frequency_range: (f32, f32),
    /// Whether to practice both ascending and descending intervals
    pub practice_both_directions: bool,
    /// Default tolerance in cents for checking responses
    pub tolerance_cents: f32,
}

impl Default for IntervalLearningConfig {
    fn default() -> Self {
        Self {
            frequency_range: (220.0, 880.0), // A3 to A5
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
            let base_freq = self.generate_base_frequency();
            IntervalExercise::new(base_freq, interval, ascending)
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

    /// Record an exercise result based on the user's produced frequency
    ///
    /// # Arguments
    /// * `exercise` - The exercise that was completed
    /// * `produced_freq` - The frequency produced by the user
    pub fn record_exercise_with_frequency(
        &mut self,
        exercise: &IntervalExercise,
        produced_freq: f32,
    ) {
        let rating = exercise.rate_response(produced_freq);
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

    /// Generate a random base frequency within the configured range
    fn generate_base_frequency(&self) -> f32 {
        // For now, use a simple middle value
        // In a real implementation, this could use random generation
        let (min, max) = self.config.frequency_range;
        (min + max) / 2.0
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

    #[test]
    fn test_interval_exercise_creation() {
        let exercise = IntervalExercise::new(440.0, Interval::PerfectFifth, true);
        assert_eq!(exercise.base_frequency, 440.0);
        assert_eq!(exercise.interval, Interval::PerfectFifth);
        assert!(exercise.ascending);
    }

    #[test]
    fn test_target_frequency_ascending() {
        let exercise = IntervalExercise::new(440.0, Interval::Octave, true);
        let target = exercise.target_frequency();
        assert!((target - 880.0).abs() < 0.1);
    }

    #[test]
    fn test_target_frequency_descending() {
        let exercise = IntervalExercise::new(880.0, Interval::Octave, false);
        let target = exercise.target_frequency();
        assert!((target - 440.0).abs() < 0.1);
    }

    #[test]
    fn test_check_response_perfect() {
        let exercise = IntervalExercise::new(440.0, Interval::PerfectFifth, true);
        let target = exercise.target_frequency();
        assert!(exercise.check_response(target, 50.0));
    }

    #[test]
    fn test_check_response_within_tolerance() {
        let exercise = IntervalExercise::new(440.0, Interval::PerfectFifth, true);
        let target = exercise.target_frequency();
        // Slightly sharp, but within 50 cents
        let sharp = target * 1.02;
        assert!(exercise.check_response(sharp, 50.0));
    }

    #[test]
    fn test_check_response_out_of_tolerance() {
        let exercise = IntervalExercise::new(440.0, Interval::PerfectFifth, true);
        // Way off - a major third instead of perfect fifth
        let wrong = apply_interval(440.0, Interval::MajorThird);
        assert!(!exercise.check_response(wrong, 50.0));
    }

    #[test]
    fn test_rate_response_perfect() {
        let exercise = IntervalExercise::new(440.0, Interval::PerfectFifth, true);
        let target = exercise.target_frequency();
        let rating = exercise.rate_response(target);
        assert_eq!(rating, PerformanceRating::Perfect);
    }

    #[test]
    fn test_rate_response_good() {
        let exercise = IntervalExercise::new(440.0, Interval::PerfectFifth, true);
        let target = exercise.target_frequency();
        // 20 cents sharp
        let sharp = target * 2.0_f32.powf(20.0 / 1200.0);
        let rating = exercise.rate_response(sharp);
        assert_eq!(rating, PerformanceRating::Good);
    }

    #[test]
    fn test_rate_response_incorrect() {
        let exercise = IntervalExercise::new(440.0, Interval::PerfectFifth, true);
        // A semitone and a half off
        let target = exercise.target_frequency();
        let wrong = target * 2.0_f32.powf(1.5 / 12.0);
        let rating = exercise.rate_response(wrong);
        assert_eq!(rating, PerformanceRating::Incorrect);
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
    fn test_learning_plan_with_frequency() {
        let mut plan = IntervalLearningPlan::new();
        let exercise = plan.next_exercise().unwrap();
        let target = exercise.target_frequency();
        
        plan.record_exercise_with_frequency(&exercise, target);
        
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
            frequency_range: (100.0, 1000.0),
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
