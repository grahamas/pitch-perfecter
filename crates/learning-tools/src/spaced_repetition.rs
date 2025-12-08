//! Spaced repetition algorithm for learning
//!
//! This module implements a spaced repetition system based on the SM-2 algorithm,
//! adapted for musical interval learning.

use std::time::{Duration, SystemTime};

/// Performance rating for an exercise attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceRating {
    /// Complete blackout (0)
    Blackout,
    /// Incorrect response (1)
    Incorrect,
    /// Correct but difficult (2)
    Difficult,
    /// Correct with hesitation (3)
    Hesitant,
    /// Correct with some effort (4)
    Good,
    /// Perfect response (5)
    Perfect,
}

impl PerformanceRating {
    /// Convert rating to a 0-5 quality score for SM-2 algorithm
    pub fn quality(&self) -> u8 {
        match self {
            PerformanceRating::Blackout => 0,
            PerformanceRating::Incorrect => 1,
            PerformanceRating::Difficult => 2,
            PerformanceRating::Hesitant => 3,
            PerformanceRating::Good => 4,
            PerformanceRating::Perfect => 5,
        }
    }
}

/// Represents the state of a learning item in the spaced repetition system
#[derive(Debug, Clone)]
pub struct ReviewItem<T> {
    /// The item being learned
    pub item: T,
    /// Easiness factor (default 2.5, range 1.3+)
    pub easiness: f32,
    /// Number of consecutive successful reviews
    pub consecutive_correct: u32,
    /// Current interval in days
    pub interval: f32,
    /// Next review time
    pub next_review: SystemTime,
    /// Total number of reviews
    pub total_reviews: u32,
}

impl<T> ReviewItem<T> {
    /// Create a new review item
    pub fn new(item: T) -> Self {
        Self {
            item,
            easiness: 2.5,
            consecutive_correct: 0,
            interval: 0.0,
            next_review: SystemTime::now(),
            total_reviews: 0,
        }
    }

    /// Check if the item is due for review
    pub fn is_due(&self) -> bool {
        self.next_review <= SystemTime::now()
    }

    /// Update the item's state based on performance using SM-2 algorithm
    pub fn record_review(&mut self, performance: PerformanceRating) {
        self.total_reviews += 1;
        let quality = performance.quality();

        // Update easiness factor
        self.easiness = (self.easiness + (0.1 - (5 - quality) as f32 * (0.08 + (5 - quality) as f32 * 0.02)))
            .max(1.3);

        // Update consecutive correct count and interval
        if quality < 3 {
            // Failed recall - reset
            self.consecutive_correct = 0;
            self.interval = 0.0;
        } else {
            // Successful recall
            self.consecutive_correct += 1;
            self.interval = match self.consecutive_correct {
                1 => 1.0,
                2 => 6.0,
                _ => self.interval * self.easiness,
            };
        }

        // Schedule next review
        let interval_seconds = (self.interval * 86400.0) as u64; // Convert days to seconds
        self.next_review = SystemTime::now() + Duration::from_secs(interval_seconds);
    }

    /// Get the time until next review
    pub fn time_until_review(&self) -> Result<Duration, ()> {
        self.next_review
            .duration_since(SystemTime::now())
            .map_err(|_| ())
    }
}

/// Manages a collection of items for spaced repetition learning
#[derive(Debug, Clone)]
pub struct SpacedRepetitionScheduler<T> {
    items: Vec<ReviewItem<T>>,
}

impl<T: Clone> SpacedRepetitionScheduler<T> {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add an item to the scheduler
    pub fn add_item(&mut self, item: T) {
        self.items.push(ReviewItem::new(item));
    }

    /// Add multiple items to the scheduler
    pub fn add_items(&mut self, items: Vec<T>) {
        for item in items {
            self.add_item(item);
        }
    }

    /// Get the next item due for review, if any
    pub fn next_due_item(&self) -> Option<&ReviewItem<T>> {
        self.items
            .iter()
            .filter(|item| item.is_due())
            .min_by_key(|item| item.next_review)
    }

    /// Get a mutable reference to the next due item
    pub fn next_due_item_mut(&mut self) -> Option<&mut ReviewItem<T>> {
        let min_review_time = self
            .items
            .iter()
            .filter(|item| item.is_due())
            .map(|item| item.next_review)
            .min()?;

        self.items
            .iter_mut()
            .find(|item| item.is_due() && item.next_review == min_review_time)
    }

    /// Get all items that are due for review
    pub fn due_items(&self) -> Vec<&ReviewItem<T>> {
        self.items
            .iter()
            .filter(|item| item.is_due())
            .collect()
    }

    /// Get the total number of items
    pub fn total_items(&self) -> usize {
        self.items.len()
    }

    /// Get the number of items due for review
    pub fn due_count(&self) -> usize {
        self.items.iter().filter(|item| item.is_due()).count()
    }

    /// Get all items
    pub fn items(&self) -> &[ReviewItem<T>] {
        &self.items
    }
}

impl<T: Clone> Default for SpacedRepetitionScheduler<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_rating_quality() {
        assert_eq!(PerformanceRating::Blackout.quality(), 0);
        assert_eq!(PerformanceRating::Incorrect.quality(), 1);
        assert_eq!(PerformanceRating::Perfect.quality(), 5);
    }

    #[test]
    fn test_review_item_creation() {
        let item = ReviewItem::new("test");
        assert_eq!(item.item, "test");
        assert_eq!(item.easiness, 2.5);
        assert_eq!(item.consecutive_correct, 0);
        assert_eq!(item.interval, 0.0);
        assert_eq!(item.total_reviews, 0);
    }

    #[test]
    fn test_review_item_is_due() {
        let item = ReviewItem::new("test");
        assert!(item.is_due()); // New items are due immediately
    }

    #[test]
    fn test_record_perfect_review() {
        let mut item = ReviewItem::new("test");
        item.record_review(PerformanceRating::Perfect);
        
        assert_eq!(item.consecutive_correct, 1);
        assert_eq!(item.interval, 1.0);
        assert_eq!(item.total_reviews, 1);
        assert!(item.easiness > 2.5); // Should increase for perfect performance
    }

    #[test]
    fn test_record_failed_review() {
        let mut item = ReviewItem::new("test");
        item.consecutive_correct = 3;
        item.interval = 10.0;
        
        item.record_review(PerformanceRating::Incorrect);
        
        assert_eq!(item.consecutive_correct, 0);
        assert_eq!(item.interval, 0.0);
        assert_eq!(item.total_reviews, 1);
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler: SpacedRepetitionScheduler<&str> = SpacedRepetitionScheduler::new();
        assert_eq!(scheduler.total_items(), 0);
        assert_eq!(scheduler.due_count(), 0);
    }

    #[test]
    fn test_scheduler_add_items() {
        let mut scheduler = SpacedRepetitionScheduler::new();
        scheduler.add_item("item1");
        scheduler.add_item("item2");
        
        assert_eq!(scheduler.total_items(), 2);
        assert_eq!(scheduler.due_count(), 2);
    }

    #[test]
    fn test_scheduler_next_due_item() {
        let mut scheduler = SpacedRepetitionScheduler::new();
        scheduler.add_item("item1");
        
        let next = scheduler.next_due_item();
        assert!(next.is_some());
        assert_eq!(next.unwrap().item, "item1");
    }

    #[test]
    fn test_easiness_bounds() {
        let mut item = ReviewItem::new("test");
        // Record many blackouts to try to push easiness below 1.3
        for _ in 0..10 {
            item.record_review(PerformanceRating::Blackout);
        }
        assert!(item.easiness >= 1.3);
    }

    #[test]
    fn test_sm2_progression() {
        let mut item = ReviewItem::new("test");
        
        // First review - should set interval to 1 day
        item.record_review(PerformanceRating::Good);
        assert_eq!(item.interval, 1.0);
        
        // Second review - should set interval to 6 days
        item.record_review(PerformanceRating::Good);
        assert_eq!(item.interval, 6.0);
        
        // Third review - should multiply by easiness
        let easiness = item.easiness;
        item.record_review(PerformanceRating::Good);
        assert!((item.interval - 6.0 * easiness).abs() < 0.01);
    }
}
