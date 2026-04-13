use super::{App, Panel};
use crate::managers::common::DeviceManager;
use crate::managers::AndroidManager;
use crate::models::{DeviceDetails, Platform};
use std::sync::Arc;

impl App {
    /// Start background device info cache loading
    pub(super) fn start_background_cache_loading(&mut self) {
        let state_clone = Arc::clone(&self.state);

        tokio::spawn(async move {
            if let Ok(android_manager) = crate::managers::AndroidManager::new() {
                let (device_types_result, api_levels_result) = tokio::join!(
                    android_manager.list_available_devices(),
                    android_manager.list_available_targets()
                );

                if let (Ok(device_types), Ok(api_levels)) = (device_types_result, api_levels_result)
                {
                    let state = state_clone.lock().await;
                    let mut cache = state.device_cache.write().await;
                    cache.android_device_cache = Some(device_types.clone());
                    cache.update_android_cache(device_types, api_levels);
                    log::info!("Android device cache updated successfully");
                }
            }

            #[cfg(target_os = "macos")]
            if let Ok(ios_manager) = crate::managers::IosManager::new() {
                let (device_types_result, runtimes_result) = tokio::join!(
                    ios_manager.list_device_types_with_names(),
                    ios_manager.list_runtimes()
                );

                if let (Ok(device_types), Ok(runtimes)) = (device_types_result, runtimes_result) {
                    let state = state_clone.lock().await;
                    let mut cache = state.device_cache.write().await;
                    cache.update_ios_cache(device_types, runtimes);
                    log::info!("iOS device cache updated successfully");
                }
            }
        });
    }

    /// Load device list in background (improve startup speed)
    pub(super) fn start_background_device_loading(&mut self) {
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        tokio::spawn({
            let state_clone = Arc::clone(&state_clone);
            let android_manager = android_manager.clone();
            async move {
                match android_manager.list_devices_parallel().await {
                    Ok(android_devices) => {
                        let mut state = state_clone.lock().await;
                        state.android_devices = android_devices;
                        state.is_loading = false;
                        state.mark_refreshed();

                        let should_update_details = state.active_panel == Panel::Android
                            && !state.android_devices.is_empty()
                            && state.cached_device_details.is_none();
                        drop(state);

                        if should_update_details {
                            let state_clone2 = Arc::clone(&state_clone);
                            let android_manager_clone = android_manager.clone();
                            tokio::spawn(async move {
                                let state = state_clone2.lock().await;
                                if let Some(device) =
                                    state.android_devices.get(state.selected_android)
                                {
                                    let device_name = device.name.clone();
                                    let cached_info = state.get_cached_android_device(&device_name);
                                    drop(state);

                                    if let Ok(details) = android_manager_clone
                                        .get_device_details(&device_name, cached_info)
                                        .await
                                    {
                                        let mut state = state_clone2.lock().await;
                                        state.update_cached_device_details(details);
                                    }
                                }
                            });
                        }

                        let state = state_clone.lock().await;
                        let should_start_logs = state.active_panel == Panel::Android
                            && state
                                .android_devices
                                .get(state.selected_android)
                                .map(|d| d.is_running)
                                .unwrap_or(false);
                        drop(state);

                        if should_start_logs {
                            let state_clone3 = Arc::clone(&state_clone);
                            let android_manager_clone2 = android_manager.clone();
                            tokio::spawn(async move {
                                Self::update_log_stream_internal(
                                    state_clone3,
                                    android_manager_clone2,
                                    None,
                                )
                                .await;
                            });
                        }
                    }
                    Err(e) => {
                        let mut state = state_clone.lock().await;
                        state.is_loading = false;
                        state
                            .add_error_notification(format!("Failed to load Android devices: {e}"));
                    }
                }
            }
        });

        tokio::spawn(async move {
            let Some(ios_manager) = ios_manager else {
                return;
            };

            match ios_manager.list_devices().await {
                Ok(ios_devices) => {
                    let mut state = state_clone.lock().await;
                    state.ios_devices = ios_devices;

                    let should_update_details = state.active_panel == Panel::Ios
                        && !state.ios_devices.is_empty()
                        && state.cached_device_details.is_none();
                    drop(state);

                    if should_update_details {
                        let state_clone2 = Arc::clone(&state_clone);
                        tokio::spawn(async move {
                            let state = state_clone2.lock().await;
                            if let Some(device) = state.ios_devices.get(state.selected_ios) {
                                let details = DeviceDetails {
                                    name: device.name.clone(),
                                    status: if device.is_running {
                                        "Running".to_string()
                                    } else {
                                        "Stopped".to_string()
                                    },
                                    platform: Platform::Ios,
                                    device_type: device.device_type.clone(),
                                    api_level_or_version: format!("iOS {}", device.ios_version),
                                    ram_size: None,
                                    storage_size: None,
                                    resolution: None,
                                    dpi: None,
                                    device_path: None,
                                    system_image: None,
                                    identifier: device.udid.clone(),
                                };
                                drop(state);

                                let mut state = state_clone2.lock().await;
                                state.update_cached_device_details(details);
                            }
                        });
                    }

                    let state = state_clone.lock().await;
                    let should_start_logs = state.active_panel == Panel::Ios
                        && state
                            .ios_devices
                            .get(state.selected_ios)
                            .map(|d| d.is_running)
                            .unwrap_or(false);
                    drop(state);

                    if should_start_logs {
                        let state_clone3 = Arc::clone(&state_clone);
                        tokio::spawn(async move {
                            Self::update_log_stream_internal(
                                state_clone3,
                                AndroidManager::new()
                                    .unwrap_or_else(|_| AndroidManager::new().unwrap()),
                                Some(ios_manager),
                            )
                            .await;
                        });
                    }
                }
                Err(e) => {
                    let mut state = state_clone.lock().await;
                    state.add_error_notification(format!("Failed to load iOS devices: {e}"));
                }
            }
        });
    }
}
