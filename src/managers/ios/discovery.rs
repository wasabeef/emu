#[cfg(target_os = "macos")]
use super::{extract_ios_version, IosManager};
#[cfg(target_os = "macos")]
use crate::constants::ios::{IOS_DEVICE_TYPE_PREFIX, IOS_INCH_PATTERN, IOS_INCH_REPLACEMENT};
#[cfg(target_os = "macos")]
use crate::constants::{
    commands::{SIMCTL, XCRUN},
    ios_devices::*,
    patterns::text_patterns::{
        APPLE_DEVICE_IPAD, APPLE_DEVICE_IPHONE, APPLE_DEVICE_IPOD, APPLE_DEVICE_PREFIX_I,
        CHIP_PREFIX_A, CHIP_PREFIX_M, INCH_INDICATOR, MEMORY_CLOSE_BRACKET, MEMORY_OPEN_BRACKET,
    },
};
#[cfg(target_os = "macos")]
use crate::models::device_info::DynamicDeviceConfig;
#[cfg(target_os = "macos")]
use anyhow::{Context, Result};
#[cfg(target_os = "macos")]
use serde_json::Value;
#[cfg(target_os = "macos")]
use std::path::Path;

#[cfg(target_os = "macos")]
impl IosManager {
    pub async fn list_device_types(&self) -> Result<Vec<String>> {
        let output = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "list", "devicetypes", "--json"])
            .await
            .context("Failed to list device types")?;
        let json: Value =
            serde_json::from_str(&output).context("Failed to parse device types JSON")?;
        let mut device_types = Vec::new();
        if let Some(types_array) = json.get("devicetypes").and_then(|v| v.as_array()) {
            for device_type_json in types_array {
                if let Some(identifier) =
                    device_type_json.get("identifier").and_then(|v| v.as_str())
                {
                    device_types.push(identifier.to_string());
                }
            }
        }
        Ok(device_types)
    }

    pub async fn list_device_types_with_names(&self) -> Result<Vec<(String, String)>> {
        let output = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "list", "devicetypes", "--json"])
            .await
            .context("Failed to list device types")?;
        let json: Value =
            serde_json::from_str(&output).context("Failed to parse device types JSON")?;
        let mut device_types = Vec::new();

        if let Some(types_array) = json.get("devicetypes").and_then(|v| v.as_array()) {
            for device_type_json in types_array {
                if let Some(identifier) =
                    device_type_json.get("identifier").and_then(|v| v.as_str())
                {
                    let display_name =
                        if let Some(name) = device_type_json.get("name").and_then(|v| v.as_str()) {
                            name.to_string()
                        } else {
                            Self::parse_device_type_display_name(identifier)
                        };

                    device_types.push((identifier.to_string(), display_name));
                }
            }
        }

        device_types.sort_by(|a, b| {
            let priority_a = DynamicDeviceConfig::calculate_ios_device_priority(&a.1);
            let priority_b = DynamicDeviceConfig::calculate_ios_device_priority(&b.1);
            priority_a.cmp(&priority_b)
        });

        Ok(device_types)
    }

    pub(super) fn parse_device_type_display_name(identifier: &str) -> String {
        let cleaned = identifier
            .replace(IOS_DEVICE_TYPE_PREFIX, "")
            .replace("-", " ")
            .replace("_", " ");

        let mut display = cleaned.replace(IOS_INCH_PATTERN, IOS_INCH_REPLACEMENT);
        display = display.replace(INCH_13_PATTERN, INCH_13_REPLACEMENT);
        display = display.replace(INCH_11_PATTERN, INCH_11_REPLACEMENT);
        display = display.replace(MEMORY_8GB_PATTERN, MEMORY_8GB_REPLACEMENT);
        display = display.replace(MEMORY_16GB_PATTERN, MEMORY_16GB_REPLACEMENT);

        display
            .split_whitespace()
            .map(|word| {
                let word_lower = word.to_lowercase();

                if word_lower == "inch"
                    || word_lower == "se"
                    || word_lower == "mini"
                    || word_lower == "max"
                    || word_lower == "plus"
                    || word_lower == "pro"
                    || word_lower == "air"
                    || word_lower == "ultra"
                    || word.contains(INCH_INDICATOR)
                    || word.contains(MEMORY_OPEN_BRACKET)
                    || word.contains(MEMORY_CLOSE_BRACKET)
                    || (word.starts_with(APPLE_DEVICE_PREFIX_I)
                        && (word.starts_with(APPLE_DEVICE_IPHONE)
                            || word.starts_with(APPLE_DEVICE_IPAD)
                            || word.starts_with(APPLE_DEVICE_IPOD)))
                    || word_lower.starts_with(CHIP_PREFIX_M) && word.len() <= 3
                    || word_lower.starts_with(CHIP_PREFIX_A)
                        && word.chars().nth(1).is_some_and(|c| c.is_ascii_digit())
                {
                    if (word_lower.starts_with(CHIP_PREFIX_M) && word.len() <= 3)
                        || (word_lower.starts_with(CHIP_PREFIX_A)
                            && word.chars().nth(1).is_some_and(|c| c.is_ascii_digit()))
                    {
                        return word.to_uppercase();
                    }

                    word.to_string()
                } else {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub async fn list_runtimes(&self) -> Result<Vec<(String, String)>> {
        let output = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "list", "runtimes", "--json"])
            .await
            .context("Failed to list runtimes")?;
        let json: Value = serde_json::from_str(&output).context("Failed to parse runtimes JSON")?;
        let mut runtimes = Vec::new();
        if let Some(runtimes_array) = json.get("runtimes").and_then(|v| v.as_array()) {
            for runtime_json in runtimes_array {
                if let Some(identifier) = runtime_json.get("identifier").and_then(|v| v.as_str()) {
                    if runtime_json
                        .get("isAvailable")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        let display_name =
                            if let Some(name) = runtime_json.get("name").and_then(|v| v.as_str()) {
                                name.to_string()
                            } else if let Some(version) =
                                runtime_json.get("version").and_then(|v| v.as_str())
                            {
                                format!("iOS {version}")
                            } else {
                                identifier
                                    .replace("com.apple.CoreSimulator.SimRuntime.", "")
                                    .replace("-", ".")
                                    .replace("iOS.", "iOS ")
                            };

                        runtimes.push((identifier.to_string(), display_name));
                    }
                }
            }
        }

        runtimes.sort_by(|a, b| {
            let version_a = extract_ios_version(&a.1);
            let version_b = extract_ios_version(&b.1);
            version_b
                .partial_cmp(&version_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(runtimes)
    }
}
