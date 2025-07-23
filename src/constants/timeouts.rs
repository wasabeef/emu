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

/// Device status check delay (used in Android manager)
pub const DEVICE_STATUS_CHECK_DELAY: Duration = Duration::from_millis(500);

/// Device start wait time
pub const DEVICE_START_WAIT_TIME: Duration = Duration::from_secs(2);

/// Log stream startup delay
pub const LOG_STREAM_STARTUP_DELAY: Duration = Duration::from_millis(500);

/// API installation wait time
pub const API_INSTALLATION_WAIT_TIME: Duration = Duration::from_secs(2);

/// Device stop wait time
pub const DEVICE_STOP_WAIT_TIME: Duration = Duration::from_millis(2000);

/// Cache expiration time (5 minutes)
pub const CACHE_EXPIRATION_TIME: Duration = Duration::from_secs(300);

/// Notification auto-dismiss time
pub const NOTIFICATION_AUTO_DISMISS_TIME: Duration = Duration::from_secs(5);

/// Default auto-refresh interval (extended to reduce UI interruption)
pub const DEFAULT_AUTO_REFRESH_INTERVAL: Duration = Duration::from_secs(5);

/// Auto-refresh check interval
pub const AUTO_REFRESH_CHECK_INTERVAL: Duration = Duration::from_millis(1000);

/// Notification check interval  
pub const NOTIFICATION_CHECK_INTERVAL: Duration = Duration::from_millis(500);

/// Event poll timeout (reduced for ultra-responsive input)
pub const EVENT_POLL_TIMEOUT: Duration = Duration::from_millis(8);

/// Log task sleep duration
pub const LOG_TASK_SLEEP_DURATION: Duration = Duration::from_millis(100);

/// Device operation wait time
pub const DEVICE_OPERATION_WAIT_TIME: Duration = Duration::from_millis(100);

/// Panel switch delay
pub const PANEL_SWITCH_DELAY: Duration = Duration::from_millis(50);
