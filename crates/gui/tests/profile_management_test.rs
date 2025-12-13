use learning_tools::{load_learning_plan, save_learning_plan, IntervalLearningPlan};
use std::env;
use std::fs;

#[test]
fn test_profile_save_and_load() {
    // Use temp directory for test
    let temp_dir = env::temp_dir();
    let profile_path = temp_dir.join("test_gui_profile.json");

    // Clean up any existing file
    let _ = fs::remove_file(&profile_path);

    // Create a new learning plan
    let plan = IntervalLearningPlan::new();

    // Save it
    save_learning_plan(&plan, &profile_path).expect("Failed to save profile");

    // Verify file exists
    assert!(
        profile_path.exists(),
        "Profile file should exist after saving"
    );

    // Load it back
    let loaded_plan = load_learning_plan(&profile_path).expect("Failed to load profile");

    // Verify the loaded plan has the same properties
    assert_eq!(plan.exercises_due(), loaded_plan.exercises_due());

    // Clean up
    let _ = fs::remove_file(&profile_path);
}

#[test]
fn test_profile_with_progress() {
    use learning_tools::PerformanceRating;

    let temp_dir = env::temp_dir();
    let profile_path = temp_dir.join("test_gui_profile_progress.json");

    // Clean up any existing file
    let _ = fs::remove_file(&profile_path);

    // Create a plan and do some exercises
    let mut plan = IntervalLearningPlan::new();

    // Record some exercises
    if let Some(exercise) = plan.next_exercise() {
        plan.record_exercise(&exercise, PerformanceRating::Good);
    }
    if let Some(exercise) = plan.next_exercise() {
        plan.record_exercise(&exercise, PerformanceRating::Perfect);
    }

    // Save the plan
    save_learning_plan(&plan, &profile_path).expect("Failed to save profile with progress");

    // Load it back
    let loaded_plan =
        load_learning_plan(&profile_path).expect("Failed to load profile with progress");

    // Verify statistics match
    let original_stats = plan.get_statistics();
    let loaded_stats = loaded_plan.get_statistics();

    assert_eq!(
        original_stats.ascending.total_intervals,
        loaded_stats.ascending.total_intervals
    );

    // Clean up
    let _ = fs::remove_file(&profile_path);
}

#[test]
fn test_load_nonexistent_profile() {
    let temp_dir = env::temp_dir();
    let profile_path = temp_dir.join("nonexistent_profile_xyz123.json");

    // Ensure file doesn't exist
    let _ = fs::remove_file(&profile_path);

    // Try to load - should fail
    let result = load_learning_plan(&profile_path);
    assert!(result.is_err(), "Loading nonexistent profile should fail");
}
