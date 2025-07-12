//! Event processing utilities for key input handling (legacy).
//!
//! This module provided event batching and debouncing functionality in previous versions.
//! The current implementation uses direct event processing in the main loop for
//! ultra-responsive input handling without any debouncing or batching delays.
//!
//! **Note**: This module is kept for compatibility but is no longer used in the main
//! application loop. The app now uses direct event processing for 120fps responsiveness.

#[allow(unused_imports)]
use crate::constants::performance::{
    EVENT_DEBOUNCE_TIMEOUT, NAVIGATION_BATCH_TIMEOUT, TEST_SLEEP_DURATION,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Processes and batches navigation events for efficient handling.
/// Accumulates multiple navigation inputs into single operations.
#[derive(Debug)]
pub struct NavigationBatcher {
    /// Accumulated vertical navigation steps (positive = down, negative = up)
    vertical_steps: i32,
    /// Accumulated horizontal navigation steps (positive = right, negative = left)
    horizontal_steps: i32,
    /// Last navigation time for timeout detection
    last_navigation: Instant,
    /// Timeout duration for navigation batching
    timeout: Duration,
}

impl NavigationBatcher {
    /// Creates a new navigation batcher with specified timeout.
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            vertical_steps: 0,
            horizontal_steps: 0,
            last_navigation: Instant::now(),
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    /// Adds a navigation event to the batch.
    pub fn add_navigation(&mut self, key_code: KeyCode) {
        self.last_navigation = Instant::now();

        match key_code {
            KeyCode::Up | KeyCode::Char('k') => self.vertical_steps -= 1,
            KeyCode::Down | KeyCode::Char('j') => self.vertical_steps += 1,
            KeyCode::Left | KeyCode::Char('h') => self.horizontal_steps -= 1,
            KeyCode::Right | KeyCode::Char('l') => self.horizontal_steps += 1,
            _ => {}
        }
    }

    /// Checks if the batch has timed out.
    pub fn is_timed_out(&self) -> bool {
        self.last_navigation.elapsed() > self.timeout
    }

    /// Takes accumulated navigation steps and resets the batcher.
    pub fn take_steps(&mut self) -> (i32, i32) {
        let steps = (self.vertical_steps, self.horizontal_steps);
        self.vertical_steps = 0;
        self.horizontal_steps = 0;
        steps
    }

    /// Checks if there are pending navigation steps.
    pub fn has_pending_steps(&self) -> bool {
        self.vertical_steps != 0 || self.horizontal_steps != 0
    }
}

/// Debounces events to prevent duplicate processing.
#[derive(Debug)]
pub struct EventDebouncer {
    /// Last processed event
    last_event: Option<Event>,
    /// Time of last event
    last_event_time: Instant,
    /// Minimum time between duplicate events
    debounce_duration: Duration,
}

impl EventDebouncer {
    /// Creates a new event debouncer with specified duration.
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            last_event: None,
            last_event_time: Instant::now(),
            debounce_duration: Duration::from_millis(debounce_ms),
        }
    }

    /// Checks if an event should be processed or ignored.
    pub fn should_process(&mut self, event: &Event) -> bool {
        let now = Instant::now();

        // Always process non-key events
        if !matches!(event, Event::Key(_)) {
            return true;
        }

        // Check if this is a duplicate event within debounce window
        if let Some(ref last) = self.last_event {
            if last == event && now.duration_since(self.last_event_time) < self.debounce_duration {
                return false;
            }
        }

        // Update last event
        self.last_event = Some(event.clone());
        self.last_event_time = now;
        true
    }
}

/// Batches multiple events for processing in a single frame.
#[derive(Debug)]
pub struct EventBatcher {
    /// Queue of pending events
    event_queue: VecDeque<Event>,
    /// Maximum events to process per batch
    max_batch_size: usize,
    /// Navigation event batcher
    navigation_batcher: NavigationBatcher,
    /// Event debouncer
    debouncer: EventDebouncer,
}

impl EventBatcher {
    /// Creates a new event batcher with specified limits.
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            event_queue: VecDeque::new(),
            max_batch_size,
            navigation_batcher: NavigationBatcher::new(NAVIGATION_BATCH_TIMEOUT.as_millis() as u64), // 50ms timeout
            debouncer: EventDebouncer::new(5), // 5ms debounce
        }
    }

    /// Adds an event to the batch queue.
    pub fn add_event(&mut self, event: Event) {
        // Apply debouncing
        if !self.debouncer.should_process(&event) {
            return;
        }

        // Handle navigation events specially
        if let Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            ..
        }) = &event
        {
            match code {
                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::Char('h')
                | KeyCode::Char('j')
                | KeyCode::Char('k')
                | KeyCode::Char('l') => {
                    self.navigation_batcher.add_navigation(*code);
                    return; // Don't add to regular queue
                }
                _ => {}
            }
        }

        // Add non-navigation events to queue
        if self.event_queue.len() < self.max_batch_size * 2 {
            self.event_queue.push_back(event);
        }
    }

    /// Takes a batch of events for processing.
    pub fn take_batch(&mut self) -> Vec<Event> {
        let mut batch = Vec::new();

        // First, check for navigation batch
        if self.navigation_batcher.has_pending_steps()
            && (self.navigation_batcher.is_timed_out() || self.event_queue.is_empty())
        {
            let (vertical, horizontal) = self.navigation_batcher.take_steps();

            // Convert accumulated steps into navigation events
            if vertical != 0 {
                batch.push(Event::Key(KeyEvent {
                    code: if vertical > 0 {
                        KeyCode::Down
                    } else {
                        KeyCode::Up
                    },
                    modifiers: KeyModifiers::NONE,
                    kind: crossterm::event::KeyEventKind::Press,
                    state: crossterm::event::KeyEventState::NONE,
                }));

                // Add step count as a special marker (we'll handle this in the app)
                if vertical.abs() > 1 {
                    for _ in 1..vertical.abs() {
                        batch.push(Event::Key(KeyEvent {
                            code: if vertical > 0 {
                                KeyCode::Down
                            } else {
                                KeyCode::Up
                            },
                            modifiers: KeyModifiers::NONE,
                            kind: crossterm::event::KeyEventKind::Press,
                            state: crossterm::event::KeyEventState::NONE,
                        }));
                    }
                }
            }

            if horizontal != 0 {
                batch.push(Event::Key(KeyEvent {
                    code: if horizontal > 0 {
                        KeyCode::Right
                    } else {
                        KeyCode::Left
                    },
                    modifiers: KeyModifiers::NONE,
                    kind: crossterm::event::KeyEventKind::Press,
                    state: crossterm::event::KeyEventState::NONE,
                }));
            }
        }

        // Then add regular events up to batch size
        while batch.len() < self.max_batch_size && !self.event_queue.is_empty() {
            if let Some(event) = self.event_queue.pop_front() {
                batch.push(event);
            }
        }

        batch
    }

    /// Checks if there are pending events to process.
    pub fn has_pending_events(&self) -> bool {
        !self.event_queue.is_empty()
            || (self.navigation_batcher.has_pending_steps()
                && self.navigation_batcher.is_timed_out())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_batcher() {
        let mut batcher = NavigationBatcher::new(NAVIGATION_BATCH_TIMEOUT.as_millis() as u64);

        // Add multiple navigation events
        batcher.add_navigation(KeyCode::Down);
        batcher.add_navigation(KeyCode::Down);
        batcher.add_navigation(KeyCode::Right);

        assert_eq!(batcher.take_steps(), (2, 1));
        assert_eq!(batcher.take_steps(), (0, 0)); // Should be reset
    }

    #[test]
    fn test_event_debouncer() {
        let mut debouncer = EventDebouncer::new(EVENT_DEBOUNCE_TIMEOUT.as_millis() as u64);
        let key_event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });

        assert!(debouncer.should_process(&key_event));
        assert!(!debouncer.should_process(&key_event)); // Should be debounced

        std::thread::sleep(TEST_SLEEP_DURATION);
        assert!(debouncer.should_process(&key_event)); // Should process after timeout
    }
}
