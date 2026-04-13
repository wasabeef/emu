use super::{AppState, Panel};
use crate::models::{DeviceDetails, Platform};

impl AppState {
    /// Gets details for the currently selected device.
    /// Returns cached details if available and matching current selection.
    /// Falls back to generating basic details from device data if cache miss.
    pub fn get_selected_device_details(&self) -> Option<DeviceDetails> {
        if let Some(ref cached) = self.cached_device_details {
            let current_identifier = match self.active_panel {
                Panel::Android => self
                    .android_devices
                    .get(self.selected_android)
                    .map(|d| d.name.clone()),
                Panel::Ios => self
                    .ios_devices
                    .get(self.selected_ios)
                    .map(|d| d.udid.clone()),
            };

            if Some(cached.identifier.clone()) == current_identifier {
                return Some(cached.clone());
            }
        }

        match self.active_panel {
            Panel::Android => self
                .android_devices
                .get(self.selected_android)
                .map(|device| DeviceDetails {
                    name: device.name.clone(),
                    status: if device.is_running {
                        "Running".to_string()
                    } else {
                        "Stopped".to_string()
                    },
                    platform: Platform::Android,
                    device_type: device.device_type.clone(),
                    api_level_or_version: format!(
                        "API {} (Android {})",
                        device.api_level, device.android_version_name
                    ),
                    ram_size: None,
                    storage_size: None,
                    resolution: None,
                    dpi: None,
                    device_path: {
                        if let Ok(home) = std::env::var("HOME") {
                            Some(format!("{home}/.android/avd/{}.avd", device.name))
                        } else {
                            None
                        }
                    },
                    system_image: None,
                    identifier: device.name.clone(),
                }),
            Panel::Ios => self
                .ios_devices
                .get(self.selected_ios)
                .map(|device| DeviceDetails {
                    name: device.name.clone(),
                    status: if device.is_running {
                        "Booted".to_string()
                    } else {
                        "Shutdown".to_string()
                    },
                    platform: Platform::Ios,
                    device_type: device.device_type.clone(),
                    api_level_or_version: device.runtime_version.clone(),
                    ram_size: None,
                    storage_size: None,
                    resolution: None,
                    dpi: None,
                    device_path: None,
                    system_image: None,
                    identifier: device.udid.clone(),
                }),
        }
    }

    /// Updates the cached device details with new information.
    /// This cache is used to avoid repeated expensive device queries.
    pub fn update_cached_device_details(&mut self, details: DeviceDetails) {
        log::debug!(
            "Updating cached device details for '{}' (platform: {:?}, has_path: {})",
            details.name,
            details.platform,
            details.device_path.is_some()
        );
        self.cached_device_details = Some(details);
    }

    /// Clears all cached device details.
    pub fn clear_cached_device_details(&mut self) {
        self.cached_device_details = None;
    }

    /// Intelligently clears cached device details only when switching platforms.
    /// Preserves cache when staying on the same platform.
    pub fn smart_clear_cached_device_details(&mut self, new_panel: Panel) {
        if let Some(ref cached) = self.cached_device_details {
            let new_platform = match new_panel {
                Panel::Android => Platform::Android,
                Panel::Ios => Platform::Ios,
            };
            if cached.platform != new_platform {
                self.cached_device_details = None;
            }
        }
    }

    /// Get cached Android device info for use in device details.
    /// This avoids calling list_devices() again when fetching details.
    pub fn get_cached_android_device(&self, name: &str) -> Option<(String, u32, String)> {
        self.android_devices
            .iter()
            .find(|d| d.name == name)
            .map(|d| {
                (
                    d.device_type.clone(),
                    d.api_level,
                    d.android_version_name.clone(),
                )
            })
    }
}
