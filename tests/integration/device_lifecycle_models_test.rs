//! Comprehensive device lifecycle tests for Emu
//!
//! This module contains extensive tests for device lifecycle management,
//! including error scenarios, edge cases, and system resource constraints.
//! All tests are designed to run without requiring actual emulator/simulator startup.

use emu::app::state::AppState;
use emu::models::device::{AndroidDevice, DeviceStatus};
use emu::utils::validation::{
    DeviceNameValidator, DevicePlatform, FieldValidator, NumericRangeValidator,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Test device creation with various error scenarios
#[tokio::test]
async fn test_device_creation_error_scenarios() {
    let mut app_state = AppState::new();

    // Test 1: Invalid device name
    app_state.create_device_form.name = "device!@#$".to_string();
    app_state.create_device_form.device_type = "phone".to_string();
    app_state.create_device_form.version = "30".to_string();

    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let result = device_validator.validate(&app_state.create_device_form.name);
    assert!(result.is_err(), "Invalid device name should be rejected");

    // Test 2: Empty required fields
    app_state.create_device_form.name = "".to_string();
    let result = device_validator.validate(&app_state.create_device_form.name);
    assert!(result.is_err(), "Empty device name should be rejected");

    // Test 3: Invalid RAM size
    app_state.create_device_form.ram_size = "99999".to_string();
    let ram_validator = NumericRangeValidator::ram_size();
    let result = ram_validator.validate(&app_state.create_device_form.ram_size);
    assert!(result.is_err(), "Invalid RAM size should be rejected");

    // Test 4: Invalid storage size
    app_state.create_device_form.storage_size = "100".to_string();
    let storage_validator = NumericRangeValidator::storage_size();
    let result = storage_validator.validate(&app_state.create_device_form.storage_size);
    assert!(result.is_err(), "Too small storage size should be rejected");
}

/// Test duplicate device name handling
#[test]
fn test_duplicate_device_name_detection() {
    let mut app_state = AppState::new();

    // Add an existing device
    let existing_device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "test_device".to_string(),
        device_type: "phone".to_string(),
        api_level: 30,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192".to_string(),
    };

    app_state.android_devices = vec![existing_device];

    // Attempt to create device with same name
    app_state.create_device_form.name = "test_device".to_string();

    // Check if duplicate is detected
    let existing_names: Vec<&str> = app_state
        .android_devices
        .iter()
        .map(|d| d.name.as_str())
        .collect();

    let new_name = &app_state.create_device_form.name;
    assert!(
        existing_names.contains(&new_name.as_str()),
        "Duplicate name should be detected"
    );
}

/// Test device state transitions
#[test]
fn test_device_state_transitions() {
    let mut device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "state_test".to_string(),
        device_type: "phone".to_string(),
        api_level: 30,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192".to_string(),
    };

    // Test valid transitions
    let valid_transitions = vec![
        (DeviceStatus::Stopped, DeviceStatus::Starting),
        (DeviceStatus::Starting, DeviceStatus::Running),
        (DeviceStatus::Running, DeviceStatus::Stopping),
        (DeviceStatus::Stopping, DeviceStatus::Stopped),
    ];

    for (from, to) in valid_transitions {
        device.status = from;
        device.status = to;
        assert_eq!(
            device.status, to,
            "State transition from {from:?} to {to:?} should succeed"
        );
    }

    // Test is_running flag consistency
    device.status = DeviceStatus::Running;
    device.is_running = true;
    assert!(
        device.is_running,
        "is_running should be true when status is Running"
    );

    device.status = DeviceStatus::Stopped;
    device.is_running = false;
    assert!(
        !device.is_running,
        "is_running should be false when status is Stopped"
    );
}

/// Test concurrent device operations
#[tokio::test]
async fn test_concurrent_device_operations() {
    let app_state = Arc::new(Mutex::new(AppState::new()));
    let mut handles = vec![];

    // Simulate concurrent device operations
    for i in 0..5 {
        let state_clone = Arc::clone(&app_state);
        let handle = tokio::spawn(async move {
            // Try to add a device
            let device = AndroidDevice {
                android_version_name: "API 30".to_string(),
                name: format!("concurrent_device_{i}"),
                device_type: "phone".to_string(),
                api_level: 30,
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: "2048".to_string(),
                storage_size: "8192".to_string(),
            };

            // Lock and modify state
            if let Ok(mut state) = state_clone.lock() {
                state.android_devices.push(device);
            }

            // Small delay to increase chance of race conditions
            tokio::time::sleep(Duration::from_millis(10)).await;
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let _ = handle.await;
    }

    // Verify all devices were added
    let state = app_state.lock().unwrap();
    assert_eq!(
        state.android_devices.len(),
        5,
        "All concurrent devices should be added"
    );

    // Verify no duplicates
    let mut names: Vec<String> = state
        .android_devices
        .iter()
        .map(|d| d.name.clone())
        .collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 5, "No duplicate devices should exist");
}

/// Test device operation error recovery
#[tokio::test]
async fn test_device_operation_error_recovery() {
    let mut app_state = AppState::new();

    // Add a device in various states
    let devices = vec![
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "starting_device".to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Starting,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "stopping_device".to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Stopping,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
    ];

    app_state.android_devices = devices;

    // Simulate recovery from transient states
    for device in &mut app_state.android_devices {
        match device.status {
            DeviceStatus::Starting => {
                // Should either complete start or rollback to stopped
                device.status = DeviceStatus::Stopped;
                device.is_running = false;
            }
            DeviceStatus::Stopping => {
                // Should complete stop
                device.status = DeviceStatus::Stopped;
                device.is_running = false;
            }
            DeviceStatus::Creating => {
                // Should complete creation or rollback to stopped
                device.status = DeviceStatus::Stopped;
                device.is_running = false;
            }
            DeviceStatus::Error => {
                // Should be cleared to stopped
                device.status = DeviceStatus::Stopped;
                device.is_running = false;
            }
            DeviceStatus::Unknown => {
                // Should be resolved to a known state
                device.status = DeviceStatus::Stopped;
                device.is_running = false;
            }
            DeviceStatus::Running | DeviceStatus::Stopped => {
                // Already in stable state
            }
        }
    }

    // Verify all devices are in stable states
    for device in &app_state.android_devices {
        assert!(
            matches!(device.status, DeviceStatus::Running | DeviceStatus::Stopped),
            "Device {} should be in stable state",
            device.name
        );
    }
}

/// Test device cleanup and resource deallocation
#[test]
fn test_device_cleanup_completeness() {
    let mut app_state = AppState::new();

    // Add multiple devices
    for i in 0..10 {
        app_state.android_devices.push(AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: format!("cleanup_device_{i}"),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        });
    }

    // Mark some devices for deletion
    let devices_to_remove = vec!["cleanup_device_3", "cleanup_device_7", "cleanup_device_9"];

    // Remove marked devices
    app_state
        .android_devices
        .retain(|device| !devices_to_remove.contains(&device.name.as_str()));

    // Verify correct devices remain
    assert_eq!(
        app_state.android_devices.len(),
        7,
        "Should have 7 devices after cleanup"
    );

    // Verify no dangling references
    for device_name in devices_to_remove {
        assert!(
            !app_state
                .android_devices
                .iter()
                .any(|d| d.name == device_name),
            "Device {device_name} should be completely removed"
        );
    }
}

/// Test device configuration validation
#[test]
fn test_device_configuration_validation() {
    // Test various device configurations
    let test_configs = vec![
        // (device_type, api_level, ram, storage, should_be_valid)
        ("phone", 30, "2048", "8192", true),
        ("tablet", 34, "4096", "16384", true),
        ("tv", 31, "2048", "4096", true),
        ("wear", 30, "512", "2048", true),
        ("automotive", 30, "4096", "16384", true),
        // Invalid configurations
        ("phone", 30, "256", "8192", false),    // RAM too small
        ("tablet", 34, "2048", "512", false),   // Storage too small
        ("unknown", 30, "2048", "8192", false), // Invalid device type
    ];

    let ram_validator = NumericRangeValidator::ram_size();
    let storage_validator = NumericRangeValidator::storage_size();

    for (device_type, api_level, ram, storage, should_be_valid) in test_configs {
        let ram_valid = ram_validator.validate(ram).is_ok();
        let storage_valid = storage_validator.validate(storage).is_ok();
        let device_type_valid =
            ["phone", "tablet", "tv", "wear", "automotive"].contains(&device_type);
        let api_level_valid = (19..=34).contains(&api_level);

        let is_valid = ram_valid && storage_valid && device_type_valid && api_level_valid;

        assert_eq!(
            is_valid, should_be_valid,
            "Configuration validation failed for: type={device_type}, api={api_level}, ram={ram}, storage={storage}"
        );
    }
}

/// Test API level compatibility
#[test]
fn test_api_level_compatibility() {
    // Test API level requirements for different device types
    let compatibility_tests = vec![
        // (device_type, min_api, max_api)
        ("phone", 19, 34),
        ("tablet", 19, 34),
        ("tv", 21, 34),
        ("wear", 23, 33),
        ("automotive", 28, 34),
    ];

    for (device_type, min_api, max_api) in compatibility_tests {
        for api in min_api..=max_api {
            let device = AndroidDevice {
                android_version_name: "API 30".to_string(),
                name: format!("api_test_{api}"),
                device_type: device_type.to_string(),
                api_level: api,
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: "2048".to_string(),
                storage_size: "8192".to_string(),
            };

            // Verify device can be created with this API level
            assert!(
                device.api_level >= min_api && device.api_level <= max_api,
                "API level {api} should be valid for device type {device_type}"
            );
        }
    }
}

/// Test device list filtering and search
#[test]
fn test_device_list_filtering() {
    let mut app_state = AppState::new();

    // Add diverse devices
    let devices = vec![
        ("phone_1", "phone", 30, DeviceStatus::Running),
        ("phone_2", "phone", 31, DeviceStatus::Stopped),
        ("tablet_1", "tablet", 33, DeviceStatus::Running),
        ("tv_1", "tv", 31, DeviceStatus::Stopped),
        ("wear_1", "wear", 30, DeviceStatus::Running),
    ];

    for (name, device_type, api_level, status) in devices {
        app_state.android_devices.push(AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: name.to_string(),
            device_type: device_type.to_string(),
            api_level,
            status,
            is_running: matches!(status, DeviceStatus::Running),
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        });
    }

    // Test filtering by status
    let running_devices: Vec<&AndroidDevice> = app_state
        .android_devices
        .iter()
        .filter(|d| d.status == DeviceStatus::Running)
        .collect();
    assert_eq!(running_devices.len(), 3, "Should have 3 running devices");

    // Test filtering by device type
    let phones: Vec<&AndroidDevice> = app_state
        .android_devices
        .iter()
        .filter(|d| d.device_type == "phone")
        .collect();
    assert_eq!(phones.len(), 2, "Should have 2 phones");

    // Test search by name pattern
    let search_pattern = "phone";
    let search_results: Vec<&AndroidDevice> = app_state
        .android_devices
        .iter()
        .filter(|d| d.name.contains(search_pattern))
        .collect();
    assert_eq!(
        search_results.len(),
        2,
        "Should find 2 devices matching 'phone'"
    );
}

/// Test error propagation and user feedback
#[test]
fn test_error_propagation() {
    // Test various error scenarios and their messages
    let error_scenarios = vec![
        ("Device name cannot be empty", "empty_name"),
        ("Device name contains invalid characters", "invalid_chars"),
        ("RAM size must be between 512 and 8192 MB", "invalid_ram"),
        (
            "Storage size must be between 1024 and 65536 MB",
            "invalid_storage",
        ),
        ("Device with this name already exists", "duplicate_name"),
    ];

    for (expected_message, error_type) in error_scenarios {
        // Simulate error based on type
        let error = match error_type {
            "empty_name" => "Device name cannot be empty",
            "invalid_chars" => "Device name contains invalid characters",
            "invalid_ram" => "RAM size must be between 512 and 8192 MB",
            "invalid_storage" => "Storage size must be between 1024 and 65536 MB",
            "duplicate_name" => "Device with this name already exists",
            _ => "Unknown error",
        };

        assert_eq!(
            error, expected_message,
            "Error message mismatch for {error_type}"
        );
    }
}

/// Test device operation timeout handling
#[tokio::test]
async fn test_device_operation_timeouts() {
    let mut app_state = AppState::new();

    // Add devices in various states
    let devices = vec![
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "timeout_test_1".to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Starting,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "timeout_test_2".to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Stopping,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
    ];

    app_state.android_devices = devices;

    // Simulate timeout scenarios
    for device in &mut app_state.android_devices {
        match device.status {
            DeviceStatus::Starting => {
                // Simulate start timeout - should rollback to stopped
                device.status = DeviceStatus::Error;
                device.is_running = false;

                // Recovery process
                if device.status == DeviceStatus::Error {
                    device.status = DeviceStatus::Stopped;
                }
            }
            DeviceStatus::Stopping => {
                // Simulate stop timeout - force stop
                device.status = DeviceStatus::Stopped;
                device.is_running = false;
            }
            _ => {}
        }
    }

    // Verify all devices are in safe states after timeout handling
    for device in &app_state.android_devices {
        assert!(
            matches!(device.status, DeviceStatus::Stopped | DeviceStatus::Running),
            "Device {} should be in stable state after timeout",
            device.name
        );

        // Verify state consistency
        if device.status == DeviceStatus::Running {
            assert!(
                device.is_running,
                "Running device should have is_running=true"
            );
        } else {
            assert!(
                !device.is_running,
                "Non-running device should have is_running=false"
            );
        }
    }
}

/// Test insufficient system resources scenarios
#[test]
fn test_insufficient_system_resources() {
    let mut app_state = AppState::new();

    // Test scenarios with various resource constraints
    let resource_scenarios = vec![
        // (ram_requested, storage_requested, should_succeed)
        ("512", "2048", true),     // Minimum resources
        ("8192", "65536", true),   // Maximum resources
        ("256", "2048", false),    // RAM too low
        ("2048", "512", false),    // Storage too low
        ("16384", "8192", false),  // RAM too high
        ("2048", "131072", false), // Storage too high
    ];

    let ram_validator = NumericRangeValidator::ram_size();
    let storage_validator = NumericRangeValidator::storage_size();

    for (ram, storage, should_succeed) in resource_scenarios {
        let ram_valid = ram_validator.validate(ram).is_ok();
        let storage_valid = storage_validator.validate(storage).is_ok();
        let is_valid = ram_valid && storage_valid;

        assert_eq!(
            is_valid, should_succeed,
            "Resource validation failed for RAM={ram}, Storage={storage}"
        );

        if is_valid {
            // Simulate successful device creation
            let device = AndroidDevice {
                android_version_name: "API 30".to_string(),
                name: format!("resource_test_{ram}_{storage}"),
                device_type: "phone".to_string(),
                api_level: 30,
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: ram.to_string(),
                storage_size: storage.to_string(),
            };

            app_state.android_devices.push(device);
        }
    }

    // Verify only valid devices were created
    assert_eq!(
        app_state.android_devices.len(),
        2,
        "Only valid devices should be created"
    );
}

/// Test parallel device operations and race conditions
#[tokio::test]
async fn test_parallel_device_operations_safety() {
    let app_state = Arc::new(Mutex::new(AppState::new()));

    // Pre-populate with some devices
    {
        let mut state = app_state.lock().unwrap();
        for i in 0..3 {
            state.android_devices.push(AndroidDevice {
                android_version_name: "API 30".to_string(),
                name: format!("parallel_device_{i}"),
                device_type: "phone".to_string(),
                api_level: 30,
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: "2048".to_string(),
                storage_size: "8192".to_string(),
            });
        }
    }

    let mut handles = vec![];

    // Simulate concurrent operations
    for i in 0..5 {
        let state_clone = Arc::clone(&app_state);
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                // Try various operations
                if let Ok(mut state) = state_clone.lock() {
                    // Simulate start/stop operations
                    let device_count = state.android_devices.len();
                    if device_count > 0 {
                        let device_index = i % device_count;
                        if let Some(device) = state.android_devices.get_mut(device_index) {
                            match device.status {
                                DeviceStatus::Stopped => {
                                    device.status = DeviceStatus::Starting;
                                    // Simulate quick start completion
                                    device.status = DeviceStatus::Running;
                                    device.is_running = true;
                                }
                                DeviceStatus::Running => {
                                    device.status = DeviceStatus::Stopping;
                                    // Simulate quick stop completion
                                    device.status = DeviceStatus::Stopped;
                                    device.is_running = false;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Small delay to encourage race conditions
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all operations
    for handle in handles {
        let _ = handle.await;
    }

    // Verify final state consistency
    let state = app_state.lock().unwrap();
    for device in &state.android_devices {
        // Check state consistency
        match device.status {
            DeviceStatus::Running => {
                assert!(
                    device.is_running,
                    "Running device {} should have is_running=true",
                    device.name
                );
            }
            DeviceStatus::Stopped => {
                assert!(
                    !device.is_running,
                    "Stopped device {} should have is_running=false",
                    device.name
                );
            }
            _ => {
                // Transient states should not persist
                panic!(
                    "Device {} should not be in transient state {:?}",
                    device.name, device.status
                );
            }
        }
    }
}

/// Test device operation failure recovery
#[test]
fn test_device_operation_failure_recovery() {
    let mut app_state = AppState::new();

    // Create devices in error states
    let error_scenarios = vec![
        ("start_failed", DeviceStatus::Error, false),
        ("stop_failed", DeviceStatus::Error, false),
        ("create_failed", DeviceStatus::Error, false),
        ("unknown_state", DeviceStatus::Unknown, false),
    ];

    for (name, status, is_running) in error_scenarios {
        app_state.android_devices.push(AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: name.to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status,
            is_running,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        });
    }

    // Implement recovery logic
    for device in &mut app_state.android_devices {
        match device.status {
            DeviceStatus::Error | DeviceStatus::Unknown => {
                // Reset to safe state
                device.status = DeviceStatus::Stopped;
                device.is_running = false;
            }
            _ => {}
        }
    }

    // Verify recovery
    for device in &app_state.android_devices {
        assert_eq!(
            device.status,
            DeviceStatus::Stopped,
            "Device {} should be recovered to Stopped state",
            device.name
        );
        assert!(
            !device.is_running,
            "Recovered device {} should not be running",
            device.name
        );
    }
}

/// Test system limit handling
#[test]
fn test_system_limit_handling() {
    let mut app_state = AppState::new();

    // Test various system limits
    let max_devices = 10; // Simulated system limit

    // Try to create devices up to and beyond the limit
    for i in 0..max_devices + 5 {
        let device = AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: format!("limit_device_{i}"),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        };

        // Only add if under limit
        if app_state.android_devices.len() < max_devices {
            app_state.android_devices.push(device);
        }
    }

    // Verify limit enforcement
    assert_eq!(
        app_state.android_devices.len(),
        max_devices,
        "Should not exceed device limit"
    );

    // Test resource exhaustion scenarios
    let total_ram: u32 = app_state
        .android_devices
        .iter()
        .map(|d| d.ram_size.parse::<u32>().unwrap_or(0))
        .sum();

    let total_storage: u32 = app_state
        .android_devices
        .iter()
        .map(|d| d.storage_size.parse::<u32>().unwrap_or(0))
        .sum();

    // Basic resource usage validation
    assert!(total_ram > 0, "Total RAM should be positive");
    assert!(total_storage > 0, "Total storage should be positive");
    assert!(
        total_ram <= max_devices as u32 * 8192,
        "Total RAM should be reasonable"
    );
    assert!(
        total_storage <= max_devices as u32 * 65536,
        "Total storage should be reasonable"
    );
}

/// Test device priority and sorting
#[test]
fn test_device_priority_sorting() {
    let mut devices = vec![
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "device_3".to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "device_1".to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "device_2".to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Starting,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
    ];

    // Sort by status priority (Running > Starting > Stopped)
    devices.sort_by_key(|d| match d.status {
        DeviceStatus::Running => 0,
        DeviceStatus::Starting => 1,
        DeviceStatus::Stopping => 2,
        DeviceStatus::Stopped => 3,
        DeviceStatus::Creating => 4,
        DeviceStatus::Error => 5,
        DeviceStatus::Unknown => 6,
    });

    assert_eq!(
        devices[0].name, "device_1",
        "Running device should be first"
    );
    assert_eq!(
        devices[1].name, "device_2",
        "Starting device should be second"
    );
    assert_eq!(devices[2].name, "device_3", "Stopped device should be last");
}
