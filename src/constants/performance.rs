//! Performance-related constants for tuning application behavior.

use std::time::Duration;

/// Maximum events to process per frame (prevents UI freezing)
pub const MAX_EVENTS_PER_FRAME: usize = 50;

/// UI update interval (16.67ms for ~60 FPS)
pub const UI_UPDATE_INTERVAL: Duration = Duration::from_millis(17);

/// Event polling timeout (100ms for responsive input)
pub const EVENT_POLL_TIMEOUT: Duration = Duration::from_millis(100);

/// Auto-refresh check interval (1 second)
pub const AUTO_REFRESH_CHECK_INTERVAL: Duration = Duration::from_millis(1000);

/// Notification check interval (500ms)
pub const NOTIFICATION_CHECK_INTERVAL: Duration = Duration::from_millis(500);

/// Device detail update debounce (100ms)
pub const DETAIL_UPDATE_DEBOUNCE: Duration = Duration::from_millis(100);

/// Log update debounce (50ms)
pub const LOG_UPDATE_DEBOUNCE: Duration = Duration::from_millis(50);

/// Progress update delay (500ms to ensure final update is shown)
pub const PROGRESS_UPDATE_DELAY: Duration = Duration::from_millis(500);

/// Initial startup delay for background loading (50ms)
pub const STARTUP_DELAY: Duration = Duration::from_millis(50);

/// Keyboard navigation repeat delay (200ms)
pub const KEY_REPEAT_DELAY: Duration = Duration::from_millis(200);

/// Keyboard navigation repeat rate (50ms)
pub const KEY_REPEAT_RATE: Duration = Duration::from_millis(50);

/// Target frame rate for rendering (125 FPS)
pub const TARGET_FPS: u32 = 125;

/// Frame duration in milliseconds (8ms for 125 FPS)
pub const FRAME_TIME_MS: u64 = 1000 / TARGET_FPS as u64;

/// Frame duration as Duration
pub const FRAME_DURATION: Duration = Duration::from_millis(FRAME_TIME_MS);

/// Fast panel switching optimization delays
/// Device detail update debounce for fast panel switching (25ms)
pub const FAST_DETAIL_UPDATE_DEBOUNCE: Duration = Duration::from_millis(25);

/// Log update debounce for fast panel switching (50ms)
pub const FAST_LOG_UPDATE_DEBOUNCE: Duration = Duration::from_millis(50);
