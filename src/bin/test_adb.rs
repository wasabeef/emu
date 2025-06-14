use std::process::Command;

fn main() {
    println!("=== Testing ADB connection ===\n");

    // Check adb version
    match Command::new("adb").arg("version").output() {
        Ok(output) => {
            println!("ADB version:");
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Err(e) => println!("Failed to run adb version: {}", e),
    }

    // Check connected devices
    match Command::new("adb").arg("devices").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("Connected devices:");
            println!("{}", stdout);

            // Parse device IDs
            let mut device_ids = Vec::new();
            for line in stdout.lines() {
                if line.contains("device")
                    && !line.contains("List of devices")
                    && !line.contains("emulator-")
                {
                    if let Some(device_id) = line.split_whitespace().next() {
                        device_ids.push(device_id.to_string());
                    }
                }
            }

            // Get device properties
            for device_id in device_ids {
                println!("\n=== Device: {} ===", device_id);

                // Get model
                if let Ok(output) = Command::new("adb")
                    .args(["-s", &device_id, "shell", "getprop", "ro.product.model"])
                    .output()
                {
                    println!("Model: {}", String::from_utf8_lossy(&output.stdout).trim());
                }

                // Get manufacturer
                if let Ok(output) = Command::new("adb")
                    .args([
                        "-s",
                        &device_id,
                        "shell",
                        "getprop",
                        "ro.product.manufacturer",
                    ])
                    .output()
                {
                    println!(
                        "Manufacturer: {}",
                        String::from_utf8_lossy(&output.stdout).trim()
                    );
                }

                // Get API level
                if let Ok(output) = Command::new("adb")
                    .args(["-s", &device_id, "shell", "getprop", "ro.build.version.sdk"])
                    .output()
                {
                    println!(
                        "API Level: {}",
                        String::from_utf8_lossy(&output.stdout).trim()
                    );
                }
            }
        }
        Err(e) => println!("Failed to run adb devices: {}", e),
    }

    // Check adb server status
    println!("\n=== ADB Server Status ===");
    match Command::new("adb").arg("get-state").output() {
        Ok(output) => {
            println!("State: {}", String::from_utf8_lossy(&output.stdout).trim());
            if !output.stderr.is_empty() {
                println!("Error: {}", String::from_utf8_lossy(&output.stderr).trim());
            }
        }
        Err(e) => println!("Failed to get adb state: {}", e),
    }
}
