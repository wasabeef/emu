#[cfg(target_os = "macos")]
use super::IosManager;
#[cfg(target_os = "macos")]
use crate::constants::{
    commands::{KILLALL, OSASCRIPT, SIMCTL, XCRUN},
    ios::{
        IOS_ALREADY_BOOTED_ERROR, IOS_ALREADY_SHUTDOWN_ERROR, IOS_DEVICE_STATUS_BOOTED,
        SIMULATOR_APP_NAME, SIMULATOR_OPEN_FLAG, SIMULATOR_QUIT_COMMAND,
    },
    numeric::IOS_DEVICE_PARSE_BATCH_SIZE,
};
#[cfg(target_os = "macos")]
use crate::managers::common::DeviceConfig;
#[cfg(target_os = "macos")]
use crate::models::{device_info::DynamicDeviceConfig, IosDevice};
#[cfg(target_os = "macos")]
use anyhow::{Context, Result};
#[cfg(target_os = "macos")]
use serde_json::Value;
#[cfg(target_os = "macos")]
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
impl IosManager {
    async fn quit_simulator_if_no_running_devices(&self) {
        match self.list_devices_internal().await {
            Ok(devices) => {
                let has_running_devices = devices.iter().any(|device| device.is_running);

                if !has_running_devices {
                    log::info!("No iOS devices are running, quitting Simulator.app");

                    if let Err(e) = self
                        .command_executor
                        .run(Path::new(OSASCRIPT), &["-e", SIMULATOR_QUIT_COMMAND])
                        .await
                    {
                        log::warn!("Failed to quit Simulator.app gracefully: {e}");

                        if let Err(e2) = self
                            .command_executor
                            .run(Path::new(KILLALL), &[SIMULATOR_APP_NAME])
                            .await
                        {
                            log::warn!("Failed to force quit Simulator.app: {e2}");
                        }
                    }
                } else {
                    log::debug!("Other iOS devices are still running, keeping Simulator.app open");
                }
            }
            Err(e) => {
                log::warn!("Failed to check device status before quitting Simulator.app: {e}");
            }
        }
    }

    pub(super) async fn list_devices_internal(&self) -> Result<Vec<IosDevice>> {
        let output = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "list", "devices", "--json"])
            .await
            .context("Failed to list iOS devices")?;
        let json: Value =
            serde_json::from_str(&output).context("Failed to parse simctl JSON output")?;

        let mut devices = Vec::new();
        if let Some(devices_obj) = json.get("devices") {
            if let Some(devices_map) = devices_obj.as_object() {
                let mut raw_devices = Vec::new();

                for (runtime, device_list_json) in devices_map {
                    if let Some(device_array_json) = device_list_json.as_array() {
                        for device_json_val in device_array_json {
                            raw_devices.push((device_json_val, runtime));
                        }
                    }
                }

                for batch in raw_devices.chunks(IOS_DEVICE_PARSE_BATCH_SIZE) {
                    for (device_json_val, runtime) in batch {
                        if let Some(parsed_device) =
                            self.parse_device_from_json(device_json_val, runtime)?
                        {
                            devices.push(parsed_device);
                        }
                    }
                }
            }
        }

        devices.sort_by(|a, b| {
            let priority_a = DynamicDeviceConfig::calculate_ios_device_priority(&a.name);
            let priority_b = DynamicDeviceConfig::calculate_ios_device_priority(&b.name);
            priority_a.cmp(&priority_b)
        });

        Ok(devices)
    }

    pub(super) async fn start_device_internal(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to start iOS device: {identifier}");

        let status_output = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "list", "devices", "-j"])
            .await
            .context("Failed to get device status")?;

        let json: Value =
            serde_json::from_str(&status_output).context("Failed to parse device status")?;

        let mut is_already_booted = false;
        if let Some(devices) = json.get("devices").and_then(|v| v.as_object()) {
            for (_, device_list) in devices {
                if let Some(devices_array) = device_list.as_array() {
                    for device in devices_array {
                        if let Some(udid) = device.get("udid").and_then(|v| v.as_str()) {
                            if udid == identifier {
                                if let Some(state) = device.get("state").and_then(|v| v.as_str()) {
                                    if state == IOS_DEVICE_STATUS_BOOTED {
                                        is_already_booted = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if is_already_booted {
            log::info!("Device {identifier} is already booted");
        } else {
            let boot_result = self
                .command_executor
                .run(Path::new(XCRUN), &[SIMCTL, "boot", identifier])
                .await;

            match boot_result {
                Ok(_) => log::info!("Successfully booted iOS device {identifier}"),
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains(IOS_ALREADY_BOOTED_ERROR) {
                        log::info!("Device {identifier} was already in the process of booting");
                    } else {
                        return Err(e).context(format!("Failed to boot iOS device {identifier}"));
                    }
                }
            }
        }

        if let Err(e) = self
            .command_executor
            .spawn(
                Path::new("open"),
                &[SIMULATOR_OPEN_FLAG, SIMULATOR_APP_NAME],
            )
            .await
        {
            log::warn!("Failed to open Simulator app: {e}. Device might be booting in headless mode or Simulator app needs to be opened manually.");
        }

        Ok(())
    }

    pub(super) async fn stop_device_internal(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to stop iOS device: {identifier}");

        let shutdown_result = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "shutdown", identifier])
            .await;

        match shutdown_result {
            Ok(_) => {
                log::info!("Successfully shut down iOS device {identifier}");
                self.quit_simulator_if_no_running_devices().await;
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains(IOS_ALREADY_SHUTDOWN_ERROR) {
                    log::info!("Device {identifier} was already shut down");
                    self.quit_simulator_if_no_running_devices().await;
                    Ok(())
                } else {
                    Err(e).context(format!("Failed to shutdown iOS device {identifier}"))
                }
            }
        }
    }

    pub(super) async fn create_device_internal(&self, config: &DeviceConfig) -> Result<()> {
        log::info!(
            "Attempting to create iOS device: {} of type {} with runtime {}",
            config.name,
            config.device_type,
            config.version
        );
        let output = self
            .command_executor
            .run(
                Path::new(XCRUN),
                &[
                    SIMCTL,
                    "create",
                    &config.name,
                    &config.device_type,
                    &config.version,
                ],
            )
            .await
            .context(format!(
                "Failed to create iOS device '{}' with type '{}' and runtime '{}'",
                config.name, config.device_type, config.version
            ))?;
        log::info!("Successfully created iOS device. UDID: {}", output.trim());
        Ok(())
    }

    pub(super) async fn delete_device_internal(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to delete iOS device: {identifier}");

        let _ = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "shutdown", identifier])
            .await;

        self.command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "delete", identifier])
            .await
            .context(format!(
                "Failed to delete iOS device {identifier}. Make sure the device exists and is not in use."
            ))?;

        log::info!("Successfully deleted iOS device {identifier}");
        Ok(())
    }

    pub(super) async fn wipe_device_internal(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to wipe iOS device: {identifier}");
        self.erase_device(identifier).await
    }

    pub(super) async fn is_available_internal(&self) -> bool {
        if which::which("xcrun").is_err() {
            return false;
        }

        self.command_executor
            .run(&PathBuf::from("xcrun"), &["simctl", "help"])
            .await
            .is_ok()
    }
}
