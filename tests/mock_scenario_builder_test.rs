//! Scenario builder tests using MockDeviceManager for emulator-independent testing.
//!
//! This demonstrates how to create complex test scenarios with pre-configured
//! mock behavior without requiring actual emulator environments.

#[cfg(any(test, feature = "test-utils"))]
use emu::managers::common::DeviceManager;
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::{common::DeviceConfig, mock::MockDeviceManager};

/// Test scenario builder for creating complex mock environments
#[cfg(any(test, feature = "test-utils"))]
pub struct MockScenarioBuilder {
    manager: MockDeviceManager,
    device_count: usize,
    platform: String,
}

#[cfg(any(test, feature = "test-utils"))]
impl MockScenarioBuilder {
    pub fn new_android() -> Self {
        Self {
            manager: MockDeviceManager::new_android(),
            device_count: 0,
            platform: "android".to_string(),
        }
    }

    pub fn new_ios() -> Self {
        Self {
            manager: MockDeviceManager::new_ios(),
            device_count: 0,
            platform: "ios".to_string(),
        }
    }

    pub fn with_devices(mut self, count: usize) -> Self {
        self.device_count = count;
        self
    }

    pub fn with_failing_operation(self, operation: &str, error_message: &str) -> Self {
        self.manager.configure_failure(operation, error_message);
        self
    }

    pub fn with_operation_delay(self, operation: &str, delay_ms: u64) -> Self {
        self.manager.configure_delay(operation, delay_ms);
        self
    }

    pub async fn build(self) -> anyhow::Result<MockDeviceManager> {
        // Create additional devices if requested
        for i in 0..self.device_count {
            let config = if self.platform == "android" {
                DeviceConfig::new(
                    format!("ScenarioDevice{i}"),
                    "pixel_8".to_string(),
                    "34".to_string(),
                )
            } else {
                DeviceConfig::new(
                    format!("iPhone Test {i}"),
                    "iPhone15,2".to_string(),
                    "17.0".to_string(),
                )
            };

            self.manager.create_device(&config).await?;
        }

        Ok(self.manager)
    }
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_scenario_with_many_devices() {
    println!("=== SCENARIO WITH MANY DEVICES TEST ===");

    let manager = MockScenarioBuilder::new_android()
        .with_devices(10)
        .build()
        .await
        .expect("Failed to build scenario");

    // Verify devices were created
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    assert_eq!(devices.len(), 12); // 2 default + 10 created

    // Verify device names
    for i in 0..10 {
        let device_name = format!("ScenarioDevice{i}");
        assert!(devices.iter().any(|d| d.name() == device_name));
    }

    // Test concurrent operations on all scenario devices
    let mut tasks = Vec::new();
    for i in 0..10 {
        let manager_clone = manager.clone();
        let task = tokio::spawn(async move {
            let device_name = format!("ScenarioDevice{i}");
            manager_clone.start_device(&device_name).await
        });
        tasks.push(task);
    }

    // Wait for all operations to complete
    for task in tasks {
        task.await.expect("Task failed").expect("Operation failed");
    }

    // Verify all devices are running
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let running_count = devices
        .iter()
        .filter(|d| d.name().starts_with("ScenarioDevice") && d.is_running())
        .count();
    assert_eq!(running_count, 10);

    println!("✅ Scenario with many devices test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_scenario_with_mixed_failures() {
    println!("=== SCENARIO WITH MIXED FAILURES TEST ===");

    let manager = MockScenarioBuilder::new_android()
        .with_devices(5)
        .with_failing_operation("start_device", "Random failure")
        .with_operation_delay("create_device", 10)
        .build()
        .await
        .expect("Failed to build scenario");

    // Test that creation worked (despite delays)
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    assert_eq!(devices.len(), 7); // 2 default + 5 created

    // Test that start operations fail
    let result = manager.start_device("ScenarioDevice0").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Random failure");

    // Verify operations were recorded
    let operations = manager.get_operations();
    assert!(operations.len() >= 6); // At least 5 creates + 1 list + 1 failed start

    println!("✅ Scenario with mixed failures test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_ios_scenario_builder() {
    println!("=== IOS SCENARIO BUILDER TEST ===");

    let manager = MockScenarioBuilder::new_ios()
        .with_devices(3)
        .with_operation_delay("start_device", 20)
        .build()
        .await
        .expect("Failed to build iOS scenario");

    // Verify iOS devices were created
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    assert_eq!(devices.len(), 5); // 2 default + 3 created

    // Test operation with delay
    let start = std::time::Instant::now();
    manager
        .start_device("iPhone Test 0")
        .await
        .expect("Failed to start device");
    let duration = start.elapsed();

    assert!(duration >= std::time::Duration::from_millis(20));
    assert!(duration < std::time::Duration::from_millis(50));

    println!("✅ iOS scenario builder test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_complex_scenario_workflow() {
    println!("=== COMPLEX SCENARIO WORKFLOW TEST ===");

    let manager = MockScenarioBuilder::new_android()
        .with_devices(3)
        .with_failing_operation("delete_device", "Device is protected")
        .with_operation_delay("wipe_device", 15)
        .build()
        .await
        .expect("Failed to build complex scenario");

    // Phase 1: Start all devices
    for i in 0..3 {
        let device_name = format!("ScenarioDevice{i}");
        manager
            .start_device(&device_name)
            .await
            .expect("Failed to start device");
    }

    // Verify all are running
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let running_count = devices
        .iter()
        .filter(|d| d.name().starts_with("ScenarioDevice") && d.is_running())
        .count();
    assert_eq!(running_count, 3);

    // Phase 2: Wipe devices (with delay)
    let start = std::time::Instant::now();
    for i in 0..3 {
        let device_name = format!("ScenarioDevice{i}");
        manager
            .wipe_device(&device_name)
            .await
            .expect("Failed to wipe device");
    }
    let wipe_duration = start.elapsed();

    // Should take at least 45ms (3 devices * 15ms delay each)
    assert!(wipe_duration >= std::time::Duration::from_millis(45));

    // Phase 3: Try to delete devices (should fail)
    for i in 0..3 {
        let device_name = format!("ScenarioDevice{i}");
        let result = manager.delete_device(&device_name).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Device is protected");
    }

    // Verify devices still exist
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    assert_eq!(devices.len(), 5); // Still have all devices

    // Check operation history
    let operations = manager.get_operations();
    assert!(operations.len() >= 12); // Many operations were performed

    println!("✅ Complex scenario workflow test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_scenario_with_zero_devices() {
    println!("=== SCENARIO WITH ZERO DEVICES TEST ===");

    let manager = MockScenarioBuilder::new_android()
        .with_devices(0) // Don't create any additional devices
        .with_failing_operation("list_devices", "Service unavailable")
        .build()
        .await
        .expect("Failed to build zero-device scenario");

    // list_devices should fail due to configuration
    let result = manager.list_devices().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Service unavailable");

    // But manager should be available
    assert!(manager.is_available().await);

    println!("✅ Scenario with zero devices test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_scenario_builder_chaining() {
    println!("=== SCENARIO BUILDER CHAINING TEST ===");

    let manager = MockScenarioBuilder::new_android()
        .with_devices(2)
        .with_failing_operation("start_device", "First failure")
        .with_operation_delay("stop_device", 25)
        .with_failing_operation("wipe_device", "Second failure")
        .build()
        .await
        .expect("Failed to build chained scenario");

    // Test multiple configured behaviors
    let result = manager.start_device("ScenarioDevice0").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "First failure");

    let start = std::time::Instant::now();
    manager
        .stop_device("ScenarioDevice0")
        .await
        .expect("Stop should work");
    let duration = start.elapsed();
    assert!(duration >= std::time::Duration::from_millis(25));

    let result = manager.wipe_device("ScenarioDevice0").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Second failure");

    println!("✅ Scenario builder chaining test completed successfully");
}
