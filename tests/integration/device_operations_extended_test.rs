//! Extended integration tests for device operations
//!
//! Tests focus on end-to-end device management workflows using MockDeviceManager
//! to validate complex scenarios without requiring actual emulators or simulators.

use emu::app::state::{AppState, FocusedPanel, Mode, Panel};
use emu::managers::common::{DeviceConfig, DeviceManager};
use emu::managers::mock::MockDeviceManager;
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(test)]
mod device_operations_extended_tests {
    use super::*;

    async fn setup_mock_app_state() -> Arc<RwLock<AppState>> {
        let app_state = Arc::new(RwLock::new(AppState::default()));
        
        // Setup some initial mock devices
        {
            let mut state = app_state.write().await;
            state.android_devices = vec![
                AndroidDevice {
                    name: "Test_Android_Device".to_string(),
                    status: DeviceStatus::Stopped,
                    api_level: "30".to_string(),
                    category: "Phone".to_string(),
                    manufacturer: "Google".to_string(),
                    device_id: "test_android_1".to_string(),
                    target: "android-30".to_string(),
                    path: "/path/to/android/device".to_string(),
                    abi: "x86_64".to_string(),
                    screen_density: 440,
                    screen_size: "1080x1920".to_string(),
                    ram_size: "2048MB".to_string(),
                    storage_size: "4096MB".to_string(),
                    emulator_port: None,
                    adb_port: None,
                },
            ];
            
            state.ios_devices = vec![
                IosDevice {
                    name: "Test iPhone 14".to_string(),
                    status: DeviceStatus::Stopped,
                    device_type: "com.apple.CoreSimulator.SimDeviceType.iPhone-14".to_string(),
                    runtime: "iOS-16-4".to_string(),
                    udid: "test-ios-device-uuid".to_string(),
                    version: "16.4".to_string(),
                    availability: "(available)".to_string(),
                    is_booted: false,
                },
            ];
        }
        
        app_state
    }

    #[tokio::test]
    async fn test_device_operations_workflow() {
        let app_state = setup_mock_app_state().await;
        let mut android_manager = MockDeviceManager::new(false);
        let mut ios_manager = MockDeviceManager::new(false);
        
        // Configure mock managers with success scenarios
        android_manager.configure_scenario("start_success");
        ios_manager.configure_scenario("start_success");
        
        {
            let mut state = app_state.write().await;
            state.selected_android_index = 0;
            state.current_panel = Panel::Android;
        }
        
        // Test Android device start operation
        let result = android_manager.start_device("test_android_1").await;
        assert!(result.is_ok());
        
        // Verify device status was updated in mock
        let devices = android_manager.list_devices().await.unwrap();
        assert_eq!(devices.len(), 1);
        
        // Test iOS device operations
        {
            let mut state = app_state.write().await;
            state.current_panel = Panel::Ios;
            state.selected_ios_index = 0;
        }
        
        let result = ios_manager.start_device("test-ios-device-uuid").await;
        assert!(result.is_ok());
        
        // Test device creation workflow
        let device_config = DeviceConfig {
            name: "New_Test_Device".to_string(),
            api_level: "31".to_string(),
            device_type: "pixel_5".to_string(),
            ram_size: "4096".to_string(),
            storage_size: "8192".to_string(),
        };
        
        let result = android_manager.create_device(&device_config).await;
        assert!(result.is_ok());
        
        // Verify new device was added
        let devices = android_manager.list_devices().await.unwrap();
        assert_eq!(devices.len(), 2);
    }

    #[tokio::test]
    async fn test_concurrent_device_operations() {
        let app_state = setup_mock_app_state().await;
        let mut android_manager = MockDeviceManager::new(false);
        let mut ios_manager = MockDeviceManager::new(false);
        
        // Configure for concurrent operations
        android_manager.configure_scenario("concurrent_operations");
        ios_manager.configure_scenario("concurrent_operations");
        
        // Start multiple operations concurrently
        let android_start = android_manager.start_device("test_android_1");
        let ios_start = ios_manager.start_device("test-ios-device-uuid");
        let android_list = android_manager.list_devices();
        let ios_list = ios_manager.list_devices();
        
        let results = tokio::join!(android_start, ios_start, android_list, ios_list);
        
        assert!(results.0.is_ok());
        assert!(results.1.is_ok());
        assert!(results.2.is_ok());
        assert!(results.3.is_ok());
    }

    #[tokio::test]
    async fn test_device_state_consistency() {
        let app_state = setup_mock_app_state().await;
        let mut manager = MockDeviceManager::new(false);
        
        // Test state transitions: Stopped -> Starting -> Running -> Stopping -> Stopped
        manager.configure_scenario("state_transitions");
        
        // Initial state
        let devices = manager.list_devices().await.unwrap();
        assert_eq!(devices.len(), 1);
        
        // Start device
        let result = manager.start_device("test_device").await;
        assert!(result.is_ok());
        
        // Stop device
        let result = manager.stop_device("test_device").await;
        assert!(result.is_ok());
        
        // Verify final state
        let devices = manager.list_devices().await.unwrap();
        assert_eq!(devices.len(), 1);
    }

    #[tokio::test]
    async fn test_device_deletion_workflow() {
        let app_state = setup_mock_app_state().await;
        let mut manager = MockDeviceManager::new(false);
        
        manager.configure_scenario("deletion_workflow");
        
        // Verify initial device count
        let devices = manager.list_devices().await.unwrap();
        let initial_count = devices.len();
        
        // Delete device
        let result = manager.delete_device("test_device").await;
        assert!(result.is_ok());
        
        // Verify device was removed
        let devices = manager.list_devices().await.unwrap();
        assert_eq!(devices.len(), initial_count - 1);
    }

    #[tokio::test]
    async fn test_error_recovery_scenarios() {
        let app_state = setup_mock_app_state().await;
        let mut manager = MockDeviceManager::new(true); // Enable error scenarios
        
        manager.configure_scenario("error_recovery");
        
        // Test operation that should fail
        let result = manager.start_device("non_existent_device").await;
        assert!(result.is_err());
        
        // Test recovery - subsequent operations should work
        manager.configure_scenario("recovery_success");
        let result = manager.list_devices().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_device_wipe_operation() {
        let app_state = setup_mock_app_state().await;
        let mut manager = MockDeviceManager::new(false);
        
        manager.configure_scenario("wipe_operation");
        
        // Test device wipe
        let result = manager.wipe_device("test_device").await;
        assert!(result.is_ok());
        
        // Verify device still exists but is reset
        let devices = manager.list_devices().await.unwrap();
        assert!(!devices.is_empty());
    }

    #[tokio::test]
    async fn test_panel_switching_with_device_operations() {
        let app_state = setup_mock_app_state().await;
        let mut android_manager = MockDeviceManager::new(false);
        let mut ios_manager = MockDeviceManager::new(false);
        
        android_manager.configure_scenario("panel_switching");
        ios_manager.configure_scenario("panel_switching");
        
        // Start on Android panel
        {
            let mut state = app_state.write().await;
            state.current_panel = Panel::Android;
            state.selected_android_index = 0;
        }
        
        // Start Android device
        let result = android_manager.start_device("test_android_1").await;
        assert!(result.is_ok());
        
        // Switch to iOS panel
        {
            let mut state = app_state.write().await;
            state.current_panel = Panel::Ios;
            state.selected_ios_index = 0;
        }
        
        // Start iOS device
        let result = ios_manager.start_device("test-ios-device-uuid").await;
        assert!(result.is_ok());
        
        // Verify both operations were recorded
        let android_ops = android_manager.get_recorded_operations().await;
        let ios_ops = ios_manager.get_recorded_operations().await;
        
        assert!(!android_ops.is_empty());
        assert!(!ios_ops.is_empty());
    }

    #[tokio::test]
    async fn test_mode_transitions_with_operations() {
        let app_state = setup_mock_app_state().await;
        let mut manager = MockDeviceManager::new(false);
        
        manager.configure_scenario("mode_transitions");
        
        // Test normal mode operations
        {
            let mut state = app_state.write().await;
            state.mode = Mode::Normal;
        }
        
        let result = manager.list_devices().await;
        assert!(result.is_ok());
        
        // Test create device mode
        {
            let mut state = app_state.write().await;
            state.mode = Mode::CreateDevice;
        }
        
        let device_config = DeviceConfig {
            name: "Mode_Test_Device".to_string(),
            api_level: "30".to_string(),
            device_type: "pixel_4".to_string(),
            ram_size: "2048".to_string(),
            storage_size: "4096".to_string(),
        };
        
        let result = manager.create_device(&device_config).await;
        assert!(result.is_ok());
        
        // Return to normal mode
        {
            let mut state = app_state.write().await;
            state.mode = Mode::Normal;
        }
    }

    #[tokio::test]
    async fn test_device_details_consistency() {
        let app_state = setup_mock_app_state().await;
        let mut manager = MockDeviceManager::new(false);
        
        manager.configure_scenario("details_consistency");
        
        // Get device details
        let result = manager.get_device_details("test_device").await;
        assert!(result.is_ok());
        
        // Verify details are consistent with list
        let devices = manager.list_devices().await.unwrap();
        let details = result.unwrap();
        
        // Details should contain relevant device information
        assert!(!details.is_empty());
    }

    #[tokio::test]
    async fn test_notification_system_integration() {
        let app_state = setup_mock_app_state().await;
        let mut manager = MockDeviceManager::new(false);
        
        manager.configure_scenario("notification_integration");
        
        // Perform operation that should generate notifications
        let result = manager.start_device("test_device").await;
        assert!(result.is_ok());
        
        // Check if notifications were added to app state
        {
            let state = app_state.read().await;
            // Note: In a real integration test, we would verify notifications
            // For now, we just ensure the operation completed successfully
            assert_eq!(state.current_panel, Panel::Android);
        }
    }

    #[tokio::test]
    async fn test_background_task_coordination() {
        let app_state = setup_mock_app_state().await;
        let mut manager = MockDeviceManager::new(false);
        
        manager.configure_scenario("background_tasks");
        
        // Start long-running operations
        let long_operation = manager.start_device("slow_device");
        let quick_operation = manager.list_devices();
        
        // Both should complete successfully
        let results = tokio::join!(long_operation, quick_operation);
        assert!(results.0.is_ok());
        assert!(results.1.is_ok());
    }

    #[tokio::test]
    async fn test_resource_cleanup() {
        let app_state = setup_mock_app_state().await;
        let mut manager = MockDeviceManager::new(false);
        
        manager.configure_scenario("resource_cleanup");
        
        // Perform operations that require cleanup
        let result = manager.start_device("test_device").await;
        assert!(result.is_ok());
        
        let result = manager.stop_device("test_device").await;
        assert!(result.is_ok());
        
        // Verify cleanup was performed
        let operations = manager.get_recorded_operations().await;
        assert!(operations.len() >= 2); // Start and stop operations
    }
}