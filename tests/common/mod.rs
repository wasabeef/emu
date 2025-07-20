//! Common test utilities for integration tests
//!
//! This module provides shared test helpers for creating mock applications,
//! terminals, and test scenarios without requiring any real emulators or simulators.

pub mod assertions;
pub mod helpers;

/// Helper function to create a mock Android SDK environment for testing
///
/// This function creates a temporary directory structure that mimics
/// an Android SDK installation, including:
/// - cmdline-tools/latest/bin/ with avdmanager and sdkmanager
/// - emulator/ with emulator binary
/// - platform-tools/ with adb binary
///
/// All executables are created as simple shell scripts and made executable
/// on Unix-like systems.
///
/// # Returns
///
/// A `TempDir` that should be kept alive for the duration of the test.
/// The caller should set `ANDROID_HOME` environment variable to the
/// temp directory path.
///
/// # Example
///
/// ```rust
/// use crate::common::setup_mock_android_sdk;
///
/// let _temp_dir = setup_mock_android_sdk();
/// std::env::set_var("ANDROID_HOME", _temp_dir.path());
/// ```
#[allow(dead_code)]
pub fn setup_mock_android_sdk() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();

    // Create necessary directory structure
    std::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin")).unwrap();
    std::fs::create_dir_all(sdk_path.join("emulator")).unwrap();
    std::fs::create_dir_all(sdk_path.join("platform-tools")).unwrap();

    // Create executable scripts that return minimal valid output
    // avdmanager: Return empty list when queried
    let avdmanager_script = r#"#!/bin/sh
case "$1" in
    "list")
        case "$2" in
            "avd")
                # Empty AVD list (no devices)
                exit 0
                ;;
            "device")
                # Return some basic device types
                echo "Available devices definitions:"
                echo "id: 0 or \"tv_1080p\""
                echo "    Name: Android TV (1080p)"
                echo "    OEM : Google"
                echo "    Tag : android-tv"
                echo "---------"
                echo "id: 1 or \"pixel_7\""
                echo "    Name: Pixel 7"
                echo "    OEM : Google" 
                echo "    Tag : google_apis"
                exit 0
                ;;
        esac
        ;;
    "create")
        # Simulate successful creation
        exit 0
        ;;
esac
exit 1
"#;
    std::fs::write(
        sdk_path.join("cmdline-tools/latest/bin/avdmanager"),
        avdmanager_script,
    )
    .unwrap();

    // sdkmanager: Return some system images with proper format
    let sdkmanager_script = r#"#!/bin/sh
# Check for --list with any additional flags
if echo "$@" | grep -q -- "--list"; then
    echo "Installed packages:"
    echo "  system-images;android-34;google_apis;arm64-v8a | 1 | Google APIs ARM 64 v8a System Image"
    echo "  system-images;android-33;google_apis;x86_64 | 1 | Google APIs Intel x86_64 System Image"
    echo ""
    echo "Available Packages:"
    echo "  system-images;android-32;google_apis;arm64-v8a | 1 | Google APIs ARM 64 v8a System Image"
    exit 0
fi
exit 1
"#;
    std::fs::write(
        sdk_path.join("cmdline-tools/latest/bin/sdkmanager"),
        sdkmanager_script,
    )
    .unwrap();

    // emulator: Basic emulator stub
    let emulator_script = r#"#!/bin/sh
# Emulator stub - just exit
exit 0
"#;
    std::fs::write(sdk_path.join("emulator/emulator"), emulator_script).unwrap();

    // adb: Return empty device list
    let adb_script = r#"#!/bin/sh
case "$1" in
    "devices")
        echo "List of devices attached"
        exit 0
        ;;
esac
exit 1
"#;
    std::fs::write(sdk_path.join("platform-tools/adb"), adb_script).unwrap();

    // Make files executable on Unix-like systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = 0o755;
        std::fs::set_permissions(
            sdk_path.join("cmdline-tools/latest/bin/avdmanager"),
            std::fs::Permissions::from_mode(mode),
        )
        .unwrap();
        std::fs::set_permissions(
            sdk_path.join("cmdline-tools/latest/bin/sdkmanager"),
            std::fs::Permissions::from_mode(mode),
        )
        .unwrap();
        std::fs::set_permissions(
            sdk_path.join("emulator/emulator"),
            std::fs::Permissions::from_mode(mode),
        )
        .unwrap();
        std::fs::set_permissions(
            sdk_path.join("platform-tools/adb"),
            std::fs::Permissions::from_mode(mode),
        )
        .unwrap();
    }

    temp_dir
}

#[cfg(feature = "test-utils")]
use emu::app::state::AppState;
#[cfg(feature = "test-utils")]
use std::sync::Arc;
#[cfg(feature = "test-utils")]
use tokio::sync::Mutex;

/// Create a test AppState with MockDeviceManager
#[cfg(feature = "test-utils")]
#[allow(dead_code)]
pub async fn setup_test_app(_android_count: usize, _ios_count: usize) -> Arc<Mutex<AppState>> {
    // For now, return a basic AppState until we implement MockDeviceManager integration
    let app_state = AppState::new();
    Arc::new(Mutex::new(app_state))
}

/// Create a test AppState with specific device configuration
#[cfg(feature = "test-utils")]
#[allow(dead_code)]
pub async fn setup_test_app_with_scenario() -> Arc<Mutex<AppState>> {
    // For now, return a basic AppState until we implement MockDeviceManager integration
    let app_state = AppState::new();
    Arc::new(Mutex::new(app_state))
}
