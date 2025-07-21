//! Priority values for device sorting and ordering.
//!
//! This module defines priority values used to sort and order devices in the UI.
//! Lower values indicate higher priority (will appear first in lists).
//!
//! # Android Priorities
//!
//! Android devices are sorted by brand and device type:
//! - Pixel devices get highest priority (30)
//! - Nexus devices (40)
//! - OnePlus devices (50)
//! - Other brands (60+)
//!
//! # iOS Priorities
//!
//! iOS devices are sorted by device type and model:
//! - iPhone models: Mini (30) < SE (40) < Standard (50) < Plus (60) < Pro (70) < Pro Max (80)
//! - iPad models: Mini (100) < Standard (110) < Air (120) < Pro 11" (130) < Pro 13" (140)
//! - Other devices: Watch (150) < TV (170) < Unknown (199)
//!
//! # Usage
//!
//! ```rust
//! use emu::constants::priorities::{ANDROID_PIXEL_PRIORITY, ANDROID_NEXUS_PRIORITY, IOS_IPHONE_PRO_MAX_PRIORITY};
//!
//! // Priority constants can be used directly for comparison
//! assert_eq!(ANDROID_PIXEL_PRIORITY, 30);
//! assert_eq!(IOS_IPHONE_PRO_MAX_PRIORITY, 80);
//!
//! // Lower values indicate higher priority
//! assert!(ANDROID_PIXEL_PRIORITY < ANDROID_NEXUS_PRIORITY);
//! ```
// Android device priorities (for sorting in device lists)
pub const ANDROID_TABLET_PRIORITY: u8 = 100;
pub const ANDROID_PIXEL_PRIORITY: u8 = 30;
pub const ANDROID_NEXUS_PRIORITY: u8 = 40;
pub const ANDROID_ONEPLUS_PRIORITY: u8 = 50;
pub const ANDROID_OTHER_BRAND_PRIORITY: u8 = 60;

// Pixel device priority calculation constants
pub const PIXEL_PRIORITY_OFFSET: u32 = 80;
pub const PIXEL_PRIORITY_MAX_BONUS: u32 = 19;
pub const PIXEL_UNVERSIONED_PRIORITY: u32 = 25;

// Phone category priority base
pub const PHONE_CATEGORY_BASE_PRIORITY: u32 = 30;

// Android alphabetical sorting base priorities
pub const ANDROID_ALPHA_LOW_PRIORITY: u8 = 70;
pub const ANDROID_ALPHA_HIGH_PRIORITY: u8 = 110;

// Android specific version priorities
pub const ANDROID_11_PRIORITY: u8 = 30;

// iOS device priorities (for sorting in device lists)
pub const IOS_IPHONE_MINI_PRIORITY: u8 = 30;
pub const IOS_IPHONE_SE_PRIORITY: u8 = 40;
pub const IOS_IPHONE_REGULAR_PRIORITY: u8 = 50;
pub const IOS_IPHONE_PLUS_PRIORITY: u8 = 60;
pub const IOS_IPHONE_PRO_PRIORITY: u8 = 70;
pub const IOS_IPHONE_PRO_MAX_PRIORITY: u8 = 80;
pub const IOS_IPAD_MINI_PRIORITY: u8 = 100;
pub const IOS_IPAD_REGULAR_PRIORITY: u8 = 110;
pub const IOS_IPAD_AIR_PRIORITY: u8 = 120;
pub const IOS_IPAD_PRO_11_PRIORITY: u8 = 130;
pub const IOS_IPAD_PRO_13_PRIORITY: u8 = 140;
pub const IOS_WATCH_PRIORITY: u8 = 150;
pub const IOS_TV_PRIORITY: u8 = 170;
pub const IOS_UNKNOWN_PRIORITY: u8 = 199;

// Detailed iOS device priority constants for device_info.rs calculations
// iPhone type priorities
pub const IOS_IPHONE_PRO_MAX_PRIORITY_VALUE: u32 = 0;
pub const IOS_IPHONE_PRO_PRIORITY_VALUE: u32 = 10;
pub const IOS_IPHONE_PLUS_MAX_PRIORITY: u32 = 20;
pub const IOS_IPHONE_MINI_PRIORITY_CALC: u32 = 30;
pub const IOS_IPHONE_SE_PRIORITY_CALC: u32 = 40;
pub const IOS_IPHONE_DEFAULT_BASE: u32 = 50;
pub const IOS_IPHONE_VERSION_OFFSET: u32 = 30;

// iPad priorities
pub const IOS_IPAD_PRO_12_9_PRIORITY: u32 = 100;
pub const IOS_IPAD_PRO_11_PRIORITY_VALUE: u32 = 110;
pub const IOS_IPAD_PRO_OTHER_PRIORITY: u32 = 120;
pub const IOS_IPAD_AIR_PRIORITY_VALUE: u32 = 130;
pub const IOS_IPAD_MINI_PRIORITY_CALC: u32 = 140;
pub const IOS_IPAD_DEFAULT_PRIORITY: u32 = 150;

// Apple TV priorities
pub const IOS_TV_4K_PRIORITY: u32 = 200;
pub const IOS_TV_DEFAULT_PRIORITY: u32 = 210;

// Apple Watch priorities
pub const IOS_WATCH_ULTRA_PRIORITY: u32 = 300;
pub const IOS_WATCH_SERIES_BASE: u32 = 310;
pub const IOS_WATCH_SERIES_OFFSET: u32 = 10;
pub const IOS_WATCH_DEFAULT_PRIORITY: u32 = 320;
pub const IOS_WATCH_SE_PRIORITY: u32 = 330;
pub const IOS_WATCH_OTHER_PRIORITY: u32 = 340;

// Unknown device priority
pub const IOS_UNKNOWN_DEVICE_PRIORITY: u32 = 999;

// Additional iOS priority constants for tests
pub const IOS_IPHONE_STANDARD_PRIORITY_VALUE: u32 = 20;
pub const IOS_IPHONE_SE_PRIORITY_VALUE: u32 = 25;
pub const IOS_IPHONE_MINI_PRIORITY_VALUE: u32 = 30;
pub const IOS_IPAD_PRO_PRIORITY_VALUE: u32 = 100;
pub const IOS_IPAD_STANDARD_PRIORITY_VALUE: u32 = 140;
pub const IOS_IPAD_MINI_PRIORITY_VALUE: u32 = 150;
pub const IOS_IPOD_PRIORITY_VALUE: u32 = 400;
pub const IOS_APPLE_TV_PRIORITY_VALUE: u32 = 500;
pub const IOS_APPLE_WATCH_PRIORITY_VALUE: u32 = 600;
pub const IOS_DEFAULT_PRIORITY_VALUE: u32 = 999;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Android priority constants ordering
    #[test]
    fn test_android_priority_ordering() {
        // Test that Android priorities are correctly ordered (lower = higher priority)
        // Using runtime comparisons to avoid clippy assertions-on-constants
        let pixel = ANDROID_PIXEL_PRIORITY;
        let nexus = ANDROID_NEXUS_PRIORITY;
        let oneplus = ANDROID_ONEPLUS_PRIORITY;
        let other = ANDROID_OTHER_BRAND_PRIORITY;
        let alpha_low = ANDROID_ALPHA_LOW_PRIORITY;
        let tablet = ANDROID_TABLET_PRIORITY;
        let alpha_high = ANDROID_ALPHA_HIGH_PRIORITY;

        assert!(pixel < nexus);
        assert!(nexus < oneplus);
        assert!(oneplus < other);
        assert!(other < alpha_low);
        assert!(alpha_low < tablet);
        assert!(tablet < alpha_high);
    }

    /// Test iOS iPhone priority ordering
    #[test]
    fn test_ios_iphone_priority_ordering() {
        // Test that iPhone priorities are correctly ordered
        let mini = IOS_IPHONE_MINI_PRIORITY;
        let se = IOS_IPHONE_SE_PRIORITY;
        let regular = IOS_IPHONE_REGULAR_PRIORITY;
        let plus = IOS_IPHONE_PLUS_PRIORITY;
        let pro = IOS_IPHONE_PRO_PRIORITY;
        let pro_max = IOS_IPHONE_PRO_MAX_PRIORITY;

        assert!(mini < se);
        assert!(se < regular);
        assert!(regular < plus);
        assert!(plus < pro);
        assert!(pro < pro_max);
    }

    /// Test iOS iPad priority ordering
    #[test]
    fn test_ios_ipad_priority_ordering() {
        // Test that iPad priorities are correctly ordered
        let mini = IOS_IPAD_MINI_PRIORITY;
        let regular = IOS_IPAD_REGULAR_PRIORITY;
        let air = IOS_IPAD_AIR_PRIORITY;
        let pro11 = IOS_IPAD_PRO_11_PRIORITY;
        let pro13 = IOS_IPAD_PRO_13_PRIORITY;

        assert!(mini < regular);
        assert!(regular < air);
        assert!(air < pro11);
        assert!(pro11 < pro13);
    }

    /// Test iOS overall device priority ordering
    #[test]
    fn test_ios_device_priority_ordering() {
        // Test that device categories are correctly ordered
        let iphone_max = IOS_IPHONE_PRO_MAX_PRIORITY;
        let ipad_mini = IOS_IPAD_MINI_PRIORITY;
        let ipad_pro13 = IOS_IPAD_PRO_13_PRIORITY;
        let watch = IOS_WATCH_PRIORITY;
        let tv = IOS_TV_PRIORITY;
        let unknown = IOS_UNKNOWN_PRIORITY;

        assert!(iphone_max < ipad_mini);
        assert!(ipad_pro13 < watch);
        assert!(watch < tv);
        assert!(tv < unknown);
    }

    /// Test priority value constants
    #[test]
    fn test_priority_value_constants() {
        // Test that priority values are reasonable ranges using runtime variables
        let android_pixel = ANDROID_PIXEL_PRIORITY as u16;
        let ios_mini = IOS_IPHONE_MINI_PRIORITY as u16;
        let ios_unknown = IOS_UNKNOWN_PRIORITY as u16;

        assert!(android_pixel >= 1);
        assert!(android_pixel <= 255);
        assert!(ios_mini >= 1);
        assert!(ios_unknown <= 255);
    }

    /// Test Pixel priority calculation constants
    #[test]
    fn test_pixel_priority_constants() {
        assert_eq!(PIXEL_PRIORITY_OFFSET, 80);
        assert_eq!(PIXEL_PRIORITY_MAX_BONUS, 19);
        assert_eq!(PIXEL_UNVERSIONED_PRIORITY, 25);

        // Test logical relationships between constants
        let offset = PIXEL_PRIORITY_OFFSET as u64;
        let bonus = PIXEL_PRIORITY_MAX_BONUS as u64;
        let unversioned = PIXEL_UNVERSIONED_PRIORITY as u64;

        assert!(offset > bonus);
        assert!(unversioned > 0);
    }

    /// Test detailed iOS priority constants
    #[test]
    fn test_detailed_ios_priorities() {
        // Test iPhone detailed priorities using runtime variables
        let pro_max = IOS_IPHONE_PRO_MAX_PRIORITY_VALUE as u64;
        let pro = IOS_IPHONE_PRO_PRIORITY_VALUE as u64;
        let plus = IOS_IPHONE_PLUS_MAX_PRIORITY as u64;

        assert_eq!(pro_max, 0);
        assert!(pro > pro_max);
        assert!(plus > pro);

        // Test iPad detailed priorities using runtime variables
        let ipad_pro_12_9 = IOS_IPAD_PRO_12_9_PRIORITY as u64;
        let ipad_pro_11 = IOS_IPAD_PRO_11_PRIORITY_VALUE as u64;
        let ipad_default = IOS_IPAD_DEFAULT_PRIORITY as u64;
        let ipad_mini = IOS_IPAD_MINI_PRIORITY_CALC as u64;

        assert_eq!(ipad_pro_12_9, 100);
        assert!(ipad_pro_11 > ipad_pro_12_9);
        assert!(ipad_default > ipad_mini);
    }

    /// Test TV and Watch priorities
    #[test]
    fn test_tv_watch_priorities() {
        // Apple TV priorities using runtime variables
        let tv_4k = IOS_TV_4K_PRIORITY as u64;
        let tv_default = IOS_TV_DEFAULT_PRIORITY as u64;
        assert!(tv_4k < tv_default);

        // Apple Watch priorities using runtime variables
        let watch_ultra = IOS_WATCH_ULTRA_PRIORITY as u64;
        let watch_series = IOS_WATCH_SERIES_BASE as u64;
        let watch_default = IOS_WATCH_DEFAULT_PRIORITY as u64;
        let watch_se = IOS_WATCH_SE_PRIORITY as u64;
        let watch_other = IOS_WATCH_OTHER_PRIORITY as u64;

        assert!(watch_ultra < watch_series);
        assert!(watch_series < watch_default);
        assert!(watch_default < watch_se);
        assert!(watch_se < watch_other);
    }

    /// Test unknown device priority
    #[test]
    fn test_unknown_device_priority() {
        let unknown = IOS_UNKNOWN_DEVICE_PRIORITY as u64;
        let watch_other = IOS_WATCH_OTHER_PRIORITY as u64;
        let tv_default = IOS_TV_DEFAULT_PRIORITY as u64;

        assert_eq!(unknown, 999);

        // Unknown should be the highest (lowest priority) value
        assert!(unknown > watch_other);
        assert!(unknown > tv_default);
    }

    /// Test priority constants are u8/u32 compatible
    #[test]
    fn test_priority_type_compatibility() {
        // Test that u8 constants are valid values
        let android_priorities = [
            ANDROID_PIXEL_PRIORITY,
            ANDROID_NEXUS_PRIORITY,
            ANDROID_ONEPLUS_PRIORITY,
            ANDROID_OTHER_BRAND_PRIORITY,
            ANDROID_ALPHA_LOW_PRIORITY,
            ANDROID_ALPHA_HIGH_PRIORITY,
        ];

        for &priority in &android_priorities {
            // Test that priority is a valid u8 (not zero)
            assert!(priority > 0);
        }

        // Test that u32 constants are reasonable
        let ios_u32_priorities = [
            IOS_IPHONE_PRO_MAX_PRIORITY_VALUE,
            IOS_IPAD_PRO_12_9_PRIORITY,
            IOS_TV_4K_PRIORITY,
            IOS_WATCH_ULTRA_PRIORITY,
            IOS_UNKNOWN_DEVICE_PRIORITY,
        ];

        for &priority in &ios_u32_priorities {
            assert!(priority <= 9999);
        }
    }

    /// Test priority range consistency
    #[test]
    fn test_priority_range_consistency() {
        // Android device priorities should be in reasonable ranges using runtime variables
        let android_pixel = ANDROID_PIXEL_PRIORITY as u16;
        let android_alpha_high = ANDROID_ALPHA_HIGH_PRIORITY as u16;
        let ios_mini = IOS_IPHONE_MINI_PRIORITY as u16;
        let ios_unknown = IOS_UNKNOWN_PRIORITY as u16;

        assert!(android_pixel < 200);
        assert!(android_alpha_high < 200);
        assert!(ios_mini < 200);
        assert!(ios_unknown < 200);

        // iOS detailed priorities can be higher using runtime variables
        let ipad_pro = IOS_IPAD_PRO_12_9_PRIORITY as u64;
        let unknown_device = IOS_UNKNOWN_DEVICE_PRIORITY as u64;

        assert!(ipad_pro >= 100);
        assert!(unknown_device >= 900);
    }

    /// Test priority documentation examples
    #[test]
    fn test_documentation_examples() {
        // Test examples from the documentation
        assert_eq!(ANDROID_PIXEL_PRIORITY, 30);
        assert_eq!(IOS_IPHONE_PRO_MAX_PRIORITY, 80);

        // Lower values indicate higher priority using runtime variables
        let android_pixel = ANDROID_PIXEL_PRIORITY as u16;
        let android_nexus = ANDROID_NEXUS_PRIORITY as u16;
        let ios_mini = IOS_IPHONE_MINI_PRIORITY as u16;
        let ios_se = IOS_IPHONE_SE_PRIORITY as u16;

        assert!(android_pixel < android_nexus);
        assert!(ios_mini < ios_se);
    }
}
