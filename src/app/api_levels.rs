use super::{state, App, Mode, Panel};
use crate::constants::{
    messages::notifications::INSTALL_PROGRESS_COMPLETE,
    performance::API_INSTALLATION_COMPLETION_DELAY, progress::PROGRESS_PHASE_100_PERCENT,
};
use crate::utils::ApiLevelCache;
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    pub(super) async fn open_api_level_management(&mut self) {
        let cached_api_levels = self.android_manager.get_cached_api_levels().await;

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

        let android_manager = self.android_manager.clone();
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            if let Ok(api_levels) = android_manager.list_api_levels().await {
                let mut state = state_clone.lock().await;
                if let Some(ref mut api_state) = state.api_level_management {
                    api_state.api_levels = api_levels;
                    api_state.is_loading = false;
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
                self.install_selected_api_level().await;
            }
            KeyCode::Char('d') => {
                self.uninstall_selected_api_level().await;
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

            let mut state = state_clone.lock().await;
            if let Some(ref mut api_mgmt) = state.api_level_management {
                api_mgmt.installing_package = None;
                api_mgmt.install_progress = None;
            }

            if let Err(error) = result {
                if let Some(ref mut api_mgmt) = state.api_level_management {
                    api_mgmt.error_message = Some(format!("Failed to install: {error}"));
                }
            } else {
                state.add_success_notification("System image installed successfully".to_string());
                {
                    let mut cache = state.device_cache.write().await;
                    cache.invalidate_android_cache();
                }
                if let Err(error) = ApiLevelCache::clear_from_disk() {
                    log::warn!("Failed to clear API level cache after install: {error}");
                }

                if let Some(ref mut api_mgmt) = state.api_level_management {
                    api_mgmt.is_loading = true;
                }

                let android_manager_refresh = android_manager.clone();
                let state_refresh = state_clone.clone();
                tokio::spawn(async move {
                    if let Ok(new_levels) = android_manager_refresh.list_api_levels().await {
                        let mut state = state_refresh.lock().await;
                        if let Some(ref mut api_mgmt) = state.api_level_management {
                            api_mgmt.api_levels = new_levels;
                            api_mgmt.is_loading = false;
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
                    for api_level in &mut api_mgmt.api_levels {
                        for variant in &mut api_level.variants {
                            if installed_variants.contains(&variant.package_id) {
                                variant.is_installed = false;
                            }
                        }
                        api_level.is_installed = api_level
                            .variants
                            .iter()
                            .any(|variant| variant.is_installed);
                    }
                    api_mgmt.installing_package = None;
                }

                state.add_success_notification(
                    "System image(s) uninstalled successfully".to_string(),
                );
            } else if let Some(ref mut api_mgmt) = state.api_level_management {
                api_mgmt.installing_package = None;
                api_mgmt.error_message = Some(format!(
                    "Failed to uninstall: {}",
                    last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error"))
                ));
            }

            {
                let mut cache = state.device_cache.write().await;
                cache.invalidate_android_cache();
            }
            if let Err(error) = ApiLevelCache::clear_from_disk() {
                log::warn!("Failed to clear API level cache after uninstall: {error}");
            }

            let android_manager_refresh = android_manager.clone();
            let state_refresh = state_clone.clone();
            tokio::spawn(async move {
                if let Ok(new_levels) = android_manager_refresh.list_api_levels().await {
                    let mut state = state_refresh.lock().await;
                    if let Some(ref mut api_mgmt) = state.api_level_management {
                        api_mgmt.api_levels = new_levels;
                        api_mgmt.is_loading = false;
                    }
                }
            });
        });
    }
}
