//! Debug module for analyzing event processing performance issues

use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Statistics for event processing performance
#[derive(Debug, Default)]
pub struct EventStats {
    /// Total number of events processed
    pub total_events: usize,
    /// Number of navigation events (up/down/j/k)
    pub navigation_events: usize,
    /// Events dropped due to debouncing
    pub dropped_events: usize,
    /// Time spent processing events
    pub processing_time: Duration,
    /// Time spent holding state lock
    pub lock_time: Duration,
    /// Recent event timings for analysis
    pub recent_events: VecDeque<EventTiming>,
    /// Maximum events processed in a single batch
    pub max_batch_size: usize,
    /// Current batch size
    pub current_batch_size: usize,
}

#[derive(Debug)]
pub struct EventTiming {
    pub timestamp: Instant,
    pub event_type: EventType,
    pub processing_duration: Duration,
    pub lock_duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    Navigation,
    PanelSwitch,
    Other,
}

impl EventStats {
    pub fn new() -> Self {
        Self {
            recent_events: VecDeque::with_capacity(100),
            ..Default::default()
        }
    }
    
    pub fn record_event(&mut self, event_type: EventType, processing_duration: Duration, lock_duration: Duration) {
        self.total_events += 1;
        self.processing_time += processing_duration;
        self.lock_time += lock_duration;
        
        if event_type == EventType::Navigation {
            self.navigation_events += 1;
        }
        
        self.recent_events.push_back(EventTiming {
            timestamp: Instant::now(),
            event_type,
            processing_duration,
            lock_duration,
        });
        
        // Keep only recent 100 events
        while self.recent_events.len() > 100 {
            self.recent_events.pop_front();
        }
    }
    
    pub fn record_batch_start(&mut self) {
        self.current_batch_size = 0;
    }
    
    pub fn record_batch_event(&mut self) {
        self.current_batch_size += 1;
        self.max_batch_size = self.max_batch_size.max(self.current_batch_size);
    }
    
    pub fn record_dropped_event(&mut self) {
        self.dropped_events += 1;
    }
    
    pub fn get_recent_navigation_rate(&self) -> f64 {
        if self.recent_events.len() < 2 {
            return 0.0;
        }
        
        let nav_events: Vec<_> = self.recent_events
            .iter()
            .filter(|e| e.event_type == EventType::Navigation)
            .collect();
        
        if nav_events.len() < 2 {
            return 0.0;
        }
        
        let duration = nav_events.last().unwrap().timestamp
            .duration_since(nav_events.first().unwrap().timestamp);
        
        if duration.as_millis() == 0 {
            return 0.0;
        }
        
        (nav_events.len() as f64 - 1.0) / duration.as_secs_f64()
    }
    
    pub fn get_average_lock_time(&self) -> Duration {
        if self.total_events == 0 {
            return Duration::ZERO;
        }
        self.lock_time / self.total_events as u32
    }
    
    pub fn print_summary(&self) {
        println!("=== Event Processing Statistics ===");
        println!("Total events: {}", self.total_events);
        println!("Navigation events: {}", self.navigation_events);
        println!("Dropped events: {}", self.dropped_events);
        println!("Max batch size: {}", self.max_batch_size);
        println!("Average lock time: {:?}", self.get_average_lock_time());
        println!("Recent navigation rate: {:.1} events/sec", self.get_recent_navigation_rate());
        
        if !self.recent_events.is_empty() {
            let recent_nav_times: Vec<_> = self.recent_events
                .iter()
                .filter(|e| e.event_type == EventType::Navigation)
                .map(|e| e.processing_duration)
                .collect();
            
            if !recent_nav_times.is_empty() {
                let avg_nav_time = recent_nav_times.iter().sum::<Duration>() / recent_nav_times.len() as u32;
                println!("Average navigation processing time: {:?}", avg_nav_time);
            }
        }
    }
}

/// Helper to detect event flooding
pub fn is_event_flooding(event_count: usize, elapsed: Duration) -> bool {
    // More than 50 events per second indicates flooding
    let rate = event_count as f64 / elapsed.as_secs_f64();
    rate > 50.0
}

/// Calculate appropriate debounce delay based on event rate
pub fn calculate_debounce_delay(event_rate: f64) -> Duration {
    if event_rate > 30.0 {
        Duration::from_millis(50) // Heavy debouncing for rapid events
    } else if event_rate > 15.0 {
        Duration::from_millis(25) // Medium debouncing
    } else {
        Duration::from_millis(10) // Light debouncing for normal usage
    }
}