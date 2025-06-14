//! Example to verify physical device detection in the TUI

use anyhow::Result;
use emu::managers::{common::DeviceManager, AndroidManager};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("Emu Physical Device Detection Check");
    println!("===================================\n");

    // Create Android manager
    let android_manager = AndroidManager::new()?;

    // List all devices
    println!("Detecting devices...");
    let devices = android_manager.list_devices().await?;

    // Separate physical and virtual
    let physical: Vec<_> = devices.iter().filter(|d| d.is_physical).collect();
    let virtual_devices: Vec<_> = devices.iter().filter(|d| !d.is_physical).collect();

    println!("\nDevice Summary:");
    println!("  Total devices: {}", devices.len());
    println!("  Virtual devices (AVDs): {}", virtual_devices.len());
    println!("  Physical devices: {}", physical.len());

    if !physical.is_empty() {
        println!("\nPhysical Devices Connected:");
        for device in &physical {
            println!(
                "  📱 {} (API {}) - {}",
                device.name, device.api_level, device.device_type
            );
        }

        println!("\n✅ Physical devices should appear in the Emu TUI with a 📱 indicator");
        println!("   Run 'emu' to see them in the Android panel");
    } else {
        println!("\n⚠️  No physical devices detected");
        println!("   Make sure your device is:");
        println!("   1. Connected via USB");
        println!("   2. Has USB debugging enabled");
        println!("   3. Has authorized this computer");
    }

    if !virtual_devices.is_empty() {
        println!("\nVirtual Devices (AVDs):");
        for device in virtual_devices.iter().take(3) {
            println!("  {} (API {})", device.name, device.api_level);
        }
        if virtual_devices.len() > 3 {
            println!("  ... and {} more", virtual_devices.len() - 3);
        }
    }

    Ok(())
}
