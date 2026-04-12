use super::{state, App, Mode, Panel};
use crate::models::error::format_user_error;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;

impl App {
    pub(super) async fn process_key_event(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
        if self.handle_quit_key(key).await {
            return Ok(true);
        }

        let mode = {
            let state = self.state.lock().await;
            state.mode
        };

        match mode {
            Mode::Normal => self.handle_normal_mode_key(key).await?,
            Mode::CreateDevice => self.handle_create_mode_key(key).await?,
            Mode::ConfirmDelete => self.handle_confirm_delete_key(key).await?,
            Mode::ConfirmWipe => self.handle_confirm_wipe_key(key).await?,
            Mode::ManageApiLevels => self.handle_api_level_mode_key(key).await,
            Mode::Help => self.handle_help_mode_key(key).await,
        }

        Ok(false)
    }

    async fn handle_quit_key(&mut self, key: KeyEvent) -> bool {
        let should_quit = matches!(key.code, KeyCode::Char('q'))
            && (key.modifiers.contains(KeyModifiers::CONTROL) || key.modifiers.is_empty())
            || matches!(key.code, KeyCode::Char('c'))
                && key.modifiers.contains(KeyModifiers::CONTROL);

        if !should_quit {
            return false;
        }

        let mut state = self.state.lock().await;
        if let Some(handle) = state.log_task_handle.take() {
            handle.abort();
        }
        true
    }

    async fn handle_normal_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Esc => {
                let mut state = self.state.lock().await;
                state.dismiss_all_notifications();
            }
            KeyCode::Char('r') => {
                self.refresh_devices_smart().await?;
            }
            KeyCode::Tab
            | KeyCode::BackTab
            | KeyCode::Char('h')
            | KeyCode::Char('l')
            | KeyCode::Left
            | KeyCode::Right => {
                self.switch_active_panel().await;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection_and_schedule_updates(true).await;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection_and_schedule_updates(false).await;
            }
            KeyCode::Enter => {
                self.toggle_device().await?;
            }
            KeyCode::Char('f') => {
                let mut state = self.state.lock().await;
                let next_filter = match &state.log_filter_level {
                    None => Some("ERROR".to_string()),
                    Some(level) if level == "ERROR" => Some("WARN".to_string()),
                    Some(level) if level == "WARN" => Some("INFO".to_string()),
                    Some(level) if level == "INFO" => Some("DEBUG".to_string()),
                    _ => None,
                };
                state.toggle_log_filter(next_filter);
            }
            KeyCode::Char('F') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                let mut state = self.state.lock().await;
                state.toggle_fullscreen_logs();
            }
            KeyCode::Char('L') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                let mut state = self.state.lock().await;
                state.clear_logs();
                state.add_info_notification("Logs cleared".to_string());
            }
            KeyCode::Char('c') => {
                self.enter_create_device_mode().await;
            }
            KeyCode::Char('d') => {
                self.open_delete_confirmation().await;
            }
            KeyCode::Char('w') => {
                self.open_wipe_confirmation().await;
            }
            KeyCode::Char('i') => {
                self.open_api_level_management().await;
            }
            _ => {}
        }

        Ok(())
    }

    async fn switch_active_panel(&mut self) {
        {
            let mut state = self.state.lock().await;
            let new_panel = state.active_panel.toggle();
            state.smart_clear_cached_device_details(new_panel);
            state.active_panel = new_panel;
        }

        self.schedule_panel_switch_updates_parallel().await;
    }

    async fn move_selection_and_schedule_updates(&mut self, move_up: bool) {
        let should_update = {
            let mut state = self.state.lock().await;
            if move_up {
                state.move_up();
            } else {
                state.move_down();
            }
            state.clear_logs();

            if let Some(handle) = state.log_task_handle.take() {
                handle.abort();
            }
            state.current_log_device = None;

            let current_device = match state.active_panel {
                Panel::Android => state
                    .android_devices
                    .get(state.selected_android)
                    .map(|device| device.name.as_str()),
                Panel::Ios => state
                    .ios_devices
                    .get(state.selected_ios)
                    .map(|device| device.udid.as_str()),
            };

            let should_update = if let Some(ref cached) = state.cached_device_details {
                current_device != Some(cached.identifier.as_str())
            } else {
                true
            };

            if should_update {
                state.clear_cached_device_details();
            }

            should_update
        };

        if should_update {
            self.schedule_non_blocking_updates();
        }
    }

    async fn enter_create_device_mode(&mut self) {
        let active_panel = {
            let mut state = self.state.lock().await;
            let active_panel = state.active_panel;
            state.mode = Mode::CreateDevice;
            state.create_device_form = match active_panel {
                Panel::Android => state::CreateDeviceForm::for_android(),
                Panel::Ios => state::CreateDeviceForm::for_ios(),
            };
            state.create_device_form.is_loading_cache = true;
            active_panel
        };

        let cache_available = {
            let state = self.state.lock().await;
            state.is_cache_available(active_panel).await
        };

        if cache_available {
            let mut state = self.state.lock().await;
            state.populate_form_from_cache(active_panel).await;
            state.create_device_form.is_loading_cache = false;
            return;
        }

        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        tokio::spawn(async move {
            match active_panel {
                Panel::Android => {
                    if let Ok(targets) = android_manager.list_available_targets().await {
                        if let Ok(devices) =
                            android_manager.list_devices_by_category(Some("all")).await
                        {
                            let mut state = state_clone.lock().await;
                            state.create_device_form.available_versions = targets.clone();
                            state.create_device_form.available_device_types = devices.clone();

                            {
                                let mut cache = state.device_cache.write().await;
                                cache.update_android_cache(devices, targets);
                            }

                            if let Some((id, display)) = state
                                .create_device_form
                                .available_device_types
                                .first()
                                .cloned()
                            {
                                state.create_device_form.device_type_id = id;
                                state.create_device_form.device_type = display;
                                state.create_device_form.selected_device_type_index = 0;
                            }

                            if let Some((value, display)) =
                                state.create_device_form.available_versions.first().cloned()
                            {
                                state.create_device_form.version = value;
                                state.create_device_form.version_display = display;
                                state.create_device_form.selected_api_level_index = 0;
                            }

                            state.create_device_form.generate_placeholder_name();
                            state.create_device_form.is_loading_cache = false;
                        }
                    }
                }
                Panel::Ios => {
                    if let Some(ref ios_manager) = ios_manager {
                        if let Ok(device_types) = ios_manager.list_device_types_with_names().await {
                            if let Ok(runtimes) = ios_manager.list_runtimes().await {
                                let mut state = state_clone.lock().await;
                                state.create_device_form.available_device_types =
                                    device_types.clone();
                                state.create_device_form.available_versions = runtimes.clone();

                                {
                                    let mut cache = state.device_cache.write().await;
                                    cache.update_ios_cache(device_types, runtimes);
                                }

                                if let Some((id, display)) = state
                                    .create_device_form
                                    .available_device_types
                                    .first()
                                    .cloned()
                                {
                                    state.create_device_form.device_type_id = id;
                                    state.create_device_form.device_type = display;
                                    state.create_device_form.selected_device_type_index = 0;
                                }

                                if let Some((value, display)) =
                                    state.create_device_form.available_versions.first().cloned()
                                {
                                    state.create_device_form.version = value;
                                    state.create_device_form.version_display = display;
                                    state.create_device_form.selected_api_level_index = 0;
                                }

                                state.create_device_form.generate_placeholder_name();
                                state.create_device_form.is_loading_cache = false;
                            }
                        }
                    }
                }
            }
        });
    }

    async fn open_delete_confirmation(&mut self) {
        let mut state = self.state.lock().await;
        let dialog =
            match state.active_panel {
                Panel::Android => state
                    .android_devices
                    .get(state.selected_android)
                    .map(|device| state::ConfirmDeleteDialog {
                        device_name: device.name.clone(),
                        device_identifier: device.name.clone(),
                        platform: Panel::Android,
                    }),
                Panel::Ios => state.ios_devices.get(state.selected_ios).map(|device| {
                    state::ConfirmDeleteDialog {
                        device_name: device.name.clone(),
                        device_identifier: device.udid.clone(),
                        platform: Panel::Ios,
                    }
                }),
            };

        if let Some(dialog) = dialog {
            state.mode = Mode::ConfirmDelete;
            state.confirm_delete_dialog = Some(dialog);
        }
    }

    async fn open_wipe_confirmation(&mut self) {
        let mut state = self.state.lock().await;
        let dialog =
            match state.active_panel {
                Panel::Android => state
                    .android_devices
                    .get(state.selected_android)
                    .map(|device| state::ConfirmWipeDialog {
                        device_name: device.name.clone(),
                        device_identifier: device.name.clone(),
                        platform: Panel::Android,
                    }),
                Panel::Ios => state.ios_devices.get(state.selected_ios).map(|device| {
                    state::ConfirmWipeDialog {
                        device_name: device.name.clone(),
                        device_identifier: device.udid.clone(),
                        platform: Panel::Ios,
                    }
                }),
            };

        if let Some(dialog) = dialog {
            state.mode = Mode::ConfirmWipe;
            state.confirm_wipe_dialog = Some(dialog);
        }
    }

    async fn handle_create_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Esc => {
                let mut state = self.state.lock().await;
                if !state.create_device_form.is_creating {
                    state.mode = Mode::Normal;
                    state.create_device_form.error_message = None;
                }
            }
            KeyCode::Tab | KeyCode::Down => {
                self.navigate_create_form(true).await;
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.navigate_create_form(false).await;
            }
            KeyCode::Enter => {
                let is_creating = {
                    let state = self.state.lock().await;
                    state.create_device_form.is_creating
                };
                if !is_creating {
                    self.submit_create_device().await?;
                }
            }
            KeyCode::Left => {
                self.change_create_device_selection(false).await?;
            }
            KeyCode::Right => {
                self.change_create_device_selection(true).await?;
            }
            KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_create_mode_ctrl_char(c).await?;
            }
            KeyCode::Char(c) => {
                let mut state = self.state.lock().await;
                if !state.create_device_form.is_creating {
                    self.handle_create_device_char(&mut state, c);
                }
            }
            KeyCode::Backspace => {
                let mut state = self.state.lock().await;
                if !state.create_device_form.is_creating {
                    self.handle_create_device_backspace(&mut state);
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn navigate_create_form(&mut self, forward: bool) {
        let mut state = self.state.lock().await;
        if state.create_device_form.is_creating {
            return;
        }

        match (state.active_panel, forward) {
            (Panel::Android, true) => state.create_device_form.next_field(),
            (Panel::Android, false) => state.create_device_form.prev_field(),
            (Panel::Ios, true) => state.create_device_form.next_field_ios(),
            (Panel::Ios, false) => state.create_device_form.prev_field_ios(),
        }
    }

    async fn change_create_device_selection(&mut self, move_right: bool) -> anyhow::Result<()> {
        let should_reload = {
            let mut state = self.state.lock().await;
            if state.create_device_form.is_creating {
                return Ok(());
            }

            let reload_category =
                state.create_device_form.active_field == state::CreateDeviceField::Category;
            let old_category = state.create_device_form.device_category_filter.clone();

            if move_right {
                self.handle_create_device_right(&mut state);
            } else {
                self.handle_create_device_left(&mut state);
            }

            reload_category && old_category != state.create_device_form.device_category_filter
        };

        if !should_reload {
            return Ok(());
        }

        if let Err(error) = self.reload_device_types_for_category().await {
            let mut state = self.state.lock().await;
            state.create_device_form.error_message = Some(format_user_error(&error));
        }

        Ok(())
    }

    async fn handle_create_mode_ctrl_char(&mut self, c: char) -> anyhow::Result<()> {
        let is_creating = {
            let state = self.state.lock().await;
            state.create_device_form.is_creating
        };

        if is_creating {
            return Ok(());
        }

        match c {
            'h' => self.change_create_device_selection(false).await?,
            'l' => self.change_create_device_selection(true).await?,
            'j' => self.navigate_create_form(true).await,
            'k' => self.navigate_create_form(false).await,
            _ => {
                let mut state = self.state.lock().await;
                self.handle_create_device_char(&mut state, c);
            }
        }

        Ok(())
    }

    async fn handle_confirm_delete_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                {
                    let mut state = self.state.lock().await;
                    state.mode = Mode::Normal;
                    if let Some(dialog) = state.confirm_delete_dialog.clone() {
                        state.set_device_operation_status(format!(
                            "Deleting device '{}'...",
                            dialog.device_name
                        ));
                    }
                }
                self.execute_delete_device().await?;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                let mut state = self.state.lock().await;
                state.mode = Mode::Normal;
                state.confirm_delete_dialog = None;
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_confirm_wipe_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                {
                    let mut state = self.state.lock().await;
                    state.mode = Mode::Normal;
                    if let Some(dialog) = state.confirm_wipe_dialog.clone() {
                        state.set_device_operation_status(format!(
                            "Wiping device '{}'...",
                            dialog.device_name
                        ));
                    }
                }
                self.execute_wipe_device().await?;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                let mut state = self.state.lock().await;
                state.mode = Mode::Normal;
                state.confirm_wipe_dialog = None;
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_help_mode_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') => {
                let mut state = self.state.lock().await;
                state.mode = Mode::Normal;
            }
            _ => {}
        }
    }
}
