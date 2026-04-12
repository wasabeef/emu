use super::{state, App, AppState};
use crate::models::error::format_user_error;
use anyhow::Result;

impl App {
    pub(super) async fn navigate_create_form(&mut self, forward: bool) {
        let mut state = self.state.lock().await;
        if state.create_device_form.is_creating {
            return;
        }

        match (state.active_panel, forward) {
            (super::Panel::Android, true) => state.create_device_form.next_field(),
            (super::Panel::Android, false) => state.create_device_form.prev_field(),
            (super::Panel::Ios, true) => state.create_device_form.next_field_ios(),
            (super::Panel::Ios, false) => state.create_device_form.prev_field_ios(),
        }
    }

    pub(super) async fn change_create_device_selection(
        &mut self,
        move_right: bool,
    ) -> anyhow::Result<()> {
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

    pub(super) async fn handle_create_mode_ctrl_char(&mut self, c: char) -> anyhow::Result<()> {
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

    pub(super) async fn reload_device_types_for_category(&mut self) -> Result<()> {
        let (current_panel, category_filter, device_cache_clone) = {
            let state = self.state.lock().await;
            let device_cache_clone = std::sync::Arc::clone(&state.device_cache);
            (
                state.active_panel,
                state.create_device_form.device_category_filter.clone(),
                device_cache_clone,
            )
        };

        match current_panel {
            super::Panel::Android => {
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
            super::Panel::Ios => {}
        }

        Ok(())
    }
}
