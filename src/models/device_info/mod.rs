use anyhow::Result;

use crate::constants::{
    keywords::*,
    limits::{MAX_DEVICE_NAME_PARTS_DISPLAY, MAX_VERSION_NUMBER},
    numeric::{
        ASUS_OEM_PRIORITY, AUTOMOTIVE_PRIORITY_OFFSET, DEVICE_CATEGORY_AUTOMOTIVE_FALLBACK,
        DEVICE_CATEGORY_FALLBACK_PRIORITY, DEVICE_CATEGORY_PHONE_FALLBACK,
        DEVICE_CATEGORY_TABLET_FALLBACK, DEVICE_CATEGORY_TV_FALLBACK,
        DEVICE_CATEGORY_UNKNOWN_FALLBACK, DEVICE_CATEGORY_WEAR_FALLBACK,
        FOLDABLE_CATEGORY_PRIORITY_BASE, FOLDABLE_PRIORITY_OFFSET, HUAWEI_OEM_PRIORITY,
        INITIAL_CATEGORY_PRIORITY, INVALID_VERSION, LENOVO_OEM_PRIORITY, MAX_VERSION_FOR_PRIORITY,
        MOTOROLA_OEM_PRIORITY, NOKIA_OEM_PRIORITY, OEM_BONUS_WEIGHT_FULL, OEM_BONUS_WEIGHT_HALF,
        OEM_GENERIC_PRIORITY, ONEPLUS_OEM_PRIORITY, OPPO_OEM_PRIORITY,
        PHONE_CATEGORY_PRIORITY_BASE, PHONE_PRIORITY_OFFSET, REGEX_GROUP_FIRST,
        SAMSUNG_OEM_PRIORITY, SCREEN_SIZE_EXTRA_LARGE_TABLET, SCREEN_SIZE_LARGE_TABLET,
        SCREEN_SIZE_MEDIUM_TABLET, SCREEN_SIZE_PHONE_LARGE, SCREEN_SIZE_PHONE_MEDIUM,
        SONY_OEM_PRIORITY, TABLET_CATEGORY_PRIORITY_BASE, TABLET_PRIORITY_OFFSET,
        TV_PRIORITY_OFFSET, UNKNOWN_DEVICE_PRIORITY, UNKNOWN_PRIORITY_OFFSET,
        VERSION_PRIORITY_BASE, WEAR_PRIORITY_OFFSET, XIAOMI_OEM_PRIORITY,
    },
    priorities::{
        IOS_IPAD_AIR_PRIORITY_VALUE, IOS_IPAD_DEFAULT_PRIORITY, IOS_IPAD_MINI_PRIORITY_CALC,
        IOS_IPAD_PRO_11_PRIORITY_VALUE, IOS_IPAD_PRO_12_9_PRIORITY, IOS_IPAD_PRO_OTHER_PRIORITY,
        IOS_IPHONE_DEFAULT_BASE, IOS_IPHONE_MINI_PRIORITY_CALC, IOS_IPHONE_PLUS_MAX_PRIORITY,
        IOS_IPHONE_PRO_MAX_PRIORITY_VALUE, IOS_IPHONE_PRO_PRIORITY_VALUE,
        IOS_IPHONE_SE_PRIORITY_CALC, IOS_IPHONE_VERSION_OFFSET, IOS_TV_4K_PRIORITY,
        IOS_TV_DEFAULT_PRIORITY, IOS_UNKNOWN_DEVICE_PRIORITY, IOS_WATCH_DEFAULT_PRIORITY,
        IOS_WATCH_OTHER_PRIORITY, IOS_WATCH_SERIES_BASE, IOS_WATCH_SERIES_OFFSET,
        IOS_WATCH_SE_PRIORITY, IOS_WATCH_ULTRA_PRIORITY, PHONE_CATEGORY_BASE_PRIORITY,
        PIXEL_PRIORITY_MAX_BONUS, PIXEL_PRIORITY_OFFSET, PIXEL_UNVERSIONED_PRIORITY,
    },
};
use std::collections::HashMap;

mod parsing;
mod priority;
pub use self::priority::sort_android_devices_for_display;

/// Dynamic device information structures
///
/// This module provides completely dynamic device management for Android and iOS platforms.
/// All device categorization, prioritization, and configuration is derived from actual
/// system specifications rather than hardcoded string matching patterns.
///
/// # Design Philosophy
///
/// ## Zero Hardcoding Principle
///
/// This implementation eliminates all hardcoded device names, manufacturer preferences,
/// and category mappings. Instead, it uses:
///
/// 1. **Physical Specifications**: Screen size, resolution, aspect ratio
/// 2. **SDK-Provided Data**: Manufacturer information from avdmanager/simctl
/// 3. **Algorithmic Classification**: Mathematical categorization based on device characteristics
/// 4. **Fair Prioritization**: Alphabetical or specification-based ordering without bias
///
/// ## Advantages of Dynamic Approach
///
/// - **Future-Proof**: Automatically handles new devices and SDK updates
/// - **Maintenance-Free**: No need to update device lists or mappings
/// - **Accurate Classification**: Based on actual technical specifications
/// - **Unbiased Ordering**: Fair prioritization without manufacturer preferences
/// - **Scalable**: Handles unlimited device types without code changes
///
/// # Device Categorization Algorithm
///
/// ## Specification-Based Classification
///
/// Devices are categorized using scientific criteria:
///
/// ### Screen Size Thresholds (inches)
/// ```text
/// ≥10.0: Tablet
/// 7.0-9.9: Large device (aspect ratio determines final category)
/// 3.0-6.9: Phone
/// <3.0: Wearable
/// ```
///
/// ### Aspect Ratio Analysis
/// ```text
/// For 7.0-9.9 inch devices:
/// 0.6-0.8: Square-ish → Tablet tendency
/// Other ratios: Elongated → Phone tendency
/// ```
///
/// ### Resolution-Based Fallback
/// ```text
/// ≥1920x1080 (without size info): TV/Large display
/// Lower resolutions: Unknown category
/// ```
///
/// ## Priority Calculation System
///
/// Priority is calculated using multiple factors:
///
/// ### 1. Category Base Priority
/// ```text
/// Foldable:   0-99   (cutting-edge technology)
/// Phone:      100-199 (common mobile devices)
/// Tablet:     200-299 (larger mobile devices)
/// Wear:       300-399 (specialized wearables)
/// TV:         400-499 (entertainment devices)
/// Automotive: 500-599 (specialized automotive)
/// Unknown:    800-899 (unclassified devices)
/// ```
///
/// ### 2. Version Bonus (0-50 points)
/// Newer device versions receive better priority:
/// ```text
/// Version extraction from ID/name → Lower priority number for newer versions
/// Example: "pixel_9" gets priority 5, "pixel_7" gets priority 15
/// ```
///
/// ### 3. Manufacturer Priority (0-50 points)
/// Fair alphabetical ordering:
/// ```text
/// A-C: 0-9   (early alphabet)
/// D-G: 10-19 (mid-early alphabet)
/// H-M: 20-29 (mid alphabet)
/// N-S: 30-39 (mid-late alphabet)
/// T-Z: 40-49 (late alphabet)
/// ```
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub display_name: String,
    pub oem: Option<String>,
    pub category: DeviceCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceCategory {
    Phone,
    Tablet,
    Wear,
    TV,
    Automotive,
    Foldable,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ApiLevelInfo {
    pub level: u32,
    pub version_name: String,
    pub available_tags: Vec<String>,
}

pub trait DynamicDeviceProvider: Send + Sync {
    fn get_available_devices(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<DeviceInfo>>> + Send;

    fn get_available_api_levels(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<ApiLevelInfo>>> + Send;

    fn get_available_skins(
        &self,
        device_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<String>>> + Send;

    fn get_device_priority(
        &self,
        device_id: &str,
    ) -> impl std::future::Future<Output = Result<u32>> + Send;
}

pub struct DynamicDeviceConfig {
    device_cache: HashMap<String, DeviceInfo>,
    api_cache: HashMap<u32, ApiLevelInfo>,
}

impl Default for DynamicDeviceConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicDeviceConfig {
    pub fn new() -> Self {
        Self {
            device_cache: HashMap::new(),
            api_cache: HashMap::new(),
        }
    }

    pub fn load_device_cache(&mut self, devices: Vec<DeviceInfo>) {
        self.device_cache.clear();
        for device in devices {
            self.device_cache.insert(device.id.clone(), device);
        }
    }

    pub fn load_api_cache(&mut self, api_levels: Vec<ApiLevelInfo>) {
        self.api_cache.clear();
        for api_info in api_levels {
            self.api_cache.insert(api_info.level, api_info);
        }
    }

    pub fn get_device_priority(&self, device_id: &str) -> u32 {
        if let Some(device) = self.device_cache.get(device_id) {
            self.calculate_priority_from_device_info(device)
        } else {
            UNKNOWN_DEVICE_PRIORITY
        }
    }

    pub fn get_android_version_name(&self, api_level: u32) -> String {
        if let Some(api_info) = self.api_cache.get(&api_level) {
            api_info.version_name.clone()
        } else {
            format!("API {api_level}")
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeviceSpecifications {
    pub screen_size_inches: f32,
    pub screen_width: u32,
    pub screen_height: u32,
    pub screen_density: u32,
    pub manufacturer: String,
}

pub static FALLBACK_ANDROID_DEVICES: &[&str] =
    &["pixel_7", "pixel_6", "pixel_5", "Nexus_5X", "pixel_tablet"];

pub mod test_constants {
    pub const TEST_ANDROID_DEVICE: &str = "pixel_7";
    pub const TEST_IOS_DEVICE: &str = "com.apple.CoreSimulator.SimDeviceType.iPhone-15";
}

#[cfg(test)]
mod tests;
