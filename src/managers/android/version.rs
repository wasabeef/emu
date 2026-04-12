use super::AndroidManager;
use regex::Regex;

impl AndroidManager {
    pub(super) fn parse_android_version_to_api_level(version: &str) -> u32 {
        let major_version = version
            .split('.')
            .next()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);

        match major_version {
            15 => 35,
            14 => 34,
            13 => 33,
            12 => 32,
            11 => 30,
            10 => 29,
            9 => 28,
            8 => 26,
            7 => 24,
            6 => 23,
            5 => 21,
            4 => 15,
            _ => 0,
        }
    }

    pub(super) fn get_android_version_name(&self, api_level: u32) -> String {
        format!("API {api_level}")
    }

    pub(super) async fn get_dynamic_android_version_name(&self, api_level: u32) -> Option<String> {
        if let Ok(targets) = self.list_available_targets().await {
            for (level_str, display) in targets {
                if let Ok(level) = level_str.parse::<u32>() {
                    if level == api_level {
                        if let Some(dash_pos) = display.find(" - Android ") {
                            return Some(display[dash_pos + 11..].to_string());
                        }
                    }
                }
            }
        }

        if let Ok(sdkmanager_path) = Self::find_tool(&self.android_home, "sdkmanager") {
            if let Ok(output) = self
                .command_executor
                .run(&sdkmanager_path, &["--list"])
                .await
            {
                let pattern = format!(
                    r"platforms;android-{api_level}\s*\|\s*\d+\s*\|\s*Android SDK Platform"
                );
                if let Ok(regex) = Regex::new(&pattern) {
                    if regex.is_match(&output) {
                        for line in output.lines() {
                            if line.contains(&format!("android-{api_level}"))
                                && line.contains("Android")
                            {
                                if let Some(version_match) = line.split("Android").nth(1) {
                                    let version = version_match
                                        .split_whitespace()
                                        .next()
                                        .unwrap_or("")
                                        .trim();
                                    if !version.is_empty() {
                                        return Some(version.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}
