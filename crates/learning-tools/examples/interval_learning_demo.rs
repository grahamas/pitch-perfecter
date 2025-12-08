//! Demonstration of the interval learning system
//!
//! This example shows how to use the interval learning plan
//! to practice musical intervals with spaced repetition.
//!
//! Run with: cargo run --package learning-tools --example interval_learning_demo

use learning_tools::interval_learning::{IntervalLearningPlan, IntervalLearningConfig};
use learning_tools::intervals::Interval;
use learning_tools::note::{Note, PitchClass};

fn main() {
    println!("=== Interval Learning Demo ===\n");

    // Create a new learning plan with default configuration
    let mut plan = IntervalLearningPlan::new();
    
    println!("Created learning plan with {} intervals", 
        Interval::all().len());
    println!("Total exercises due: {}\n", plan.exercises_due());

    // Simulate a practice session
    println!("=== Practice Session ===\n");
    
    for session_num in 1..=5 {
        println!("Exercise {}:", session_num);
        
        if let Some(exercise) = plan.next_exercise() {
            // Display the exercise
            println!("  Direction: {}", 
                if exercise.ascending { "Ascending" } else { "Descending" });
            println!("  Interval: {}", exercise.interval);
            println!("  Base note: {}", exercise.base_note);
            println!("  Target note: {}", exercise.target_note());
            
            // Simulate user response (in real app, this would be pitch detection)
            // Here we simulate different levels of accuracy
            let target = exercise.target_note();
            let user_note = match session_num {
                1 => target,                              // Perfect
                2 => target.transpose(0),                 // Perfect (same note)
                3 => target.transpose(1),                 // Off by 1 semitone
                4 => target.transpose(2),                 // Off by 2 semitones
                _ => target.transpose(-3),                // Off by 3 semitones
            };
            
            // Rate and record the response
            let rating = exercise.rate_response(user_note);
            println!("  User sang: {}", user_note);
            println!("  Rating: {:?}", rating);
            plan.record_exercise(&exercise, rating);
            println!();
        } else {
            println!("  No exercises due!");
            break;
        }
    }

    // Show statistics
    println!("=== Progress Statistics ===\n");
    let stats = plan.get_statistics();
    
    println!("Ascending intervals:");
    println!("  Total: {}", stats.ascending.total_intervals);
    println!("  Due for review: {}", stats.ascending.due_for_review);
    println!("  Mastered: {}", stats.ascending.mastered_intervals);
    println!("  Average easiness: {:.2}", stats.ascending.average_easiness);
    
    if stats.practice_both_directions {
        println!("\nDescending intervals:");
        println!("  Total: {}", stats.descending.total_intervals);
        println!("  Due for review: {}", stats.descending.due_for_review);
        println!("  Mastered: {}", stats.descending.mastered_intervals);
        println!("  Average easiness: {:.2}", stats.descending.average_easiness);
    }

    // Demonstrate custom configuration
    println!("\n=== Custom Configuration Demo ===\n");
    
    let custom_config = IntervalLearningConfig {
        note_range: (Note::new(PitchClass::C, 3), Note::new(PitchClass::C, 5)),
        practice_both_directions: false,  // Only ascending
        tolerance_cents: 30.0,            // Stricter tolerance
    };
    
    let custom_plan = IntervalLearningPlan::with_config(custom_config);
    println!("Custom plan configuration:");
    println!("  Note range: {} to {}", 
        custom_plan.config().note_range.0,
        custom_plan.config().note_range.1);
    println!("  Both directions: {}", 
        custom_plan.config().practice_both_directions);
    println!("  Tolerance: {} cents", 
        custom_plan.config().tolerance_cents);
    
    let stats = custom_plan.get_statistics();
    println!("  Total exercises: {}", 
        stats.ascending.total_intervals + stats.descending.total_intervals);
}
