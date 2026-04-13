use super::{
    parser::AvdListParser, AndroidManager, API_LEVEL_REGEX, API_OR_ANDROID_REGEX, BASED_ON_REGEX,
    IMAGE_SYSDIR_REGEX, TARGET_CONFIG_REGEX,
};
use crate::{
    constants::{
        commands, defaults,
        env_vars::HOME,
        files,
        limits::STORAGE_MB_TO_GB_DIVISOR,
        timeouts::{DEVICE_START_WAIT_TIME, DEVICE_STATUS_CHECK_DELAY},
    },
    models::{device_info::DynamicDeviceConfig, AndroidDevice, DeviceStatus},
};
use anyhow::{Context, Result};
use std::cmp::Reverse;
use std::path::{Path, PathBuf};
use tokio::fs;

impl AndroidManager {
    /// Optimized parallel version of list_devices
    pub async fn list_devices_parallel(&self) -> Result<Vec<AndroidDevice>> {
        let avd_list_future = self
            .command_executor
            .run(&self.avdmanager_path, &["list", "avd"]);
        let running_avds_future = self.get_running_avd_names();
        let targets_future = self.list_available_targets();

        let (avd_output_result, running_avds_result, targets_result) =
            tokio::join!(avd_list_future, running_avds_future, targets_future);

        let avd_output = avd_output_result.context("Failed to list Android AVDs")?;
        let running_avds = running_avds_result?;
        let targets = targets_result.unwrap_or_default();

        let mut version_map = std::collections::HashMap::new();
        for (level_str, display) in targets {
            if let Ok(level) = level_str.parse::<u32>() {
                if let Some(dash_pos) = display.find(" - Android ") {
                    version_map.insert(level, display[dash_pos + 11..].to_string());
                }
            }
        }

        let mut parser = AvdListParser::new(&avd_output);
        let mut devices = Vec::new();

        while let Some((name, _path, target, _abi, device)) = parser.parse_next_device() {
            let is_running = running_avds.contains_key(&name);
            let api_level = self.detect_api_level_for_device(&name, &target).await;
            let ram_size = format!("{}", defaults::DEFAULT_RAM_MB);
            let storage_size = format!(
                "{}M",
                defaults::DEFAULT_STORAGE_MB / STORAGE_MB_TO_GB_DIVISOR
            );
            let android_version_name = version_map
                .get(&api_level)
                .cloned()
                .unwrap_or_else(|| self.get_android_version_name(api_level));

            devices.push(AndroidDevice {
                name,
                device_type: device,
                api_level,
                android_version_name,
                status: if is_running {
                    DeviceStatus::Running
                } else {
                    DeviceStatus::Stopped
                },
                is_running,
                ram_size,
                storage_size,
            });
        }

        sort_discovered_devices(&mut devices);
        Ok(devices)
    }

    pub(super) async fn detect_api_level_for_device(&self, name: &str, target: &str) -> u32 {
        let mut api = 0u32;

        if let Ok(home) = std::env::var(HOME) {
            let config_path = PathBuf::from(home)
                .join(files::android::AVD_DIR)
                .join("avd")
                .join(format!("{name}.avd"))
                .join(files::CONFIG_FILE);

            if let Ok(config_content) = fs::read_to_string(&config_path).await {
                if let Some(caps) = IMAGE_SYSDIR_REGEX.captures(&config_content) {
                    if let Ok(parsed_api) = caps[1].parse::<u32>() {
                        api = parsed_api;
                    }
                } else if let Some(caps) = TARGET_CONFIG_REGEX.captures(&config_content) {
                    if let Ok(parsed_api) = caps[1].parse::<u32>() {
                        api = parsed_api;
                    }
                }
            }
        }

        if api == 0 {
            if let Ok(Some(avd_path)) = self.get_avd_path(name).await {
                let config_path = avd_path.join(files::CONFIG_FILE);
                if let Ok(config_content) = fs::read_to_string(&config_path).await {
                    if let Some(caps) = IMAGE_SYSDIR_REGEX.captures(&config_content) {
                        if let Ok(parsed_api) = caps[1].parse::<u32>() {
                            api = parsed_api;
                        }
                    } else if let Some(caps) = TARGET_CONFIG_REGEX.captures(&config_content) {
                        if let Ok(parsed_api) = caps[1].parse::<u32>() {
                            api = parsed_api;
                        }
                    }
                }
            }
        }

        if api == 0 {
            if let Some(caps) = API_LEVEL_REGEX.captures(target) {
                api = caps[1].parse().unwrap_or(0);
            } else if let Some(caps) = API_OR_ANDROID_REGEX.captures(target) {
                api = caps[1].parse().unwrap_or(0);
            } else if let Some(caps) = BASED_ON_REGEX.captures(target) {
                let version = &caps[1];
                api = Self::parse_android_version_to_api_level(version);
            }
        }

        api
    }

    pub(super) async fn start_device_internal(&self, identifier: &str) -> Result<()> {
        let args = vec![
            "-avd",
            identifier,
            "-no-audio",
            "-no-snapshot-save",
            "-no-boot-anim",
            "-netfast",
        ];

        self.command_executor
            .spawn(&self.emulator_path, &args)
            .await?;
        Ok(())
    }

    pub(super) async fn stop_device_internal(&self, identifier: &str) -> Result<()> {
        let running_avds = self.get_running_avd_names().await?;

        if let Some(emulator_id) = running_avds.get(identifier) {
            let shutdown_result = self
                .command_executor
                .run(
                    Path::new(commands::ADB),
                    &[
                        "-s",
                        emulator_id,
                        "shell",
                        "am",
                        "broadcast",
                        "-a",
                        "android.intent.action.ACTION_SHUTDOWN",
                    ],
                )
                .await;

            if shutdown_result.is_ok() {
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    DEVICE_STATUS_CHECK_DELAY.as_millis() as u64,
                ))
                .await;

                let _ = self
                    .command_executor
                    .run(
                        Path::new(commands::ADB),
                        &["-s", emulator_id, "shell", "reboot", "-p"],
                    )
                    .await;
            } else {
                self.command_executor
                    .run(
                        Path::new(commands::ADB),
                        &["-s", emulator_id, "emu", "kill"],
                    )
                    .await
                    .context(format!("Failed to stop emulator {emulator_id}"))?;
            }
        }

        Ok(())
    }

    pub(super) async fn delete_device_internal(&self, identifier: &str) -> Result<()> {
        let running_avds = self.get_running_avd_names().await.unwrap_or_default();
        if running_avds.contains_key(identifier) {
            log::info!("Device '{identifier}' is running, stopping before deletion");
            if let Err(e) = self.stop_device_internal(identifier).await {
                log::warn!("Failed to stop device '{identifier}' before deletion: {e}");
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(
                DEVICE_START_WAIT_TIME.as_secs(),
            ))
            .await;
        }

        self.command_executor
            .run(&self.avdmanager_path, &["delete", "avd", "-n", identifier])
            .await
            .context(format!("Failed to delete Android AVD '{identifier}'"))?;
        Ok(())
    }

    pub(super) async fn wipe_device_internal(&self, identifier: &str) -> Result<()> {
        let running_avds = self.get_running_avd_names().await?;
        if running_avds.contains_key(identifier) {
            log::info!("Device '{identifier}' is running, stopping before wipe");
            self.stop_device_internal(identifier).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(
                DEVICE_STATUS_CHECK_DELAY.as_millis() as u64,
            ))
            .await;
        }

        if let Ok(home_dir) = std::env::var(HOME) {
            let avd_path = PathBuf::from(home_dir)
                .join(files::android::AVD_DIR)
                .join("avd")
                .join(format!("{identifier}.avd"));

            if avd_path.exists() {
                let files_to_delete = [
                    "userdata.img",
                    "userdata-qemu.img",
                    "cache.img",
                    "cache.img.qcow2",
                    "userdata.img.qcow2",
                    "sdcard.img",
                    "sdcard.img.qcow2",
                    "multiinstance.lock",
                ];

                for file_name in &files_to_delete {
                    let file_path = avd_path.join(file_name);
                    if file_path.exists() {
                        if let Err(e) = tokio::fs::remove_file(&file_path).await {
                            log::warn!("Failed to remove {}: {}", file_path.display(), e);
                        } else {
                            log::debug!("Removed user data file: {}", file_path.display());
                        }
                    }
                }

                let snapshots_dir = avd_path.join("snapshots");
                if snapshots_dir.exists() {
                    if let Err(e) = tokio::fs::remove_dir_all(&snapshots_dir).await {
                        log::warn!("Failed to remove snapshots directory: {e}");
                    } else {
                        log::debug!("Removed snapshots directory");
                    }
                }

                log::info!("Successfully wiped user data for device '{identifier}'");
            } else {
                return Err(anyhow::anyhow!(
                    "AVD directory not found: {}",
                    avd_path.display()
                ));
            }
        } else {
            return Err(anyhow::anyhow!("HOME environment variable not set"));
        }

        Ok(())
    }
}

fn sort_discovered_devices(devices: &mut [AndroidDevice]) {
    devices.sort_by(|left, right| {
        let left_priority =
            DynamicDeviceConfig::calculate_android_device_priority(&left.device_type, &left.name);
        let right_priority =
            DynamicDeviceConfig::calculate_android_device_priority(&right.device_type, &right.name);

        (
            Reverse(left.api_level),
            left_priority,
            left.name.to_lowercase(),
            left.device_type.to_lowercase(),
        )
            .cmp(&(
                Reverse(right.api_level),
                right_priority,
                right.name.to_lowercase(),
                right.device_type.to_lowercase(),
            ))
    });
}
