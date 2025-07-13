//! Comprehensive integration tests using MockDeviceManager for emulator-independent testing.
//!
//! This file extends the existing comprehensive integration test with mock-based
//! device operations to provide complete end-to-end workflow testing.

use emu::app::state::{AppState, CreateDeviceField, CreateDeviceForm, Mode, Panel};
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::common::DeviceManager;
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::{common::DeviceConfig, mock::MockDeviceManager};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};

/// Integration test that verifies the complete workflow with mock device operations
#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_comprehensive_mock_device_workflow() {
    println!("=== COMPREHENSIVE MOCK DEVICE WORKFLOW TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();
    let mut state = AppState::new();

    // Phase 1: Initialize with mock device data
    let android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let ios_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list iOS devices");

    println!(
        "Initial devices - Android: {}, iOS: {}",
        android_devices.len(),
        ios_devices.len()
    );

    // Convert mock devices to app state format
    state.android_devices = android_devices
        .into_iter()
        .map(|device| AndroidDevice {
            name: device.name().to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status: *device.status(),
            is_running: device.is_running(),
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        })
        .collect();

    state.ios_devices = ios_devices
        .into_iter()
        .map(|device| IosDevice {
            name: device.name().to_string(),
            udid: device.id().to_string(),
            device_type: "iPhone".to_string(),
            ios_version: "16.0".to_string(),
            runtime_version: "iOS 16.0".to_string(),
            status: *device.status(),
            is_running: device.is_running(),
            is_available: true,
        })
        .collect();

    // Phase 2: Panel navigation workflow
    println!("Testing panel navigation...");
    assert_eq!(state.active_panel, Panel::Android);
    state.next_panel();
    assert_eq!(state.active_panel, Panel::Ios);
    state.next_panel();
    assert_eq!(state.active_panel, Panel::Android);

    // Phase 3: Device creation workflow with mock backend
    println!("Testing device creation workflow...");

    let device_config = DeviceConfig::new(
        "WorkflowTestDevice".to_string(),
        "pixel_8".to_string(),
        "34".to_string(),
    );

    android_manager
        .create_device(&device_config)
        .await
        .expect("Failed to create device");

    // Simulate form workflow
    state.mode = Mode::CreateDevice;
    state.create_device_form = CreateDeviceForm::for_android();

    // Test field navigation
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::ApiLevel
    );
    state.create_device_form.next_field();
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::Category
    );

    // Fill form
    state.create_device_form.name = device_config.name.clone();
    state.create_device_form.ram_size = "4096".to_string();
    state.create_device_form.storage_size = "16384".to_string();

    // Validate form
    assert!(!state.create_device_form.name.trim().is_empty());
    assert!(state.create_device_form.ram_size.parse::<u32>().is_ok());

    state.mode = Mode::Normal;

    // Phase 4: Device operation workflow
    println!("Testing device operation workflow...");

    // Start device operation
    state.set_device_operation_status("Starting device 'WorkflowTestDevice'...".to_string());
    assert!(state.get_device_operation_status().is_some());

    android_manager
        .start_device("WorkflowTestDevice")
        .await
        .expect("Failed to start device");

    // Update state to reflect operation completion
    state.clear_device_operation_status();
    assert!(state.get_device_operation_status().is_none());

    // Phase 5: List and status operations
    println!("Testing list and status operations...");

    let updated_android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let running_device = updated_android_devices
        .iter()
        .find(|d| d.name() == "WorkflowTestDevice")
        .expect("Device not found");

    assert!(running_device.is_running(), "Device should be running");

    // Phase 6: Device lifecycle completion
    println!("Testing device lifecycle completion...");

    android_manager
        .stop_device("WorkflowTestDevice")
        .await
        .expect("Failed to stop device");
    android_manager
        .wipe_device("WorkflowTestDevice")
        .await
        .expect("Failed to wipe device");
    android_manager
        .delete_device("WorkflowTestDevice")
        .await
        .expect("Failed to delete device");

    // Verify cleanup
    let final_android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    assert!(!final_android_devices
        .iter()
        .any(|d| d.name() == "WorkflowTestDevice"));

    println!("✅ Comprehensive mock device workflow completed successfully!");
}

/// Test complete UI state management without mock dependencies
#[test]
fn test_ui_state_management_workflow() {
    println!("=== UI STATE MANAGEMENT WORKFLOW TEST ===");

    let mut state = AppState::new();

    // Setup test devices
    state.android_devices = vec![
        AndroidDevice {
            name: "Pixel_7_API_31".to_string(),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            name: "Pixel_8_API_33".to_string(),
            device_type: "phone".to_string(),
            api_level: 33,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        },
    ];

    state.ios_devices = vec![IosDevice {
        name: "iPhone 14".to_string(),
        udid: "test-udid-1".to_string(),
        device_type: "iPhone".to_string(),
        ios_version: "16.0".to_string(),
        runtime_version: "iOS 16.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    }];

    // Test device selection and navigation
    assert_eq!(state.selected_android, 0);
    state.move_down();
    assert_eq!(state.selected_android, 1);
    state.move_down(); // Circular navigation
    assert_eq!(state.selected_android, 0);
    state.move_up(); // Circular navigation
    assert_eq!(state.selected_android, 1);

    // Test panel switching
    state.next_panel();
    assert_eq!(state.active_panel, Panel::Ios);
    assert_eq!(state.selected_ios, 0);

    // Test iOS device navigation
    state.move_down(); // Should stay at 0 (only one device)
    assert_eq!(state.selected_ios, 0);

    // Test mode switching
    state.mode = Mode::CreateDevice;
    assert_eq!(state.mode, Mode::CreateDevice);

    state.mode = Mode::Normal;
    assert_eq!(state.mode, Mode::Normal);

    println!("✅ UI state management workflow completed successfully!");
}

/// Test error handling and recovery workflow with mocks
#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_error_handling_workflow() {
    println!("=== ERROR HANDLING WORKFLOW TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let mut state = AppState::new();

    // Configure failure scenarios
    android_manager.configure_failure("start_device", "Device is busy");
    android_manager.configure_failure("create_device", "Insufficient disk space");

    // Test device creation failure
    let config = DeviceConfig::new(
        "FailureTestDevice".to_string(),
        "pixel_7".to_string(),
        "34".to_string(),
    );

    state.set_device_operation_status("Creating device...".to_string());

    let create_result = android_manager.create_device(&config).await;
    assert!(create_result.is_err(), "Device creation should fail");
    assert_eq!(
        create_result.unwrap_err().to_string(),
        "Insufficient disk space"
    );

    // Update UI state to reflect error
    state.set_device_operation_status("Error: Insufficient disk space".to_string());
    assert!(state
        .get_device_operation_status()
        .unwrap()
        .contains("Error"));

    // Test device start failure
    let start_result = android_manager.start_device("emulator-5554").await;
    assert!(start_result.is_err(), "Device start should fail");
    assert_eq!(start_result.unwrap_err().to_string(), "Device is busy");

    // Clear error state
    state.clear_device_operation_status();

    // Test recovery after clearing failures
    android_manager.clear_behavior();

    let recovery_result = android_manager.create_device(&config).await;
    assert!(
        recovery_result.is_ok(),
        "Device creation should succeed after recovery"
    );

    state.set_device_operation_status("Device created successfully".to_string());
    assert!(state
        .get_device_operation_status()
        .unwrap()
        .contains("successfully"));

    println!("✅ Error handling workflow completed successfully!");
}

/// Test concurrent operations workflow
#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_concurrent_operations_workflow() {
    println!("=== CONCURRENT OPERATIONS WORKFLOW TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    // Create devices for concurrent testing
    for i in 0..5 {
        let android_config = DeviceConfig::new(
            format!("ConcurrentAndroid{i}"),
            "pixel_8".to_string(),
            "34".to_string(),
        );
        android_manager
            .create_device(&android_config)
            .await
            .expect("Failed to create Android device");

        let ios_config = DeviceConfig::new(
            format!("ConcurrentiOS{i}"),
            "iPhone15,2".to_string(),
            "17.0".to_string(),
        );
        ios_manager
            .create_device(&ios_config)
            .await
            .expect("Failed to create iOS device");
    }

    // Test concurrent device operations
    let mut tasks = Vec::new();

    // Android operations
    for i in 0..5 {
        let manager_clone = android_manager.clone();
        let task = tokio::spawn(async move {
            let device_name = format!("ConcurrentAndroid{i}");
            manager_clone.start_device(&device_name).await?;
            manager_clone.list_devices().await?;
            manager_clone.stop_device(&device_name).await?;
            anyhow::Ok(())
        });
        tasks.push(task);
    }

    // iOS operations
    for i in 0..5 {
        let manager_clone = ios_manager.clone();
        let task = tokio::spawn(async move {
            let device_name = format!("ConcurrentiOS{i}");
            manager_clone.start_device(&device_name).await?;
            manager_clone.list_devices().await?;
            manager_clone.stop_device(&device_name).await?;
            anyhow::Ok(())
        });
        tasks.push(task);
    }

    // Wait for all operations
    for task in tasks {
        task.await.expect("Task failed").expect("Operation failed");
    }

    // Verify final state
    let android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let ios_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list iOS devices");

    assert_eq!(android_devices.len(), 7); // 2 default + 5 created
    assert_eq!(ios_devices.len(), 7); // 2 default + 5 created

    // Verify operations were recorded
    let android_ops = android_manager.get_operations().len();
    let ios_ops = ios_manager.get_operations().len();

    assert!(android_ops >= 20); // create + start + list + stop for each device
    assert!(ios_ops >= 20);

    println!("Android operations: {android_ops}, iOS operations: {ios_ops}");
    println!("✅ Concurrent operations workflow completed successfully!");
}

/// Test device details and state synchronization
#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_device_details_workflow() {
    println!("=== DEVICE DETAILS WORKFLOW TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let mut state = AppState::new();

    // Create a test device
    let config = DeviceConfig::new(
        "DetailsTestDevice".to_string(),
        "pixel_8_pro".to_string(),
        "34".to_string(),
    );

    android_manager
        .create_device(&config)
        .await
        .expect("Failed to create device");

    // Get updated device list
    let devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list devices");

    // Convert to app state format
    state.android_devices = devices
        .into_iter()
        .map(|device| AndroidDevice {
            name: device.name().to_string(),
            device_type: "phone".to_string(),
            api_level: 34,
            status: *device.status(),
            is_running: device.is_running(),
            ram_size: "6144".to_string(),
            storage_size: "32768".to_string(),
        })
        .collect();

    // Test device selection
    let test_device_index = state
        .android_devices
        .iter()
        .position(|d| d.name == "DetailsTestDevice")
        .expect("Test device not found");

    state.selected_android = test_device_index;

    // Test getting selected device details
    let selected_device = &state.android_devices[state.selected_android];
    assert_eq!(selected_device.name, "DetailsTestDevice");
    assert_eq!(selected_device.device_type, "phone");
    assert_eq!(selected_device.api_level, 34);
    assert_eq!(selected_device.status, DeviceStatus::Stopped);
    assert!(!selected_device.is_running);

    // Test device state update after operation
    android_manager
        .start_device("DetailsTestDevice")
        .await
        .expect("Failed to start device");

    // Simulate state update after operation
    let updated_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let updated_device = updated_devices
        .iter()
        .find(|d| d.name() == "DetailsTestDevice")
        .expect("Device not found");

    // Update app state to reflect changes
    state.android_devices[state.selected_android].status = *updated_device.status();
    state.android_devices[state.selected_android].is_running = updated_device.is_running();

    // Verify state synchronization
    assert_eq!(
        state.android_devices[state.selected_android].status,
        DeviceStatus::Running
    );
    assert!(state.android_devices[state.selected_android].is_running);

    println!("✅ Device details workflow completed successfully!");
}
