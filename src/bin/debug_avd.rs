//! Debug tool for Android AVD creation
//!
//! This binary helps debug AVD creation issues by providing detailed logging
//! and checking system configuration.

use anyhow::Result;
use emu::managers::{
    common::{DeviceConfig, DeviceManager},
    AndroidManager,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== EMU Android AVD Debug Tool ===\n");

    // Test AndroidManager initialization
    println!("1. Initializing Android Manager...");
    let android_manager = match AndroidManager::new() {
        Ok(manager) => {
            println!("✓ Android Manager initialized successfully");
            manager
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize Android Manager: {e}");
            return Err(e);
        }
    };

    // Test listing available system images
    println!("\n2. Checking available system images...");
    match android_manager.list_available_system_images().await {
        Ok(images) => {
            if images.is_empty() {
                println!("⚠ No system images found. You may need to install some system images.");
                println!("  Example: sdkmanager \"system-images;android-34;google_apis_playstore;arm64-v8a\"");
            } else {
                println!("✓ Found {} system images:", images.len());
                for image in &images {
                    println!("  - {image}");
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to list system images: {e}");
        }
    }

    // Test existing AVDs
    println!("\n3. Listing existing AVDs...");
    match android_manager.list_devices().await {
        Ok(devices) => {
            if devices.is_empty() {
                println!("ℹ No existing AVDs found");
            } else {
                println!("✓ Found {} existing AVDs:", devices.len());
                for device in &devices {
                    println!(
                        "  - {} (API {}) - Status: {:?}",
                        device.name, device.api_level, device.status
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to list AVDs: {e}");
        }
    }

    // Test creating a simple AVD
    println!("\n4. Testing AVD creation...");
    let test_config = DeviceConfig::new(
        "test_debug_device".to_string(),
        "phone".to_string(),
        "34".to_string(), // Android 14
    );

    println!("Attempting to create test AVD with config:");
    println!("  Name: {}", test_config.name);
    println!("  Type: {}", test_config.device_type);
    println!("  API Level: {}", test_config.version);

    match android_manager.create_device(&test_config).await {
        Ok(()) => {
            println!("✓ Test AVD created successfully!");

            // Clean up - delete the test device
            println!("\n5. Cleaning up test device...");
            match android_manager.delete_device(&test_config.name).await {
                Ok(()) => println!("✓ Test device cleaned up successfully"),
                Err(e) => eprintln!("⚠ Failed to clean up test device: {e}"),
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to create test AVD: {e}");

            // Try with a different API level
            println!("\n   Trying with API level 33 (Android 13)...");
            let test_config_33 = DeviceConfig::new(
                "test_debug_device_33".to_string(),
                "phone".to_string(),
                "33".to_string(),
            );

            match android_manager.create_device(&test_config_33).await {
                Ok(()) => {
                    println!("✓ Test AVD with API 33 created successfully!");

                    // Clean up
                    match android_manager.delete_device(&test_config_33.name).await {
                        Ok(()) => println!("✓ Test device cleaned up successfully"),
                        Err(e) => eprintln!("⚠ Failed to clean up test device: {e}"),
                    }
                }
                Err(e) => {
                    eprintln!("✗ Failed to create test AVD with API 33: {e}");
                }
            }
        }
    }

    println!("\n=== Debug Complete ===");
    println!("If you're still experiencing issues, check:");
    println!("1. Android SDK is properly installed");
    println!("2. ANDROID_HOME or ANDROID_SDK_ROOT environment variable is set");
    println!("3. System images are installed (use `sdkmanager --list` to check)");
    println!("4. Android SDK licenses are accepted (run `sdkmanager --licenses`)");

    Ok(())
}
