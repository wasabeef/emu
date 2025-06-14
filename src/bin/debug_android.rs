//! Android SDK and emulator diagnostic tool
//!
//! This tool checks the Android development environment and diagnoses
//! common issues with emulator startup.

use anyhow::Result;
use emu::managers::android::AndroidManager;
use emu::managers::common::DeviceManager;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    println!("🔧 Android Development Environment Diagnostics");
    println!("===============================================\n");

    // 1. Check environment variables
    println!("📍 Environment Variables:");
    if let Ok(android_home) = env::var("ANDROID_HOME") {
        println!("  ✅ ANDROID_HOME: {}", android_home);
    } else {
        println!("  ❌ ANDROID_HOME: Not set");
    }

    if let Ok(android_sdk) = env::var("ANDROID_SDK_ROOT") {
        println!("  ✅ ANDROID_SDK_ROOT: {}", android_sdk);
    } else {
        println!("  ❌ ANDROID_SDK_ROOT: Not set");
    }

    if let Ok(path) = env::var("PATH") {
        let has_android_tools = path.contains("Android") || path.contains("android");
        if has_android_tools {
            println!("  ✅ PATH: Contains Android tools");
        } else {
            println!("  ⚠️  PATH: No Android tools detected");
        }
    }

    println!();

    // 2. Try to initialize AndroidManager
    println!("🏗️  Android Manager Initialization:");
    match AndroidManager::new() {
        Ok(manager) => {
            println!("  ✅ AndroidManager initialized successfully");

            // 3. List available AVDs
            println!("\n📱 Available AVDs:");
            match manager.list_devices().await {
                Ok(devices) => {
                    if devices.is_empty() {
                        println!("  ⚠️  No AVDs found. Create an AVD first.");
                    } else {
                        for device in &devices {
                            let status = if device.is_running {
                                "🟢 Running"
                            } else {
                                "⚪ Stopped"
                            };
                            println!("  {} {} (API {})", status, device.name, device.api_level);
                        }

                        // 4. Test emulator startup with first available AVD
                        if let Some(device) = devices.first() {
                            if !device.is_running {
                                println!("\n🚀 Testing Emulator Startup:");
                                println!("  Testing with AVD: {}", device.name);

                                match manager.start_device(&device.name).await {
                                    Ok(()) => {
                                        println!(
                                            "  ✅ Emulator startup command executed successfully"
                                        );
                                        println!("  ℹ️  Note: Emulator may take 1-2 minutes to fully boot");
                                    }
                                    Err(e) => {
                                        println!("  ❌ Emulator startup failed: {}", e);
                                        println!("  💡 Troubleshooting tips:");
                                        println!("     - Check if Android Emulator is installed via SDK Manager");
                                        println!("     - Ensure hardware acceleration (Intel HAXM/Hyper-V) is enabled");
                                        println!(
                                            "     - Try running manually: emulator -avd {}",
                                            device.name
                                        );
                                    }
                                }
                            } else {
                                println!("\n✅ First AVD is already running: {}", device.name);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ Failed to list devices: {}", e);
                }
            }

            // 5. Check system images
            println!("\n🖼️  System Images:");
            match manager.list_available_api_levels_filtered(true).await {
                Ok(api_levels) => {
                    if api_levels.is_empty() {
                        println!("  ❌ No system images installed");
                        println!("  💡 Install system images using Android Studio SDK Manager");
                    } else {
                        println!("  ✅ {} system image(s) available:", api_levels.len());
                        for api in api_levels.iter().take(5) {
                            println!("    - {}", api.display_name());
                        }
                        if api_levels.len() > 5 {
                            println!("    ... and {} more", api_levels.len() - 5);
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ Failed to check system images: {}", e);
                }
            }

            // 6. Check skins directory
            println!("\n🎨 Skins Directory:");
            if let Ok(android_home) = env::var("ANDROID_HOME") {
                let android_home_path = std::path::Path::new(&android_home);
                let skins_path = android_home_path.join("skins");
                let platform_skins_path = android_home_path.join("platforms");

                if skins_path.exists() {
                    println!("  ✅ Skins directory found: {:?}", skins_path);
                    if let Ok(entries) = std::fs::read_dir(&skins_path) {
                        let skins: Vec<_> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().is_dir())
                            .collect();
                        println!("  📦 {} skin(s) found", skins.len());
                        for skin in skins.iter().take(5) {
                            if let Some(name) = skin.file_name().to_str() {
                                println!("    - {}", name);
                            }
                        }
                    }
                } else {
                    println!("  ⚠️  No skins directory at {:?}", skins_path);
                }

                // Check platform skins
                if platform_skins_path.exists() {
                    println!("\n  📱 Platform skins:");
                    if let Ok(platforms) = std::fs::read_dir(&platform_skins_path) {
                        for platform in platforms.filter_map(|e| e.ok()) {
                            let platform_skin_dir = platform.path().join("skins");
                            if platform_skin_dir.exists() {
                                if let Some(platform_name) = platform.file_name().to_str() {
                                    println!("    - {}: {:?}", platform_name, platform_skin_dir);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("  ❌ AndroidManager initialization failed: {}", e);
            println!("\n💡 Troubleshooting steps:");
            println!("  1. Install Android Studio and SDK");
            println!("  2. Set ANDROID_HOME environment variable");
            println!("  3. Ensure cmdline-tools are installed");
            println!("  4. Add SDK tools to PATH");
        }
    }

    println!("\n🏁 Diagnostics complete!");
    Ok(())
}
