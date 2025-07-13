//! Tests for CreateDeviceForm functionality
//!
//! Tests focus on form field navigation, validation, and state management
//! for both Android and iOS device creation workflows.

use emu::app::state::{CreateDeviceForm, CreateDeviceField};
use emu::constants::defaults::{DEFAULT_RAM_MB, DEFAULT_STORAGE_MB};

#[cfg(test)]
mod android_form_tests {
    use super::*;

    #[test]
    fn test_android_form_creation() {
        let form = CreateDeviceForm::for_android();
        
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
        assert_eq!(form.name, "");
        assert_eq!(form.device_type, "");
        assert_eq!(form.ram_size, DEFAULT_RAM_MB.to_string());
        assert_eq!(form.storage_size, DEFAULT_STORAGE_MB.to_string());
        assert!(!form.is_loading_cache);
        assert!(!form.is_creating);
        assert!(form.error_message.is_none());
    }

    #[test]
    fn test_android_field_navigation_forward() {
        let mut form = CreateDeviceForm::for_android();
        
        // Start at ApiLevel
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
        
        // Navigate through all fields
        form.next_field();
        assert_eq!(form.active_field, CreateDeviceField::Category);
        
        form.next_field();
        assert_eq!(form.active_field, CreateDeviceField::DeviceType);
        
        form.next_field();
        assert_eq!(form.active_field, CreateDeviceField::RamSize);
        
        form.next_field();
        assert_eq!(form.active_field, CreateDeviceField::StorageSize);
        
        form.next_field();
        assert_eq!(form.active_field, CreateDeviceField::Name);
        
        // Should cycle back to start
        form.next_field();
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
    }

    #[test]
    fn test_android_field_navigation_backward() {
        let mut form = CreateDeviceForm::for_android();
        
        // Start at ApiLevel, go back to Name
        form.prev_field();
        assert_eq!(form.active_field, CreateDeviceField::Name);
        
        form.prev_field();
        assert_eq!(form.active_field, CreateDeviceField::StorageSize);
        
        form.prev_field();
        assert_eq!(form.active_field, CreateDeviceField::RamSize);
        
        form.prev_field();
        assert_eq!(form.active_field, CreateDeviceField::DeviceType);
        
        form.prev_field();
        assert_eq!(form.active_field, CreateDeviceField::Category);
        
        form.prev_field();
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
    }

    #[test]
    fn test_android_field_navigation_cycling() {
        let mut form = CreateDeviceForm::for_android();
        
        // Test multiple cycles
        for _ in 0..3 {
            for _ in 0..6 {
                form.next_field();
            }
            assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
        }
    }
}

#[cfg(test)]
mod ios_form_tests {
    use super::*;

    #[test]
    fn test_ios_form_creation() {
        let form = CreateDeviceForm::for_ios();
        
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
        assert_eq!(form.name, "");
        assert_eq!(form.device_type, "");
        // iOS doesn't use RAM/Storage, but they're set to defaults
        assert_eq!(form.ram_size, DEFAULT_RAM_MB.to_string());
        assert_eq!(form.storage_size, DEFAULT_STORAGE_MB.to_string());
    }

    #[test]
    fn test_ios_field_navigation_forward() {
        let mut form = CreateDeviceForm::for_ios();
        
        // Start at ApiLevel
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
        
        // iOS only has 3 fields: ApiLevel -> DeviceType -> Name
        form.next_field_ios();
        assert_eq!(form.active_field, CreateDeviceField::DeviceType);
        
        form.next_field_ios();
        assert_eq!(form.active_field, CreateDeviceField::Name);
        
        // Should cycle back to start
        form.next_field_ios();
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
    }

    #[test]
    fn test_ios_field_navigation_backward() {
        let mut form = CreateDeviceForm::for_ios();
        
        // Start at ApiLevel, go back to Name
        form.prev_field_ios();
        assert_eq!(form.active_field, CreateDeviceField::Name);
        
        form.prev_field_ios();
        assert_eq!(form.active_field, CreateDeviceField::DeviceType);
        
        form.prev_field_ios();
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
    }

    #[test]
    fn test_ios_field_navigation_cycling() {
        let mut form = CreateDeviceForm::for_ios();
        
        // Test multiple cycles (3 fields)
        for _ in 0..5 {
            for _ in 0..3 {
                form.next_field_ios();
            }
            assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
        }
    }

    #[test]
    fn test_ios_fallback_fields() {
        let mut form = CreateDeviceForm::for_ios();
        
        // Set to Android-only field and test fallback behavior
        form.active_field = CreateDeviceField::Category;
        form.next_field_ios();
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
        
        form.active_field = CreateDeviceField::RamSize;
        form.prev_field_ios();
        assert_eq!(form.active_field, CreateDeviceField::Name);
    }
}

#[cfg(test)]
mod form_state_tests {
    use super::*;

    #[test]
    fn test_default_form_state() {
        let form = CreateDeviceForm::default();
        
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
        assert!(form.name.is_empty());
        assert!(form.device_type.is_empty());
        assert!(form.device_type_id.is_empty());
        assert!(form.version.is_empty());
        assert!(form.version_display.is_empty());
        assert!(form.available_device_types.is_empty());
        assert!(form.available_versions.is_empty());
        assert_eq!(form.selected_api_level_index, 0);
        assert_eq!(form.selected_device_type_index, 0);
        assert!(form.error_message.is_none());
        assert!(!form.is_loading_cache);
        assert!(!form.is_creating);
        assert!(form.creation_status.is_none());
    }

    #[test]
    fn test_form_field_modification() {
        let mut form = CreateDeviceForm::default();
        
        // Test field modifications
        form.name = "TestDevice".to_string();
        form.device_type = "Pixel 4".to_string();
        form.device_type_id = "pixel_4".to_string();
        form.version = "30".to_string();
        form.version_display = "API 30".to_string();
        form.ram_size = "4096".to_string();
        form.storage_size = "8192".to_string();
        
        assert_eq!(form.name, "TestDevice");
        assert_eq!(form.device_type, "Pixel 4");
        assert_eq!(form.device_type_id, "pixel_4");
        assert_eq!(form.version, "30");
        assert_eq!(form.version_display, "API 30");
        assert_eq!(form.ram_size, "4096");
        assert_eq!(form.storage_size, "8192");
    }

    #[test]
    fn test_form_error_state() {
        let mut form = CreateDeviceForm::default();
        
        // Test error state
        form.error_message = Some("Invalid configuration".to_string());
        assert_eq!(form.error_message, Some("Invalid configuration".to_string()));
        
        // Clear error
        form.error_message = None;
        assert!(form.error_message.is_none());
    }

    #[test]
    fn test_form_loading_state() {
        let mut form = CreateDeviceForm::default();
        
        // Test loading states
        form.is_loading_cache = true;
        form.is_creating = true;
        form.creation_status = Some("Creating device...".to_string());
        
        assert!(form.is_loading_cache);
        assert!(form.is_creating);
        assert_eq!(form.creation_status, Some("Creating device...".to_string()));
    }

    #[test]
    fn test_form_selection_indices() {
        let mut form = CreateDeviceForm::default();
        
        // Test selection indices
        form.selected_api_level_index = 5;
        form.selected_device_type_index = 3;
        
        assert_eq!(form.selected_api_level_index, 5);
        assert_eq!(form.selected_device_type_index, 3);
    }
}

#[cfg(test)]
mod form_validation_tests {
    use super::*;

    #[test]
    fn test_move_selection_methods() {
        let mut form = CreateDeviceForm::default();
        
        // These methods always return false (placeholder behavior)
        assert!(!form.move_selection_up());
        assert!(!form.move_selection_down());
        
        // State should not change
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
    }

    #[test]
    fn test_form_data_persistence() {
        let mut form = CreateDeviceForm::for_android();
        
        // Set form data
        form.name = "MyDevice".to_string();
        form.ram_size = "8192".to_string();
        form.active_field = CreateDeviceField::Name;
        
        // Navigate through fields - data should persist
        form.next_field();
        form.next_field();
        form.prev_field();
        
        assert_eq!(form.name, "MyDevice");
        assert_eq!(form.ram_size, "8192");
    }

    #[test]
    fn test_available_options_management() {
        let mut form = CreateDeviceForm::default();
        
        // Test available options
        form.available_device_types = vec![
            ("pixel_4".to_string(), "Pixel 4".to_string()),
            ("pixel_5".to_string(), "Pixel 5".to_string()),
        ];
        
        form.available_versions = vec![
            ("29".to_string(), "API 29".to_string()),
            ("30".to_string(), "API 30".to_string()),
            ("31".to_string(), "API 31".to_string()),
        ];
        
        assert_eq!(form.available_device_types.len(), 2);
        assert_eq!(form.available_versions.len(), 3);
        
        // Test specific entries
        assert_eq!(form.available_device_types[0].0, "pixel_4");
        assert_eq!(form.available_device_types[0].1, "Pixel 4");
        assert_eq!(form.available_versions[1].0, "30");
        assert_eq!(form.available_versions[1].1, "API 30");
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_mixed_navigation_methods() {
        let mut form = CreateDeviceForm::for_android();
        
        // Mix Android and iOS navigation methods
        form.next_field(); // Android method
        assert_eq!(form.active_field, CreateDeviceField::Category);
        
        form.next_field_ios(); // iOS method from Category
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel); // Should fallback to ApiLevel
        
        form.active_field = CreateDeviceField::DeviceType;
        form.prev_field_ios(); // iOS method
        assert_eq!(form.active_field, CreateDeviceField::ApiLevel);
    }

    #[test]
    fn test_field_enum_coverage() {
        // Ensure all CreateDeviceField variants are handled
        let fields = vec![
            CreateDeviceField::ApiLevel,
            CreateDeviceField::Category,
            CreateDeviceField::DeviceType,
            CreateDeviceField::RamSize,
            CreateDeviceField::StorageSize,
            CreateDeviceField::Name,
        ];
        
        for field in fields {
            let mut form = CreateDeviceForm::default();
            form.active_field = field;
            
            // Should not panic
            form.next_field();
            form.prev_field();
            form.next_field_ios();
            form.prev_field_ios();
        }
    }

    #[test]
    fn test_empty_string_handling() {
        let mut form = CreateDeviceForm::default();
        
        // Test empty string handling
        form.name = "".to_string();
        form.device_type = "".to_string();
        form.ram_size = "".to_string();
        form.storage_size = "".to_string();
        
        // Should handle empty strings gracefully
        assert_eq!(form.name, "");
        assert_eq!(form.device_type, "");
        assert_eq!(form.ram_size, "");
        assert_eq!(form.storage_size, "");
    }
}