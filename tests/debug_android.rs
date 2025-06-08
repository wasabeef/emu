use emu::models::device_config::test_constants::TEST_ANDROID_DEVICE;
use tokio::process::Command;

#[tokio::test]
async fn debug_android_system_images() {
    println!("🔍 Debugging Android system images...");

    // Check available system images
    println!("📋 Checking available system images...");
    let output = Command::new("avdmanager")
        .args(&["list", "targets"])
        .output()
        .await;

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            println!("✅ Available targets:");
            println!("{}", stdout);

            if !stderr.is_empty() {
                println!("⚠️  Stderr:");
                println!("{}", stderr);
            }
        }
        Err(e) => {
            println!("❌ Failed to check targets: {}", e);
            return;
        }
    }

    // Check available packages
    println!("\n📦 Checking available packages...");
    let output = Command::new("sdkmanager")
        .args(&["--list", "--verbose"])
        .output()
        .await;

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);

            // Filter for system images
            let system_images: Vec<&str> = stdout
                .lines()
                .filter(|line| line.contains("system-images") && line.contains("google_apis"))
                .take(10) // Limit output
                .collect();

            println!("✅ Available system images (first 10):");
            for image in system_images {
                println!("   {}", image.trim());
            }
        }
        Err(e) => {
            println!("❌ Failed to check packages: {}", e);
        }
    }

    // Try to create with the most basic configuration
    println!("\n🧪 Testing basic device creation...");
    let test_name = format!("debug_test_{}", chrono::Utc::now().timestamp());

    let output = Command::new("avdmanager")
        .args(&[
            "create",
            "avd",
            "--name",
            &test_name,
            "--package",
            "system-images;android-34;google_apis;arm64-v8a",
            "--device",
            TEST_ANDROID_DEVICE,
            "--force",
        ])
        .output()
        .await;

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            if result.status.success() {
                println!("✅ Test device created successfully");

                // Clean up
                let _ = Command::new("avdmanager")
                    .args(&["delete", "avd", "--name", &test_name])
                    .output()
                    .await;
                println!("🧹 Test device cleaned up");
            } else {
                println!("❌ Failed to create test device");
                println!("Exit code: {}", result.status);
                println!("Stdout: {}", stdout);
                println!("Stderr: {}", stderr);
            }
        }
        Err(e) => {
            println!("❌ Failed to execute avdmanager: {}", e);
        }
    }
}
