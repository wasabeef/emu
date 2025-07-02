//! Cache models for persistent storage of device information.
//!
//! This module provides structures for caching device lists to disk,
//! enabling faster startup times by avoiding redundant device queries.

use super::device::{AndroidDevice, IosDevice};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Persistent cache for device lists.
///
/// Stores device information with timestamps to enable cache invalidation
/// and background refresh strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCache {
    /// Timestamp when the cache was last updated
    pub last_updated: SystemTime,
    /// Cached Android devices
    pub android_devices: Vec<AndroidDevice>,
    /// Cached iOS devices
    pub ios_devices: Vec<IosDevice>,
    /// Version of the cache format for future compatibility
    pub version: u32,
}

impl DeviceCache {
    /// Current cache format version
    pub const CURRENT_VERSION: u32 = 1;

    /// Creates a new device cache with current timestamp
    pub fn new(android_devices: Vec<AndroidDevice>, ios_devices: Vec<IosDevice>) -> Self {
        Self {
            last_updated: SystemTime::now(),
            android_devices,
            ios_devices,
            version: Self::CURRENT_VERSION,
        }
    }

    /// Checks if the cache is still valid based on age
    pub fn is_valid(&self, max_age_secs: u64) -> bool {
        if let Ok(elapsed) = self.last_updated.elapsed() {
            elapsed.as_secs() < max_age_secs
        } else {
            false
        }
    }

    /// Updates the cache with new device lists
    pub fn update(&mut self, android_devices: Vec<AndroidDevice>, ios_devices: Vec<IosDevice>) {
        self.last_updated = SystemTime::now();
        self.android_devices = android_devices;
        self.ios_devices = ios_devices;
    }
}

/// Configuration for cache behavior
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum age in seconds before cache is considered stale
    pub max_age_secs: u64,
    /// Whether to use cache on startup
    pub enabled: bool,
    /// Cache file path
    pub cache_path: std::path::PathBuf,
}

impl Default for CacheConfig {
    fn default() -> Self {
        let cache_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("emu")
            .join("cache");

        Self {
            max_age_secs: 300, // 5 minutes
            enabled: true,
            cache_path: cache_dir.join("devices.json"),
        }
    }
}
