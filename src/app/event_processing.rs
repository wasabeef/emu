//! Optimized event processing for handling rapid key repeats

use crossterm::event::{Event as CrosstermEvent, KeyCode};
use std::time::{Duration, Instant};

/// Configuration for event processing optimization
#[derive(Debug, Clone)]
pub struct EventProcessingConfig {
    /// Maximum events to process per frame
    pub max_events_per_frame: usize,
    /// Minimum time between navigation events
    pub navigation_debounce: Duration,
    /// Maximum time to spend processing events per frame
    pub max_processing_time: Duration,
    /// Whether to enable event batching for navigation
    pub batch_navigation: bool,
}

impl Default for EventProcessingConfig {
    fn default() -> Self {
        Self {
            max_events_per_frame: 5, // Process at most 5 events per frame
            navigation_debounce: Duration::from_millis(30), // 30ms between navigation events
            max_processing_time: Duration::from_millis(5), // Spend at most 5ms processing events
            batch_navigation: true,
        }
    }
}

/// Tracks timing for event debouncing
#[derive(Debug)]
pub struct EventDebouncer {
    last_navigation: Instant,
    last_panel_switch: Instant,
    navigation_count: usize,
    config: EventProcessingConfig,
}

impl Default for EventDebouncer {
    fn default() -> Self {
        Self {
            last_navigation: Instant::now(),
            last_panel_switch: Instant::now(),
            navigation_count: 0,
            config: EventProcessingConfig::default(),
        }
    }
}

impl EventDebouncer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: EventProcessingConfig) -> Self {
        Self {
            last_navigation: Instant::now(),
            last_panel_switch: Instant::now(),
            navigation_count: 0,
            config,
        }
    }

    /// Check if a navigation event should be processed
    pub fn should_process_navigation(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_navigation) >= self.config.navigation_debounce {
            self.last_navigation = now;
            self.navigation_count = 0;
            true
        } else {
            self.navigation_count += 1;
            false
        }
    }

    /// Check if a panel switch event should be processed
    pub fn should_process_panel_switch(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_panel_switch) >= Duration::from_millis(100) {
            self.last_panel_switch = now;
            true
        } else {
            false
        }
    }

    /// Get the current navigation rate (events per second)
    pub fn get_navigation_rate(&self) -> f64 {
        self.navigation_count as f64 / self.config.navigation_debounce.as_secs_f64()
    }

    /// Reset all debounce timers
    pub fn reset(&mut self) {
        self.last_navigation = Instant::now();
        self.last_panel_switch = Instant::now();
        self.navigation_count = 0;
    }
}

/// Result of processing a batch of events
#[derive(Debug)]
pub struct EventBatchResult {
    /// Number of events processed
    pub events_processed: usize,
    /// Number of events skipped due to debouncing
    pub events_skipped: usize,
    /// Whether a render is needed
    pub needs_render: bool,
    /// Whether there are more events pending
    pub has_pending_events: bool,
    /// Total time spent processing
    pub processing_time: Duration,
}

/// Process events with optimizations for rapid key repeats
pub async fn process_event_batch<F>(
    debouncer: &mut EventDebouncer,
    mut event_handler: F,
) -> anyhow::Result<EventBatchResult>
where
    F: FnMut(CrosstermEvent, &mut EventDebouncer) -> anyhow::Result<bool>,
{
    let start = Instant::now();
    let mut result = EventBatchResult {
        events_processed: 0,
        events_skipped: 0,
        needs_render: false,
        has_pending_events: false,
        processing_time: Duration::ZERO,
    };

    // Process events with limits
    while result.events_processed < debouncer.config.max_events_per_frame {
        // Check if we've spent too much time
        if start.elapsed() > debouncer.config.max_processing_time {
            result.has_pending_events = crossterm::event::poll(Duration::ZERO)?;
            break;
        }

        // Poll for event with zero timeout
        if !crossterm::event::poll(Duration::ZERO)? {
            break;
        }

        // Read the event
        match crossterm::event::read()? {
            event @ CrosstermEvent::Key(key) => {
                // Check if this is a navigation event that should be debounced
                let is_navigation = matches!(
                    key.code,
                    KeyCode::Up | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('k')
                );

                if is_navigation && !debouncer.should_process_navigation() {
                    result.events_skipped += 1;
                    continue;
                }

                // Process the event
                let needs_render = event_handler(event, debouncer)?;
                if needs_render {
                    result.needs_render = true;
                }
                result.events_processed += 1;
            }
            event => {
                // Process non-key events normally
                let needs_render = event_handler(event, debouncer)?;
                if needs_render {
                    result.needs_render = true;
                }
                result.events_processed += 1;
            }
        }
    }

    // Check if there are more events pending
    result.has_pending_events = crossterm::event::poll(Duration::ZERO)?;
    result.processing_time = start.elapsed();

    Ok(result)
}

/// Helper to batch consecutive navigation events
#[derive(Default)]
pub struct NavigationBatcher {
    pending_moves: i32,
    last_direction: Option<NavigationDirection>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavigationDirection {
    Up,
    Down,
}

impl NavigationBatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a navigation event to the batch
    pub fn add_navigation(&mut self, direction: NavigationDirection) {
        if Some(direction) == self.last_direction {
            match direction {
                NavigationDirection::Up => self.pending_moves -= 1,
                NavigationDirection::Down => self.pending_moves += 1,
            }
        } else {
            // Direction changed, apply pending moves first
            self.pending_moves = match direction {
                NavigationDirection::Up => -1,
                NavigationDirection::Down => 1,
            };
            self.last_direction = Some(direction);
        }
    }

    /// Get the net navigation offset
    pub fn get_net_moves(&mut self) -> i32 {
        let moves = self.pending_moves;
        self.pending_moves = 0;
        self.last_direction = None;
        moves
    }

    /// Check if there are pending moves
    pub fn has_pending_moves(&self) -> bool {
        self.pending_moves != 0
    }
}
