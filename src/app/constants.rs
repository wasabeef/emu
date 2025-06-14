//! Application constants for performance and configuration.

use std::time::Duration;

/// Performance-related constants
pub mod performance {
    use super::*;

    /// Target frame rate for rendering (125 FPS)
    pub const TARGET_FPS: u32 = 125;

    /// Frame duration in milliseconds (8ms for 125 FPS)
    pub const FRAME_TIME_MS: u64 = 1000 / TARGET_FPS as u64;

    /// Frame duration as Duration
    pub const FRAME_DURATION: Duration = Duration::from_millis(FRAME_TIME_MS);

    /// Maximum events to process per frame
    pub const MAX_EVENTS_PER_FRAME: usize = 50;

    /// Navigation batch timeout in milliseconds
    pub const NAVIGATION_BATCH_TIMEOUT_MS: u64 = 50;

    /// Event debounce duration in milliseconds
    pub const EVENT_DEBOUNCE_MS: u64 = 5;

    /// Background update debounce duration in milliseconds
    pub const BACKGROUND_UPDATE_DEBOUNCE_MS: u64 = 50;
}

/// UI-related constants
pub mod ui {
    /// Maximum log entries to keep in memory
    pub const MAX_LOG_ENTRIES: usize = 1000;

    /// Maximum notifications to display
    pub const MAX_NOTIFICATIONS: usize = 5;

    /// Notification display duration in seconds
    pub const NOTIFICATION_DURATION_SECS: u64 = 5;
}

/// Device operation timeouts
pub mod timeouts {
    use super::*;

    /// Device start operation timeout
    pub const DEVICE_START_TIMEOUT: Duration = Duration::from_secs(60);

    /// Device stop operation timeout  
    pub const DEVICE_STOP_TIMEOUT: Duration = Duration::from_secs(30);

    /// Device creation operation timeout
    pub const DEVICE_CREATE_TIMEOUT: Duration = Duration::from_secs(120);

    /// Cache refresh timeout
    pub const CACHE_REFRESH_TIMEOUT: Duration = Duration::from_secs(10);
}
