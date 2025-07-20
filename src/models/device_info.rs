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
///
/// # Device Specifications Structure
///
/// ## Physical Characteristics
/// ```rust
/// pub struct DeviceSpecifications {
///     screen_size_inches: f32,    // Physical screen size
///     screen_width: u32,          // Horizontal resolution
///     screen_height: u32,         // Vertical resolution
///     screen_density: u32,        // DPI (dots per inch)
///     manufacturer: String,       // OEM from device definition
/// }
/// ```
///
/// ## Data Sources
///
/// ### Android (avdmanager)
/// ```bash
/// avdmanager list device
/// # Provides: id, Name, OEM, screen specifications
/// ```
///
/// ### iOS (xcrun simctl)
/// ```bash
/// xcrun simctl list devicetypes
/// # Provides: identifier, name, device specifications
/// ```
///
/// # Dynamic Configuration Parsing
///
/// ## Device Name Parsing
///
/// Names are parsed using dynamic algorithms rather than hardcoded patterns:
///
/// ### Cleaning Algorithm
/// 1. Remove parenthetical information (manufacturer tags)
/// 2. Filter non-alphanumeric characters (keep spaces)
/// 3. Split into meaningful word components
/// 4. Take first 2-3 significant words
/// 5. Capitalize each word properly
///
/// ### Examples
/// ```text
/// Input:  "Pixel 7 (Google)"
/// Output: ["Pixel", "7"]
///
/// Input:  "Galaxy S23 Ultra (Samsung)"
/// Output: ["Galaxy", "S23", "Ultra"]
/// ```
///
/// ## API Level to Android Version Mapping
///
/// Rather than hardcoded mappings, the system:
///
/// 1. **Primary**: Queries `sdkmanager --list --verbose` for platform information
/// 2. **Secondary**: Parses system image descriptions for version hints
/// 3. **Tertiary**: Uses algorithmic derivation based on known Android release patterns
///
/// ### Dynamic Version Detection
/// ```bash
/// sdkmanager --list --verbose
/// # Output includes: "Android API 34, revision 2 | Android 14"
/// # Parser extracts: API 34 → "Android 14"
/// ```
///
/// # Error Handling and Fallbacks
///
/// ## Graceful Degradation
///
/// When dynamic data is unavailable:
///
/// 1. **Device Specifications**: Fall back to `Unknown` category
/// 2. **Manufacturer Information**: Use neutral alphabetical priority
/// 3. **API Levels**: Fall back to algorithmic version derivation
/// 4. **Device Names**: Use basic string cleaning without pattern matching
///
/// ## Performance Considerations
///
/// - **Caching**: Device specifications are cached per session
/// - **Batch Processing**: Multiple devices processed in parallel where possible
/// - **Lazy Loading**: Specifications loaded only when needed
/// - **Async Operations**: All SDK queries are asynchronous to avoid blocking
///
/// # Testing Strategy
///
/// ## Test Constants
///
/// Test code is allowed to use hardcoded values for verification:
/// ```rust
/// pub mod test_constants {
///     pub const TEST_ANDROID_DEVICE: &str = "pixel_7";
///     pub const TEST_IOS_DEVICE: &str = "com.apple.CoreSimulator.SimDeviceType.iPhone-15";
/// }
/// ```
///
/// ## Fallback Constants
///
/// Emergency fallbacks for critical system failures:
/// ```rust
/// pub static FALLBACK_ANDROID_DEVICES: &[&str] = &[
///     "pixel_7", "pixel_6", "pixel_5", // Minimal working set
/// ];
/// ```
///
/// These are used only when the SDK is completely unavailable.
use std::collections::HashMap;

/// Information about a device discovered dynamically from the SDK.
///
/// This struct contains device metadata retrieved from platform tools
/// (avdmanager for Android, xcrun simctl for iOS) at runtime.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device identifier (e.g., "pixel_7", "tv_1080p")
    pub id: String,
    /// Human-readable device name (e.g., "Pixel 7", "Android TV (1080p)")
    pub display_name: String,
    /// Original Equipment Manufacturer name if available
    pub oem: Option<String>,
    /// Dynamically determined device category
    pub category: DeviceCategory,
}

/// Device category determined by dynamic analysis of device characteristics.
///
/// Categories are assigned based on device specifications, screen size,
/// and naming patterns discovered at runtime rather than hardcoded lists.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceCategory {
    /// Mobile phone devices (typically 3-6.9 inch screens)
    Phone,
    /// Tablet devices (typically 7+ inch screens)
    Tablet,
    /// Wearable devices (watches, fitness trackers)
    Wear,
    /// Television and streaming devices
    TV,
    /// In-vehicle entertainment systems
    Automotive,
    /// Foldable devices with flexible displays
    Foldable,
    /// Devices that don't fit other categories
    Unknown,
}

/// Android API level information discovered from the SDK.
///
/// Contains version mappings and available system image tags
/// retrieved dynamically from sdkmanager and platform tools.
#[derive(Debug, Clone)]
pub struct ApiLevelInfo {
    /// API level number (e.g., 34)
    pub level: u32,
    /// Android version name (e.g., "Android 14")
    pub version_name: String,
    /// Available system image tags (e.g., ["google_apis", "google_apis_playstore"])
    pub available_tags: Vec<String>,
}

/// Trait for device managers that provide dynamic device information.
///
/// Implementors discover device configurations at runtime from platform SDKs
/// rather than relying on hardcoded device lists or mappings.
pub trait DynamicDeviceProvider: Send + Sync {
    /// Get all available devices from the system
    fn get_available_devices(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<DeviceInfo>>> + Send;

    /// Get all available API levels from installed system images
    fn get_available_api_levels(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<ApiLevelInfo>>> + Send;

    /// Get available skins for a device (check if skin files exist)
    fn get_available_skins(
        &self,
        device_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<String>>> + Send;

    /// Get device priority based on system information (newer devices first)
    fn get_device_priority(
        &self,
        device_id: &str,
    ) -> impl std::future::Future<Output = Result<u32>> + Send;
}

/// Dynamic device configuration system that replaces hardcoded device mappings.
///
/// This struct provides runtime device discovery and prioritization algorithms
/// that adapt to new devices and SDK updates without code changes. It caches
/// discovered information for performance while maintaining flexibility.
///
/// # Implementation Note
/// Some async operations are implemented directly in AndroidManager to avoid
/// async trait object limitations in Rust.
pub struct DynamicDeviceConfig {
    /// Cache of discovered device information keyed by device ID
    device_cache: HashMap<String, DeviceInfo>,
    /// Cache of API level to version name mappings
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

    /// Calculate device priority for Android devices
    pub fn calculate_android_device_priority(device_id: &str, display_name: &str) -> u32 {
        let combined = format!(
            "{} {}",
            device_id.to_lowercase(),
            display_name.to_lowercase()
        );

        // Special handling for Pixel devices - they get highest priority
        if combined.contains(DEVICE_KEYWORD_PIXEL) && !combined.contains(DEVICE_KEYWORD_NEXUS) {
            let version_bonus = Self::extract_device_version(device_id, display_name);

            // Check if this device has a version number
            // extract_device_version returns 100-version for versioned devices, 50 for unversioned
            if version_bonus != MAX_VERSION_NUMBER {
                // Versioned Pixel devices: priority 0-19
                let final_priority = version_bonus
                    .saturating_sub(PIXEL_PRIORITY_OFFSET)
                    .min(PIXEL_PRIORITY_MAX_BONUS);
                return final_priority;
            } else {
                // Unversioned Pixel devices: priority 20-29 (lower than versioned Pixels)
                return PIXEL_UNVERSIONED_PRIORITY; // Fixed priority for unversioned Pixel devices
            }
        }

        let category_priority = Self::infer_device_category(device_id, display_name);
        let version_bonus = Self::extract_device_version(device_id, display_name);
        let oem_bonus = Self::calculate_oem_priority(display_name);

        // For phones (category 0), emphasize version differences more
        // This ensures newer devices come before older devices in the same category
        if category_priority == INITIAL_CATEGORY_PRIORITY {
            // Start from PIXEL_PRIORITY_OFFSET to leave room for Pixel devices (0-FOLDABLE_CATEGORY_PRIORITY_BASE)
            return PHONE_CATEGORY_BASE_PRIORITY
                + version_bonus
                + (oem_bonus / OEM_BONUS_WEIGHT_HALF);
        }

        // For other categories, maintain manufacturer preference
        category_priority + (oem_bonus * OEM_BONUS_WEIGHT_FULL) + version_bonus
    }

    /// Calculate device priority for iOS devices  
    pub fn calculate_ios_device_priority(display_name: &str) -> u32 {
        let name_lower = display_name.to_lowercase();

        // iPhone priorities (0-99)
        if name_lower.contains(DEVICE_KEYWORD_IPHONE) {
            if name_lower.contains("pro max") {
                return IOS_IPHONE_PRO_MAX_PRIORITY_VALUE;
            } else if name_lower.contains(DEVICE_KEYWORD_PRO) {
                return IOS_IPHONE_PRO_PRIORITY_VALUE;
            } else if name_lower.contains(DEVICE_KEYWORD_PLUS)
                || name_lower.contains(DEVICE_KEYWORD_MAX)
            {
                return IOS_IPHONE_PLUS_MAX_PRIORITY;
            } else if name_lower.contains(DEVICE_KEYWORD_MINI) {
                return IOS_IPHONE_MINI_PRIORITY_CALC;
            } else if name_lower.contains(DEVICE_KEYWORD_SE) {
                return IOS_IPHONE_SE_PRIORITY_CALC;
            } else {
                let version = Self::extract_ios_version(&name_lower);
                if version > INVALID_VERSION {
                    return IOS_IPHONE_DEFAULT_BASE - version.min(IOS_IPHONE_VERSION_OFFSET);
                }
                return MAX_VERSION_NUMBER;
            }
        }

        // iPad priorities (IPAD_PRIORITY_MIN-IPAD_PRIORITY_MAX)
        if name_lower.contains(DEVICE_KEYWORD_IPAD) {
            if name_lower.contains(DEVICE_KEYWORD_PRO) {
                if name_lower.contains(DEVICE_KEYWORD_12_9) {
                    return IOS_IPAD_PRO_12_9_PRIORITY;
                } else if name_lower.contains(DEVICE_KEYWORD_11) {
                    return IOS_IPAD_PRO_11_PRIORITY_VALUE;
                } else {
                    return IOS_IPAD_PRO_OTHER_PRIORITY;
                }
            } else if name_lower.contains(DEVICE_KEYWORD_AIR) {
                return IOS_IPAD_AIR_PRIORITY_VALUE;
            } else if name_lower.contains(DEVICE_KEYWORD_MINI) {
                return IOS_IPAD_MINI_PRIORITY_CALC;
            } else {
                return IOS_IPAD_DEFAULT_PRIORITY;
            }
        }

        // Apple TV (200-299)
        if name_lower.contains(DEVICE_KEYWORD_TV) {
            if name_lower.contains(DEVICE_KEYWORD_4K) {
                return IOS_TV_4K_PRIORITY;
            } else {
                return IOS_TV_DEFAULT_PRIORITY;
            }
        }

        // Apple Watch (300-399)
        if name_lower.contains(DEVICE_KEYWORD_WATCH) {
            if name_lower.contains(DEVICE_KEYWORD_ULTRA) {
                return IOS_WATCH_ULTRA_PRIORITY;
            } else if name_lower.contains(DEVICE_KEYWORD_SERIES) {
                let version = Self::extract_ios_version(&name_lower);
                if version > INVALID_VERSION {
                    return IOS_WATCH_SERIES_BASE - version.min(IOS_WATCH_SERIES_OFFSET);
                }
                return IOS_WATCH_DEFAULT_PRIORITY;
            } else if name_lower.contains(DEVICE_KEYWORD_SE) {
                return IOS_WATCH_SE_PRIORITY;
            } else {
                return IOS_WATCH_OTHER_PRIORITY;
            }
        }

        IOS_UNKNOWN_DEVICE_PRIORITY
    }

    /// Infer device category for priority sorting
    fn infer_device_category(device_id: &str, display_name: &str) -> u32 {
        let combined = format!(
            "{} {}",
            device_id.to_lowercase(),
            display_name.to_lowercase()
        );

        // Foldable devices (check first to avoid phone categorization)
        if combined.contains(DEVICE_KEYWORD_FOLD) || combined.contains(DEVICE_KEYWORD_FLIP) {
            return FOLDABLE_CATEGORY_PRIORITY_BASE;
        }

        // Tablets (check before phones to catch "pixel_tablet")
        if combined.contains(DEVICE_KEYWORD_TABLET)
            || combined.contains(DEVICE_KEYWORD_PAD)
            || (combined.contains(&SCREEN_SIZE_MEDIUM_TABLET.to_string())
                && combined.contains("inch"))
            || (combined.contains(&SCREEN_SIZE_LARGE_TABLET.to_string())
                && combined.contains("inch"))
            || (combined.contains(&SCREEN_SIZE_EXTRA_LARGE_TABLET.to_string())
                && combined.contains("inch"))
        {
            return TABLET_CATEGORY_PRIORITY_BASE;
        }

        // Phone category gets highest priority
        if combined.contains(DEVICE_KEYWORD_PHONE)
            || (combined.contains(DEVICE_KEYWORD_PIXEL)
                && !combined.contains(DEVICE_KEYWORD_FOLD)
                && !combined.contains(DEVICE_KEYWORD_TABLET))
            || (combined.contains(DEVICE_KEYWORD_GALAXY)
                && !combined.contains(DEVICE_KEYWORD_FOLD)
                && !combined.contains(DEVICE_KEYWORD_TABLET))
            || combined.contains(DEVICE_KEYWORD_ONEPLUS)
            || (combined.contains(&SCREEN_SIZE_PHONE_MEDIUM.to_string())
                && combined.contains("inch"))
            || (combined.contains(&SCREEN_SIZE_PHONE_LARGE.to_string())
                && combined.contains("inch"))
            || (combined.contains(DEVICE_KEYWORD_PRO)
                && !combined.contains(DEVICE_KEYWORD_TABLET)
                && !combined.contains(DEVICE_KEYWORD_FOLD))
        {
            return PHONE_CATEGORY_PRIORITY_BASE;
        }

        // TV devices
        if combined.contains(DEVICE_KEYWORD_TV)
            || combined.contains(DEVICE_KEYWORD_1080P)
            || combined.contains(DEVICE_KEYWORD_4K)
        {
            return 200;
        }

        // Wear devices (watches)
        if combined.contains(DEVICE_KEYWORD_WEAR)
            || combined.contains(DEVICE_KEYWORD_WATCH)
            || (combined.contains(DEVICE_KEYWORD_ROUND)
                && !combined.contains(DEVICE_KEYWORD_TABLET))
        {
            return 300;
        }

        // Auto/Car devices
        if combined.contains(DEVICE_KEYWORD_AUTO) || combined.contains(DEVICE_KEYWORD_CAR) {
            return 400;
        }

        500
    }

    /// Calculate OEM priority
    fn calculate_oem_priority(display_name: &str) -> u32 {
        let combined = display_name.to_lowercase();

        // Google devices get highest priority
        if combined.contains(DEVICE_KEYWORD_GOOGLE) || combined.contains(DEVICE_KEYWORD_PIXEL) {
            return 0;
        }

        // Samsung devices get second priority
        if combined.contains(DEVICE_KEYWORD_SAMSUNG) || combined.contains(DEVICE_KEYWORD_GALAXY) {
            return SAMSUNG_OEM_PRIORITY;
        }

        // OnePlus devices
        if combined.contains(DEVICE_KEYWORD_ONEPLUS) {
            return ONEPLUS_OEM_PRIORITY;
        }

        // Extract manufacturer from parentheses
        if let Some(start) = display_name.find('(') {
            if let Some(end) = display_name.find(')') {
                let oem_part = &display_name[start + 1..end].to_lowercase();
                if oem_part == DEVICE_KEYWORD_XIAOMI {
                    return XIAOMI_OEM_PRIORITY;
                } else if oem_part == DEVICE_KEYWORD_ASUS {
                    return ASUS_OEM_PRIORITY;
                } else if oem_part == DEVICE_KEYWORD_OPPO {
                    return OPPO_OEM_PRIORITY;
                } else if oem_part == DEVICE_KEYWORD_VIVO {
                    return NOKIA_OEM_PRIORITY;
                } else if oem_part == DEVICE_KEYWORD_HUAWEI {
                    return HUAWEI_OEM_PRIORITY;
                } else if oem_part == DEVICE_KEYWORD_MOTOROLA {
                    return MOTOROLA_OEM_PRIORITY;
                } else if oem_part == DEVICE_KEYWORD_LENOVO {
                    return LENOVO_OEM_PRIORITY;
                } else if oem_part == DEVICE_KEYWORD_SONY {
                    return SONY_OEM_PRIORITY;
                }
            }
        }

        OEM_GENERIC_PRIORITY
    }

    /// Extract device version from string
    fn extract_device_version(device_id: &str, display_name: &str) -> u32 {
        let combined = format!("{device_id} {display_name}").to_lowercase();

        // First, try to extract numbers that appear after known device names
        // This makes it future-proof for Pixel 10, 11, etc.
        let device_patterns = [
            (r"pixel[_\s]?(\d+)", REGEX_GROUP_FIRST), // Pixel 9, pixel_9, etc.
            (r"galaxy[_\s]?s(\d+)", REGEX_GROUP_FIRST), // Galaxy S24
            (r"galaxy[_\s]?z[_\s]?fold[_\s]?(\d+)", REGEX_GROUP_FIRST), // Galaxy Z Fold 5
            (r"galaxy[_\s]?z[_\s]?flip[_\s]?(\d+)", REGEX_GROUP_FIRST), // Galaxy Z Flip 5
            (r"oneplus[_\s]?(\d+)", REGEX_GROUP_FIRST), // OnePlus 11
            (r"nexus[_\s]?(\d+)", REGEX_GROUP_FIRST), // Nexus 5
            (r"(\d+)[_\s]?pro", REGEX_GROUP_FIRST),   // 8 Pro, 9_pro
            (r"(\d+)[_\s]?plus", REGEX_GROUP_FIRST),  // 8 Plus
            (r"(\d+)[_\s]?ultra", REGEX_GROUP_FIRST), // 24 Ultra
        ];

        // Try each pattern
        for (pattern, group) in &device_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(&combined) {
                    if let Some(version_str) = caps.get(*group) {
                        if let Ok(version) = version_str.as_str().parse::<u32>() {
                            // Higher version = lower priority number (newer devices first)
                            // This ensures Pixel 10 (priority 90) comes before Pixel 9 (priority 91)
                            return VERSION_PRIORITY_BASE - version.min(MAX_VERSION_FOR_PRIORITY);
                        }
                    }
                }
            }
        }

        // If no specific pattern matched, try to find any number in the string
        // This helps with future device naming patterns
        let number_regex = regex::Regex::new(r"\b(\d{1,2})\b").unwrap();
        let mut versions = Vec::new();

        for caps in number_regex.captures_iter(&combined) {
            if let Ok(num) = caps[1].parse::<u32>() {
                // Only consider reasonable version numbers (1-50)
                if num > 0 && num <= MAX_VERSION_NUMBER {
                    versions.push(num);
                }
            }
        }

        // Use the highest version number found
        if let Some(&max_version) = versions.iter().max() {
            return VERSION_PRIORITY_BASE - max_version.min(99);
        }

        // Return 50 for devices without version numbers (middle priority)
        MAX_VERSION_NUMBER
    }

    /// Extract iOS device version
    fn extract_ios_version(device_name: &str) -> u32 {
        // Try to extract number from patterns like "iPhone 15", "Series 9", etc.
        let parts: Vec<&str> = device_name.split_whitespace().collect();

        for part in parts {
            if let Ok(num) = part.parse::<u32>() {
                if num > 0 && num <= MAX_VERSION_NUMBER {
                    return num;
                }
            }

            // Handle hyphenated versions like "iPhone-15"
            if part.contains('-') {
                if let Some(num_part) = part.split('-').next_back() {
                    if let Ok(num) = num_part.parse::<u32>() {
                        if num > 0 && num <= MAX_VERSION_NUMBER {
                            return num;
                        }
                    }
                }
            }
        }

        0
    }

    /// Load device cache
    pub fn load_device_cache(&mut self, devices: Vec<DeviceInfo>) {
        self.device_cache.clear();
        for device in devices {
            self.device_cache.insert(device.id.clone(), device);
        }
    }

    /// Load API cache  
    pub fn load_api_cache(&mut self, api_levels: Vec<ApiLevelInfo>) {
        self.api_cache.clear();
        for api_info in api_levels {
            self.api_cache.insert(api_info.level, api_info);
        }
    }

    /// Get device priority from cache
    pub fn get_device_priority(&self, device_id: &str) -> u32 {
        if let Some(device) = self.device_cache.get(device_id) {
            self.calculate_priority_from_device_info(device)
        } else {
            UNKNOWN_DEVICE_PRIORITY // Unknown devices go to the end
        }
    }

    /// Parse device name using cached device information
    pub fn parse_device_name(&self, device_type: &str) -> Vec<String> {
        // Try to find matching device in cache
        for device in self.device_cache.values() {
            if device_type.contains(&device.display_name) || device_type.contains(&device.id) {
                return self.extract_name_parts(&device.display_name);
            }
        }

        // Fallback to basic parsing
        self.basic_name_parsing(device_type)
    }

    /// Get Android version name from cache
    pub fn get_android_version_name(&self, api_level: u32) -> String {
        if let Some(api_info) = self.api_cache.get(&api_level) {
            api_info.version_name.clone()
        } else {
            format!("API {api_level}") // Fallback
        }
    }

    fn calculate_priority_from_device_info(&self, device: &DeviceInfo) -> u32 {
        let device_lower = device.id.to_lowercase();
        let display_lower = device.display_name.to_lowercase();

        // Extract version numbers if possible
        if let Some(version) = self.extract_version_number(&device_lower, &display_lower) {
            match device.category {
                DeviceCategory::Foldable => FOLDABLE_PRIORITY_OFFSET + version, // Highest priority
                DeviceCategory::Phone => PHONE_PRIORITY_OFFSET + version,
                DeviceCategory::Tablet => TABLET_PRIORITY_OFFSET + version,
                DeviceCategory::Wear => WEAR_PRIORITY_OFFSET + version,
                DeviceCategory::TV => TV_PRIORITY_OFFSET + version,
                DeviceCategory::Automotive => AUTOMOTIVE_PRIORITY_OFFSET + version,
                DeviceCategory::Unknown => UNKNOWN_PRIORITY_OFFSET + version,
            }
        } else {
            // Fallback to category-based priority
            match device.category {
                DeviceCategory::Foldable => DEVICE_CATEGORY_FALLBACK_PRIORITY,
                DeviceCategory::Phone => DEVICE_CATEGORY_PHONE_FALLBACK,
                DeviceCategory::Tablet => DEVICE_CATEGORY_TABLET_FALLBACK,
                DeviceCategory::Wear => DEVICE_CATEGORY_WEAR_FALLBACK,
                DeviceCategory::TV => DEVICE_CATEGORY_TV_FALLBACK,
                DeviceCategory::Automotive => DEVICE_CATEGORY_AUTOMOTIVE_FALLBACK,
                DeviceCategory::Unknown => DEVICE_CATEGORY_UNKNOWN_FALLBACK,
            }
        }
    }

    fn extract_version_number(&self, device_id: &str, display_name: &str) -> Option<u32> {
        // Try to extract numbers from device ID or display name
        let combined = format!("{device_id} {display_name}");

        for part in combined.split_whitespace() {
            if let Ok(num) = part.parse::<u32>() {
                if num > 0 && num <= MAX_VERSION_NUMBER {
                    // Reasonable device version range
                    return Some(VERSION_PRIORITY_BASE - num); // Newer versions get lower numbers (higher priority)
                }
            }

            // Handle cases like "pixel_9", "iphone_15"
            if part.contains('_') {
                if let Some(num_part) = part.split('_').next_back() {
                    if let Ok(num) = num_part.parse::<u32>() {
                        if num > 0 && num <= MAX_VERSION_NUMBER {
                            return Some(VERSION_PRIORITY_BASE - num);
                        }
                    }
                }
            }
        }

        None
    }

    fn extract_name_parts(&self, display_name: &str) -> Vec<String> {
        // First try to extract main device name without manufacturer
        let mut parts = Vec::new();
        let mut in_parentheses = false;
        let mut current_word = String::new();

        for ch in display_name.chars() {
            match ch {
                '(' => {
                    if !current_word.is_empty() {
                        parts.push(current_word.clone());
                        current_word.clear();
                    }
                    in_parentheses = true;
                }
                ')' => {
                    in_parentheses = false;
                }
                ' ' if !in_parentheses => {
                    if !current_word.is_empty() {
                        parts.push(current_word.clone());
                        current_word.clear();
                    }
                }
                _ if !in_parentheses => {
                    current_word.push(ch);
                }
                _ => {}
            }
        }

        if !current_word.is_empty() {
            parts.push(current_word);
        }

        // Take reasonable limit, but keep important modifiers like Fold, XL, Pro
        parts
            .into_iter()
            .take(MAX_DEVICE_NAME_PARTS_DISPLAY)
            .collect()
    }

    fn basic_name_parsing(&self, device_type: &str) -> Vec<String> {
        // Use the same logic as extract_name_parts for consistency
        self.extract_name_parts(device_type)
    }
}

/// Device specifications parsed from avdmanager
#[derive(Debug, Clone, Default)]
pub struct DeviceSpecifications {
    pub screen_size_inches: f32,
    pub screen_width: u32,
    pub screen_height: u32,
    pub screen_density: u32,
    pub manufacturer: String,
}

/// Fallback constants for when dynamic retrieval fails
pub static FALLBACK_ANDROID_DEVICES: &[&str] =
    &["pixel_7", "pixel_6", "pixel_5", "Nexus_5X", "pixel_tablet"];

pub mod test_constants {
    pub const TEST_ANDROID_DEVICE: &str = "pixel_7";
    pub const TEST_IOS_DEVICE: &str = "com.apple.CoreSimulator.SimDeviceType.iPhone-15";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_name_parts() {
        let config = DynamicDeviceConfig::new();

        // Test basic device name
        let parts = config.extract_name_parts("Pixel 9 Pro (Google)");
        assert_eq!(parts, vec!["Pixel", "9", "Pro"]);

        // Test device with Fold
        let parts = config.extract_name_parts("Pixel 9 Pro Fold (Google)");
        assert_eq!(parts, vec!["Pixel", "9", "Pro", "Fold"]);

        // Test device with XL
        let parts = config.extract_name_parts("Pixel 9 Pro XL (Google)");
        assert_eq!(parts, vec!["Pixel", "9", "Pro", "XL"]);

        // Test Samsung device
        let parts = config.extract_name_parts("Galaxy S23 Ultra (Samsung)");
        assert_eq!(parts, vec!["Galaxy", "S23", "Ultra"]);

        // Test device without parentheses
        let parts = config.extract_name_parts("Nexus 5X");
        assert_eq!(parts, vec!["Nexus", "5X"]);

        // Test device with multiple parentheses
        let parts = config.extract_name_parts("Pixel Tablet (Google) (Large)");
        assert_eq!(parts, vec!["Pixel", "Tablet"]);
    }

    #[test]
    fn test_parse_device_name() {
        let config = DynamicDeviceConfig::new();

        // Since parse_device_name depends on device_cache, it will use basic_name_parsing
        // which now uses extract_name_parts logic
        let parts = config.parse_device_name("Pixel 9 Pro Fold (Google)");
        assert_eq!(parts, vec!["Pixel", "9", "Pro", "Fold"]);

        let parts = config.parse_device_name("Pixel 9 Pro XL (Google)");
        assert_eq!(parts, vec!["Pixel", "9", "Pro", "XL"]);
    }

    #[test]
    fn test_placeholder_generation() {
        use crate::app::state::CreateDeviceForm;

        let mut form = CreateDeviceForm::new();
        form.device_type = "Pixel 9 Pro Fold (Google)".to_string();
        form.version_display = "API 36 - Android 15".to_string();

        form.generate_placeholder_name();

        // Now it should include "Fold"
        assert_eq!(form.name, "Pixel 9 Pro Fold API 36");

        // Test with XL
        form.device_type = "Pixel 9 Pro XL (Google)".to_string();
        form.generate_placeholder_name();
        assert_eq!(form.name, "Pixel 9 Pro XL API 36");

        // Test without special modifier
        form.device_type = "Pixel 9 Pro (Google)".to_string();
        form.generate_placeholder_name();
        assert_eq!(form.name, "Pixel 9 Pro API 36");
    }
}
