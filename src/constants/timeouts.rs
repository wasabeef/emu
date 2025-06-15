//! Timeout and delay constants for various operations.

use std::time::Duration;

/// Initial retry delay for command execution
pub const INITIAL_RETRY_DELAY: Duration = Duration::from_millis(100);

/// Maximum retry delay for command execution
pub const MAX_RETRY_DELAY: Duration = Duration::from_secs(2);

/// Thread sleep duration for event processing
pub const EVENT_PROCESSING_SLEEP: Duration = Duration::from_millis(15);

/// Device detail update debounce delay
pub const DEVICE_DETAIL_UPDATE_DELAY: Duration = Duration::from_millis(50);

/// Log update debounce delay
pub const LOG_UPDATE_DELAY: Duration = Duration::from_millis(100);

/// Progress display delay after completion
pub const PROGRESS_COMPLETION_DELAY: Duration = Duration::from_millis(500);

/// Cache invalidation time offset
pub const CACHE_INVALIDATION_OFFSET_SECS: u64 = 400;

/// Auto-refresh interval for fast updates
pub const FAST_REFRESH_INTERVAL_SECS: u64 = 1;

/// Navigation batcher timeout
pub const NAVIGATION_BATCH_TIMEOUT_MS: u64 = 50;

/// Event debouncer timeout
pub const EVENT_DEBOUNCE_TIMEOUT_MS: u64 = 5;

/// Android device status check timeout
pub const ANDROID_STATUS_CHECK_TIMEOUT: Duration = Duration::from_secs(2);

/// AVD creation wait timeout
pub const AVD_CREATION_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
