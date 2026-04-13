use super::{App, Panel};
use crate::managers::common::DeviceManager;
use crate::models::{AndroidDevice, IosDevice};
use anyhow::Result;
use std::collections::HashMap;

impl App {
    /// Refresh devices using incremental update for optimal performance
    pub(super) async fn refresh_devices_smart(&mut self) -> Result<()> {
        let (has_android_devices, pending_device) = {
            let state = self.state.lock().await;
            (
                !state.android_devices.is_empty(),
                state.get_pending_device_start().cloned(),
            )
        };

        let should_full_refresh = pending_device.is_some()
            || !has_android_devices
            || self.last_full_device_refresh.elapsed()
                >= crate::constants::performance::FULL_DEVICE_REFRESH_INTERVAL;

        if should_full_refresh {
            self.refresh_devices_incremental().await
        } else {
            self.refresh_device_statuses_only().await
        }
    }

    /// Incrementally refresh device lists by only updating changed devices
    /// This is more efficient than full refresh for large device counts
    pub(super) async fn refresh_devices_incremental(&mut self) -> Result<()> {
        let (existing_android, existing_ios, pending_device) = {
            let state = self.state.lock().await;
            let existing_android: HashMap<String, AndroidDevice> = state
                .android_devices
                .iter()
                .map(|d| (d.name.clone(), d.clone()))
                .collect();
            let existing_ios: HashMap<String, IosDevice> = state
                .ios_devices
                .iter()
                .map(|d| (d.name.clone(), d.clone()))
                .collect();
            let pending_device = state.get_pending_device_start().cloned();
            (existing_android, existing_ios, pending_device)
        };

        let new_android_devices = self.android_manager.list_devices().await?;
        let new_ios_devices = if let Some(ref ios_manager) = self.ios_manager {
            ios_manager.list_devices().await?
        } else {
            Vec::new()
        };

        let updated_android = self.process_android_updates(existing_android, new_android_devices);
        let updated_ios = self.process_ios_updates(existing_ios, new_ios_devices);

        {
            let mut state = self.state.lock().await;
            let mut device_started = None;
            if let Some(ref pending_name) = pending_device {
                let device_running = updated_android
                    .iter()
                    .any(|d| &d.name == pending_name && d.is_running)
                    || updated_ios
                        .iter()
                        .any(|d| &d.name == pending_name && d.is_running);

                if device_running {
                    state.add_success_notification(
                        crate::constants::messages::notifications::DEVICE_START_SUCCESS
                            .replace("{}", pending_name),
                    );
                    state.clear_pending_device_start();
                    device_started = Some(pending_name.clone());
                }
            }

            state.android_devices = updated_android;
            state.ios_devices = updated_ios;

            if state.selected_android >= state.android_devices.len() {
                state.selected_android = state.android_devices.len().saturating_sub(1);
            }
            if state.selected_ios >= state.ios_devices.len() {
                state.selected_ios = state.ios_devices.len().saturating_sub(1);
            }

            state.is_loading = false;
            state.mark_refreshed();

            let need_detail_update = if let Some(ref started_name) = device_started {
                match state.active_panel {
                    Panel::Android => state
                        .android_devices
                        .get(state.selected_android)
                        .map(|d| &d.name == started_name)
                        .unwrap_or(false),
                    Panel::Ios => state
                        .ios_devices
                        .get(state.selected_ios)
                        .map(|d| &d.name == started_name)
                        .unwrap_or(false),
                }
            } else {
                false
            };

            if need_detail_update {
                drop(state);
                self.update_device_details().await;
            }
        }

        self.last_full_device_refresh = std::time::Instant::now();

        Ok(())
    }

    /// Refresh only running status for existing devices and avoid rebuilding Android metadata.
    pub(super) async fn refresh_device_statuses_only(&mut self) -> Result<()> {
        let (existing_android, existing_ios) = {
            let state = self.state.lock().await;
            (
                state.android_devices.clone(),
                state
                    .ios_devices
                    .iter()
                    .map(|d| (d.name.clone(), d.clone()))
                    .collect::<HashMap<String, IosDevice>>(),
            )
        };

        let running_avds = self.android_manager.get_running_avd_names().await?;
        let updated_android = self.process_android_status_updates(existing_android, &running_avds);
        let new_ios_devices = if let Some(ref ios_manager) = self.ios_manager {
            ios_manager.list_devices().await?
        } else {
            Vec::new()
        };
        let updated_ios = self.process_ios_updates(existing_ios, new_ios_devices);

        let mut state = self.state.lock().await;
        state.android_devices = updated_android;
        state.ios_devices = updated_ios;

        if state.selected_android >= state.android_devices.len() {
            state.selected_android = state.android_devices.len().saturating_sub(1);
        }
        if state.selected_ios >= state.ios_devices.len() {
            state.selected_ios = state.ios_devices.len().saturating_sub(1);
        }

        state.is_loading = false;
        state.mark_refreshed();

        Ok(())
    }

    /// Process Android device updates in background (no state lock)
    pub(super) fn process_android_updates(
        &self,
        existing_android: HashMap<String, AndroidDevice>,
        new_android_devices: Vec<AndroidDevice>,
    ) -> Vec<AndroidDevice> {
        let mut updated_android = Vec::with_capacity(new_android_devices.len());
        for new_device in new_android_devices {
            if let Some(existing) = existing_android.get(&new_device.name) {
                if existing.status != new_device.status
                    || existing.is_running != new_device.is_running
                {
                    let mut updated = existing.clone();
                    updated.status = new_device.status;
                    updated.is_running = new_device.is_running;
                    updated_android.push(updated);
                } else {
                    updated_android.push(existing.clone());
                }
            } else {
                updated_android.push(new_device);
            }
        }
        updated_android
    }

    pub(super) fn process_android_status_updates(
        &self,
        existing_android: Vec<AndroidDevice>,
        running_avds: &HashMap<String, String>,
    ) -> Vec<AndroidDevice> {
        existing_android
            .into_iter()
            .map(|mut device| {
                let is_running = running_avds.contains_key(&device.name)
                    || running_avds.contains_key(&device.name.replace(' ', "_"));
                device.is_running = is_running;
                device.status = if is_running {
                    crate::models::DeviceStatus::Running
                } else {
                    crate::models::DeviceStatus::Stopped
                };
                device
            })
            .collect()
    }

    /// Process iOS device updates in background (no state lock)
    pub(super) fn process_ios_updates(
        &self,
        existing_ios: HashMap<String, IosDevice>,
        new_ios_devices: Vec<IosDevice>,
    ) -> Vec<IosDevice> {
        let mut updated_ios = Vec::with_capacity(new_ios_devices.len());
        for new_device in new_ios_devices {
            if let Some(existing) = existing_ios.get(&new_device.name) {
                if existing.status != new_device.status
                    || existing.is_running != new_device.is_running
                {
                    let mut updated = existing.clone();
                    updated.status = new_device.status;
                    updated.is_running = new_device.is_running;
                    updated_ios.push(updated);
                } else {
                    updated_ios.push(existing.clone());
                }
            } else {
                updated_ios.push(new_device);
            }
        }
        updated_ios
    }
}
