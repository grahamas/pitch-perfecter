//! Learning pane for interval training
//!
//! This module provides the UI for the interval learning system,
//! integrating the learning-tools crate with the GUI.

use eframe::egui;
use learning_tools::{
    interval_learning::{IntervalExercise, IntervalLearningPlan, LearningStatistics},
    spaced_repetition::PerformanceRating,
    Note,
};
use std::sync::mpsc::Receiver;

use crate::pitch_processor::PitchResult;

/// State of the learning session
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningState {
    /// No exercise active
    Idle,
    /// Exercise displayed, waiting for user to hear the reference
    ShowingExercise,
    /// Recording user's attempt
    Recording,
    /// Showing feedback on the attempt
    ShowingFeedback,
}

/// Manages the learning pane UI and state
pub struct LearningPane {
    /// The learning plan with spaced repetition
    learning_plan: IntervalLearningPlan,
    
    /// Current exercise being practiced
    current_exercise: Option<IntervalExercise>,
    
    /// Current state of the learning session
    state: LearningState,
    
    /// Last pitch result from user
    user_pitch: Option<PitchResult>,
    
    /// Last performance rating
    last_rating: Option<PerformanceRating>,
    
    /// Statistics for display
    statistics: LearningStatistics,
    
    /// Message to display to user
    feedback_message: String,
}

impl LearningPane {
    /// Create a new learning pane
    pub fn new() -> Self {
        let learning_plan = IntervalLearningPlan::new();
        let statistics = learning_plan.get_statistics();
        
        Self {
            learning_plan,
            current_exercise: None,
            state: LearningState::Idle,
            user_pitch: None,
            last_rating: None,
            statistics,
            feedback_message: String::new(),
        }
    }
    
    /// Start a new exercise
    pub fn start_exercise(&mut self) {
        if let Some(exercise) = self.learning_plan.next_exercise() {
            self.current_exercise = Some(exercise);
            self.state = LearningState::ShowingExercise;
            self.user_pitch = None;
            self.last_rating = None;
            self.feedback_message = "Listen to the interval and sing it!".to_string();
        } else {
            self.feedback_message = "Great job! All intervals reviewed for now.".to_string();
            self.state = LearningState::Idle;
        }
    }
    
    /// Start recording the user's attempt
    pub fn start_recording(&mut self) {
        if self.state == LearningState::ShowingExercise {
            self.state = LearningState::Recording;
            self.user_pitch = None;
            self.feedback_message = "Recording... Sing the target note!".to_string();
        }
    }
    
    /// Update with new pitch data from recording
    pub fn update_pitch(&mut self, pitch_receiver: &Receiver<PitchResult>) {
        if self.state == LearningState::Recording {
            // Get the latest pitch result
            while let Ok(pitch) = pitch_receiver.try_recv() {
                self.user_pitch = Some(pitch);
            }
        }
    }
    
    /// Check the user's response and provide feedback
    /// Returns true if recording should be stopped
    pub fn check_response(&mut self) -> bool {
        if self.state != LearningState::Recording {
            return false;
        }
        
        let should_stop_recording = true;
        
        if let Some(exercise) = &self.current_exercise {
            if let Some(pitch) = &self.user_pitch {
                // Try to convert frequency to note
                if let Some(user_note) = Note::from_frequency(pitch.frequency) {
                    let rating = exercise.rate_response(user_note);
                    self.last_rating = Some(rating);
                    
                    // Record the result
                    self.learning_plan.record_exercise(exercise, rating);
                    
                    // Update statistics
                    self.statistics = self.learning_plan.get_statistics();
                    
                    // Set feedback message
                    let target = exercise.target_note();
                    self.feedback_message = format!(
                        "{:?}! Target: {} ({:.2} Hz), You sang: {} ({:.2} Hz)",
                        rating,
                        target,
                        target.to_frequency(),
                        user_note,
                        pitch.frequency
                    );
                    
                    self.state = LearningState::ShowingFeedback;
                } else {
                    self.feedback_message = "Could not detect a clear note. Try again!".to_string();
                }
            } else {
                self.feedback_message = "No pitch detected. Please sing louder!".to_string();
            }
        }
        
        should_stop_recording
    }
    
    /// Move to the next exercise
    pub fn next_exercise(&mut self) {
        self.start_exercise();
    }
    
    /// Skip current exercise without recording
    pub fn skip_exercise(&mut self) {
        self.start_exercise();
    }
    
    /// Get whether recording should be active
    pub fn should_be_recording(&self) -> bool {
        self.state == LearningState::Recording
    }
    
    /// Render the learning pane UI
    /// Returns true if recording should be started
    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let mut start_recording = false;
        ui.heading("Interval Learning");
        ui.add_space(10.0);
        
        // Statistics panel
        ui.group(|ui| {
            ui.heading("Progress");
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Ascending:");
                ui.label(format!(
                    "{}/{} mastered",
                    self.statistics.ascending.mastered_intervals,
                    self.statistics.ascending.total_intervals
                ));
            });
            
            if self.statistics.practice_both_directions {
                ui.horizontal(|ui| {
                    ui.label("Descending:");
                    ui.label(format!(
                        "{}/{} mastered",
                        self.statistics.descending.mastered_intervals,
                        self.statistics.descending.total_intervals
                    ));
                });
            }
            
            ui.horizontal(|ui| {
                ui.label("Due for review:");
                ui.label(format!(
                    "{}",
                    self.statistics.ascending.due_for_review
                        + self.statistics.descending.due_for_review
                ));
            });
        });
        
        ui.add_space(10.0);
        
        // Current exercise panel
        ui.group(|ui| {
            ui.heading("Current Exercise");
            ui.add_space(5.0);
            
            if let Some(exercise) = &self.current_exercise {
                ui.horizontal(|ui| {
                    ui.label("Direction:");
                    ui.heading(if exercise.ascending {
                        "Ascending ↑"
                    } else {
                        "Descending ↓"
                    });
                });
                
                ui.horizontal(|ui| {
                    ui.label("Interval:");
                    ui.heading(format!("{}", exercise.interval));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Base Note:");
                    ui.heading(format!("{}", exercise.base_note));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Target Note:");
                    ui.heading(format!("{}", exercise.target_note()));
                });
                
                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);
                
                // Current pitch display during recording
                if self.state == LearningState::Recording {
                    if let Some(pitch) = &self.user_pitch {
                        ui.horizontal(|ui| {
                            ui.label("Detected:");
                            ui.heading(&pitch.note_name);
                            ui.label(format!("({:.2} Hz)", pitch.frequency));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Clarity:");
                            ui.add(egui::ProgressBar::new(pitch.clarity).show_percentage());
                        });
                    } else {
                        ui.label("Listening for your voice...");
                    }
                }
                
                // Feedback display
                if !self.feedback_message.is_empty() {
                    ui.add_space(5.0);
                    let color = match self.last_rating {
                        Some(PerformanceRating::Perfect) => egui::Color32::GREEN,
                        Some(PerformanceRating::Good) => egui::Color32::LIGHT_GREEN,
                        Some(PerformanceRating::Hesitant) => egui::Color32::YELLOW,
                        Some(PerformanceRating::Difficult) => egui::Color32::LIGHT_BLUE,
                        Some(PerformanceRating::Incorrect) => egui::Color32::LIGHT_RED,
                        Some(PerformanceRating::Blackout) => egui::Color32::RED,
                        None => egui::Color32::GRAY,
                    };
                    ui.colored_label(color, &self.feedback_message);
                }
            } else {
                ui.label("No exercise active. Click 'Start Exercise' to begin!");
            }
        });
        
        ui.add_space(10.0);
        
        // Controls
        ui.group(|ui| {
            ui.heading("Controls");
            ui.add_space(5.0);
            
            match self.state {
                LearningState::Idle => {
                    if ui.button("Start Exercise").clicked() {
                        self.start_exercise();
                    }
                }
                LearningState::ShowingExercise => {
                    ui.horizontal(|ui| {
                        if ui.button("🎤 Start Recording").clicked() {
                            self.start_recording();
                            start_recording = true;
                        }
                        if ui.button("⏭ Skip").clicked() {
                            self.skip_exercise();
                        }
                    });
                }
                LearningState::Recording => {
                    if ui.button("✓ Check Answer").clicked() {
                        let _should_stop = self.check_response();
                        // Note: stopping recording is handled by main app checking should_be_recording()
                    }
                }
                LearningState::ShowingFeedback => {
                    if ui.button("Next Exercise").clicked() {
                        self.next_exercise();
                    }
                }
            }
        });
        
        ui.add_space(10.0);
        
        // Instructions
        ui.group(|ui| {
            ui.heading("How to Practice");
            ui.add_space(5.0);
            
            ui.label("1. Click 'Start Exercise' to get a new interval");
            ui.label("2. Sing the base note, then sing the target interval");
            ui.label("3. Click 'Start Recording' when ready");
            ui.label("4. Hold the target note steadily");
            ui.label("5. Click 'Check Answer' to get feedback");
            ui.label("6. Progress to the next exercise");
            
            ui.add_space(5.0);
            ui.label("💡 Tip: Enable 'Bandpass Filter' in the Pitch Detection tab for better accuracy!");
        });
        
        start_recording
    }
}

impl Default for LearningPane {
    fn default() -> Self {
        Self::new()
    }
}
