use super::{state, App, Mode, Panel};
use crate::constants::{
    messages::{
        errors::SYSTEM_IMAGE_OPERATION_IN_PROGRESS,
        notifications::{
            INSTALL_PROGRESS_COMPLETE, SYSTEM_IMAGE_INSTALLED, SYSTEM_IMAGE_UNINSTALLED,
        },
    },
    performance::API_INSTALLATION_COMPLETION_DELAY,
    progress::PROGRESS_PHASE_100_PERCENT,
};
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    fn update_api_level_installation_state(
        api_mgmt: &mut state::ApiLevelManagementState,
        package_ids: &[String],
        is_installed: bool,
    ) {
        for api_level in &mut api_mgmt.api_levels {
            for variant in &mut api_level.variants {
                if package_ids.contains(&variant.package_id) {
                    variant.is_installed = is_installed;
                }
            }

            api_level.is_installed = api_level
                .variants
                .iter()
                .any(|variant| variant.is_installed);
        }
    }

    fn set_api_level_busy_error(api_mgmt: &mut state::ApiLevelManagementState) {
        api_mgmt.error_message = Some(SYSTEM_IMAGE_OPERATION_IN_PROGRESS.to_string());
    }

    pub(super) async fn open_api_level_management(&mut self) {
        let cached_api_levels = self.android_manager.get_cached_api_levels().await;
        let has_warm_cache = cached_api_levels.is_some();

        let should_open = {
            let mut state = self.state.lock().await;
            if state.active_panel != Panel::Android {
                false
            } else {
                let mut api_state = state::ApiLevelManagementState::new();
                if let Some(cached_api_levels) = cached_api_levels {
                    api_state.api_levels = cached_api_levels;
                    api_state.is_loading = false;
                }
                state.mode = Mode::ManageApiLevels;
                state.api_level_management = Some(api_state);
                true
            }
        };

        if !should_open {
            return;
        }

        if has_warm_cache {
            return;
        }

        let android_manager = self.android_manager.clone();
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            let result = android_manager.list_api_levels().await;
            let mut state = state_clone.lock().await;
            if let Some(ref mut api_state) = state.api_level_management {
                api_state.is_loading = false;
                match result {
                    Ok(api_levels) => api_state.api_levels = api_levels,
                    Err(error) => {
                        api_state.error_message =
                            Some(format!("Failed to load API levels: {error}"));
                    }
                }
            }
        });
    }

    pub(super) async fn handle_api_level_mode_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                let mut state = self.state.lock().await;
                if let Some(ref api_mgmt) = state.api_level_management {
                    if !api_mgmt.is_busy() {
                        state.mode = Mode::Normal;
                        state.api_level_management = None;
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let mut state = self.state.lock().await;
                if let Some(ref mut api_state) = state.api_level_management {
                    api_state.move_up();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let mut state = self.state.lock().await;
                if let Some(ref mut api_state) = state.api_level_management {
                    api_state.move_down();
                }
            }
            KeyCode::Enter => {
                let mut state = self.state.lock().await;
                let can_install = if let Some(api_mgmt) = state.api_level_management.as_mut() {
                    if api_mgmt.is_busy() {
                        Self::set_api_level_busy_error(api_mgmt);
                        false
                    } else {
                        true
                    }
                } else {
                    false
                };
                drop(state);

                if can_install {
                    self.install_selected_api_level().await;
                }
            }
            KeyCode::Char('d') => {
                let mut state = self.state.lock().await;
                let can_uninstall = if let Some(api_mgmt) = state.api_level_management.as_mut() {
                    if api_mgmt.is_busy() {
                        Self::set_api_level_busy_error(api_mgmt);
                        false
                    } else {
                        true
                    }
                } else {
                    false
                };
                drop(state);

                if can_uninstall {
                    self.uninstall_selected_api_level().await;
                }
            }
            _ => {}
        }
    }

    async fn install_selected_api_level(&mut self) {
        let package_id = {
            let mut state = self.state.lock().await;
            let Some(ref api_state) = state.api_level_management else {
                return;
            };
            let Some(api_level) = api_state.get_selected_api_level() else {
                return;
            };
            let Some(variant) = api_level.get_recommended_variant() else {
                return;
            };
            if variant.is_installed {
                return;
            }

            let package_id = variant.package_id.clone();
            if let Some(ref mut api_mgmt) = state.api_level_management {
                api_mgmt.installing_package = Some(package_id.clone());
                api_mgmt.error_message = None;
            }
            package_id
        };

        let android_manager = self.android_manager.clone();
        let state_clone = self.state.clone();
        let state_clone_for_progress = state_clone.clone();

        tokio::spawn(async move {
            let result = android_manager
                .install_system_image(&package_id, move |progress| {
                    let state_clone = state_clone_for_progress.clone();
                    tokio::spawn(async move {
                        let mut state = state_clone.lock().await;
                        if let Some(ref mut api_mgmt) = state.api_level_management {
                            let already_complete = api_mgmt
                                .install_progress
                                .as_ref()
                                .map(|progress| progress.percentage >= PROGRESS_PHASE_100_PERCENT)
                                .unwrap_or(false);
                            if !already_complete {
                                api_mgmt.install_progress = Some(progress);
                            }
                        }
                    });
                })
                .await;

            if result.is_ok() {
                let mut state = state_clone.lock().await;
                if let Some(ref mut api_mgmt) = state.api_level_management {
                    api_mgmt.install_progress = Some(crate::models::InstallProgress {
                        operation: INSTALL_PROGRESS_COMPLETE.to_string(),
                        percentage: PROGRESS_PHASE_100_PERCENT,
                        eta_seconds: None,
                    });
                }
            }

            tokio::time::sleep(API_INSTALLATION_COMPLETION_DELAY).await;

            if let Err(error) = result {
                let mut state = state_clone.lock().await;
                if let Some(ref mut api_mgmt) = state.api_level_management {
                    api_mgmt.installing_package = None;
                    api_mgmt.install_progress = None;
                    api_mgmt.error_message = Some(format!("Failed to install: {error}"));
                }
            } else {
                let mut state = state_clone.lock().await;
                if let Some(ref mut api_mgmt) = state.api_level_management {
                    Self::update_api_level_installation_state(
                        api_mgmt,
                        std::slice::from_ref(&package_id),
                        true,
                    );
                    api_mgmt.is_loading = true;
                }

                state.add_success_notification(SYSTEM_IMAGE_INSTALLED.to_string());
                {
                    let mut cache = state.device_cache.write().await;
                    cache.invalidate_android_cache();
                }
                drop(state);

                let android_manager_refresh = android_manager.clone();
                let state_refresh = state_clone.clone();
                tokio::spawn(async move {
                    let refresh_result = android_manager_refresh.list_api_levels_fresh().await;
                    let mut state = state_refresh.lock().await;
                    if let Some(ref mut api_mgmt) = state.api_level_management {
                        api_mgmt.installing_package = None;
                        api_mgmt.install_progress = None;
                        api_mgmt.is_loading = false;
                        match refresh_result {
                            Ok(new_levels) => api_mgmt.api_levels = new_levels,
                            Err(error) => {
                                log::warn!("Failed to refresh API levels after install: {error}");
                            }
                        }
                    }
                });
            }
        });
    }

    async fn uninstall_selected_api_level(&mut self) {
        let installed_variants = {
            let mut state = self.state.lock().await;
            let Some(ref api_state) = state.api_level_management else {
                return;
            };
            let Some(api_level) = api_state.get_selected_api_level() else {
                return;
            };
            let installed_variants: Vec<_> = api_level
                .variants
                .iter()
                .filter(|variant| variant.is_installed)
                .map(|variant| variant.package_id.clone())
                .collect();

            if installed_variants.is_empty() {
                return;
            }

            if let Some(ref mut api_mgmt) = state.api_level_management {
                api_mgmt.installing_package = Some(installed_variants[0].clone());
                api_mgmt.error_message = None;
            }

            installed_variants
        };

        let android_manager = self.android_manager.clone();
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            let mut success = true;
            let mut last_error = None;

            for package_id in &installed_variants {
                if let Err(error) = android_manager.uninstall_system_image(package_id).await {
                    success = false;
                    last_error = Some(error);
                }
            }

            let mut state = state_clone.lock().await;
            if success {
                if let Some(ref mut api_mgmt) = state.api_level_management {
                    Self::update_api_level_installation_state(api_mgmt, &installed_variants, false);
                    api_mgmt.is_loading = true;
                }

                state.add_success_notification(SYSTEM_IMAGE_UNINSTALLED.to_string());
            } else if let Some(ref mut api_mgmt) = state.api_level_management {
                api_mgmt.installing_package = None;
                api_mgmt.install_progress = None;
                api_mgmt.error_message = Some(format!(
                    "Failed to uninstall: {}",
                    last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error"))
                ));
            }

            {
                let mut cache = state.device_cache.write().await;
                cache.invalidate_android_cache();
            }

            let android_manager_refresh = android_manager.clone();
            let state_refresh = state_clone.clone();
            tokio::spawn(async move {
                let refresh_result = android_manager_refresh.list_api_levels_fresh().await;
                let mut state = state_refresh.lock().await;
                if let Some(ref mut api_mgmt) = state.api_level_management {
                    api_mgmt.installing_package = None;
                    api_mgmt.install_progress = None;
                    api_mgmt.is_loading = false;
                    match refresh_result {
                        Ok(new_levels) => api_mgmt.api_levels = new_levels,
                        Err(error) => {
                            log::warn!("Failed to refresh API levels after uninstall: {error}");
                        }
                    }
                }
            });
        });
    }
}
