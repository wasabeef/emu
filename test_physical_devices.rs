use emu::managers::android::AndroidManager;
use emu::managers::ios::IosManager;
use emu::managers::common::DeviceManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Testing physical device detection...\n");

    // Test Android physical devices
    println!("=== Android Physical Devices ===");
    match AndroidManager::new() {
        Ok(android_manager) => {
            match android_manager.list_physical_devices().await {
                Ok(devices) => {
                    if devices.is_empty() {
                        println!("No Android physical devices found.");
                    } else {
                        for device in devices {
                            println!("Found Android device: {} (API {}) - Physical: {}", 
                                     device.name, device.api_level, device.is_physical);
                        }
                    }
                }
                Err(e) => println!("Error listing Android physical devices: {}", e),
            }
            
            // Also test full device list
            println!("\n=== All Android Devices (Virtual + Physical) ===");
            match android_manager.list_devices().await {
                Ok(devices) => {
                    for device in devices {
                        let device_type = if device.is_physical { "Physical" } else { "Virtual" };
                        println!("{}: {} (API {})", device_type, device.name, device.api_level);
                    }
                }
                Err(e) => println!("Error listing all Android devices: {}", e),
            }
        }
        Err(e) => println!("Failed to create Android manager: {}", e),
    }

    // Test iOS physical devices (macOS only)
    #[cfg(target_os = "macos")]
    {
        println!("\n=== iOS Physical Devices ===");
        match IosManager::new() {
            Ok(ios_manager) => {
                match ios_manager.list_physical_devices().await {
                    Ok(devices) => {
                        if devices.is_empty() {
                            println!("No iOS physical devices found.");
                        } else {
                            for device in devices {
                                println!("Found iOS device: {} ({}) - Physical: {}", 
                                         device.name, device.ios_version, device.is_physical);
                            }
                        }
                    }
                    Err(e) => println!("Error listing iOS physical devices: {}", e),
                }
                
                // Also test full device list
                println!("\n=== All iOS Devices (Virtual + Physical) ===");
                match ios_manager.list_devices().await {
                    Ok(devices) => {
                        for device in devices {
                            let device_type = if device.is_physical { "Physical" } else { "Virtual" };
                            println!("{}: {} ({})", device_type, device.name, device.ios_version);
                        }
                    }
                    Err(e) => println!("Error listing all iOS devices: {}", e),
                }
            }
            Err(e) => println!("Failed to create iOS manager: {}", e),
        }
    }

    Ok(())
}