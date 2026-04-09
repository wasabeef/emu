//! JSON fixture loader using runtime fs::read_to_string.
//!
//! Consolidated from tests/fixtures/fixture_loader.rs.
//! Uses runtime path resolution so tests work from both src/ and tests/.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Fixture loader with an in-memory cache.
pub struct FixtureLoader {
    cache: HashMap<String, Value>,
    base_path: PathBuf,
}

impl FixtureLoader {
    /// Create a loader rooted at `tests/fixtures/`.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            base_path: PathBuf::from("tests/fixtures"),
        }
    }

    /// Load a JSON fixture file, returning the parsed value (cached).
    pub fn load(&mut self, filename: &str) -> Result<&Value> {
        if !self.cache.contains_key(filename) {
            let path = self.base_path.join(filename);
            let content = std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("Failed to read fixture {}: {e}", path.display()))?;
            let json: Value = serde_json::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse fixture {filename}: {e}"))?;
            self.cache.insert(filename.to_string(), json);
        }
        Ok(self.cache.get(filename).unwrap())
    }

    /// Navigate a dot-separated JSON path within a fixture.
    pub fn get(&mut self, filename: &str, path: &[&str]) -> Result<Option<Value>> {
        let root = self.load(filename)?.clone();
        Ok(navigate(&root, path).cloned())
    }

    /// Get a string value from a fixture.
    pub fn get_string(&mut self, filename: &str, path: &[&str]) -> Result<Option<String>> {
        Ok(self.get(filename, path)?.and_then(|v| v.as_str().map(String::from)))
    }
}

impl Default for FixtureLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Load a plain-text fixture file from `tests/fixtures/`.
pub fn load_text_fixture(relative_path: &str) -> Result<String> {
    let path = Path::new("tests/fixtures").join(relative_path);
    std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read fixture {}: {e}", path.display()))
}

fn navigate<'a>(mut current: &'a Value, path: &[&str]) -> Option<&'a Value> {
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}
