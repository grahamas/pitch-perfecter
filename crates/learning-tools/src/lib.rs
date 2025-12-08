//! Learning Tools
//! 
//! This crate provides sight-singing learning logic and exercises.
//! Features include:
//! - Musical note representation (A4, C#5, etc.)
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
//! use learning_tools::note::{Note, PitchClass};
//!
//! let mut plan = IntervalLearningPlan::new();
//! 
//! // Get the next exercise
//! if let Some(exercise) = plan.next_exercise() {
//!     println!("Practice: {} {} from {}",
//!         if exercise.ascending { "ascending" } else { "descending" },
//!         exercise.interval,
//!         exercise.base_note
//!     );
//!     
//!     // User produces a pitch and we record the result
//!     let user_note = exercise.target_note(); // Perfect response
//!     plan.record_exercise_with_note(&exercise, user_note);
//! }
//! 
//! // Check progress
//! let stats = plan.get_statistics();
//! println!("Mastered intervals: {}", stats.ascending.mastered_intervals);
//! ```

pub mod note;
pub mod intervals;
pub mod spaced_repetition;
pub mod interval_learning;

pub use note::{Note, PitchClass};
pub use intervals::{Interval, apply_interval, calculate_interval_semitones, closest_interval};
pub use spaced_repetition::{PerformanceRating, ReviewItem, SpacedRepetitionScheduler};
pub use interval_learning::{
    IntervalExercise, IntervalLearningConfig, IntervalLearningPlan, LearningStatistics,
};
