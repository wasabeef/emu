use super::AndroidManager;
use crate::{
    constants::{
        defaults,
        limits::{
            MAX_DEVICE_NAME_CREATE_LENGTH, MAX_DEVICE_NAME_PARTS_PROCESS, MAX_ERROR_MESSAGE_LENGTH,
            MIN_STRING_LENGTH_FOR_MATCH,
        },
    },
    managers::common::{DeviceConfig, DeviceManager},
};
use anyhow::Result;

impl AndroidManager {
    /// Get appropriate skin name for device type using dynamic lookup
    pub(super) async fn get_appropriate_skin(
        &self,
        device_id: &str,
        device_display: &str,
    ) -> Option<String> {
        if device_id.is_empty() {
            return None;
        }

        let primary_skin = device_id.to_string();
        let available_skins = self
            .get_available_skins_from_sdk(device_id)
            .await
            .unwrap_or_default();

        if available_skins.iter().any(|skin| skin == &primary_skin) {
            return Some(primary_skin);
        }

        let display_based_skin = device_display
            .split('(')
            .next()
            .unwrap_or(device_display)
            .trim()
            .replace(' ', "_")
            .to_lowercase();

        if available_skins
            .iter()
            .any(|skin| skin == &display_based_skin)
        {
            return Some(display_based_skin);
        }

        let device_lower = device_id.to_lowercase();
        for skin in &available_skins {
            let skin_lower = skin.to_lowercase();
            if (device_lower.len() > MIN_STRING_LENGTH_FOR_MATCH
                && skin_lower.contains(&device_lower))
                || (skin_lower.len() > MIN_STRING_LENGTH_FOR_MATCH
                    && device_lower.contains(&skin_lower))
            {
                return Some(skin.clone());
            }
        }

        None
    }

    /// Diagnose AVD creation issues and provide specific solutions
    pub async fn diagnose_avd_creation_issues(&self, config: &DeviceConfig) -> Result<String> {
        let mut diagnosis = Vec::new();

        diagnosis.push("=== Android SDK Diagnosis ===".to_string());

        let available_images = self.list_available_system_images().await?;
        diagnosis.push(format!(
            "Available system images: {}",
            available_images.len()
        ));
        if available_images.is_empty() {
            diagnosis.push("❌ No system images found! Install with: sdkmanager \"system-images;android-XX;google_apis_playstore;arm64-v8a\"".to_string());
        } else {
            diagnosis.push("✅ System images available".to_string());
            diagnosis.push(format!(
                "First 3: {}",
                available_images
                    .iter()
                    .take(MAX_DEVICE_NAME_PARTS_PROCESS)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let available_devices = self.list_available_devices().await?;
        diagnosis.push(format!(
            "Available device types: {}",
            available_devices.len()
        ));
        if available_devices.is_empty() {
            diagnosis.push("❌ No device types found! Check Android SDK installation".to_string());
        } else {
            diagnosis.push("✅ Device types available".to_string());
            diagnosis.push(format!(
                "First 3: {}",
                available_devices
                    .iter()
                    .take(MAX_DEVICE_NAME_PARTS_PROCESS)
                    .map(|(id, display)| format!("{display} ({id})"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let (tag, abi) = if let Some((found_tag, found_abi)) = self
            .get_first_available_system_image(&config.version)
            .await?
        {
            (found_tag, found_abi)
        } else {
            (
                "google_apis_playstore".to_string(),
                defaults::default_abi().to_string(),
            )
        };

        let package_path = format!(
            "system-images;android-{version};{tag};{abi}",
            version = config.version,
        );
        let image_available = self
            .check_system_image_available(&config.version, &tag, &abi)
            .await
            .unwrap_or(false);

        diagnosis.push(format!("Required system image: {package_path}"));
        if image_available {
            diagnosis.push("✅ Required system image is available".to_string());
        } else {
            diagnosis.push("❌ Required system image NOT available".to_string());
            diagnosis.push(format!("Install with: sdkmanager \"{package_path}\""));
        }

        let device_id = Self::find_matching_device_id(&available_devices, &config.device_type);
        diagnosis.push(format!(
            "Required device type: {} ({})",
            config.device_type,
            device_id.as_deref().unwrap_or("NOT FOUND")
        ));
        if device_id.is_some() {
            diagnosis.push("✅ Required device type is available".to_string());
        } else {
            diagnosis.push("❌ Required device type NOT found".to_string());
            diagnosis.push("Suggestion: Use one of the available device types above".to_string());
        }

        Ok(diagnosis.join("\n"))
    }

    /// Find matching device ID from available devices list
    pub(super) fn find_matching_device_id(
        available_devices: &[(String, String)],
        device_type: &str,
    ) -> Option<String> {
        if let Some((id, _)) = available_devices.iter().find(|(id, _)| id == device_type) {
            return Some(id.clone());
        }

        if let Some((id, _)) = available_devices
            .iter()
            .find(|(_, display)| display == device_type)
        {
            return Some(id.clone());
        }

        let cleaned_config = device_type
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .to_lowercase();

        available_devices.iter().find_map(|(id, display)| {
            let cleaned_display = display
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect::<String>()
                .to_lowercase();

            if cleaned_config == cleaned_display {
                return Some(id.clone());
            }

            let config_words: Vec<&str> = cleaned_config.split_whitespace().collect();
            let display_words: Vec<&str> = cleaned_display.split_whitespace().collect();
            let important_words = ["galaxy", "pixel", "nexus", "tv", "wear", "automotive"];
            for word in &important_words {
                if cleaned_config.contains(word) && cleaned_display.contains(word) {
                    let config_specific: Vec<&str> = config_words
                        .iter()
                        .filter(|word| word.chars().any(|c| c.is_ascii_digit()) || word.len() > 4)
                        .cloned()
                        .collect();
                    let display_specific: Vec<&str> = display_words
                        .iter()
                        .filter(|word| word.chars().any(|c| c.is_ascii_digit()) || word.len() > 4)
                        .cloned()
                        .collect();

                    if !config_specific.is_empty() && !display_specific.is_empty() {
                        if config_specific
                            .iter()
                            .any(|word| display_specific.contains(word))
                        {
                            return Some(id.clone());
                        }
                    } else if config_specific.is_empty() && display_specific.is_empty() {
                        return Some(id.clone());
                    }
                }
            }

            None
        })
    }

    pub(super) async fn create_device_internal(&self, config: &DeviceConfig) -> Result<()> {
        let safe_name = config
            .name
            .chars()
            .filter_map(|c| match c {
                c if c.is_ascii_alphanumeric() || c == '.' || c == '-' => Some(c),
                ' ' | '_' => Some('_'),
                _ => None,
            })
            .collect::<String>()
            .trim_matches('_')
            .to_string();

        if safe_name.is_empty() {
            return Err(anyhow::anyhow!(
                "Device name '{}' contains only invalid characters and cannot be used for AVD creation.",
                config.name
            ));
        }

        let existing_devices = self.list_devices().await?;
        if existing_devices
            .iter()
            .any(|device| device.name == safe_name)
        {
            return Err(anyhow::anyhow!(
                "Device with name '{safe_name}' already exists. Please choose a different name or delete the existing device first."
            ));
        }

        let (tag, abi) = if let Some((found_tag, found_abi)) = self
            .get_first_available_system_image(&config.version)
            .await?
        {
            (found_tag, found_abi)
        } else {
            let default_tag = config
                .additional_options
                .get("tag")
                .map_or("google_apis_playstore", |value| value.as_str());
            let default_abi = config
                .additional_options
                .get("abi")
                .map_or(defaults::default_abi(), |value| value.as_str());
            (default_tag.to_string(), default_abi.to_string())
        };

        let package_path = format!("system-images;android-{};{};{}", config.version, tag, abi);

        let image_available = self
            .check_system_image_available(&config.version, &tag, &abi)
            .await
            .unwrap_or(false);

        if !image_available {
            let available_images = self.list_available_system_images().await?;
            return Err(anyhow::anyhow!(
                "System image '{}' not found. Install it with: sdkmanager \"{}\"\nAvailable images: {}",
                package_path, package_path, available_images.join(", ")
            ));
        }

        let mut args = vec!["create", "avd", "-n", &safe_name, "-k", &package_path];

        let device_param =
            if !config.device_type.is_empty() && config.device_type.to_lowercase() != "custom" {
                let available_devices = self.list_available_devices().await?;
                Self::find_matching_device_id(&available_devices, &config.device_type)
            } else {
                None
            };

        if let Some(ref device_id) = device_param {
            args.push("--device");
            args.push(device_id);
        } else {
            log::warn!(
                "Device type '{}' not found, using default device",
                config.device_type
            );
        }

        let skin_name = if let Some(ref device_id) = device_param {
            self.get_appropriate_skin(device_id, &config.device_type)
                .await
        } else {
            self.get_appropriate_skin(&config.device_type, &config.device_type)
                .await
        };

        if let Some(ref skin) = skin_name {
            args.push("--skin");
            args.push(skin);
        }

        let result = self
            .command_executor
            .run(&self.avdmanager_path, &args)
            .await;

        let result = if result.is_err() && skin_name.is_some() {
            let error_str = result.as_ref().unwrap_err().to_string();
            if error_str.to_lowercase().contains("skin") {
                log::warn!(
                    "Skin '{}' failed, retrying without skin",
                    skin_name.as_ref().unwrap()
                );
                let mut fallback_args =
                    vec!["create", "avd", "-n", &safe_name, "-k", &package_path];
                if let Some(ref device_id) = device_param {
                    fallback_args.push("--device");
                    fallback_args.push(device_id);
                }
                self.command_executor
                    .run(&self.avdmanager_path, &fallback_args)
                    .await
            } else {
                result
            }
        } else {
            result
        };

        match result {
            Ok(_) => {
                if let Err(error) = self
                    .fine_tune_avd_config(&safe_name, config, &tag, &abi)
                    .await
                {
                    eprintln!("Warning: Failed to fine-tune AVD configuration: {error}");
                }
                self.invalidate_device_metadata_cache(Some(&safe_name))
                    .await;
                Ok(())
            }
            Err(error) => {
                let error_str = error.to_string();
                let short_name = if safe_name.len() > MAX_DEVICE_NAME_CREATE_LENGTH {
                    format!(
                        "{name}...",
                        name = &safe_name[..MAX_DEVICE_NAME_CREATE_LENGTH - 3]
                    )
                } else {
                    safe_name.clone()
                };

                if error_str.contains("system image")
                    || error_str.contains("package path")
                    || error_str.contains("not installed")
                {
                    Err(anyhow::anyhow!(
                        "System image not installed for API {}\nRun: sdkmanager \"{}\"",
                        config.version,
                        package_path
                    ))
                } else if error_str.contains("license") || error_str.contains("accept") {
                    Err(anyhow::anyhow!(
                        "Android SDK licenses not accepted\nRun: sdkmanager --licenses"
                    ))
                } else if error_str.contains("already exists") {
                    Err(anyhow::anyhow!(
                        "AVD '{}' already exists\nDelete existing or choose different name",
                        config.name
                    ))
                } else if error_str.contains("device") && error_str.contains("not found") {
                    Err(anyhow::anyhow!(
                        "Device type '{}' not found\nCheck available types in device list",
                        config.device_type
                    ))
                } else {
                    let key_error = if error_str.contains("Error:") {
                        error_str
                            .split("Error:")
                            .nth(1)
                            .unwrap_or(&error_str)
                            .trim()
                    } else if error_str.contains("failed") {
                        error_str
                            .split("failed")
                            .nth(0)
                            .unwrap_or(&error_str)
                            .trim()
                    } else {
                        &error_str
                    };

                    let short_error = if key_error.len() > MAX_ERROR_MESSAGE_LENGTH {
                        format!(
                            "{error}...",
                            error = &key_error[..MAX_ERROR_MESSAGE_LENGTH - 3]
                        )
                    } else {
                        key_error.to_string()
                    };

                    Err(anyhow::anyhow!(
                        "AVD creation failed: {}\nAVD: {} | API: {}",
                        short_error,
                        short_name,
                        config.version
                    ))
                }
            }
        }
    }
}
