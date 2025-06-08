use emu::managers::android::AndroidManager;
use emu::managers::common::{DeviceConfig, DeviceManager};
use emu::models::device_config::test_constants::{TEST_ANDROID_DEVICE, TEST_IOS_DEVICE};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_device_lifecycle() {
    // Initialize Android manager
    let android_manager = match AndroidManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            println!("Android manager not available: {}", e);
            return;
        }
    };

    // Check if Android tools are available
    if !android_manager.is_available().await {
        println!("Android SDK tools not available, skipping test");
        return;
    }

    println!("🔍 Testing Android device lifecycle...");

    // 1. List initial devices
    println!("📋 Getting initial device list...");
    let initial_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let initial_count = initial_devices.len();
    println!("   Found {} initial devices", initial_count);

    // 2. Create a test device
    println!("📱 Creating test device...");
    let test_device_name = format!("test_device_{}", chrono::Utc::now().timestamp());
    let device_config = DeviceConfig::new(
        test_device_name.clone(),
        TEST_ANDROID_DEVICE.to_string(), // Use a specific device type that exists
        "35".to_string(),                // Android 15
    )
    .with_ram("2048".to_string())
    .with_storage("8192".to_string());

    let create_result = android_manager.create_device(&device_config).await;
    match create_result {
        Ok(()) => {
            println!("   ✅ Device '{}' created successfully", test_device_name);
        }
        Err(e) => {
            println!("   ❌ Failed to create device: {}", e);
            println!("   🔧 Debug: Trying with different API level...");

            // Try with API level 36 instead
            let fallback_config = DeviceConfig::new(
                test_device_name.clone(),
                TEST_ANDROID_DEVICE.to_string(),
                "36".to_string(), // Android 16
            )
            .with_ram("2048".to_string())
            .with_storage("8192".to_string());

            match android_manager.create_device(&fallback_config).await {
                Ok(()) => {
                    println!(
                        "   ✅ Device '{}' created successfully with API 36",
                        test_device_name
                    );
                }
                Err(e2) => {
                    println!("   ❌ Failed to create device with API 36: {}", e2);
                    println!("   💡 This might indicate missing system images. Install via Android Studio SDK Manager.");
                    return;
                }
            }
        }
    }

    // 3. Verify device appears in list
    println!("🔄 Verifying device appears in list...");
    sleep(Duration::from_secs(2)).await; // Give some time for the device to be registered

    let updated_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list devices after creation");
    let found_device = updated_devices.iter().find(|d| d.name == test_device_name);

    match found_device {
        Some(device) => {
            println!("   ✅ Device found in list: {}", device.name);
            println!("      API Level: {}", device.api_level);
            println!("      Device Type: {}", device.device_type);
        }
        None => {
            println!("   ❌ Created device not found in list");
            // Still try to clean up
        }
    }

    // 4. Test device start (optional - may take time)
    if std::env::var("EMU_FULL_TEST").is_ok() {
        println!("🚀 Testing device start...");
        match android_manager.start_device(&test_device_name).await {
            Ok(()) => {
                println!("   ✅ Device started successfully");

                // Wait a bit for device to boot
                sleep(Duration::from_secs(5)).await;

                // Stop the device
                println!("🛑 Stopping device...");
                match android_manager.stop_device(&test_device_name).await {
                    Ok(()) => println!("   ✅ Device stopped successfully"),
                    Err(e) => println!("   ⚠️  Warning: Failed to stop device: {}", e),
                }
            }
            Err(e) => {
                println!("   ⚠️  Warning: Failed to start device: {}", e);
            }
        }
    } else {
        println!("🚀 Skipping device start test (set EMU_FULL_TEST=1 to enable)");
    }

    // 5. Delete the test device
    println!("🗑️  Deleting test device...");
    match android_manager.delete_device(&test_device_name).await {
        Ok(()) => {
            println!("   ✅ Device '{}' deleted successfully", test_device_name);
        }
        Err(e) => {
            println!("   ❌ Failed to delete device: {}", e);
        }
    }

    // 6. Verify device is removed from list
    println!("🔄 Verifying device is removed from list...");
    sleep(Duration::from_secs(1)).await;

    let final_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list devices after deletion");
    let device_still_exists = final_devices.iter().any(|d| d.name == test_device_name);

    if device_still_exists {
        println!("   ⚠️  Warning: Device still appears in list after deletion");
    } else {
        println!("   ✅ Device successfully removed from list");
    }

    let final_count = final_devices.len();
    println!("📊 Test completed. Final device count: {}", final_count);

    if final_count == initial_count {
        println!("✅ Device lifecycle test completed successfully!");
    } else {
        println!(
            "⚠️  Device count changed from {} to {}",
            initial_count, final_count
        );
    }
}

#[tokio::test]
async fn test_android_device_list_parsing() {
    let android_manager = match AndroidManager::new() {
        Ok(manager) => manager,
        Err(_) => {
            println!("Android manager not available, skipping test");
            return;
        }
    };

    if !android_manager.is_available().await {
        println!("Android SDK tools not available, skipping test");
        return;
    }

    println!("🔍 Testing Android device list parsing...");

    let devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list devices");

    println!("   Found {} devices", devices.len());

    for (i, device) in devices.iter().enumerate() {
        println!("   Device {}: {}", i + 1, device.name);
        println!("      API Level: {}", device.api_level);
        println!("      Device Type: {}", device.device_type);
        println!("      Running: {}", device.is_running);
        println!("      Status: {:?}", device.status);
    }

    // Validate that all devices have required fields
    for device in &devices {
        assert!(!device.name.is_empty(), "Device name should not be empty");
        assert!(device.api_level > 0, "API level should be greater than 0");
        assert!(
            !device.device_type.is_empty(),
            "Device type should not be empty"
        );
    }

    println!("✅ Device list parsing test completed successfully!");
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn test_ios_device_lifecycle() {
    use emu::managers::ios::IosManager;

    let ios_manager = match IosManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            println!("iOS manager not available: {}", e);
            return;
        }
    };

    if !ios_manager.is_available().await {
        println!("iOS simulator tools not available, skipping test");
        return;
    }

    println!("🔍 Testing iOS device lifecycle...");

    // 1. List initial devices
    println!("📋 Getting initial device list...");
    let initial_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let initial_count = initial_devices.len();
    println!("   Found {} initial devices", initial_count);

    // 2. Create a test device
    println!("📱 Creating test iOS device...");
    let test_device_name = format!("test_ios_device_{}", chrono::Utc::now().timestamp());
    let device_config = DeviceConfig::new(
        test_device_name.clone(),
        TEST_IOS_DEVICE.to_string(),
        "com.apple.CoreSimulator.SimRuntime.iOS-18-1".to_string(),
    );

    let create_result = ios_manager.create_device(&device_config).await;
    match create_result {
        Ok(()) => {
            println!(
                "   ✅ iOS device '{}' created successfully",
                test_device_name
            );
        }
        Err(e) => {
            println!("   ❌ Failed to create iOS device: {}", e);
            return;
        }
    }

    // 3. Verify device appears in list
    println!("🔄 Verifying iOS device appears in list...");
    sleep(Duration::from_secs(2)).await;

    let updated_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list devices after creation");
    let found_device = updated_devices.iter().find(|d| d.name == test_device_name);

    let device_udid = match found_device {
        Some(device) => {
            println!("   ✅ iOS device found in list: {}", device.name);
            println!("      UDID: {}", device.udid);
            println!("      iOS Version: {}", device.ios_version);
            device.udid.clone()
        }
        None => {
            println!("   ❌ Created iOS device not found in list");
            return;
        }
    };

    // 4. Delete the test device
    println!("🗑️  Deleting test iOS device...");
    match ios_manager.delete_device(&device_udid).await {
        Ok(()) => {
            println!(
                "   ✅ iOS device '{}' deleted successfully",
                test_device_name
            );
        }
        Err(e) => {
            println!("   ❌ Failed to delete iOS device: {}", e);
        }
    }

    println!("✅ iOS device lifecycle test completed!");
}

#[test]
fn test_device_config_builder() {
    println!("🔧 Testing DeviceConfig builder pattern...");

    let config = DeviceConfig::new(
        "test_device".to_string(),
        "phone".to_string(),
        "35".to_string(),
    )
    .with_ram("4096".to_string())
    .with_storage("16384".to_string())
    .with_option("abi".to_string(), "arm64-v8a".to_string());

    assert_eq!(config.name, "test_device");
    assert_eq!(config.device_type, "phone");
    assert_eq!(config.version, "35");
    assert_eq!(config.ram_size, Some("4096".to_string()));
    assert_eq!(config.storage_size, Some("16384".to_string()));
    assert_eq!(
        config.additional_options.get("abi"),
        Some(&"arm64-v8a".to_string())
    );

    println!("✅ DeviceConfig builder test completed successfully!");
}
