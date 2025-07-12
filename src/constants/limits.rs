//! Size limits and validation constants.
//!
//! This module defines various limits and constraints used throughout the application,
//! including device configuration limits, string processing limits, and validation ranges.
//!
//! # Categories
//!
//! ## Memory and Storage Limits
//! - RAM size limits (MIN_RAM_MB, MAX_RAM_MB)
//! - Storage size limits (MIN_STORAGE_MB, MAX_STORAGE_MB)
//! - Memory validation tiers for different device configurations
//!
//! ## String and Text Limits
//! - Device name length constraints
//! - Display name truncation limits
//! - Word count limits for name processing
//!
//! ## UI and Display Limits
//! - Maximum log entries and notifications
//! - Error message display lengths
//! - Device type display constraints
//!
//! ## Validation Constants
//! - Array index constants for safe array access
//! - Version number limits
//! - String matching minimum lengths

/// Minimum RAM size in MB for Android devices
pub const MIN_RAM_MB: u32 = 512;

/// Maximum RAM size in MB for Android devices
pub const MAX_RAM_MB: u32 = 8192;

/// Minimum storage size in MB for Android devices
pub const MIN_STORAGE_MB: u32 = 1024;

/// Maximum storage size in MB for Android devices
pub const MAX_STORAGE_MB: u32 = 65536;

/// Upper limit for storage validation testing
pub const STORAGE_UPPER_LIMIT_TEST: u32 = 16384;

/// Maximum device name length in characters
pub const MAX_DEVICE_NAME_LENGTH: usize = 50;

/// Maximum device type display name length for UI
pub const MAX_DEVICE_TYPE_DISPLAY_LENGTH: usize = 25;

/// Truncated device type name length
pub const TRUNCATED_DEVICE_TYPE_LENGTH: usize = 22;

/// Maximum log entries to keep in memory
pub const MAX_LOG_ENTRIES: usize = 1000;

/// Maximum notification queue size
pub const MAX_NOTIFICATIONS: usize = 10;

/// Percentage calculation multiplier
pub const PERCENTAGE_MULTIPLIER: f64 = 100.0;

/// Maximum device name length for creation (shorter than display)
pub const MAX_DEVICE_NAME_CREATE_LENGTH: usize = 20;

/// Maximum error message display length
pub const MAX_ERROR_MESSAGE_LENGTH: usize = 60;

/// Maximum version number value
pub const MAX_VERSION_NUMBER: u32 = 50;

/// Maximum device name parts to take (for display)
pub const MAX_DEVICE_NAME_PARTS_DISPLAY: usize = 4;

/// Maximum device name parts to take (for processing)
pub const MAX_DEVICE_NAME_PARTS_PROCESS: usize = 3;

/// Validation limits for device configuration
/// Device name character limit for validation
pub const DEVICE_NAME_VALIDATION_LIMIT: usize = 50;

/// Memory size validation limits (in MB)
pub const MEMORY_VALIDATION_MIN_MB: u32 = 512;
pub const MEMORY_VALIDATION_BASE_MB: u32 = 1024;
pub const MEMORY_VALIDATION_HIGH_MB: u32 = 8192;
pub const MEMORY_VALIDATION_MAX_MB: u32 = 65536;

/// Storage conversion factor (MB to GB)
pub const STORAGE_MB_TO_GB_DIVISOR: u32 = 1024;

// Word count limits for name generation
pub const MAX_WORDS_IN_DEVICE_NAME: usize = 3;
pub const MAX_WORDS_IN_API_DISPLAY: usize = 2;

// String matching minimum length
pub const MIN_STRING_LENGTH_FOR_MATCH: usize = 3;

// Additional validation limits
pub const MIN_DEVICE_NAME_LENGTH: usize = 1;
pub const MIN_STORAGE_GB: u32 = 1;
pub const MAX_STORAGE_GB: u32 = 64;

// Word count requirements
pub const MIN_WORDS_FOR_DEVICE_NAME: usize = 1;

// Array index constants
pub const FIRST_WORD_INDEX: usize = 0;
pub const SECOND_WORD_INDEX: usize = 1;
pub const LAST_MATCH_INDEX: usize = 0;
