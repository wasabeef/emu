use super::*;

impl DynamicDeviceConfig {
    pub fn calculate_android_device_priority(device_id: &str, display_name: &str) -> u32 {
        let combined = format!(
            "{} {}",
            device_id.to_lowercase(),
            display_name.to_lowercase()
        );

        if combined.contains(DEVICE_KEYWORD_PIXEL) && !combined.contains(DEVICE_KEYWORD_NEXUS) {
            let version_bonus = Self::extract_device_version(device_id, display_name);

            if version_bonus != MAX_VERSION_NUMBER {
                let final_priority = version_bonus
                    .saturating_sub(PIXEL_PRIORITY_OFFSET)
                    .min(PIXEL_PRIORITY_MAX_BONUS);
                return final_priority;
            } else {
                return PIXEL_UNVERSIONED_PRIORITY;
            }
        }

        let category_priority = Self::infer_device_category(device_id, display_name);
        let version_bonus = Self::extract_device_version(device_id, display_name);
        let oem_bonus = Self::calculate_oem_priority(display_name);

        if category_priority == INITIAL_CATEGORY_PRIORITY {
            return PHONE_CATEGORY_BASE_PRIORITY
                + version_bonus
                + (oem_bonus / OEM_BONUS_WEIGHT_HALF);
        }

        category_priority + (oem_bonus * OEM_BONUS_WEIGHT_FULL) + version_bonus
    }

    pub fn calculate_ios_device_priority(display_name: &str) -> u32 {
        let name_lower = display_name.to_lowercase();

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

        if name_lower.contains(DEVICE_KEYWORD_TV) {
            if name_lower.contains(DEVICE_KEYWORD_4K) {
                return IOS_TV_4K_PRIORITY;
            } else {
                return IOS_TV_DEFAULT_PRIORITY;
            }
        }

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

    fn infer_device_category(device_id: &str, display_name: &str) -> u32 {
        let combined = format!(
            "{} {}",
            device_id.to_lowercase(),
            display_name.to_lowercase()
        );

        if combined.contains(DEVICE_KEYWORD_FOLD) || combined.contains(DEVICE_KEYWORD_FLIP) {
            return FOLDABLE_CATEGORY_PRIORITY_BASE;
        }

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

        if combined.contains(DEVICE_KEYWORD_TV)
            || combined.contains(DEVICE_KEYWORD_1080P)
            || combined.contains(DEVICE_KEYWORD_4K)
        {
            return 200;
        }

        if combined.contains(DEVICE_KEYWORD_WEAR)
            || combined.contains(DEVICE_KEYWORD_WATCH)
            || (combined.contains(DEVICE_KEYWORD_ROUND)
                && !combined.contains(DEVICE_KEYWORD_TABLET))
        {
            return 300;
        }

        if combined.contains(DEVICE_KEYWORD_AUTO) || combined.contains(DEVICE_KEYWORD_CAR) {
            return 400;
        }

        500
    }

    fn calculate_oem_priority(display_name: &str) -> u32 {
        let combined = display_name.to_lowercase();

        if combined.contains(DEVICE_KEYWORD_GOOGLE) || combined.contains(DEVICE_KEYWORD_PIXEL) {
            return 0;
        }

        if combined.contains(DEVICE_KEYWORD_SAMSUNG) || combined.contains(DEVICE_KEYWORD_GALAXY) {
            return SAMSUNG_OEM_PRIORITY;
        }

        if combined.contains(DEVICE_KEYWORD_ONEPLUS) {
            return ONEPLUS_OEM_PRIORITY;
        }

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

    fn extract_device_version(device_id: &str, display_name: &str) -> u32 {
        let combined = format!("{device_id} {display_name}").to_lowercase();

        let device_patterns = [
            (r"pixel[_\s]?(\d+)", REGEX_GROUP_FIRST),
            (r"galaxy[_\s]?s(\d+)", REGEX_GROUP_FIRST),
            (r"galaxy[_\s]?z[_\s]?fold[_\s]?(\d+)", REGEX_GROUP_FIRST),
            (r"galaxy[_\s]?z[_\s]?flip[_\s]?(\d+)", REGEX_GROUP_FIRST),
            (r"oneplus[_\s]?(\d+)", REGEX_GROUP_FIRST),
            (r"nexus[_\s]?(\d+)", REGEX_GROUP_FIRST),
            (r"(\d+)[_\s]?pro", REGEX_GROUP_FIRST),
            (r"(\d+)[_\s]?plus", REGEX_GROUP_FIRST),
            (r"(\d+)[_\s]?ultra", REGEX_GROUP_FIRST),
        ];

        for (pattern, group) in &device_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(&combined) {
                    if let Some(version_str) = caps.get(*group) {
                        if let Ok(version) = version_str.as_str().parse::<u32>() {
                            return VERSION_PRIORITY_BASE - version.min(MAX_VERSION_FOR_PRIORITY);
                        }
                    }
                }
            }
        }

        let number_regex = regex::Regex::new(r"\b(\d{1,2})\b").unwrap();
        let mut versions = Vec::new();

        for caps in number_regex.captures_iter(&combined) {
            if let Ok(num) = caps[1].parse::<u32>() {
                if num > 0 && num <= MAX_VERSION_NUMBER {
                    versions.push(num);
                }
            }
        }

        if let Some(&max_version) = versions.iter().max() {
            return VERSION_PRIORITY_BASE - max_version.min(99);
        }

        MAX_VERSION_NUMBER
    }

    fn extract_ios_version(device_name: &str) -> u32 {
        let parts: Vec<&str> = device_name.split_whitespace().collect();

        for part in parts {
            if let Ok(num) = part.parse::<u32>() {
                if num > 0 && num <= MAX_VERSION_NUMBER {
                    return num;
                }
            }

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

    pub(super) fn calculate_priority_from_device_info(&self, device: &DeviceInfo) -> u32 {
        let device_lower = device.id.to_lowercase();
        let display_lower = device.display_name.to_lowercase();

        if let Some(version) = self.extract_version_number(&device_lower, &display_lower) {
            match device.category {
                DeviceCategory::Foldable => FOLDABLE_PRIORITY_OFFSET + version,
                DeviceCategory::Phone => PHONE_PRIORITY_OFFSET + version,
                DeviceCategory::Tablet => TABLET_PRIORITY_OFFSET + version,
                DeviceCategory::Wear => WEAR_PRIORITY_OFFSET + version,
                DeviceCategory::TV => TV_PRIORITY_OFFSET + version,
                DeviceCategory::Automotive => AUTOMOTIVE_PRIORITY_OFFSET + version,
                DeviceCategory::Unknown => UNKNOWN_PRIORITY_OFFSET + version,
            }
        } else {
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
        let combined = format!("{device_id} {display_name}");

        for part in combined.split_whitespace() {
            if let Ok(num) = part.parse::<u32>() {
                if num > 0 && num <= MAX_VERSION_NUMBER {
                    return Some(VERSION_PRIORITY_BASE - num);
                }
            }

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
}
