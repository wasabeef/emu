//! Test helpers for creating App instances with mock dependencies.
//!
//! This module provides utilities for creating App instances that don't
//! require actual emulator environments, enabling true unit testing.

#[cfg(test)]
use crate::{
    app::{App, AppState},
    managers::{AndroidManager, IosManager},
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(test)]
impl App {
    /// Creates a new App instance configured for testing.
    ///
    /// This method creates an App instance suitable for testing environments.
    /// It assumes that test setup (like `setup_mock_android_sdk()`) has been
    /// done to provide mock SDK tools.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use crate::common::setup_mock_android_sdk;
    /// 
    /// #[tokio::test]
    /// async fn test_app_functionality() {
    ///     let _temp_dir = setup_mock_android_sdk();
    ///     std::env::set_var("ANDROID_HOME", _temp_dir.path());
    ///     
    ///     let app = App::new_for_testing().await.expect("Failed to create test app");
    ///     // Test app functionality without needing emulators
    /// }
    /// ```
    pub async fn new_for_testing() -> anyhow::Result<Self> {
        // Use the regular App::new() which will work with our mock SDK tools
        // The mock tools are set up to return appropriate responses for testing
        App::new().await
    }

    // Note: Due to the App struct using concrete types (AndroidManager, IosManager)
    // instead of trait objects, we cannot directly inject MockDeviceManager.
    // Tests will need to use the real managers but in a controlled environment,
    // or we need to refactor App to use trait objects.
}

// TestScenarioBuilder removed as it relies on MockDeviceManager injection
// which is not possible with the current App architecture.

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_new_for_testing() {
        // Set up mock Android SDK for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let sdk_path = temp_dir.path();
        
        // Create minimal directory structure
        std::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin")).unwrap();
        std::fs::create_dir_all(sdk_path.join("emulator")).unwrap();
        std::fs::create_dir_all(sdk_path.join("platform-tools")).unwrap();
        
        // Create mock executables
        let script = "#!/bin/sh\nexit 0\n";
        std::fs::write(sdk_path.join("cmdline-tools/latest/bin/avdmanager"), script).unwrap();
        std::fs::write(sdk_path.join("cmdline-tools/latest/bin/sdkmanager"), script).unwrap();
        std::fs::write(sdk_path.join("emulator/emulator"), script).unwrap();
        std::fs::write(sdk_path.join("platform-tools/adb"), script).unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = 0o755;
            std::fs::set_permissions(
                sdk_path.join("cmdline-tools/latest/bin/avdmanager"),
                std::fs::Permissions::from_mode(mode),
            ).unwrap();
            std::fs::set_permissions(
                sdk_path.join("emulator/emulator"),
                std::fs::Permissions::from_mode(mode),
            ).unwrap();
            std::fs::set_permissions(
                sdk_path.join("platform-tools/adb"),
                std::fs::Permissions::from_mode(mode),
            ).unwrap();
        }
        
        std::env::set_var("ANDROID_HOME", sdk_path);
        
        let app = App::new_for_testing().await.expect("Failed to create test app");
        
        // The app should be created successfully with the mock SDK
        let state = app.state.lock().await;
        assert!(!state.android_manager_name.is_empty());
    }

    // TestScenarioBuilder tests removed as it's not implemented yet
}