//! Unit tests for defaults.rs module
//!
//! This module contains comprehensive tests for the defaults.rs constants module,
//! focusing on the executable code and runtime behavior.

use emu::constants::defaults::*;
use std::time::Duration;

#[cfg(test)]
mod defaults_function_tests {
    use super::*;

    #[test]
    fn test_default_abi_returns_valid_string() {
        let abi = default_abi();
        assert!(!abi.is_empty(), "default_abi() should return a non-empty string");
        assert!(abi.len() > 0, "ABI string should have length > 0");
    }

    #[test]
    fn test_default_abi_returns_expected_values() {
        let abi = default_abi();
        
        // Test that it returns one of the expected values
        let valid_abis = ["x86_64", "arm64-v8a"];
        assert!(
            valid_abis.contains(&abi),
            "default_abi() returned '{}', expected one of: {:?}",
            abi,
            valid_abis
        );
    }

    #[test]
    fn test_default_abi_architecture_specific() {
        let abi = default_abi();
        
        // Test the specific architecture mappings
        #[cfg(target_arch = "x86_64")]
        assert_eq!(abi, "x86_64", "On x86_64, default_abi() should return 'x86_64'");
        
        #[cfg(target_arch = "aarch64")]
        assert_eq!(abi, "arm64-v8a", "On aarch64, default_abi() should return 'arm64-v8a'");
        
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        assert_eq!(abi, "x86_64", "On other architectures, default_abi() should return 'x86_64' as fallback");
    }

    #[test]
    fn test_default_abi_is_consistent() {
        // Test that multiple calls return the same value
        let abi1 = default_abi();
        let abi2 = default_abi();
        assert_eq!(abi1, abi2, "default_abi() should return consistent values");
    }

    #[test]
    fn test_default_abi_static_lifetime() {
        // Test that the returned string has static lifetime
        let abi: &'static str = default_abi();
        assert!(!abi.is_empty(), "Static string should not be empty");
        
        // Test that we can use the string in different contexts
        let owned_string = abi.to_string();
        assert_eq!(owned_string, abi, "Static string should convert to owned string correctly");
    }
}

#[cfg(test)]
mod constant_value_tests {
    use super::*;

    #[test]
    fn test_ram_constants() {
        assert_eq!(DEFAULT_RAM_MB, 2048, "DEFAULT_RAM_MB should be 2048");
        assert!(DEFAULT_RAM_MB > 0, "DEFAULT_RAM_MB should be positive");
        assert!(DEFAULT_RAM_MB <= 8192, "DEFAULT_RAM_MB should be reasonable");
    }

    #[test]
    fn test_storage_constants() {
        assert_eq!(DEFAULT_STORAGE_MB, 8192, "DEFAULT_STORAGE_MB should be 8192");
        assert!(DEFAULT_STORAGE_MB > 0, "DEFAULT_STORAGE_MB should be positive");
        assert!(DEFAULT_STORAGE_MB >= DEFAULT_RAM_MB, "Storage should be at least as large as RAM");
    }

    #[test]
    fn test_gpu_mode_constant() {
        assert_eq!(DEFAULT_GPU_MODE, "auto", "DEFAULT_GPU_MODE should be 'auto'");
        assert!(!DEFAULT_GPU_MODE.is_empty(), "DEFAULT_GPU_MODE should not be empty");
    }

    #[test]
    fn test_skin_constant() {
        assert_eq!(DEFAULT_SKIN, "pixel_5", "DEFAULT_SKIN should be 'pixel_5'");
        assert!(!DEFAULT_SKIN.is_empty(), "DEFAULT_SKIN should not be empty");
    }

    #[test]
    fn test_duration_constants() {
        assert_eq!(CACHE_EXPIRATION, Duration::from_secs(300), "CACHE_EXPIRATION should be 5 minutes");
        assert_eq!(DEVICE_REFRESH_INTERVAL, Duration::from_secs(5), "DEVICE_REFRESH_INTERVAL should be 5 seconds");
        assert_eq!(NOTIFICATION_DURATION, Duration::from_secs(3), "NOTIFICATION_DURATION should be 3 seconds");
        
        // Test that durations are reasonable
        assert!(CACHE_EXPIRATION.as_secs() > 0, "CACHE_EXPIRATION should be positive");
        assert!(DEVICE_REFRESH_INTERVAL.as_secs() > 0, "DEVICE_REFRESH_INTERVAL should be positive");
        assert!(NOTIFICATION_DURATION.as_secs() > 0, "NOTIFICATION_DURATION should be positive");
    }

    #[test]
    fn test_api_levels_array() {
        assert_eq!(DEFAULT_API_LEVELS, &[35, 34, 33, 32, 31, 30, 29, 28], "DEFAULT_API_LEVELS should match expected array");
        assert!(!DEFAULT_API_LEVELS.is_empty(), "DEFAULT_API_LEVELS should not be empty");
        assert!(DEFAULT_API_LEVELS.len() > 0, "DEFAULT_API_LEVELS should have elements");
        
        // Test that API levels are in descending order
        for i in 1..DEFAULT_API_LEVELS.len() {
            assert!(
                DEFAULT_API_LEVELS[i-1] > DEFAULT_API_LEVELS[i],
                "API levels should be in descending order: {} should be > {}",
                DEFAULT_API_LEVELS[i-1],
                DEFAULT_API_LEVELS[i]
            );
        }
        
        // Test that API levels are reasonable
        for &level in DEFAULT_API_LEVELS {
            assert!(level >= 21, "API level {} should be >= 21", level);
            assert!(level <= 40, "API level {} should be <= 40", level);
        }
    }

    #[test]
    fn test_log_level_constant() {
        assert_eq!(DEFAULT_LOG_LEVEL, "info", "DEFAULT_LOG_LEVEL should be 'info'");
        assert!(!DEFAULT_LOG_LEVEL.is_empty(), "DEFAULT_LOG_LEVEL should not be empty");
    }

    #[test]
    fn test_android_logging_constant() {
        assert_eq!(ANDROID_LOGGING_DISABLED_VALUE, "0", "ANDROID_LOGGING_DISABLED_VALUE should be '0'");
        assert!(!ANDROID_LOGGING_DISABLED_VALUE.is_empty(), "ANDROID_LOGGING_DISABLED_VALUE should not be empty");
    }

    #[test]
    fn test_test_device_constants() {
        assert_eq!(TEST_DEVICE_NAME_BASE, "test_debug_device", "TEST_DEVICE_NAME_BASE should be 'test_debug_device'");
        assert_eq!(TEST_DEVICE_NAME_33, "test_debug_device_33", "TEST_DEVICE_NAME_33 should be 'test_debug_device_33'");
        assert_eq!(TEST_DEVICE_TYPE, "phone", "TEST_DEVICE_TYPE should be 'phone'");
        assert_eq!(TEST_API_LEVEL_34, "34", "TEST_API_LEVEL_34 should be '34'");
        assert_eq!(TEST_API_LEVEL_33, "33", "TEST_API_LEVEL_33 should be '33'");
        
        // Test that test constants are not empty
        assert!(!TEST_DEVICE_NAME_BASE.is_empty(), "TEST_DEVICE_NAME_BASE should not be empty");
        assert!(!TEST_DEVICE_NAME_33.is_empty(), "TEST_DEVICE_NAME_33 should not be empty");
        assert!(!TEST_DEVICE_TYPE.is_empty(), "TEST_DEVICE_TYPE should not be empty");
        assert!(!TEST_API_LEVEL_34.is_empty(), "TEST_API_LEVEL_34 should not be empty");
        assert!(!TEST_API_LEVEL_33.is_empty(), "TEST_API_LEVEL_33 should not be empty");
    }
}

#[cfg(test)]
mod duration_arithmetic_tests {
    use super::*;

    #[test]
    fn test_duration_relationships() {
        // Test logical relationships between durations
        assert!(CACHE_EXPIRATION > DEVICE_REFRESH_INTERVAL, "Cache expiration should be longer than device refresh");
        assert!(DEVICE_REFRESH_INTERVAL > NOTIFICATION_DURATION, "Device refresh should be longer than notification");
        
        // Test duration arithmetic
        assert!(CACHE_EXPIRATION.as_secs() % DEVICE_REFRESH_INTERVAL.as_secs() == 0, 
               "Cache expiration should be a multiple of device refresh interval");
    }

    #[test]
    fn test_duration_conversions() {
        // Test various duration conversions
        assert_eq!(CACHE_EXPIRATION.as_millis(), 300_000, "CACHE_EXPIRATION should be 300,000 milliseconds");
        assert_eq!(DEVICE_REFRESH_INTERVAL.as_millis(), 5_000, "DEVICE_REFRESH_INTERVAL should be 5,000 milliseconds");
        assert_eq!(NOTIFICATION_DURATION.as_millis(), 3_000, "NOTIFICATION_DURATION should be 3,000 milliseconds");
    }
}

#[cfg(test)]
mod memory_size_tests {
    use super::*;

    #[test]
    fn test_memory_size_logic() {
        // Test that memory sizes are powers of 2 or common sizes
        assert_eq!(DEFAULT_RAM_MB, 2048, "DEFAULT_RAM_MB should be 2048 (2^11)");
        assert_eq!(DEFAULT_STORAGE_MB, 8192, "DEFAULT_STORAGE_MB should be 8192 (2^13)");
        
        // Test that storage is reasonable multiple of RAM
        assert_eq!(DEFAULT_STORAGE_MB / DEFAULT_RAM_MB, 4, "Storage should be 4x RAM size");
    }

    #[test]
    fn test_memory_size_conversions() {
        // Test MB to GB conversions
        assert_eq!(DEFAULT_RAM_MB / 1024, 2, "DEFAULT_RAM_MB should be 2 GB");
        assert_eq!(DEFAULT_STORAGE_MB / 1024, 8, "DEFAULT_STORAGE_MB should be 8 GB");
    }
}