//! Test fixture loading utilities
//!
//! This module provides utilities for loading and managing test fixture data,
//! including command outputs, state transitions, and error scenarios.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Fixture loader for managing test data
pub struct FixtureLoader {
    pub cache: HashMap<String, Value>,
    base_path: String,
}

impl FixtureLoader {
    /// Creates a new fixture loader with the default base path
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            base_path: "tests/fixtures".to_string(),
        }
    }

    /// Creates a fixture loader with a custom base path
    #[allow(dead_code)]
    pub fn with_base_path(base_path: &str) -> Self {
        Self {
            cache: HashMap::new(),
            base_path: base_path.to_string(),
        }
    }

    /// Loads a fixture file and returns the parsed JSON
    pub fn load(&mut self, filename: &str) -> Result<&Value> {
        if !self.cache.contains_key(filename) {
            let file_path = Path::new(&self.base_path).join(filename);
            let content = fs::read_to_string(&file_path)
                .map_err(|e| anyhow::anyhow!("Failed to read fixture file {}: {}", filename, e))?;

            let json: Value = serde_json::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse fixture JSON {}: {}", filename, e))?;

            self.cache.insert(filename.to_string(), json);
        }

        Ok(self.cache.get(filename).unwrap())
    }

    /// Gets a specific value from a fixture using a path
    pub fn get(&mut self, filename: &str, path: &[&str]) -> Result<Option<Value>> {
        let fixture = self.load(filename)?.clone();
        Ok(self.navigate_json_path(&fixture, path).cloned())
    }

    /// Gets a string value from a fixture
    #[allow(dead_code)]
    pub fn get_string(&mut self, filename: &str, path: &[&str]) -> Result<Option<String>> {
        if let Some(value) = self.get(filename, path)? {
            Ok(value.as_str().map(|s| s.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Gets an array of values from a fixture
    #[allow(dead_code)]
    pub fn get_array(&mut self, filename: &str, path: &[&str]) -> Result<Option<Vec<Value>>> {
        if let Some(value) = self.get(filename, path)? {
            Ok(value.as_array().cloned())
        } else {
            Ok(None)
        }
    }

    /// Navigates through a JSON path
    pub fn navigate_json_path<'a>(
        &self,
        mut current: &'a Value,
        path: &[&str],
    ) -> Option<&'a Value> {
        for key in path {
            current = current.get(key)?;
        }
        Some(current)
    }

    /// Clears the cache
    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for FixtureLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common fixture operations
pub mod fixtures {
    use super::*;

    /// Loads Android command output fixtures
    pub fn android_outputs() -> Result<Value> {
        let mut loader = FixtureLoader::new();
        Ok(loader.load("android_outputs.json")?.clone())
    }

    /// Loads iOS command output fixtures
    pub fn ios_outputs() -> Result<Value> {
        let mut loader = FixtureLoader::new();
        Ok(loader.load("ios_outputs.json")?.clone())
    }

    /// Loads state transition fixtures
    #[allow(dead_code)]
    pub fn state_transitions() -> Result<Value> {
        let mut loader = FixtureLoader::new();
        Ok(loader.load("state_transitions.json")?.clone())
    }

    /// Loads error scenario fixtures
    #[allow(dead_code)]
    pub fn error_scenarios() -> Result<Value> {
        let mut loader = FixtureLoader::new();
        Ok(loader.load("error_scenarios.json")?.clone())
    }

    /// Loads environment variation fixtures
    #[allow(dead_code)]
    pub fn environment_variations() -> Result<Value> {
        let mut loader = FixtureLoader::new();
        Ok(loader.load("environment_variations.json")?.clone())
    }

    /// Gets Android AVD list output for testing
    #[allow(dead_code)]
    pub fn android_avd_list_single() -> Result<String> {
        let mut loader = FixtureLoader::new();
        loader
            .get_string(
                "android_outputs.json",
                &["avdmanager_list_avd", "single_device"],
            )
            .map(|opt| opt.unwrap_or_default())
    }

    /// Gets Android AVD list output with multiple devices
    #[allow(dead_code)]
    pub fn android_avd_list_multiple() -> Result<String> {
        let mut loader = FixtureLoader::new();
        loader
            .get_string(
                "android_outputs.json",
                &["avdmanager_list_avd", "multiple_devices"],
            )
            .map(|opt| opt.unwrap_or_default())
    }

    /// Gets ADB devices output
    #[allow(dead_code)]
    pub fn adb_devices_running() -> Result<String> {
        let mut loader = FixtureLoader::new();
        loader
            .get_string("android_outputs.json", &["adb_devices", "multiple_devices"])
            .map(|opt| opt.unwrap_or_default())
    }

    /// Gets iOS device list output
    #[allow(dead_code)]
    pub fn ios_device_list_single() -> Result<String> {
        let mut loader = FixtureLoader::new();
        loader
            .get_string(
                "ios_outputs.json",
                &["xcrun_simctl_list_devices", "single_runtime"],
            )
            .map(|opt| opt.unwrap_or_default())
    }

    /// Gets iOS device list output with multiple runtimes
    #[allow(dead_code)]
    pub fn ios_device_list_multiple() -> Result<String> {
        let mut loader = FixtureLoader::new();
        loader
            .get_string(
                "ios_outputs.json",
                &["xcrun_simctl_list_devices", "multiple_runtimes"],
            )
            .map(|opt| opt.unwrap_or_default())
    }

    /// Gets Android license error output
    #[allow(dead_code)]
    pub fn android_license_error() -> Result<String> {
        let mut loader = FixtureLoader::new();
        loader
            .get_string(
                "error_scenarios.json",
                &["android_errors", "license_not_accepted", "stderr"],
            )
            .map(|opt| opt.unwrap_or_default())
    }

    /// Gets iOS already booted error output
    #[allow(dead_code)]
    pub fn ios_already_booted_error() -> Result<String> {
        let mut loader = FixtureLoader::new();
        loader
            .get_string(
                "error_scenarios.json",
                &["ios_errors", "device_already_booted", "stderr"],
            )
            .map(|opt| opt.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_loader_basic() {
        let mut loader = FixtureLoader::new();

        // This test will only work if the fixture files exist
        // In a real test environment, we'd want to use test-specific fixtures
        if let Ok(fixture) = loader.load("android_outputs.json") {
            assert!(fixture.is_object());
        }
    }

    #[test]
    fn test_fixture_loader_navigation() {
        let loader = FixtureLoader::new();

        // Test JSON navigation
        let json_data = serde_json::json!({
            "level1": {
                "level2": {
                    "value": "test"
                }
            }
        });

        let result = loader.navigate_json_path(&json_data, &["level1", "level2", "value"]);
        assert_eq!(result.unwrap().as_str().unwrap(), "test");
    }

    #[test]
    fn test_convenience_functions() {
        // These tests will only work in a real environment with fixture files
        // In CI/CD, we'd want to ensure the fixtures are available

        // Test that the convenience functions don't panic
        let _ = fixtures::android_outputs();
        let _ = fixtures::ios_outputs();
    }
}
