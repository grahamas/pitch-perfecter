//! Learning Tools
//! 
//! This crate provides sight-singing learning logic and exercises.
//! Features include:
//! - Musical interval definitions and utilities
//! - Spaced repetition system for learning
//! - Interval learning plan with automatic scheduling
//! - Exercise generation and scoring
//! - Progress tracking
//!
//! # Example
//!
//! ```
//! use learning_tools::interval_learning::IntervalLearningPlan;
//! use learning_tools::spaced_repetition::PerformanceRating;
//!
//! let mut plan = IntervalLearningPlan::new();
//! 
//! // Get the next exercise
//! if let Some(exercise) = plan.next_exercise() {
//!     println!("Practice: {} {} from {:.2} Hz",
//!         if exercise.ascending { "ascending" } else { "descending" },
//!         exercise.interval,
//!         exercise.base_frequency
//!     );
//!     
//!     // User produces a pitch and we record the result
//!     let user_freq = exercise.target_frequency(); // Perfect response
//!     plan.record_exercise_with_frequency(&exercise, user_freq);
//! }
//! 
//! // Check progress
//! let stats = plan.get_statistics();
//! println!("Mastered intervals: {}", stats.ascending.mastered_intervals);
//! ```

pub mod intervals;
pub mod spaced_repetition;
pub mod interval_learning;

pub use intervals::{Interval, apply_interval, calculate_interval_semitones, closest_interval};
pub use spaced_repetition::{PerformanceRating, ReviewItem, SpacedRepetitionScheduler};
pub use interval_learning::{
    IntervalExercise, IntervalLearningConfig, IntervalLearningPlan, LearningStatistics,
};
