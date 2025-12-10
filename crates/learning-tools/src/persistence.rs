//! Persistence module for saving and loading learning profiles
//!
//! This module provides functionality to save and load learning plans
//! to and from JSON files, enabling progress to be preserved across sessions.

use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use crate::interval_learning::IntervalLearningPlan;

/// Error type for persistence operations
#[derive(Debug)]
pub enum PersistenceError {
    /// Failed to read file
    IoError(std::io::Error),
    /// Failed to serialize/deserialize JSON
    JsonError(serde_json::Error),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::IoError(e) => write!(f, "IO error: {}", e),
            PersistenceError::JsonError(e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<std::io::Error> for PersistenceError {
    fn from(error: std::io::Error) -> Self {
        PersistenceError::IoError(error)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(error: serde_json::Error) -> Self {
        PersistenceError::JsonError(error)
    }
}

/// Save an `IntervalLearningPlan` to a JSON file
///
/// # Arguments
/// * `plan` - The learning plan to save
/// * `path` - Path where the JSON file should be created
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(PersistenceError)` on failure
///
/// # Example
/// ```no_run
/// use learning_tools::interval_learning::IntervalLearningPlan;
/// use learning_tools::persistence::save_learning_plan;
///
/// let plan = IntervalLearningPlan::new();
/// save_learning_plan(&plan, "my_profile.json").unwrap();
/// ```
pub fn save_learning_plan<P: AsRef<Path>>(
    plan: &IntervalLearningPlan,
    path: P,
) -> Result<(), PersistenceError> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, plan)?;
    Ok(())
}

/// Load an `IntervalLearningPlan` from a JSON file
///
/// # Arguments
/// * `path` - Path to the JSON file to load
///
/// # Returns
/// * `Ok(IntervalLearningPlan)` on success
/// * `Err(PersistenceError)` on failure
///
/// # Example
/// ```no_run
/// use learning_tools::persistence::load_learning_plan;
///
/// let plan = load_learning_plan("my_profile.json").unwrap();
/// ```
pub fn load_learning_plan<P: AsRef<Path>>(
    path: P,
) -> Result<IntervalLearningPlan, PersistenceError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let plan = serde_json::from_reader(reader)?;
    Ok(plan)
}

/// Check if a learning plan file exists
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `true` if the file exists, `false` otherwise
pub fn learning_plan_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Delete a learning plan file
///
/// # Arguments
/// * `path` - Path to the file to delete
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(PersistenceError)` on failure
pub fn delete_learning_plan<P: AsRef<Path>>(path: P) -> Result<(), PersistenceError> {
    fs::remove_file(path)?;
    Ok(())
}

impl IntervalLearningPlan {
    /// Save this learning plan to a JSON file
    ///
    /// # Arguments
    /// * `path` - Path where the JSON file should be created
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(PersistenceError)` on failure
    ///
    /// # Example
    /// ```no_run
    /// use learning_tools::interval_learning::IntervalLearningPlan;
    ///
    /// let plan = IntervalLearningPlan::new();
    /// plan.save("my_profile.json").unwrap();
    /// ```
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), PersistenceError> {
        save_learning_plan(self, path)
    }

    /// Load a learning plan from a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to the JSON file to load
    ///
    /// # Returns
    /// * `Ok(IntervalLearningPlan)` on success
    /// * `Err(PersistenceError)` on failure
    ///
    /// # Example
    /// ```no_run
    /// use learning_tools::interval_learning::IntervalLearningPlan;
    ///
    /// let plan = IntervalLearningPlan::load("my_profile.json").unwrap();
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, PersistenceError> {
        load_learning_plan(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spaced_repetition::PerformanceRating;
    use std::fs;
    use std::env;

    #[test]
    fn test_save_and_load_new_plan() {
        let temp_path = env::temp_dir().join("test_plan_new.json");
        
        // Clean up any existing file
        let _ = fs::remove_file(&temp_path);
        
        // Create and save a new plan
        let plan = IntervalLearningPlan::new();
        save_learning_plan(&plan, &temp_path).expect("Failed to save plan");
        
        // Load it back
        let loaded_plan = load_learning_plan(&temp_path).expect("Failed to load plan");
        
        // Verify basic properties match
        assert_eq!(plan.exercises_due(), loaded_plan.exercises_due());
        
        // Clean up
        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_save_and_load_with_progress() {
        let temp_path = env::temp_dir().join("test_plan_progress.json");
        
        // Clean up any existing file
        let _ = fs::remove_file(&temp_path);
        
        // Create a plan and record some exercises
        let mut plan = IntervalLearningPlan::new();
        if let Some(exercise) = plan.next_exercise() {
            plan.record_exercise(&exercise, PerformanceRating::Perfect);
        }
        
        // Save it
        plan.save(&temp_path).expect("Failed to save plan");
        
        // Load it back
        let loaded_plan = IntervalLearningPlan::load(&temp_path).expect("Failed to load plan");
        
        // Verify the progress was preserved
        let original_stats = plan.get_statistics();
        let loaded_stats = loaded_plan.get_statistics();
        
        assert_eq!(original_stats.ascending.total_intervals, loaded_stats.ascending.total_intervals);
        assert_eq!(original_stats.descending.total_intervals, loaded_stats.descending.total_intervals);
        
        // Clean up
        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_learning_plan_exists() {
        let temp_path = env::temp_dir().join("test_plan_exists.json");
        
        // Clean up any existing file
        let _ = fs::remove_file(&temp_path);
        
        assert!(!learning_plan_exists(&temp_path));
        
        let plan = IntervalLearningPlan::new();
        save_learning_plan(&plan, &temp_path).expect("Failed to save plan");
        
        assert!(learning_plan_exists(&temp_path));
        
        // Clean up
        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_delete_learning_plan() {
        let temp_path = env::temp_dir().join("test_plan_delete.json");
        
        // Create a plan file
        let plan = IntervalLearningPlan::new();
        save_learning_plan(&plan, &temp_path).expect("Failed to save plan");
        
        assert!(learning_plan_exists(&temp_path));
        
        // Delete it
        delete_learning_plan(&temp_path).expect("Failed to delete plan");
        
        assert!(!learning_plan_exists(&temp_path));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_learning_plan(env::temp_dir().join("nonexistent_plan_xyz.json"));
        assert!(result.is_err());
    }
}
