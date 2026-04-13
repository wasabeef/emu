use super::{AndroidManager, ID_REGEX, NAME_REGEX, OEM_REGEX};
use crate::{
    constants::{
        commands, env_vars,
        limits::{ANDROID_COMMAND_PARTS_MINIMUM, SYSTEM_IMAGE_PARTS_REQUIRED},
    },
    models::{
        device_info::{
            ApiLevelInfo, DeviceCategory, DeviceInfo, DynamicDeviceConfig, DynamicDeviceProvider,
        },
        ApiLevel,
    },
    utils::ApiLevelCache,
};
use anyhow::{Context, Result};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use tokio::fs;

impl AndroidManager {
    pub async fn get_running_avd_names(&self) -> Result<HashMap<String, String>> {
        let mut avd_map = HashMap::new();
        let mut normalized_map = HashMap::new();

        let adb_output = self
            .command_executor
            .run(Path::new(commands::ADB), &[commands::adb::DEVICES])
            .await
            .unwrap_or_default();

        for line in adb_output.lines() {
            if line.contains("emulator-") && line.contains("device") {
                if let Some(emulator_id) = line.split_whitespace().next() {
                    if let Ok(boot_prop_output) = self
                        .command_executor
                        .run(
                            Path::new(commands::ADB),
                            &[
                                "-s",
                                emulator_id,
                                "shell",
                                "getprop",
                                "ro.boot.qemu.avd_name",
                            ],
                        )
                        .await
                    {
                        let avd_name = boot_prop_output.trim().to_string();
                        if !avd_name.is_empty() {
                            avd_map.insert(avd_name.clone(), emulator_id.to_string());
                            let normalized = avd_name.replace(' ', "_");
                            if normalized != avd_name {
                                normalized_map.insert(normalized, emulator_id.to_string());
                            }
                            continue;
                        }
                    }

                    if let Ok(avd_name_output) = self
                        .command_executor
                        .run(
                            Path::new(commands::ADB),
                            &["-s", emulator_id, "emu", "avd", "name"],
                        )
                        .await
                    {
                        let avd_name = avd_name_output
                            .lines()
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();

                        if !avd_name.is_empty()
                            && !avd_name.contains("error")
                            && !avd_name.contains("KO")
                            && !avd_name.contains("unknown command")
                            && avd_name != "OK"
                        {
                            avd_map.insert(avd_name.clone(), emulator_id.to_string());
                            let normalized = avd_name.replace(' ', "_");
                            if normalized != avd_name {
                                normalized_map.insert(normalized, emulator_id.to_string());
                            }
                            continue;
                        }
                    }

                    if let Ok(prop_output) = self
                        .command_executor
                        .run(
                            Path::new(commands::ADB),
                            &[
                                "-s",
                                emulator_id,
                                "shell",
                                "getprop",
                                "ro.kernel.qemu.avd_name",
                            ],
                        )
                        .await
                    {
                        let avd_name = prop_output.trim().to_string();
                        if !avd_name.is_empty() {
                            avd_map.insert(avd_name.clone(), emulator_id.to_string());
                            let normalized = avd_name.replace(' ', "_");
                            if normalized != avd_name {
                                normalized_map.insert(normalized, emulator_id.to_string());
                            }
                            continue;
                        }
                    }
                }
            }
        }

        for (normalized_name, serial) in normalized_map {
            avd_map.entry(normalized_name).or_insert(serial);
        }

        Ok(avd_map)
    }

    pub async fn list_available_targets(&self) -> Result<Vec<(String, String)>> {
        if let Some(cached_targets) = self.get_cached_available_targets().await {
            return Ok(cached_targets);
        }

        log::debug!("list_available_targets called");
        let installed_images = self.list_available_system_images().await?;
        let mut targets = std::collections::HashMap::new();

        for image in installed_images {
            let parts: Vec<&str> = image.split(';').collect();
            if parts.len() >= SYSTEM_IMAGE_PARTS_REQUIRED {
                if let Some(api_level) = parts.get(1).and_then(|part| part.strip_prefix("android-"))
                {
                    let api_num: u32 = api_level.parse().unwrap_or(0);
                    let android_version = self.get_android_version_name(api_num);
                    let display = format!("API {api_level} - {android_version}");
                    targets.insert(api_level.to_string(), display);
                }
            }
        }

        let mut result: Vec<(String, String)> = targets.into_iter().collect();
        result.sort_by(|a, b| {
            let api_a: u32 = a.0.parse().unwrap_or(0);
            let api_b: u32 = b.0.parse().unwrap_or(0);
            api_b.cmp(&api_a)
        });

        let api_levels: Vec<ApiLevel> = result
            .iter()
            .map(|(level_str, display)| {
                let api: u32 = level_str.parse().unwrap_or(0);
                let version = if let Some(dash_pos) = display.find(" - ") {
                    display[dash_pos + 3..].to_string()
                } else {
                    format!("API {api}")
                };
                ApiLevel {
                    api,
                    version,
                    display_name: display.clone(),
                    system_image_id: format!("android-{api}"),
                    is_installed: true,
                    variants: vec![],
                }
            })
            .collect();

        let cache = ApiLevelCache {
            api_levels,
            timestamp: std::time::SystemTime::now(),
        };

        if result.is_empty() {
            if let Err(error) = ApiLevelCache::clear_from_disk() {
                log::warn!("Failed to clear API level cache: {error}");
            } else {
                log::debug!("Cleared API level cache because no installed targets were found");
            }
        } else if let Err(error) = cache.save_to_disk() {
            log::warn!("Failed to save API level cache: {error}");
        } else {
            log::debug!("Saved {} API levels to cache", result.len());
        }

        self.set_cached_available_targets(result.clone()).await;
        log::debug!(
            "list_available_targets completed, returning {} targets",
            result.len()
        );

        Ok(result)
    }

    pub async fn list_available_devices(&self) -> Result<Vec<(String, String)>> {
        let output = self
            .command_executor
            .run(&self.avdmanager_path, &["list", "device"])
            .await
            .context("Failed to list Android devices")?;

        let mut devices = Vec::new();
        let mut current_id = String::new();
        let mut current_name = String::new();
        let mut current_oem = String::new();

        for line in output.lines() {
            if let Some(caps) = ID_REGEX.captures(line) {
                current_id = caps[1].to_string();
            } else if let Some(caps) = NAME_REGEX.captures(line) {
                current_name = caps[1].to_string();
            } else if let Some(caps) = OEM_REGEX.captures(line) {
                current_oem = caps[1].to_string();
            } else if line.contains("-----") && !current_id.is_empty() {
                let display = if !current_oem.is_empty() && current_oem != "Generic" {
                    format!("{current_name} ({current_oem})")
                } else {
                    current_name.clone()
                };

                devices.push((current_id.clone(), display));
                current_id.clear();
                current_name.clear();
                current_oem.clear();
            }
        }

        if !current_id.is_empty() {
            let display = if !current_oem.is_empty() && current_oem != "Generic" {
                format!("{current_name} ({current_oem})")
            } else {
                current_name.clone()
            };
            devices.push((current_id, display));
        }

        if devices.is_empty() {
            log::warn!(
                "No Android device definitions found. Please check your Android SDK installation."
            );
        }

        devices.sort_by(|a, b| {
            let priority_a = DynamicDeviceConfig::calculate_android_device_priority(&a.0, &a.1);
            let priority_b = DynamicDeviceConfig::calculate_android_device_priority(&b.0, &b.1);
            priority_a.cmp(&priority_b)
        });

        Ok(devices)
    }

    pub fn get_device_category(&self, device_id: &str, device_display: &str) -> String {
        let combined = format!(
            "{} {}",
            device_id.to_lowercase(),
            device_display.to_lowercase()
        );

        if combined.contains("phone")
            || combined.contains("pixel")
                && !combined.contains("fold")
                && !combined.contains("tablet")
            || combined.contains("galaxy")
                && !combined.contains("fold")
                && !combined.contains("tablet")
            || combined.contains("oneplus")
            || combined.contains("iphone")
            || Self::is_phone_size(&combined)
            || (combined.contains("pro")
                && !combined.contains("tablet")
                && !combined.contains("fold"))
        {
            return "phone".to_string();
        }

        if combined.contains("tablet")
            || combined.contains("pad")
            || Self::is_tablet_size(&combined)
        {
            return "tablet".to_string();
        }

        if combined.contains("wear")
            || combined.contains("watch")
            || combined.contains("round") && !combined.contains("tablet")
            || combined.contains("square") && !combined.contains("tablet")
        {
            return "wear".to_string();
        }

        if combined.contains("tv")
            || combined.contains("1080p")
            || combined.contains("4k")
            || combined.contains("720p")
        {
            return "tv".to_string();
        }

        if combined.contains("auto") || combined.contains("car") || combined.contains("automotive")
        {
            return "automotive".to_string();
        }

        if combined.contains("desktop")
            || combined.contains("foldable") && combined.contains("large")
            || Self::is_desktop_size(&combined)
        {
            return "desktop".to_string();
        }

        "phone".to_string()
    }

    fn is_phone_size(combined: &str) -> bool {
        if !combined.contains("inch") {
            return false;
        }

        for size in ["5", "6"] {
            if combined.contains(size) {
                return true;
            }
        }
        false
    }

    fn is_tablet_size(combined: &str) -> bool {
        if !combined.contains("inch") {
            return false;
        }

        for size in ["10", "11", "12", "13"] {
            if combined.contains(size) {
                return true;
            }
        }
        false
    }

    fn is_desktop_size(combined: &str) -> bool {
        if !combined.contains("inch") {
            return false;
        }

        for size in ["15", "17"] {
            if combined.contains(size) {
                return true;
            }
        }
        false
    }

    pub async fn list_devices_by_category(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<(String, String)>> {
        let all_devices = self.list_available_devices().await?;

        if let Some(filter_category) = category {
            if filter_category == "all" {
                return Ok(all_devices);
            }

            let filtered_devices: Vec<(String, String)> = all_devices
                .into_iter()
                .filter(|(id, display)| self.get_device_category(id, display) == filter_category)
                .collect();

            Ok(filtered_devices)
        } else {
            Ok(all_devices)
        }
    }

    pub(super) async fn get_available_skins_from_sdk(
        &self,
        _device_id: &str,
    ) -> Result<Vec<String>> {
        let mut skins = Vec::new();

        if let Ok(android_home) = std::env::var(env_vars::ANDROID_HOME) {
            let android_path = std::path::PathBuf::from(&android_home);

            let standard_skins = android_path.join("skins");
            if standard_skins.exists() {
                self.scan_skin_directory(&standard_skins, &mut skins).await;
            }

            let platforms_dir = android_path.join("platforms");
            if platforms_dir.exists() {
                if let Ok(mut platform_entries) = fs::read_dir(&platforms_dir).await {
                    while let Some(platform_entry) =
                        platform_entries.next_entry().await.ok().flatten()
                    {
                        if let Ok(file_type) = platform_entry.file_type().await {
                            if file_type.is_dir() {
                                let platform_skins = platform_entry.path().join("skins");
                                if platform_skins.exists() {
                                    self.scan_skin_directory(&platform_skins, &mut skins).await;
                                }
                            }
                        }
                    }
                }
            }

            let system_images_dir = android_path.join("system-images");
            if system_images_dir.exists() {
                self.scan_system_images_for_skins(&system_images_dir, &mut skins)
                    .await;
            }
        }

        if let Ok(available_devices) = self.list_available_devices().await {
            for (id, _) in available_devices {
                skins.push(id);
            }
        }

        skins.sort();
        skins.dedup();

        Ok(skins)
    }

    async fn scan_skin_directory(&self, skin_dir: &std::path::Path, skins: &mut Vec<String>) {
        if let Ok(mut entries) = fs::read_dir(skin_dir).await {
            while let Some(entry) = entries.next_entry().await.ok().flatten() {
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_dir() {
                        if let Some(skin_name) = entry.file_name().to_str() {
                            skins.push(skin_name.to_string());
                        }
                    }
                }
            }
        }
    }

    async fn scan_system_images_for_skins(
        &self,
        system_images_dir: &std::path::Path,
        skins: &mut Vec<String>,
    ) {
        if let Ok(mut api_entries) = fs::read_dir(system_images_dir).await {
            while let Some(api_entry) = api_entries.next_entry().await.ok().flatten() {
                if let Ok(file_type) = api_entry.file_type().await {
                    if file_type.is_dir() {
                        let api_dir = api_entry.path();
                        if let Ok(mut tag_entries) = fs::read_dir(&api_dir).await {
                            while let Some(tag_entry) =
                                tag_entries.next_entry().await.ok().flatten()
                            {
                                if let Ok(file_type) = tag_entry.file_type().await {
                                    if file_type.is_dir() {
                                        let tag_dir = tag_entry.path();
                                        if let Ok(mut abi_entries) = fs::read_dir(&tag_dir).await {
                                            while let Some(abi_entry) =
                                                abi_entries.next_entry().await.ok().flatten()
                                            {
                                                if let Ok(file_type) = abi_entry.file_type().await {
                                                    if file_type.is_dir() {
                                                        let skins_dir =
                                                            abi_entry.path().join("skins");
                                                        if skins_dir.exists() {
                                                            self.scan_skin_directory(
                                                                &skins_dir, skins,
                                                            )
                                                            .await;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub(super) async fn get_available_tags_for_api_level(
        &self,
        api_level: u32,
    ) -> Result<Vec<String>> {
        let images = self.list_available_system_images().await?;
        let mut tags = HashSet::new();

        for image in images {
            if image.contains(&format!("android-{api_level}")) {
                let parts: Vec<&str> = image.split(';').collect();
                if parts.len() >= ANDROID_COMMAND_PARTS_MINIMUM {
                    tags.insert(parts[2].to_string());
                }
            }
        }

        Ok(tags.into_iter().collect())
    }
}

impl DynamicDeviceProvider for AndroidManager {
    async fn get_available_devices(&self) -> Result<Vec<DeviceInfo>> {
        let devices = self.list_available_devices().await?;

        let mut device_infos = Vec::new();
        for (id, display_name) in devices {
            let category = DeviceCategory::Unknown;
            let oem = if display_name.contains('(') && display_name.contains(')') {
                let start = display_name.find('(').unwrap() + 1;
                let end = display_name.find(')').unwrap();
                Some(display_name[start..end].to_string())
            } else {
                None
            };

            device_infos.push(DeviceInfo {
                id,
                display_name,
                oem,
                category,
            });
        }

        Ok(device_infos)
    }

    async fn get_available_api_levels(&self) -> Result<Vec<ApiLevelInfo>> {
        let targets = self.list_available_targets().await?;

        let mut api_infos = Vec::new();
        for (api_level_str, display) in targets {
            if let Ok(level) = api_level_str.parse::<u32>() {
                let version_name = if let Some(dash_pos) = display.find(" - ") {
                    display[dash_pos + 3..].to_string()
                } else {
                    self.get_dynamic_android_version_name(level)
                        .await
                        .unwrap_or_else(|| format!("API {level}"))
                };

                let available_tags = self
                    .get_available_tags_for_api_level(level)
                    .await
                    .unwrap_or_default();

                api_infos.push(ApiLevelInfo {
                    level,
                    version_name,
                    available_tags,
                });
            }
        }

        api_infos.sort_by(|a, b| b.level.cmp(&a.level));
        Ok(api_infos)
    }

    async fn get_available_skins(&self, device_id: &str) -> Result<Vec<String>> {
        self.get_available_skins_from_sdk(device_id).await
    }

    async fn get_device_priority(&self, device_id: &str) -> Result<u32> {
        Ok(DynamicDeviceConfig::calculate_android_device_priority(
            device_id, "",
        ))
    }
}
