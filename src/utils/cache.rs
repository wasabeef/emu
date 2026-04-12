//! Shared cache helpers that are used outside of application UI state.

use crate::models::ApiLevel;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persistent API level cache stored on disk for faster device creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLevelCache {
    pub api_levels: Vec<ApiLevel>,
    pub timestamp: std::time::SystemTime,
}

impl ApiLevelCache {
    /// Get the API cache file path in the user's config directory.
    fn cache_file_path() -> Result<PathBuf, anyhow::Error> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        let emu_config_dir = config_dir.join("emu");
        fs::create_dir_all(&emu_config_dir)?;
        Ok(emu_config_dir.join("api_level_cache.json"))
    }

    /// Load API level cache from disk if it exists and is valid.
    pub fn load_from_disk() -> Option<Self> {
        let cache_path = Self::cache_file_path().ok()?;
        if !cache_path.exists() {
            return None;
        }

        let cache_content = fs::read_to_string(cache_path).ok()?;
        let cache: Self = serde_json::from_str(&cache_content).ok()?;

        let cache_age = cache.timestamp.elapsed().ok()?;
        if cache_age.as_secs() < 86400 {
            Some(cache)
        } else {
            None
        }
    }

    /// Save API level cache to disk.
    pub fn save_to_disk(&self) -> Result<(), anyhow::Error> {
        let cache_path = Self::cache_file_path()?;
        let cache_json = serde_json::to_string_pretty(self)?;
        fs::write(cache_path, cache_json)?;
        Ok(())
    }
}
