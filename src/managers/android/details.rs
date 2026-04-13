use super::{AndroidManager, AVD_NAME_REGEX, IMAGE_SYSDIR_REGEX, PATH_REGEX};
use crate::{
    constants::{defaults, env_vars::HOME, files, limits::STORAGE_MB_TO_GB_DIVISOR},
    managers::common::DeviceConfig,
    models::{DeviceDetails, Platform},
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

impl AndroidManager {
    /// Get the AVD directory path for a given AVD name
    pub(super) async fn get_avd_path(&self, avd_name: &str) -> Result<Option<PathBuf>> {
        let avd_output = self
            .command_executor
            .run(&self.avdmanager_path, &["list", "avd"])
            .await
            .context("Failed to list Android AVDs")?;

        let mut current_name = String::new();

        for line in avd_output.lines() {
            let trimmed = line.trim();
            if let Some(caps) = AVD_NAME_REGEX.captures(trimmed) {
                current_name = caps[1].to_string();
            } else if let Some(caps) = PATH_REGEX.captures(trimmed) {
                if current_name == avd_name {
                    return Ok(Some(PathBuf::from(caps[1].to_string())));
                }
            }
        }

        Ok(None)
    }

    /// Fine-tune AVD configuration after creation with avdmanager
    pub(super) async fn fine_tune_avd_config(
        &self,
        avd_name: &str,
        config: &DeviceConfig,
        _tag: &str,
        _abi: &str,
    ) -> Result<()> {
        if let Some(avd_path) = self.get_avd_path(avd_name).await? {
            let config_path = avd_path.join(files::CONFIG_FILE);

            let mut config_content = fs::read_to_string(&config_path)
                .await
                .context("Failed to read existing AVD configuration")?;

            let device_display_name = &config.name;

            let ram_mb = if let Some(ram) = &config.ram_size {
                ram.parse::<u32>().unwrap_or(0)
            } else {
                0
            };

            let storage_mb = if let Some(storage) = &config.storage_size {
                storage.parse::<u32>().unwrap_or(0)
            } else {
                0
            };

            let avd_id = device_display_name.replace(' ', "_");

            if !device_display_name.is_empty() {
                if config_content.contains("avd.ini.displayname=") {
                    if let Some(start) = config_content.find("avd.ini.displayname=") {
                        if let Some(end) = config_content[start..].find('\n') {
                            let line_end = start + end;
                            config_content.replace_range(
                                start..line_end,
                                &format!("avd.ini.displayname={device_display_name}"),
                            );
                        }
                    }
                } else if let Some(encoding_pos) = config_content.find("avd.ini.encoding=UTF-8\n") {
                    let insert_pos = encoding_pos + "avd.ini.encoding=UTF-8\n".len();
                    config_content.insert_str(
                        insert_pos,
                        &format!("avd.ini.displayname={device_display_name}\n"),
                    );
                } else {
                    config_content = format!(
                        "avd.ini.displayname={device_display_name}\navd.ini.encoding=UTF-8\n{config_content}"
                    );
                }
            }

            if !avd_id.is_empty() {
                if config_content.contains("AvdId=") {
                    if let Some(start) = config_content.find("AvdId=") {
                        if let Some(end) = config_content[start..].find('\n') {
                            let line_end = start + end;
                            config_content
                                .replace_range(start..line_end, &format!("AvdId={avd_id}"));
                        }
                    }
                } else if let Some(displayname_pos) = config_content.find("avd.ini.displayname=") {
                    if let Some(line_end) = config_content[displayname_pos..].find('\n') {
                        let insert_pos = displayname_pos + line_end + 1;
                        config_content.insert_str(insert_pos, &format!("AvdId={avd_id}\n"));
                    }
                }
            }

            if ram_mb > 0 {
                if let Some(start) = config_content.find("hw.ramSize=") {
                    if let Some(end) = config_content[start..].find('\n') {
                        let line_end = start + end;
                        config_content
                            .replace_range(start..line_end, &format!("hw.ramSize={ram_mb}"));
                    }
                }
            }

            if storage_mb > 0 {
                if let Some(start) = config_content.find("disk.dataPartition.size=") {
                    if let Some(end) = config_content[start..].find('\n') {
                        let line_end = start + end;
                        config_content.replace_range(
                            start..line_end,
                            &format!(
                                "disk.dataPartition.size={}G",
                                storage_mb / STORAGE_MB_TO_GB_DIVISOR
                            ),
                        );
                    }
                }
            }

            if config_content.contains("image.sysdir.1=")
                && !config_content.contains("image.sysdir.1=system-images/android-")
            {
                // Safety check for unexpected config values.
            } else if let Some(start) = config_content.find("image.sysdir.1=") {
                if let Some(end) = config_content[start..].find('\n') {
                    let line = &config_content[start..start + end];
                    if !line.ends_with('/') {
                        let line_end = start + end;
                        config_content.replace_range(start..line_end, &format!("{line}/"));
                    }
                }
            }

            fs::write(&config_path, config_content)
                .await
                .context("Failed to write updated AVD configuration")?;
        }

        Ok(())
    }

    /// Get detailed information for a specific AVD
    pub async fn get_device_details(
        &self,
        avd_name: &str,
        cached_info: Option<(String, u32, String)>,
    ) -> Result<DeviceDetails> {
        log::debug!("Getting device details for AVD: '{avd_name}'");

        let mut details = DeviceDetails {
            name: avd_name.to_string(),
            status: "Unknown".to_string(),
            platform: Platform::Android,
            device_type: cached_info
                .as_ref()
                .map(|(device_type, _, _)| device_type.clone())
                .unwrap_or_default(),
            api_level_or_version: cached_info
                .as_ref()
                .map(|(_, api_level, version)| format!("API {api_level} (Android {version})"))
                .unwrap_or_default(),
            ram_size: None,
            storage_size: None,
            resolution: None,
            dpi: None,
            device_path: None,
            system_image: None,
            identifier: avd_name.to_string(),
        };

        let running_avds = self.get_running_avd_names().await?;
        let is_running = running_avds.contains_key(avd_name);
        details.status = if is_running {
            "Running".to_string()
        } else {
            "Stopped".to_string()
        };

        if let Ok(home_dir) = std::env::var(HOME) {
            let config_path = PathBuf::from(&home_dir)
                .join(files::android::AVD_DIR)
                .join(files::android::AVD_SUBDIR)
                .join(format!("{avd_name}.avd"))
                .join(files::CONFIG_FILE);

            log::debug!("Checking config path: {config_path:?}");
            if config_path.exists() {
                log::debug!("Config file found, reading details for {avd_name}");
                if let Ok(config_content) = fs::read_to_string(&config_path).await {
                    let mut api_level = 0u32;
                    for line in config_content.lines() {
                        if let Some((key, value)) = line.split_once('=') {
                            match key.trim() {
                                "hw.ramSize" => {
                                    if let Ok(ram_mb) = value.trim().parse::<u64>() {
                                        details.ram_size = Some(format!("{ram_mb} MB"));
                                    }
                                }
                                "disk.dataPartition.size" => {
                                    let value = value.trim();
                                    if let Some(size_str) = value.strip_suffix('M') {
                                        if let Ok(size_mb) = size_str.parse::<u64>() {
                                            details.storage_size = Some(format!("{size_mb} MB"));
                                        }
                                    } else if let Some(size_str) = value.strip_suffix('G') {
                                        if let Ok(size_gb) = size_str.parse::<u64>() {
                                            details.storage_size = Some(format!(
                                                "{} MB",
                                                size_gb * STORAGE_MB_TO_GB_DIVISOR as u64
                                            ));
                                        }
                                    }
                                }
                                "hw.lcd.width" => {
                                    if let Ok(width) = value.trim().parse::<u32>() {
                                        details.resolution = Some(format!("{width}x?"));
                                    }
                                }
                                "hw.lcd.height" => {
                                    if let Ok(height) = value.trim().parse::<u32>() {
                                        if let Some(ref res) = details.resolution {
                                            if res.contains("x?") {
                                                let width = res.replace("x?", "");
                                                details.resolution =
                                                    Some(format!("{width}x{height}"));
                                            }
                                        } else {
                                            details.resolution = Some(format!("?x{height}"));
                                        }
                                    }
                                }
                                "hw.lcd.density" => {
                                    details.dpi = Some(format!("{dpi} DPI", dpi = value.trim()));
                                }
                                "image.sysdir.1" => {
                                    details.system_image = Some(value.trim().to_string());
                                    if let Some(caps) = IMAGE_SYSDIR_REGEX.captures(value.trim()) {
                                        if let Ok(parsed_api) = caps[1].parse::<u32>() {
                                            api_level = parsed_api;
                                        }
                                    }
                                }
                                "hw.device.name" => {
                                    details.device_type = value.trim().to_string();
                                }
                                _ => {}
                            }
                        }
                    }

                    if api_level > 0 {
                        let version_name = self.get_android_version_name(api_level);
                        details.api_level_or_version =
                            format!("API {api_level} (Android {version_name})");
                    }
                }

                details.device_path =
                    Some(config_path.parent().unwrap().to_string_lossy().to_string());
            } else {
                log::debug!("Config file not found for {avd_name}: {config_path:?}");
                let avd_path = PathBuf::from(&home_dir)
                    .join(files::android::AVD_DIR)
                    .join(files::android::AVD_SUBDIR)
                    .join(format!("{avd_name}.avd"));
                details.device_path = Some(avd_path.to_string_lossy().to_string());

                if details.ram_size.is_none() {
                    details.ram_size = Some(format!("{} MB", defaults::DEFAULT_RAM_MB));
                }
                if details.storage_size.is_none() {
                    details.storage_size = Some(format!("{} MB", defaults::DEFAULT_STORAGE_MB));
                }
                if details.device_type.is_empty() {
                    details.device_type = "Unknown Device".to_string();
                }
                if details.api_level_or_version.is_empty() {
                    details.api_level_or_version = "Unknown Version".to_string();
                }
            }
        } else {
            log::warn!("HOME environment variable not set, cannot determine device path");
        }

        if let Some(ref res) = details.resolution {
            if res.contains('?') {
                details.resolution = None;
            }
        }

        Ok(details)
    }
}
