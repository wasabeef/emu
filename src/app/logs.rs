use super::{App, AppState, Panel};
use crate::constants::{
    keywords::{LOG_LEVEL_ERROR, LOG_LEVEL_WARNING},
    performance::DETAIL_UPDATE_DEBOUNCE,
};
use crate::managers::{AndroidManager, IosManager};
use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

impl App {
    #[allow(dead_code)]
    pub(super) async fn update_log_stream(&mut self) -> Result<()> {
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        Self::update_log_stream_internal(state_clone, android_manager, ios_manager).await;
        Ok(())
    }

    pub(super) async fn update_log_stream_internal(
        state: Arc<Mutex<AppState>>,
        android_manager: AndroidManager,
        _ios_manager: Option<IosManager>,
    ) {
        let (
            active_panel,
            selected_android,
            selected_ios,
            android_devices,
            ios_devices,
            _current_log_device,
        ) = {
            let state_lock = state.lock().await;
            (
                state_lock.active_panel,
                state_lock.selected_android,
                state_lock.selected_ios,
                state_lock.android_devices.clone(),
                state_lock.ios_devices.clone(),
                state_lock.current_log_device.clone(),
            )
        };

        let (device_name, device_is_running) = match active_panel {
            Panel::Android => {
                if let Some(device) = android_devices.get(selected_android) {
                    (device.name.clone(), device.is_running)
                } else {
                    return;
                }
            }
            Panel::Ios => {
                if let Some(device) = ios_devices.get(selected_ios) {
                    (device.name.clone(), device.is_running)
                } else {
                    return;
                }
            }
        };

        if !device_is_running {
            let mut state_lock = state.lock().await;
            state_lock.current_log_device = None;
            return;
        }

        {
            let mut state_lock = state.lock().await;
            state_lock.current_log_device = Some((active_panel, device_name.clone()));

            if let Some(handle) = state_lock.log_task_handle.take() {
                handle.abort();
            }
        }

        match active_panel {
            Panel::Android => {
                if let Some(device) = android_devices.get(selected_android) {
                    if device.is_running {
                        {
                            let mut state_lock = state.lock().await;
                            state_lock.clear_logs();
                            state_lock.reset_log_scroll();
                        }

                        let device_name = device.name.clone();
                        let state_clone = Arc::clone(&state);

                        if let Ok(running_avds) = android_manager.get_running_avd_names().await {
                            if let Some(emulator_serial) = running_avds.get(&device_name) {
                                let serial = emulator_serial.clone();
                                let handle = tokio::spawn(async move {
                                    Self::stream_android_logs(state_clone, device_name, serial)
                                        .await;
                                });
                                let mut state_lock = state.lock().await;
                                state_lock.log_task_handle = Some(handle);
                            } else {
                                let normalized_name = device_name.replace(' ', "_");
                                if let Some(emulator_serial) = running_avds.get(&normalized_name) {
                                    let serial = emulator_serial.clone();
                                    let handle = tokio::spawn(async move {
                                        Self::stream_android_logs(state_clone, device_name, serial)
                                            .await;
                                    });
                                    let mut state_lock = state.lock().await;
                                    state_lock.log_task_handle = Some(handle);
                                } else if device.is_running && !running_avds.is_empty() {
                                    if let Some((_, serial)) = running_avds.iter().next() {
                                        let serial = serial.clone();
                                        let handle = tokio::spawn(async move {
                                            Self::stream_android_logs(
                                                state_clone,
                                                device_name,
                                                serial,
                                            )
                                            .await;
                                        });
                                        let mut state_lock = state.lock().await;
                                        state_lock.log_task_handle = Some(handle);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Panel::Ios => {
                if let Some(device) = ios_devices.get(selected_ios) {
                    if device.is_running {
                        {
                            let mut state_lock = state.lock().await;
                            state_lock.clear_logs();
                            state_lock.reset_log_scroll();
                        }

                        let device_udid = device.udid.clone();
                        let device_name = device.name.clone();
                        let state_clone = Arc::clone(&state);
                        let handle = tokio::spawn(async move {
                            Self::stream_ios_logs(state_clone, device_udid, device_name).await;
                        });
                        let mut state_lock = state.lock().await;
                        state_lock.log_task_handle = Some(handle);
                    }
                }
            }
        }
    }

    pub(super) async fn stream_android_logs(
        state: Arc<Mutex<AppState>>,
        device_name: String,
        emulator_serial: String,
    ) {
        let result = Command::new("adb")
            .args(["-s", &emulator_serial, "logcat", "-v", "time"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .stdin(std::process::Stdio::null())
            .spawn();

        if let Ok(mut child) = result {
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                loop {
                    tokio::select! {
                        result = lines.next_line() => {
                            match result {
                                Ok(Some(line)) => {
                                    if line.trim().is_empty() {
                                        continue;
                                    }

                                    let level = if line.contains(" E ") || line.contains("ERROR") {
                                        "ERROR"
                                    } else if line.contains(" W ") || line.contains("WARN") {
                                        "WARN"
                                    } else if line.contains(" I ") || line.contains("INFO") {
                                        "INFO"
                                    } else if line.contains(" D ") || line.contains("DEBUG") {
                                        "DEBUG"
                                    } else {
                                        "INFO"
                                    };

                                    let mut state = state.lock().await;
                                    state.add_log(level.to_string(), line);
                                }
                                Ok(None) => break,
                                Err(_) => break,
                            }
                        }
                        _ = tokio::time::sleep(DETAIL_UPDATE_DEBOUNCE) => {
                            let should_continue = {
                                let state_lock = state.lock().await;
                                if let Some((panel, name)) = &state_lock.current_log_device {
                                    panel == &crate::app::Panel::Android && name == &device_name
                                } else {
                                    false
                                }
                            };
                            if !should_continue {
                                break;
                            }
                        }
                    }
                }
            }

            let _ = child.kill().await;
        }
    }

    pub(super) async fn stream_ios_logs(
        state: Arc<Mutex<AppState>>,
        device_udid: String,
        _device_name: String,
    ) {
        let log_commands = [
            (
                "xcrun",
                vec!["simctl", "spawn", &device_udid, "log", "stream"],
            ),
            ("log", vec!["stream", "--style", "compact"]),
            ("log", vec!["stream"]),
        ];

        for (command, args) in log_commands.iter() {
            let result = tokio::process::Command::new(command)
                .args(args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();

            match result {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        let mut lines = reader.lines();

                        while let Ok(Some(line_content)) = lines.next_line().await {
                            if line_content.trim().is_empty() {
                                continue;
                            }

                            let level = if line_content.contains("error")
                                || line_content.contains(LOG_LEVEL_ERROR)
                            {
                                "ERROR"
                            } else if line_content.contains("warning")
                                || line_content.contains(LOG_LEVEL_WARNING)
                            {
                                "WARN"
                            } else {
                                "INFO"
                            };

                            let mut app_state = state.lock().await;
                            app_state.add_log(level.to_string(), line_content);
                        }
                        break;
                    }

                    let _ = child.kill().await;
                }
                Err(_) => {
                    continue;
                }
            }
        }
    }
}
