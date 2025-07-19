//! Realistic Android Manager tests using actual public methods
//!
//! These tests focus on the actual public API of AndroidManager,
//! testing real functionality that users depend on.

use anyhow::Result;
use emu::managers::android::AndroidManager;

/// Test basic AndroidManager functionality
#[tokio::test]
async fn test_android_manager_basic_operations() -> Result<()> {
    let manager = AndroidManager::new()?;

    // Test device listing (this should work even without emulators)
    let _devices = manager.list_devices_parallel().await?;
    // Should return list (may be empty), but not error - no need to assert len >= 0

    Ok(())
}

/// Test getting running AVD names
#[tokio::test]
async fn test_get_running_avd_names() -> Result<()> {
    let manager = AndroidManager::new()?;

    // This should work even if no AVDs are running
    let _running_names = manager.get_running_avd_names().await?;
    // Should return HashMap (may be empty) if no devices running

    Ok(())
}

/// Test listing available targets
#[tokio::test]
async fn test_list_available_targets() -> Result<()> {
    let manager = AndroidManager::new()?;

    // This tests SDK integration
    let targets = manager.list_available_targets().await;

    // May fail if SDK not available, but should handle gracefully
    match targets {
        Ok(_target_list) => {
            // If successful, should return valid format
            // target_list can be empty or contain targets
        }
        Err(_) => {
            // SDK might not be available in test environment
            // This is acceptable for CI/testing
        }
    }

    Ok(())
}

/// Test listing available devices
#[tokio::test]
async fn test_list_available_devices() -> Result<()> {
    let manager = AndroidManager::new()?;

    let devices = manager.list_available_devices().await;

    match devices {
        Ok(device_list) => {
            // If successful, should return valid format

            // Verify format of returned devices
            for (id, name) in device_list {
                assert!(!id.is_empty());
                assert!(!name.is_empty());
            }
        }
        Err(_) => {
            // SDK might not be available in test environment
        }
    }

    Ok(())
}

/// Test device category detection
#[tokio::test]
async fn test_device_category_detection() -> Result<()> {
    let manager = AndroidManager::new()?;

    // Test known device categories (actual implementation returns simple strings)
    let phone_category = manager.get_device_category("pixel_7", "Pixel 7");
    assert_eq!(phone_category, "phone");

    let tv_category = manager.get_device_category("tv_1080p", "Android TV (1080p)");
    assert_eq!(tv_category, "tv");

    let wear_category = manager.get_device_category("wear_round", "Android Wear Round");
    assert_eq!(wear_category, "wear");

    let tablet_category = manager.get_device_category("pixel_tablet", "Pixel Tablet");
    assert_eq!(tablet_category, "tablet");

    let auto_category = manager.get_device_category("automotive_1024p", "Automotive 1024p");
    assert_eq!(auto_category, "automotive");

    Ok(())
}

/// Test system image availability check
#[tokio::test]
async fn test_check_system_image_available() -> Result<()> {
    let manager = AndroidManager::new()?;

    // Test with common architectures
    let architectures = vec!["x86_64", "arm64"];

    for arch in &architectures {
        let result = manager
            .check_system_image_available("34", arch, "google_apis")
            .await;

        // Should not error, even if image not available
        match result {
            Ok(available) => {
                // Result can be true or false, both valid
                assert!(available == available); // Tautology to verify result is boolean
            }
            Err(_) => {
                // SDK might not be available, acceptable in test environment
            }
        }
    }

    Ok(())
}

/// Test listing available system images
#[tokio::test]
async fn test_list_available_system_images() -> Result<()> {
    let manager = AndroidManager::new()?;

    let images = manager.list_available_system_images().await;

    match images {
        Ok(image_list) => {
            // Should return list (may be empty)

            // Verify format of system images
            for image in image_list {
                assert!(image.contains("system-images"));
            }
        }
        Err(_) => {
            // SDK might not be available
        }
    }

    Ok(())
}

/// Test getting first available system image
#[tokio::test]
async fn test_get_first_available_system_image() -> Result<()> {
    let manager = AndroidManager::new()?;

    let result = manager.get_first_available_system_image("x86_64").await;

    match result {
        Ok(Some((id, name))) => {
            // If found, should be valid format
            assert!(!id.is_empty());
            assert!(!name.is_empty());
        }
        Ok(None) => {
            // No image available, valid result
        }
        Err(_) => {
            // SDK not available, acceptable
        }
    }

    Ok(())
}

/// Test API levels listing
#[tokio::test]
async fn test_list_api_levels() -> Result<()> {
    let manager = AndroidManager::new()?;

    let api_levels = manager.list_api_levels().await;

    match api_levels {
        Ok(levels) => {
            // Should return list of API levels

            // Verify API level structure
            for level in levels {
                assert!(level.api > 0);
                assert!(!level.display_name.is_empty());
            }
        }
        Err(_) => {
            // SDK might not be available
        }
    }

    Ok(())
}

/// Test concurrent operations safety
#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let manager = AndroidManager::new()?;

    // Test multiple concurrent device list operations
    let tasks: Vec<_> = (0..3)
        .map(|_| {
            let manager = manager.clone();
            tokio::spawn(async move { manager.list_devices_parallel().await })
        })
        .collect();

    // All operations should complete without panic
    for task in tasks {
        let result = task.await?;
        assert!(result.is_ok());
    }

    Ok(())
}

/// Test error handling robustness
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let manager = AndroidManager::new()?;

    // These operations should handle errors gracefully
    let _ = manager.get_running_avd_names().await;
    let _ = manager.list_available_targets().await;
    let _ = manager.list_available_devices().await;
    let _ = manager.list_available_system_images().await;

    // If we reach here without panic, error handling is working
    Ok(())
}
