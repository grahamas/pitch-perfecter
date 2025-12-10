//! Demonstration of saving and loading learning profiles
//!
//! This example shows how to:
//! 1. Create a new learning plan
//! 2. Practice some exercises and record progress
//! 3. Save the plan to a file
//! 4. Load it back later
//! 5. Continue practicing with preserved progress

use learning_tools::{
    IntervalLearningPlan,
    PerformanceRating,
    save_learning_plan,
    load_learning_plan,
    learning_plan_exists,
};

fn main() {
    println!("=== Learning Profile Persistence Demo ===\n");

    let profile_path = "/tmp/my_learning_profile.json";

    // Check if a profile already exists
    if learning_plan_exists(profile_path) {
        println!("📂 Found existing profile at {}", profile_path);
        println!("   Loading profile...\n");

        // Load the existing profile
        let mut plan = load_learning_plan(profile_path)
            .expect("Failed to load learning profile");

        println!("✅ Profile loaded successfully!");
        display_progress(&plan);

        // Continue practicing
        println!("\n🎵 Continuing practice session...");
        practice_exercises(&mut plan, 2);

        // Save the updated progress
        println!("\n💾 Saving updated progress...");
        save_learning_plan(&plan, profile_path)
            .expect("Failed to save profile");
        println!("✅ Progress saved!");

    } else {
        println!("📝 No existing profile found. Creating a new one...\n");

        // Create a new learning plan
        let mut plan = IntervalLearningPlan::new();

        println!("✅ New learning plan created!");
        display_progress(&plan);

        // Do some practice
        println!("\n🎵 Starting practice session...");
        practice_exercises(&mut plan, 3);

        // Save the profile
        println!("\n💾 Saving learning profile...");
        save_learning_plan(&plan, profile_path)
            .expect("Failed to save profile");
        println!("✅ Profile saved to: {}", profile_path);
    }

    println!("\n═══════════════════════════════════════");
    println!("💡 Tip: Run this example again to see your");
    println!("   progress loaded from the saved file!");
    println!("═══════════════════════════════════════\n");
}

/// Display current learning progress
fn display_progress(plan: &IntervalLearningPlan) {
    let stats = plan.get_statistics();

    println!("\n📊 Current Progress:");
    println!("   ┌─ Ascending Intervals");
    println!("   │  • Total: {}", stats.ascending.total_intervals);
    println!("   │  • Due for review: {}", stats.ascending.due_for_review);
    println!("   │  • Mastered: {}", stats.ascending.mastered_intervals);
    println!("   │  • Avg easiness: {:.2}", stats.ascending.average_easiness);

    if stats.practice_both_directions {
        println!("   │");
        println!("   └─ Descending Intervals");
        println!("      • Total: {}", stats.descending.total_intervals);
        println!("      • Due for review: {}", stats.descending.due_for_review);
        println!("      • Mastered: {}", stats.descending.mastered_intervals);
        println!("      • Avg easiness: {:.2}", stats.descending.average_easiness);
    }

    println!("\n   Exercises due: {}", plan.exercises_due());
}

/// Practice a few exercises and record results
fn practice_exercises(plan: &mut IntervalLearningPlan, count: usize) {
    for i in 0..count {
        if let Some(exercise) = plan.next_exercise() {
            println!("\n   Exercise {}:", i + 1);
            println!("   🎼 Practice: {} {}",
                if exercise.ascending { "Ascending" } else { "Descending" },
                exercise.interval
            );
            println!("   📍 Base note: {}", exercise.base_note);
            println!("   🎯 Target note: {}", exercise.target_note());

            // Simulate user performance (alternating between Perfect and Good)
            let rating = if i % 2 == 0 {
                PerformanceRating::Perfect
            } else {
                PerformanceRating::Good
            };

            plan.record_exercise(&exercise, rating);
            println!("   ✓ Recorded as: {:?}", rating);
        } else {
            println!("\n   ℹ️  No more exercises due at this time!");
            break;
        }
    }
}
