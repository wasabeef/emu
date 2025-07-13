//! Detailed tests for app/event_processing.rs
//!
//! Tests focus on event batching, debouncing, and navigation processing
//! that are part of the legacy event processing system.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind, KeyEventState};
use emu::app::event_processing::{EventBatcher, EventDebouncer, NavigationBatcher};
use std::time::Duration;

#[cfg(test)]
mod navigation_batcher_tests {
    use super::*;

    #[test]
    fn test_navigation_batcher_creation() {
        let batcher = NavigationBatcher::new(50);
        assert!(!batcher.has_pending_steps());
        assert!(!batcher.is_timed_out());
    }

    #[test]
    fn test_vertical_navigation_up() {
        let mut batcher = NavigationBatcher::new(50);
        batcher.add_navigation(KeyCode::Up);
        batcher.add_navigation(KeyCode::Char('k'));
        
        assert!(batcher.has_pending_steps());
        let (vertical, horizontal) = batcher.take_steps();
        assert_eq!(vertical, -2); // Two up movements
        assert_eq!(horizontal, 0);
        assert!(!batcher.has_pending_steps());
    }

    #[test]
    fn test_vertical_navigation_down() {
        let mut batcher = NavigationBatcher::new(50);
        batcher.add_navigation(KeyCode::Down);
        batcher.add_navigation(KeyCode::Char('j'));
        
        let (vertical, horizontal) = batcher.take_steps();
        assert_eq!(vertical, 2); // Two down movements
        assert_eq!(horizontal, 0);
    }

    #[test]
    fn test_horizontal_navigation_left() {
        let mut batcher = NavigationBatcher::new(50);
        batcher.add_navigation(KeyCode::Left);
        batcher.add_navigation(KeyCode::Char('h'));
        
        let (vertical, horizontal) = batcher.take_steps();
        assert_eq!(vertical, 0);
        assert_eq!(horizontal, -2); // Two left movements
    }

    #[test]
    fn test_horizontal_navigation_right() {
        let mut batcher = NavigationBatcher::new(50);
        batcher.add_navigation(KeyCode::Right);
        batcher.add_navigation(KeyCode::Char('l'));
        
        let (vertical, horizontal) = batcher.take_steps();
        assert_eq!(vertical, 0);
        assert_eq!(horizontal, 2); // Two right movements
    }

    #[test]
    fn test_mixed_navigation() {
        let mut batcher = NavigationBatcher::new(50);
        batcher.add_navigation(KeyCode::Up);
        batcher.add_navigation(KeyCode::Down);
        batcher.add_navigation(KeyCode::Down);
        batcher.add_navigation(KeyCode::Left);
        batcher.add_navigation(KeyCode::Right);
        batcher.add_navigation(KeyCode::Right);
        
        let (vertical, horizontal) = batcher.take_steps();
        assert_eq!(vertical, 1); // -1 + 1 + 1 = 1
        assert_eq!(horizontal, 1); // -1 + 1 + 1 = 1
    }

    #[test]
    fn test_non_navigation_keys_ignored() {
        let mut batcher = NavigationBatcher::new(50);
        batcher.add_navigation(KeyCode::Enter);
        batcher.add_navigation(KeyCode::Char('q'));
        batcher.add_navigation(KeyCode::Space);
        
        assert!(!batcher.has_pending_steps());
        let (vertical, horizontal) = batcher.take_steps();
        assert_eq!(vertical, 0);
        assert_eq!(horizontal, 0);
    }

    #[test]
    fn test_timeout_detection() {
        let mut batcher = NavigationBatcher::new(1); // 1ms timeout
        batcher.add_navigation(KeyCode::Up);
        
        // Wait longer than timeout
        std::thread::sleep(Duration::from_millis(5));
        assert!(batcher.is_timed_out());
    }

    #[test]
    fn test_reset_after_take_steps() {
        let mut batcher = NavigationBatcher::new(50);
        batcher.add_navigation(KeyCode::Up);
        batcher.add_navigation(KeyCode::Right);
        
        assert!(batcher.has_pending_steps());
        batcher.take_steps();
        assert!(!batcher.has_pending_steps());
    }
}

#[cfg(test)]
mod event_debouncer_tests {
    use super::*;

    fn create_key_event(key_code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code: key_code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[test]
    fn test_debouncer_creation() {
        let mut debouncer = EventDebouncer::new(50);
        let event = create_key_event(KeyCode::Char('q'));
        assert!(debouncer.should_process(&event));
    }

    #[test]
    fn test_first_event_always_processed() {
        let mut debouncer = EventDebouncer::new(50);
        let event = create_key_event(KeyCode::Char('q'));
        assert!(debouncer.should_process(&event));
    }

    #[test]
    fn test_duplicate_event_within_debounce_window() {
        let mut debouncer = EventDebouncer::new(50);
        let event = create_key_event(KeyCode::Char('q'));
        
        // First event should be processed
        assert!(debouncer.should_process(&event));
        
        // Immediate duplicate should be ignored
        assert!(!debouncer.should_process(&event));
    }

    #[test]
    fn test_different_events_processed() {
        let mut debouncer = EventDebouncer::new(50);
        let event1 = create_key_event(KeyCode::Char('q'));
        let event2 = create_key_event(KeyCode::Char('w'));
        
        assert!(debouncer.should_process(&event1));
        assert!(debouncer.should_process(&event2));
    }

    #[test]
    fn test_duplicate_event_after_timeout() {
        let mut debouncer = EventDebouncer::new(1); // 1ms debounce
        let event = create_key_event(KeyCode::Char('q'));
        
        // First event processed
        assert!(debouncer.should_process(&event));
        
        // Wait for debounce timeout
        std::thread::sleep(Duration::from_millis(5));
        
        // Same event should now be processed
        assert!(debouncer.should_process(&event));
    }

    #[test]
    fn test_non_key_events_always_processed() {
        let mut debouncer = EventDebouncer::new(50);
        let resize_event = Event::Resize(80, 24);
        
        // Non-key events should always be processed
        assert!(debouncer.should_process(&resize_event));
        assert!(debouncer.should_process(&resize_event));
        assert!(debouncer.should_process(&resize_event));
    }

    #[test]
    fn test_modifier_key_events() {
        let mut debouncer = EventDebouncer::new(50);
        let event_with_ctrl = Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        
        assert!(debouncer.should_process(&event_with_ctrl));
        assert!(!debouncer.should_process(&event_with_ctrl));
    }
}

#[cfg(test)]
mod event_batcher_tests {
    use super::*;

    #[test]
    fn test_event_batcher_creation() {
        let batcher = EventBatcher::new(10);
        let batch = batcher.take_batch();
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_add_non_navigation_event() {
        let mut batcher = EventBatcher::new(10);
        let event = create_key_event(KeyCode::Char('q'));
        
        batcher.add_event(event.clone());
        let batch = batcher.take_batch();
        
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0], event);
    }

    #[test]
    fn test_navigation_events_batched() {
        let mut batcher = EventBatcher::new(10);
        let up_event = create_key_event(KeyCode::Up);
        let down_event = create_key_event(KeyCode::Down);
        
        batcher.add_event(up_event);
        batcher.add_event(down_event);
        
        // Navigation events should not appear in immediate batch
        let immediate_batch = batcher.take_batch();
        assert_eq!(immediate_batch.len(), 0);
        
        // Wait for navigation timeout
        std::thread::sleep(Duration::from_millis(55));
        let delayed_batch = batcher.take_batch();
        
        // Should contain processed navigation (net zero movement)
        assert_eq!(delayed_batch.len(), 0);
    }

    #[test]
    fn test_vim_navigation_keys() {
        let mut batcher = EventBatcher::new(10);
        batcher.add_event(create_key_event(KeyCode::Char('j'))); // down
        batcher.add_event(create_key_event(KeyCode::Char('k'))); // up
        batcher.add_event(create_key_event(KeyCode::Char('h'))); // left
        batcher.add_event(create_key_event(KeyCode::Char('l'))); // right
        
        // All navigation events should be batched
        let immediate_batch = batcher.take_batch();
        assert_eq!(immediate_batch.len(), 0);
    }

    #[test]
    fn test_mixed_events() {
        let mut batcher = EventBatcher::new(10);
        let quit_event = create_key_event(KeyCode::Char('q'));
        let nav_event = create_key_event(KeyCode::Up);
        let enter_event = create_key_event(KeyCode::Enter);
        
        batcher.add_event(quit_event.clone());
        batcher.add_event(nav_event);
        batcher.add_event(enter_event.clone());
        
        let batch = batcher.take_batch();
        
        // Should contain non-navigation events
        assert_eq!(batch.len(), 2);
        assert!(batch.contains(&quit_event));
        assert!(batch.contains(&enter_event));
    }

    #[test]
    fn test_batch_size_limit() {
        let mut batcher = EventBatcher::new(2); // Small batch size
        let events: Vec<Event> = (0..5)
            .map(|i| create_key_event(KeyCode::Char((b'a' + i as u8) as char)))
            .collect();
        
        for event in &events {
            batcher.add_event(event.clone());
        }
        
        let batch = batcher.take_batch();
        
        // Should respect batch size limits
        assert!(batch.len() <= 4); // max_batch_size * 2
    }

    #[test]
    fn test_debouncing_in_batcher() {
        let mut batcher = EventBatcher::new(10);
        let event = create_key_event(KeyCode::Char('q'));
        
        // Add the same event multiple times rapidly
        batcher.add_event(event.clone());
        batcher.add_event(event.clone());
        batcher.add_event(event.clone());
        
        let batch = batcher.take_batch();
        
        // Should be debounced to single event
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0], event);
    }

    fn create_key_event(key_code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code: key_code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complex_navigation_sequence() {
        let mut batcher = NavigationBatcher::new(50);
        
        // Simulate complex navigation: down 3, up 1, right 2, left 1
        batcher.add_navigation(KeyCode::Down);
        batcher.add_navigation(KeyCode::Char('j'));
        batcher.add_navigation(KeyCode::Down);
        batcher.add_navigation(KeyCode::Up);
        batcher.add_navigation(KeyCode::Right);
        batcher.add_navigation(KeyCode::Char('l'));
        batcher.add_navigation(KeyCode::Left);
        
        let (vertical, horizontal) = batcher.take_steps();
        assert_eq!(vertical, 2); // 3 - 1 = 2
        assert_eq!(horizontal, 1); // 2 - 1 = 1
    }

    #[test]
    fn test_event_processing_pipeline() {
        let mut debouncer = EventDebouncer::new(10);
        let mut nav_batcher = NavigationBatcher::new(50);
        
        let events = vec![
            create_key_event(KeyCode::Char('q')),
            create_key_event(KeyCode::Up),
            create_key_event(KeyCode::Up), // Duplicate
            create_key_event(KeyCode::Down),
            create_key_event(KeyCode::Enter),
        ];
        
        let mut processed_events = Vec::new();
        
        for event in events {
            if debouncer.should_process(&event) {
                if let Event::Key(KeyEvent { code, .. }) = &event {
                    match code {
                        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
                        | KeyCode::Char('h') | KeyCode::Char('j') | KeyCode::Char('k') | KeyCode::Char('l') => {
                            nav_batcher.add_navigation(*code);
                        }
                        _ => {
                            processed_events.push(event);
                        }
                    }
                }
            }
        }
        
        // Should have processed non-navigation events and debounced duplicates
        assert!(processed_events.len() >= 2);
        assert!(nav_batcher.has_pending_steps());
    }

    fn create_key_event(key_code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code: key_code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }
}