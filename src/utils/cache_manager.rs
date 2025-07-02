//! Cache management utilities for persistent device storage.
//!
//! This module provides functionality to save and load device information
//! to/from disk, enabling faster startup times.

use crate::models::cache::{CacheConfig, DeviceCache};
use crate::models::device::{AndroidDevice, IosDevice};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Manages persistent caching of device information.
pub struct CacheManager {
    config: CacheConfig,
}

impl CacheManager {
    /// Creates a new cache manager with default configuration.
    pub fn new() -> Self {
        Self {
            config: CacheConfig::default(),
        }
    }

    /// Creates a new cache manager with custom configuration.
    pub fn with_config(config: CacheConfig) -> Self {
        Self { config }
    }

    /// Loads cached device information from disk.
    ///
    /// Returns None if:
    /// - Cache is disabled
    /// - Cache file doesn't exist
    /// - Cache is invalid or corrupted
    /// - Cache is older than max_age_secs
    pub async fn load_cache(&self) -> Option<DeviceCache> {
        if !self.config.enabled {
            return None;
        }

        if !self.config.cache_path.exists() {
            log::debug!("Cache file does not exist: {:?}", self.config.cache_path);
            return None;
        }

        match fs::read_to_string(&self.config.cache_path) {
            Ok(contents) => match serde_json::from_str::<DeviceCache>(&contents) {
                Ok(cache) => {
                    if cache.version != DeviceCache::CURRENT_VERSION {
                        log::warn!(
                            "Cache version mismatch: expected {}, got {}",
                            DeviceCache::CURRENT_VERSION,
                            cache.version
                        );
                        return None;
                    }

                    if !cache.is_valid(self.config.max_age_secs) {
                        log::debug!("Cache is expired");
                        return None;
                    }

                    log::info!(
                        "Loaded cache with {} Android and {} iOS devices",
                        cache.android_devices.len(),
                        cache.ios_devices.len()
                    );
                    Some(cache)
                }
                Err(e) => {
                    log::warn!("Failed to deserialize cache: {}", e);
                    None
                }
            },
            Err(e) => {
                log::warn!("Failed to read cache file: {}", e);
                None
            }
        }
    }

    /// Saves device information to disk cache.
    ///
    /// Creates the cache directory if it doesn't exist.
    pub async fn save_cache(
        &self,
        android_devices: Vec<AndroidDevice>,
        ios_devices: Vec<IosDevice>,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Create cache directory if it doesn't exist
        if let Some(parent) = self.config.cache_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create cache directory: {:?}", parent))?;
        }

        let cache = DeviceCache::new(android_devices, ios_devices);
        let json = serde_json::to_string_pretty(&cache).context("Failed to serialize cache")?;

        fs::write(&self.config.cache_path, json)
            .with_context(|| format!("Failed to write cache file: {:?}", self.config.cache_path))?;

        log::debug!("Cache saved to {:?}", self.config.cache_path);
        Ok(())
    }

    /// Updates an existing cache with new device information.
    ///
    /// Loads the existing cache, updates it, and saves it back to disk.
    pub async fn update_cache(
        &self,
        android_devices: Vec<AndroidDevice>,
        ios_devices: Vec<IosDevice>,
    ) -> Result<()> {
        self.save_cache(android_devices, ios_devices).await
    }

    /// Clears the cache by deleting the cache file.
    pub fn clear_cache(&self) -> Result<()> {
        if self.config.cache_path.exists() {
            fs::remove_file(&self.config.cache_path).with_context(|| {
                format!("Failed to delete cache file: {:?}", self.config.cache_path)
            })?;
            log::info!("Cache cleared");
        }
        Ok(())
    }

    /// Gets the cache file path.
    pub fn cache_path(&self) -> &Path {
        &self.config.cache_path
    }

    /// Checks if cache exists on disk.
    pub fn cache_exists(&self) -> bool {
        self.config.cache_path.exists()
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::device::DeviceStatus;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cache_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.json");

        let config = CacheConfig {
            max_age_secs: 300,
            enabled: true,
            cache_path: cache_path.clone(),
        };

        let manager = CacheManager::with_config(config);

        // Create test devices
        let android_device = AndroidDevice {
            name: "test_avd".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        };

        let ios_device = IosDevice {
            name: "iPhone 15".to_string(),
            udid: "test-udid".to_string(),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        };

        // Save cache
        manager
            .save_cache(vec![android_device.clone()], vec![ios_device.clone()])
            .await
            .unwrap();

        // Load cache
        let loaded_cache = manager.load_cache().await.unwrap();
        assert_eq!(loaded_cache.android_devices.len(), 1);
        assert_eq!(loaded_cache.ios_devices.len(), 1);
        assert_eq!(loaded_cache.android_devices[0].name, "test_avd");
        assert_eq!(loaded_cache.ios_devices[0].name, "iPhone 15");
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("test_cache.json");

        let config = CacheConfig {
            max_age_secs: 0, // Expire immediately
            enabled: true,
            cache_path,
        };

        let manager = CacheManager::with_config(config);

        // Save cache
        manager.save_cache(vec![], vec![]).await.unwrap();

        // Try to load expired cache
        let loaded_cache = manager.load_cache().await;
        assert!(loaded_cache.is_none());
    }
}
