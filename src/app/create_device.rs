use super::{state, App, AppState, Mode, Panel};
use crate::constants::performance::DETAIL_UPDATE_DEBOUNCE;
use crate::managers::common::{DeviceConfig, DeviceManager};
use crate::models::error::format_user_error;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;

impl App {
    pub(super) async fn enter_create_device_mode(&mut self) {
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

    #[allow(dead_code)]
    pub(super) async fn load_available_versions(&mut self) -> Result<()> {
        let state = self.state.lock().await;

        match state.active_panel {
            Panel::Android => {
                drop(state);

                let available_devices = {
                    let state = self.state.lock().await;
                    let category_filter =
                        if state.create_device_form.device_category_filter == "all" {
                            None
                        } else {
                            Some(state.create_device_form.device_category_filter.clone())
                        };
                    drop(state);
                    self.android_manager
                        .list_devices_by_category(category_filter.as_deref())
                        .await?
                };
                if available_devices.is_empty() {
                    let mut state = self.state.lock().await;
                    state.create_device_form.error_message = Some(
                        "No Android device definitions found. Check your Android SDK installation."
                            .to_string(),
                    );
                    return Ok(());
                }

                let available_targets = self.android_manager.list_available_targets().await?;
                if available_targets.is_empty() {
                    let mut state = self.state.lock().await;
                    state.create_device_form.error_message = Some("No Android targets found. Use Android Studio SDK Manager to install system images.".to_string());
                    return Ok(());
                }

                let mut state = self.state.lock().await;
                state.create_device_form.available_device_types = available_devices;
                state.create_device_form.available_versions = available_targets;

                if !state.create_device_form.available_device_types.is_empty() {
                    let (id, display) = state.create_device_form.available_device_types[0].clone();
                    state.create_device_form.device_type_id = id;
                    state.create_device_form.device_type = display;
                    state.create_device_form.selected_device_type_index = 0;
                }

                if !state.create_device_form.available_versions.is_empty() {
                    let (value, display) = state.create_device_form.available_versions[0].clone();
                    state.create_device_form.version = value;
                    state.create_device_form.version_display = display;
                    state.create_device_form.selected_api_level_index = 0;
                }

                state.create_device_form.generate_placeholder_name();
            }
            Panel::Ios => {
                if let Some(ref ios_manager) = self.ios_manager {
                    drop(state);

                    let available_device_types = ios_manager.list_device_types_with_names().await?;
                    if available_device_types.is_empty() {
                        let mut state = self.state.lock().await;
                        state.create_device_form.error_message =
                            Some("No iOS device types available.".to_string());
                        return Ok(());
                    }

                    let available_runtimes = ios_manager.list_runtimes().await?;
                    if available_runtimes.is_empty() {
                        let mut state = self.state.lock().await;
                        state.create_device_form.error_message = Some(
                            "No iOS runtimes available. Install iOS runtimes using Xcode."
                                .to_string(),
                        );
                        return Ok(());
                    }

                    let mut state = self.state.lock().await;
                    state.create_device_form.available_device_types = available_device_types;
                    state.create_device_form.available_versions = available_runtimes;

                    if !state.create_device_form.available_device_types.is_empty() {
                        let (id, display) =
                            state.create_device_form.available_device_types[0].clone();
                        state.create_device_form.device_type_id = id;
                        state.create_device_form.device_type = display;
                        state.create_device_form.selected_device_type_index = 0;
                    }

                    if !state.create_device_form.available_versions.is_empty() {
                        let (value, display) =
                            state.create_device_form.available_versions[0].clone();
                        state.create_device_form.version = value;
                        state.create_device_form.version_display = display;
                        state.create_device_form.selected_api_level_index = 0;
                    }

                    state.create_device_form.generate_placeholder_name();
                } else {
                    let mut state = self.state.lock().await;
                    state.create_device_form.error_message =
                        Some("iOS simulator not available on this platform.".to_string());
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    pub(super) async fn handle_create_mode_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
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

    pub(super) fn handle_create_device_char(&self, state: &mut AppState, c: char) {
        use crate::app::state::CreateDeviceField;

        match state.create_device_form.active_field {
            CreateDeviceField::Name => {
                state.create_device_form.name.push(c);
            }
            CreateDeviceField::Category => {}
            CreateDeviceField::DeviceType => {}
            CreateDeviceField::ApiLevel => {}
            CreateDeviceField::RamSize => {
                if c.is_ascii_digit() {
                    state.create_device_form.ram_size.push(c);
                }
            }
            CreateDeviceField::StorageSize => {
                if c.is_ascii_digit() {
                    state.create_device_form.storage_size.push(c);
                }
            }
        }
        state.create_device_form.error_message = None;
    }

    pub(super) fn handle_create_device_backspace(&self, state: &mut AppState) {
        use crate::app::state::CreateDeviceField;

        match state.create_device_form.active_field {
            CreateDeviceField::Name => {
                state.create_device_form.name.pop();
            }
            CreateDeviceField::Category => {}
            CreateDeviceField::DeviceType => {}
            CreateDeviceField::ApiLevel => {}
            CreateDeviceField::RamSize => {
                state.create_device_form.ram_size.pop();
            }
            CreateDeviceField::StorageSize => {
                state.create_device_form.storage_size.pop();
            }
        }
        state.create_device_form.error_message = None;
    }

    pub(super) fn handle_create_device_left(&self, state: &mut AppState) {
        use crate::app::state::CreateDeviceField;

        match state.create_device_form.active_field {
            CreateDeviceField::Category => {
                if state.create_device_form.selected_category_index > 0 {
                    state.create_device_form.selected_category_index -= 1;
                } else {
                    state.create_device_form.selected_category_index =
                        state.create_device_form.available_categories.len() - 1;
                }
                state.create_device_form.update_selected_category();
            }
            CreateDeviceField::DeviceType => {
                let options = &state.create_device_form.available_device_types;
                if let Some(current_index) = options
                    .iter()
                    .position(|(id, _)| id == &state.create_device_form.device_type_id)
                {
                    let new_index = if current_index == 0 {
                        options.len() - 1
                    } else {
                        current_index - 1
                    };
                    let (id, display) = options[new_index].clone();
                    state.create_device_form.device_type_id = id;
                    state.create_device_form.device_type = display;
                    state.create_device_form.selected_device_type_index = new_index;
                    state.create_device_form.generate_placeholder_name();
                }
            }
            CreateDeviceField::ApiLevel => {
                let options = &state.create_device_form.available_versions;
                if !options.is_empty() {
                    if let Some(current_index) = options
                        .iter()
                        .position(|(value, _)| value == &state.create_device_form.version)
                    {
                        let new_index = if current_index == 0 {
                            options.len() - 1
                        } else {
                            current_index - 1
                        };
                        let (value, display) = options[new_index].clone();
                        state.create_device_form.version = value;
                        state.create_device_form.version_display = display;
                        state.create_device_form.selected_api_level_index = new_index;
                        state.create_device_form.generate_placeholder_name();
                    }
                }
            }
            _ => {}
        }
        state.create_device_form.error_message = None;
    }

    pub(super) fn handle_create_device_right(&self, state: &mut AppState) {
        use crate::app::state::CreateDeviceField;

        match state.create_device_form.active_field {
            CreateDeviceField::Category => {
                let len = state.create_device_form.available_categories.len();
                state.create_device_form.selected_category_index =
                    (state.create_device_form.selected_category_index + 1) % len;
                state.create_device_form.update_selected_category();
            }
            CreateDeviceField::DeviceType => {
                let options = &state.create_device_form.available_device_types;
                if let Some(current_index) = options
                    .iter()
                    .position(|(id, _)| id == &state.create_device_form.device_type_id)
                {
                    let new_index = (current_index + 1) % options.len();
                    let (id, display) = options[new_index].clone();
                    state.create_device_form.device_type_id = id;
                    state.create_device_form.device_type = display;
                    state.create_device_form.selected_device_type_index = new_index;
                    state.create_device_form.generate_placeholder_name();
                }
            }
            CreateDeviceField::ApiLevel => {
                let options = &state.create_device_form.available_versions;
                if !options.is_empty() {
                    if let Some(current_index) = options
                        .iter()
                        .position(|(value, _)| value == &state.create_device_form.version)
                    {
                        let new_index = (current_index + 1) % options.len();
                        let (value, display) = options[new_index].clone();
                        state.create_device_form.version = value;
                        state.create_device_form.version_display = display;
                        state.create_device_form.selected_api_level_index = new_index;
                        state.create_device_form.generate_placeholder_name();
                    }
                }
            }
            _ => {}
        }
        state.create_device_form.error_message = None;
    }

    pub(super) async fn submit_create_device(&mut self) -> Result<()> {
        let (active_panel, form_data, config) = {
            let state = self.state.lock().await;
            let form_data = state.create_device_form.clone();

            if form_data.name.trim().is_empty() {
                drop(state);
                let mut state = self.state.lock().await;
                state.create_device_form.error_message =
                    Some("Device name is required".to_string());
                return Ok(());
            }

            if form_data.version.trim().is_empty() {
                drop(state);
                let mut state = self.state.lock().await;
                state.create_device_form.error_message = Some("Version is required".to_string());
                return Ok(());
            }

            let device_name = form_data.name.clone();
            let device_type = form_data.device_type_id.clone();
            let mut config = DeviceConfig::new(device_name, device_type, form_data.version.clone());

            if matches!(state.active_panel, Panel::Android) {
                if !form_data.ram_size.is_empty() {
                    config = config.with_ram(form_data.ram_size.clone());
                }
                if !form_data.storage_size.is_empty() {
                    config = config.with_storage(form_data.storage_size.clone());
                }
            }

            (state.active_panel, form_data, config)
        };

        {
            let mut state = self.state.lock().await;
            state.create_device_form.is_creating = true;
            state.create_device_form.creation_status =
                Some("Initializing device creation...".to_string());
            state.create_device_form.error_message = None;
        }

        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();
        let device_name_for_display = form_data.name.clone();

        tokio::spawn(async move {
            {
                let mut state = state_clone.lock().await;
                state.create_device_form.creation_status =
                    Some(format!("Creating device '{device_name_for_display}'..."));
            }

            let result = match active_panel {
                Panel::Android => {
                    tokio::time::sleep(DETAIL_UPDATE_DEBOUNCE).await;
                    android_manager.create_device(&config).await
                }
                Panel::Ios => {
                    if let Some(ref ios_manager) = ios_manager {
                        tokio::time::sleep(DETAIL_UPDATE_DEBOUNCE).await;
                        ios_manager.create_device(&config).await
                    } else {
                        Err(anyhow::anyhow!("iOS manager not available"))
                    }
                }
            };

            match result {
                Ok(()) => {
                    {
                        let mut state = state_clone.lock().await;
                        state.create_device_form.creation_status =
                            Some("Finalizing...".to_string());
                    }

                    match active_panel {
                        Panel::Android => {
                            if let Ok(devices) = android_manager.list_devices().await {
                                let mut state = state_clone.lock().await;
                                state.android_devices = devices;
                                state.mode = Mode::Normal;
                                state.create_device_form.is_creating = false;
                                state.create_device_form.creation_status = None;
                                state.add_success_notification(format!(
                                    "Device '{device_name_for_display}' created successfully"
                                ));
                            } else {
                                let mut state = state_clone.lock().await;
                                state.mode = Mode::Normal;
                                state.create_device_form.is_creating = false;
                                state.create_device_form.creation_status = None;
                                state.add_success_notification(format!(
                                    "Device '{device_name_for_display}' created successfully"
                                ));
                            }
                        }
                        Panel::Ios => {
                            if let Some(ref ios_manager) = ios_manager {
                                if let Ok(devices) = ios_manager.list_devices().await {
                                    let mut state = state_clone.lock().await;
                                    state.ios_devices = devices;
                                    state.mode = Mode::Normal;
                                    state.create_device_form.is_creating = false;
                                    state.create_device_form.creation_status = None;
                                    state.add_success_notification(format!(
                                        "Device '{device_name_for_display}' created successfully"
                                    ));
                                } else {
                                    let mut state = state_clone.lock().await;
                                    state.mode = Mode::Normal;
                                    state.create_device_form.is_creating = false;
                                    state.create_device_form.creation_status = None;
                                    state.add_success_notification(format!(
                                        "Device '{device_name_for_display}' created successfully"
                                    ));
                                }
                            } else {
                                let mut state = state_clone.lock().await;
                                state.mode = Mode::Normal;
                                state.create_device_form.is_creating = false;
                                state.create_device_form.creation_status = None;
                                state.add_error_notification(
                                    "iOS manager not available (only available on macOS)"
                                        .to_string(),
                                );
                            }
                        }
                    }
                }
                Err(error) => {
                    let mut state = state_clone.lock().await;
                    state.create_device_form.is_creating = false;
                    state.create_device_form.creation_status = None;
                    state.add_error_notification(format!(
                        "Device creation error: {}",
                        format_user_error(&error)
                    ));
                    state.create_device_form.error_message = Some(format_user_error(&error));
                }
            }
        });

        Ok(())
    }

    pub(super) async fn reload_device_types_for_category(&mut self) -> Result<()> {
        let (current_panel, category_filter, device_cache_clone) = {
            let state = self.state.lock().await;
            let device_cache_clone = Arc::clone(&state.device_cache);
            (
                state.active_panel,
                state.create_device_form.device_category_filter.clone(),
                device_cache_clone,
            )
        };

        match current_panel {
            Panel::Android => {
                let cache = device_cache_clone.read().await;
                let cached_devices = cache.android_device_cache.clone();
                drop(cache);

                if let Some(all_devices) = cached_devices {
                    let filtered_devices = if category_filter == "all" {
                        all_devices
                    } else {
                        all_devices
                            .into_iter()
                            .filter(|(id, display)| {
                                let device_category =
                                    self.android_manager.get_device_category(id, display);
                                device_category == category_filter
                            })
                            .collect()
                    };

                    let mut state = self.state.lock().await;
                    state.create_device_form.available_device_types = filtered_devices;

                    state.create_device_form.selected_device_type_index = 0;
                    if !state.create_device_form.available_device_types.is_empty() {
                        let (id, display) =
                            state.create_device_form.available_device_types[0].clone();
                        state.create_device_form.device_type_id = id;
                        state.create_device_form.device_type = display;
                        state.create_device_form.generate_placeholder_name();
                    }
                } else {
                    let filtered_devices = self
                        .android_manager
                        .list_devices_by_category(if category_filter == "all" {
                            None
                        } else {
                            Some(&category_filter)
                        })
                        .await?;

                    let mut state = self.state.lock().await;
                    state.create_device_form.available_device_types = filtered_devices;

                    state.create_device_form.selected_device_type_index = 0;
                    if !state.create_device_form.available_device_types.is_empty() {
                        let (id, display) =
                            state.create_device_form.available_device_types[0].clone();
                        state.create_device_form.device_type_id = id;
                        state.create_device_form.device_type = display;
                        state.create_device_form.generate_placeholder_name();
                    }
                }
            }
            Panel::Ios => {}
        }

        Ok(())
    }
}
