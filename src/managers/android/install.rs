use super::AndroidManager;
use crate::{
    constants::{
        android, commands,
        keywords::{LOG_LEVEL_ERROR, LOG_LEVEL_FAILED},
        limits::SYSTEM_IMAGE_PARTS_REQUIRED,
        progress::{
            COMPLETION_THRESHOLD_PERCENTAGE, DOWNLOAD_PHASE_INCREMENT,
            DOWNLOAD_PHASE_START_PERCENTAGE, DOWNLOAD_PROGRESS_MULTIPLIER, EXTRACT_PHASE_INCREMENT,
            EXTRACT_PHASE_START_PERCENTAGE, INSTALL_PHASE_START_PERCENTAGE,
            LOADING_PHASE_INCREMENT, PROGRESS_PHASE_100_PERCENT, PROGRESS_PHASE_75_PERCENT,
            PROGRESS_PHASE_85_PERCENT,
        },
        timeouts::DEVICE_START_WAIT_TIME,
    },
    models::{ApiLevel, InstallProgress, SystemImageVariant},
};
use anyhow::Result;

impl AndroidManager {
    /// Lists available API levels with their installation status and Android version names.
    pub async fn list_api_levels(&self) -> Result<Vec<ApiLevel>> {
        if let Some(cached_levels) = self.get_cached_api_levels().await {
            return Ok(cached_levels);
        }

        let output = self.get_sdkmanager_verbose_output().await?;
        let api_levels = self.parse_api_levels_from_output(&output);
        self.set_cached_api_levels(api_levels.clone()).await;

        Ok(api_levels)
    }

    pub(crate) async fn list_api_levels_fresh(&self) -> Result<Vec<ApiLevel>> {
        let output = self.refresh_sdkmanager_verbose_output().await?;
        let api_levels = self.parse_api_levels_from_output(&output);
        self.set_cached_api_levels(api_levels.clone()).await;

        Ok(api_levels)
    }

    fn parse_api_levels_from_output(&self, output_str: &str) -> Vec<ApiLevel> {
        let mut api_levels_map: std::collections::HashMap<u32, ApiLevel> =
            std::collections::HashMap::new();
        let mut in_installed_section = false;
        let mut found_system_images = false;

        for line in output_str.lines() {
            let line = line.trim();

            if line.contains("Installed packages") || line.contains("Installed Packages") {
                in_installed_section = true;
                continue;
            } else if line.contains("Available Packages") || line.contains("Available Updates") {
                in_installed_section = false;
                continue;
            }

            if line.starts_with("system-images;android-") {
                found_system_images = true;
                let package_id = line.split_whitespace().next().unwrap_or(line);

                if let Some(api_level) = self.parse_api_level_from_package(package_id) {
                    let is_installed = in_installed_section;
                    let parts: Vec<&str> = package_id.split(';').collect();
                    if parts.len() >= SYSTEM_IMAGE_PARTS_REQUIRED {
                        let variant = parts[2].to_string();
                        let architecture = parts[3].to_string();

                        let system_variant = SystemImageVariant::new(
                            variant.clone(),
                            architecture,
                            package_id.to_string(),
                        );

                        let api_entry = api_levels_map.entry(api_level).or_insert_with(|| {
                            let version_name = self.get_android_version_name(api_level);
                            ApiLevel::new(
                                api_level,
                                version_name,
                                format!("system-images;android-{api_level};google_apis;x86_64"),
                            )
                        });

                        let mut variant_clone = system_variant;
                        variant_clone.is_installed = is_installed;
                        api_entry.variants.push(variant_clone);

                        if is_installed {
                            api_entry.is_installed = true;
                        }
                    }
                }
            }
        }

        if !found_system_images {
            let max_api = api_levels_map.keys().max().copied().unwrap_or(35);
            let start_api = max_api.saturating_sub(android::DEFAULT_API_LEVELS_COUNT as u32 - 1);
            let start_api = start_api.max(android::DEFAULT_MIN_API_LEVEL);

            for api in start_api..=max_api {
                api_levels_map.entry(api).or_insert_with(|| {
                    let version_name = self.get_android_version_name(api);
                    ApiLevel::new(
                        api,
                        version_name,
                        format!("system-images;android-{api};google_apis;x86_64"),
                    )
                });
            }
        }

        let mut api_levels: Vec<ApiLevel> = api_levels_map.into_values().collect();
        api_levels.sort_by(|a, b| b.api.cmp(&a.api));
        api_levels
    }

    /// Installs a system image with progress callback.
    pub async fn install_system_image<F>(
        &self,
        package_id: &str,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(InstallProgress) + Send + Sync + 'static,
    {
        progress_callback(InstallProgress {
            operation: "Preparing installation...".to_string(),
            percentage: 0,
            eta_seconds: None,
        });

        let sdkmanager_path = Self::find_tool(&self.android_home, commands::SDKMANAGER)?;
        let mut child = tokio::process::Command::new(&sdkmanager_path)
            .args([package_id])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(b"y\n").await?;
            stdin.flush().await?;
        }

        progress_callback(InstallProgress {
            operation: "Starting installation process...".to_string(),
            percentage: 5,
            eta_seconds: None,
        });

        let progress_callback = std::sync::Arc::new(progress_callback);
        let progress_clone = progress_callback.clone();
        let stop_timer = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_timer_clone = stop_timer.clone();

        tokio::spawn(async move {
            let mut progress = 10u8;
            let mut stage = 0;

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(
                    DEVICE_START_WAIT_TIME.as_secs(),
                ))
                .await;

                if stop_timer_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                match stage {
                    0 => {
                        progress_clone(InstallProgress {
                            operation: "Loading package information...".to_string(),
                            percentage: progress,
                            eta_seconds: None,
                        });
                        progress += LOADING_PHASE_INCREMENT;
                        if progress >= DOWNLOAD_PHASE_START_PERCENTAGE {
                            stage = 1;
                            progress = DOWNLOAD_PHASE_START_PERCENTAGE;
                        }
                    }
                    1 => {
                        progress_clone(InstallProgress {
                            operation: "Downloading system image...".to_string(),
                            percentage: progress,
                            eta_seconds: None,
                        });
                        progress += DOWNLOAD_PHASE_INCREMENT;
                        if progress >= EXTRACT_PHASE_START_PERCENTAGE {
                            stage = 2;
                            progress = EXTRACT_PHASE_START_PERCENTAGE;
                        }
                    }
                    2 => {
                        progress_clone(InstallProgress {
                            operation: "Extracting system image...".to_string(),
                            percentage: progress,
                            eta_seconds: None,
                        });
                        progress += EXTRACT_PHASE_INCREMENT;
                        if progress >= INSTALL_PHASE_START_PERCENTAGE {
                            stage = 3;
                            progress = INSTALL_PHASE_START_PERCENTAGE;
                        }
                    }
                    3 => {
                        progress_clone(InstallProgress {
                            operation: "Installing system image...".to_string(),
                            percentage: progress,
                            eta_seconds: None,
                        });
                        progress += 2;
                        if progress >= COMPLETION_THRESHOLD_PERCENTAGE {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        });

        if let Some(stdout) = child.stdout.take() {
            let progress_stdout = progress_callback.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if line.contains("Downloading") {
                        if line.contains(" MiB") || line.contains(" MB") {
                            if let Some(start) = line.find('(') {
                                if let Some(end) = line.find('%') {
                                    if let Ok(pct) = line[start + 1..end].trim().parse::<u8>() {
                                        progress_stdout(InstallProgress {
                                            operation: "Downloading system image...".to_string(),
                                            percentage: (DOWNLOAD_PHASE_START_PERCENTAGE
                                                + (pct * DOWNLOAD_PROGRESS_MULTIPLIER
                                                    / PROGRESS_PHASE_100_PERCENT))
                                                .min(EXTRACT_PHASE_START_PERCENTAGE),
                                            eta_seconds: None,
                                        });
                                    }
                                }
                            }
                        }
                    } else if line.contains("Unzipping") || line.contains("Extracting") {
                        progress_stdout(InstallProgress {
                            operation: "Extracting system image...".to_string(),
                            percentage: PROGRESS_PHASE_75_PERCENT,
                            eta_seconds: None,
                        });
                    } else if line.contains("Installing") {
                        progress_stdout(InstallProgress {
                            operation: "Installing system image...".to_string(),
                            percentage: PROGRESS_PHASE_85_PERCENT,
                            eta_seconds: None,
                        });
                    }
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if line.contains(LOG_LEVEL_ERROR)
                        || line.contains("error")
                        || line.contains(LOG_LEVEL_FAILED)
                    {
                        eprintln!("sdkmanager error: {line}");
                    }
                }
            });
        }

        let output = child.wait_with_output().await?;
        stop_timer.store(true, std::sync::atomic::Ordering::Relaxed);

        if output.status.success() {
            self.invalidate_sdk_list_caches().await;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to install system image: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Uninstalls a system image.
    pub async fn uninstall_system_image(&self, package_id: &str) -> Result<()> {
        let sdkmanager_path = Self::find_tool(&self.android_home, commands::SDKMANAGER)?;
        let output = tokio::process::Command::new(&sdkmanager_path)
            .args(["--uninstall", package_id])
            .output()
            .await?;

        if output.status.success() {
            self.invalidate_sdk_list_caches().await;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to uninstall system image: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Parses API level from package ID.
    pub(super) fn parse_api_level_from_package(&self, package_id: &str) -> Option<u32> {
        if let Some(start) = package_id.find("android-") {
            let api_part = &package_id[start + 8..];
            if let Some(end) = api_part.find(';') {
                api_part[..end].parse().ok()
            } else {
                api_part.parse().ok()
            }
        } else {
            None
        }
    }
}
