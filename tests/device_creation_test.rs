use emu::managers::android::AndroidManager;
use emu::managers::common::DeviceManager;
use emu::models::device_info::DynamicDeviceProvider;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_device_creation_flow() {
    // Test the device creation flow that was freezing
    println!("🔍 Testing device creation flow...");

    let android_manager = match AndroidManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            println!("Android manager not available: {e}");
            return;
        }
    };

    if !android_manager.is_available().await {
        println!("Android SDK tools not available, skipping test");
        return;
    }

    // Test list available devices (this was causing the freeze)
    println!("📋 Testing list_available_devices...");
    let list_result = timeout(
        Duration::from_secs(30),
        android_manager.list_available_devices(),
    )
    .await;

    match list_result {
        Ok(Ok(devices)) => {
            let device_count = devices.len();
            println!("   ✅ Successfully listed {device_count} devices");
            for (i, (id, name)) in devices.iter().enumerate().take(3) {
                let device_num = i + 1;
                println!("      Device {device_num}: {name} ({id})");
            }
        }
        Ok(Err(e)) => {
            println!("   ❌ Failed to list devices: {e}");
        }
        Err(_) => {
            println!("   ❌ TIMEOUT: list_available_devices took too long");
            panic!("Device listing timeout - this indicates the freeze issue");
        }
    }

    // Test list available targets
    println!("🎯 Testing list_available_targets...");
    let targets_result = timeout(
        Duration::from_secs(30),
        android_manager.list_available_targets(),
    )
    .await;

    match targets_result {
        Ok(Ok(targets)) => {
            let target_count = targets.len();
            println!("   ✅ Successfully listed {target_count} API targets");
            for (api, display) in targets.iter().take(3) {
                println!("      API {api}: {display}");
            }
        }
        Ok(Err(e)) => {
            println!("   ❌ Failed to list targets: {e}");
        }
        Err(_) => {
            println!("   ❌ TIMEOUT: list_available_targets took too long");
        }
    }

    // Test basic device info operations
    println!("📱 Testing get_available_devices...");
    let info_result = timeout(
        Duration::from_secs(30),
        android_manager.get_available_devices(),
    )
    .await;

    match info_result {
        Ok(Ok(device_infos)) => {
            let info_count = device_infos.len();
            println!("   ✅ Successfully got {info_count} device infos");
            for info in device_infos.iter().take(3) {
                println!(
                    "      Device: {} (OEM: {:?}, Category: {:?})",
                    info.display_name, info.oem, info.category
                );
            }
        }
        Ok(Err(e)) => {
            println!("   ❌ Failed to get device infos: {e}");
        }
        Err(_) => {
            println!("   ❌ TIMEOUT: get_available_devices took too long");
        }
    }

    println!("✅ Device creation flow test completed!");
}

#[tokio::test]
async fn test_android_manager_basic_operations() {
    println!("🔧 Testing basic AndroidManager operations...");

    let android_manager = match AndroidManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            println!("Android manager not available: {e}");
            return;
        }
    };

    // Test availability check
    println!("🔍 Testing is_available...");
    let is_available = android_manager.is_available().await;
    println!("   Available: {is_available}");

    if !is_available {
        println!("Android SDK not available, skipping further tests");
        return;
    }

    // Test listing existing devices (should not freeze)
    println!("📋 Testing list_devices...");
    let list_result = timeout(Duration::from_secs(30), android_manager.list_devices()).await;

    match list_result {
        Ok(Ok(devices)) => {
            let avd_count = devices.len();
            println!("   ✅ Successfully listed {avd_count} existing AVDs");
            for device in devices.iter().take(2) {
                println!(
                    "      AVD: {} (API {}, Type: {})",
                    device.name, device.api_level, device.device_type
                );
            }
        }
        Ok(Err(e)) => {
            println!("   ❌ Failed to list existing AVDs: {e}");
        }
        Err(_) => {
            println!("   ❌ TIMEOUT: list_devices took too long");
            panic!("AVD listing timeout - this indicates a serious issue");
        }
    }

    println!("✅ Basic operations test completed!");
}
