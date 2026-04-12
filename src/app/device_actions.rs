use super::{App, Panel};
use crate::managers::common::DeviceManager;
use crate::models::error::format_user_error;
use anyhow::Result;

impl App {
    pub(super) async fn toggle_device(&mut self) -> Result<()> {
        let (active_panel, selected_android, selected_ios, android_devices, ios_devices) = {
            let state = self.state.lock().await;
            (
                state.active_panel,
                state.selected_android,
                state.selected_ios,
                state.android_devices.clone(),
                state.ios_devices.clone(),
            )
        };

        let result = match active_panel {
            Panel::Android => {
                if let Some(device) = android_devices.get(selected_android) {
                    let name = device.name.clone();
                    let is_running = device.is_running;

                    if is_running {
                        {
                            let mut state = self.state.lock().await;
                            state.set_device_operation_status(format!(
                                "Stopping device '{name}'..."
                            ));
                        }

                        match self.android_manager.stop_device(&name).await {
                            Ok(()) => {
                                let mut state = self.state.lock().await;
                                state.clear_device_operation_status();
                                state.add_success_notification(format!("Device '{name}' stopped"));
                                state.update_single_android_device_status(&name, false);

                                if let Some(ref cached) = state.cached_device_details {
                                    if cached.identifier == name {
                                        state.clear_cached_device_details();
                                    }
                                }
                                Ok(())
                            }
                            Err(error) => {
                                let mut state = self.state.lock().await;
                                state.clear_device_operation_status();
                                state.add_error_notification(format!(
                                    "Failed to stop device '{name}': {}",
                                    format_user_error(&error)
                                ));
                                Err(error)
                            }
                        }
                    } else {
                        let mut state = self.state.lock().await;
                        state.set_pending_device_start(name.clone());
                        state.set_device_operation_status(format!("Starting device '{name}'..."));
                        drop(state);

                        match self.android_manager.start_device(&name).await {
                            Ok(()) => {
                                let mut state = self.state.lock().await;
                                state.clear_device_operation_status();
                                state.add_info_notification(format!("Starting device '{name}'..."));
                                state.update_single_android_device_status(&name, true);

                                if let Some(ref cached) = state.cached_device_details {
                                    if cached.identifier == name {
                                        state.clear_cached_device_details();
                                    }
                                }
                                Ok(())
                            }
                            Err(error) => {
                                let mut state = self.state.lock().await;
                                state.clear_pending_device_start();
                                state.clear_device_operation_status();
                                state.add_error_notification(format!(
                                    "Failed to start device '{name}': {}",
                                    format_user_error(&error)
                                ));
                                Err(error)
                            }
                        }
                    }
                } else {
                    Ok(())
                }
            }
            Panel::Ios => {
                if let Some(ref ios_manager) = self.ios_manager {
                    if let Some(device) = ios_devices.get(selected_ios) {
                        let name = device.name.clone();
                        let udid = device.udid.clone();
                        let is_running = device.is_running;

                        if is_running {
                            {
                                let mut state = self.state.lock().await;
                                state.set_device_operation_status(format!(
                                    "Stopping device '{name}'..."
                                ));
                            }

                            match ios_manager.stop_device(&udid).await {
                                Ok(()) => {
                                    let mut state = self.state.lock().await;
                                    state.clear_device_operation_status();
                                    state.add_success_notification(format!(
                                        "Device '{name}' stopped"
                                    ));
                                    state.update_single_ios_device_status(&udid, false);

                                    if let Some(ref cached) = state.cached_device_details {
                                        if cached.identifier == udid {
                                            state.clear_cached_device_details();
                                        }
                                    }
                                    Ok(())
                                }
                                Err(error) => {
                                    let mut state = self.state.lock().await;
                                    state.clear_device_operation_status();
                                    state.add_error_notification(format!(
                                        "Failed to stop device '{name}': {error}"
                                    ));
                                    Err(error)
                                }
                            }
                        } else {
                            let mut state = self.state.lock().await;
                            state.set_pending_device_start(name.clone());
                            state.set_device_operation_status(format!(
                                "Starting device '{name}'..."
                            ));
                            drop(state);

                            match ios_manager.start_device(&udid).await {
                                Ok(()) => {
                                    let mut state = self.state.lock().await;
                                    state.clear_device_operation_status();
                                    state.add_info_notification(format!(
                                        "Starting device '{name}'..."
                                    ));
                                    state.update_single_ios_device_status(&udid, true);

                                    if let Some(ref cached) = state.cached_device_details {
                                        if cached.identifier == udid {
                                            state.clear_cached_device_details();
                                        }
                                    }
                                    Ok(())
                                }
                                Err(error) => {
                                    let mut state = self.state.lock().await;
                                    state.clear_pending_device_start();
                                    state.clear_device_operation_status();
                                    state.add_error_notification(format!(
                                        "Failed to start device '{name}': {error}"
                                    ));
                                    Err(error)
                                }
                            }
                        }
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            }
        };

        if result.is_ok() {
            self.schedule_background_device_status_check().await;
        }
        Ok(())
    }

    pub(super) async fn execute_delete_device(&mut self) -> Result<()> {
        let dialog_info = {
            let mut state = self.state.lock().await;
            state.confirm_delete_dialog.take()
        };

        if let Some(dialog) = dialog_info {
            let result = match dialog.platform {
                Panel::Android => {
                    self.android_manager
                        .delete_device(&dialog.device_identifier)
                        .await
                }
                Panel::Ios => {
                    if let Some(ref ios_manager) = self.ios_manager {
                        ios_manager.delete_device(&dialog.device_identifier).await
                    } else {
                        return Err(anyhow::anyhow!("iOS manager not available"));
                    }
                }
            };

            match result {
                Ok(()) => {
                    let mut state = self.state.lock().await;

                    match dialog.platform {
                        Panel::Android => {
                            state
                                .android_devices
                                .retain(|device| device.name != dialog.device_identifier);
                            if state.selected_android >= state.android_devices.len() {
                                state.selected_android =
                                    state.android_devices.len().saturating_sub(1);
                            }
                        }
                        Panel::Ios => {
                            state
                                .ios_devices
                                .retain(|device| device.udid != dialog.device_identifier);
                            if state.selected_ios >= state.ios_devices.len() {
                                state.selected_ios = state.ios_devices.len().saturating_sub(1);
                            }
                        }
                    }

                    state.clear_device_operation_status();
                    state.add_success_notification(format!(
                        "Device '{}' deleted successfully",
                        dialog.device_name
                    ));
                }
                Err(error) => {
                    let mut state = self.state.lock().await;
                    state.clear_device_operation_status();
                    state.add_error_notification(format!(
                        "Failed to delete device '{}': {}",
                        dialog.device_name, error
                    ));
                }
            }
        }

        Ok(())
    }

    pub(super) async fn execute_wipe_device(&mut self) -> Result<()> {
        let dialog_info = {
            let mut state = self.state.lock().await;
            state.confirm_wipe_dialog.take()
        };

        if let Some(dialog) = dialog_info {
            let result = match dialog.platform {
                Panel::Android => {
                    self.android_manager
                        .wipe_device(&dialog.device_identifier)
                        .await
                }
                Panel::Ios => {
                    if let Some(ref ios_manager) = self.ios_manager {
                        ios_manager.wipe_device(&dialog.device_identifier).await
                    } else {
                        let mut state = self.state.lock().await;
                        state.clear_device_operation_status();
                        return Err(anyhow::anyhow!("iOS manager not available"));
                    }
                }
            };

            match result {
                Ok(()) => {
                    let mut state = self.state.lock().await;
                    state.clear_device_operation_status();
                    state.add_success_notification(format!(
                        "Device '{}' wiped successfully",
                        dialog.device_name
                    ));

                    match dialog.platform {
                        Panel::Android => {
                            drop(state);
                            self.update_single_android_device_status(&dialog.device_identifier)
                                .await;
                        }
                        Panel::Ios => {
                            drop(state);
                            self.update_single_ios_device_status(&dialog.device_identifier)
                                .await;
                        }
                    }
                    self.update_device_details().await;
                }
                Err(error) => {
                    let mut state = self.state.lock().await;
                    state.clear_device_operation_status();
                    state.add_error_notification(format!(
                        "Failed to wipe device '{}': {}",
                        dialog.device_name,
                        format_user_error(&error)
                    ));
                }
            }
        }

        Ok(())
    }
}
