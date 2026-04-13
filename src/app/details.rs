use super::{App, AppState, Panel};
use crate::constants::{
    performance::{FAST_DETAIL_UPDATE_DEBOUNCE, FAST_LOG_UPDATE_DEBOUNCE},
    timeouts::DEVICE_STOP_WAIT_TIME,
};
use crate::managers::common::DeviceManager;
use crate::managers::{AndroidManager, IosManager};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

impl App {
    /// Update device details for the currently selected device
    pub(super) async fn update_device_details(&mut self) {
        let (active_panel, device_identifier, cached_device_info) = {
            let state = self.state.lock().await;
            let (identifier, cached_info) = match state.active_panel {
                Panel::Android => {
                    let device_name = state
                        .android_devices
                        .get(state.selected_android)
                        .map(|d| d.name.clone());
                    let cached_info = device_name
                        .as_ref()
                        .and_then(|name| state.get_cached_android_device(name));
                    (device_name, cached_info)
                }
                Panel::Ios => (
                    state
                        .ios_devices
                        .get(state.selected_ios)
                        .map(|d| d.udid.clone()),
                    None,
                ),
            };
            (state.active_panel, identifier, cached_info)
        };

        if let Some(identifier) = device_identifier {
            match active_panel {
                Panel::Android => {
                    if let Ok(details) = self
                        .android_manager
                        .get_device_details(&identifier, cached_device_info)
                        .await
                    {
                        let mut state = self.state.lock().await;
                        state.update_cached_device_details(details);
                    }
                }
                Panel::Ios => {
                    // TODO: Implement iOS device details
                }
            }
        }
    }

    /// Schedule device details update with delay to avoid performance issues
    #[allow(dead_code)]
    pub(super) async fn schedule_device_details_update(&mut self) {
        if let Some(handle) = self.detail_update_handle.take() {
            handle.abort();
        }

        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();
        let delay = FAST_DETAIL_UPDATE_DEBOUNCE;

        let update_handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            Self::update_device_details_internal(state_clone, android_manager, ios_manager).await;
        });

        self.detail_update_handle = Some(update_handle);
    }

    /// Schedule both log stream and device details updates in parallel for fast panel switching
    pub(super) async fn schedule_panel_switch_updates_parallel(&mut self) {
        if let Some(handle) = self.log_update_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.detail_update_handle.take() {
            handle.abort();
        }

        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        let log_delay = FAST_LOG_UPDATE_DEBOUNCE;
        let detail_delay = FAST_DETAIL_UPDATE_DEBOUNCE;

        let state_clone_log = Arc::clone(&state_clone);
        let android_manager_log = android_manager.clone();
        let ios_manager_log = ios_manager.clone();

        let log_handle = tokio::spawn(async move {
            tokio::time::sleep(log_delay).await;
            Self::update_log_stream_internal(state_clone_log, android_manager_log, ios_manager_log)
                .await;
        });

        let detail_handle = tokio::spawn(async move {
            tokio::time::sleep(detail_delay).await;
            Self::update_device_details_internal(state_clone, android_manager, ios_manager).await;
        });

        self.log_update_handle = Some(log_handle);
        self.detail_update_handle = Some(detail_handle);
    }

    /// Schedule background device status check for smart device start mode.
    /// This performs a lightweight status check after a delay to ensure accuracy.
    pub(super) async fn update_single_android_device_status(&mut self, device_name: &str) {
        if let Ok(devices) = self.android_manager.list_devices().await {
            if let Some(device) = devices.iter().find(|d| d.name == device_name) {
                let mut state = self.state.lock().await;
                state.update_single_android_device_status(device_name, device.is_running);
            }
        }
    }

    pub(super) async fn update_single_ios_device_status(&mut self, device_udid: &str) {
        if let Some(ref ios_manager) = self.ios_manager {
            if let Ok(devices) = ios_manager.list_devices().await {
                if let Some(device) = devices.iter().find(|d| d.udid == device_udid) {
                    let mut state = self.state.lock().await;
                    state.update_single_ios_device_status(device_udid, device.is_running);
                }
            }
        }
    }

    pub(super) async fn schedule_background_device_status_check(&mut self) {
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        tokio::spawn(async move {
            tokio::time::sleep(DEVICE_STOP_WAIT_TIME).await;

            let (active_panel, device_identifier) = {
                let state = state_clone.lock().await;
                let identifier = match state.active_panel {
                    Panel::Android => state
                        .android_devices
                        .get(state.selected_android)
                        .map(|d| d.name.clone()),
                    Panel::Ios => state
                        .ios_devices
                        .get(state.selected_ios)
                        .map(|d| d.udid.clone()),
                };
                (state.active_panel, identifier)
            };

            if let Some(identifier) = device_identifier {
                match active_panel {
                    Panel::Android => {
                        if let Ok(devices) = android_manager.list_devices().await {
                            if let Some(device) = devices.iter().find(|d| d.name == identifier) {
                                let mut state = state_clone.lock().await;
                                state.update_single_android_device_status(
                                    &identifier,
                                    device.is_running,
                                );
                            }
                        }
                    }
                    Panel::Ios => {
                        if let Some(ios_manager) = ios_manager {
                            if let Ok(devices) = ios_manager.list_devices().await {
                                if let Some(device) = devices.iter().find(|d| d.udid == identifier)
                                {
                                    let mut state = state_clone.lock().await;
                                    state.update_single_ios_device_status(
                                        &identifier,
                                        device.is_running,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub(super) async fn update_device_details_internal(
        state: Arc<Mutex<AppState>>,
        android_manager: AndroidManager,
        ios_manager: Option<IosManager>,
    ) {
        let (active_panel, device_identifier, cached_device_info) = {
            let state_lock = state.lock().await;
            let (identifier, cached_info) = match state_lock.active_panel {
                Panel::Android => {
                    let device_name = state_lock
                        .android_devices
                        .get(state_lock.selected_android)
                        .map(|d| d.name.clone());
                    let cached_info = device_name
                        .as_ref()
                        .and_then(|name| state_lock.get_cached_android_device(name));
                    (device_name, cached_info)
                }
                Panel::Ios => (
                    state_lock
                        .ios_devices
                        .get(state_lock.selected_ios)
                        .map(|d| d.udid.clone()),
                    None,
                ),
            };
            log::debug!(
                "update_device_details_internal: panel={:?}, selected_index={}, identifier={:?}",
                state_lock.active_panel,
                if state_lock.active_panel == Panel::Android {
                    state_lock.selected_android
                } else {
                    state_lock.selected_ios
                },
                identifier
            );
            (state_lock.active_panel, identifier, cached_info)
        };

        if let Some(identifier) = device_identifier {
            match active_panel {
                Panel::Android => match android_manager
                    .get_device_details(&identifier, cached_device_info)
                    .await
                {
                    Ok(details) => {
                        log::debug!(
                            "Got device details successfully: RAM={:?}, Storage={:?}, Path={:?}",
                            details.ram_size,
                            details.storage_size,
                            details.device_path
                        );
                        let mut state_lock = state.lock().await;
                        state_lock.update_cached_device_details(details);
                    }
                    Err(e) => {
                        log::error!("Failed to get device details for '{identifier}': {e}");
                    }
                },
                Panel::Ios => {
                    if let Some(ios_manager) = ios_manager {
                        match ios_manager.get_device_details(&identifier).await {
                            Ok(details) => {
                                let mut state_lock = state.lock().await;
                                state_lock.update_cached_device_details(details);
                            }
                            Err(e) => {
                                log::error!(
                                    "Failed to get iOS device details for '{identifier}': {e}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Schedule non-blocking updates for device details and log streams
    /// to prevent UI stuttering during continuous navigation
    pub(super) fn schedule_non_blocking_updates(&self) {
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(25)).await;

            Self::update_device_details_internal(
                state_clone.clone(),
                android_manager.clone(),
                ios_manager.clone(),
            )
            .await;

            Self::update_log_stream_internal(state_clone, android_manager, ios_manager).await;
        });
    }
}
