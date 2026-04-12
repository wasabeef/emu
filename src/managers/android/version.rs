use super::AndroidManager;
use crate::constants::commands;
use regex::Regex;

impl AndroidManager {
    pub(super) fn parse_android_version_to_api_level(version: &str) -> u32 {
        match version.trim() {
            "15" | "15.0" => 35,
            "14" | "14.0" => 34,
            "13" | "13.0" => 33,
            "11" | "11.0" => 30,
            "10" | "10.0" => 29,
            "9" | "9.0" => 28,
            "8.1" => 27,
            "8.0" => 26,
            "7.1" => 25,
            "7.0" => 24,
            "6" | "6.0" => 23,
            "5.1" => 22,
            "5.0" => 21,
            "4.4" => 19,
            "4.3" => 18,
            "4.2" => 17,
            "4.1" => 16,
            "4.0.3" | "4.0.4" => 15,
            "4.0" => 14,
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

        if let Ok(sdkmanager_path) = Self::find_tool(&self.android_home, commands::SDKMANAGER) {
            if let Ok(output) = self
                .command_executor
                .run(&sdkmanager_path, &[commands::sdkmanager::LIST])
                .await
            {
                let package_name = format!("platforms;android-{api_level}");
                let pattern = format!(r"{package_name}\s*\|");
                if let Ok(regex) = Regex::new(&pattern) {
                    for line in output.lines() {
                        if regex.is_match(line) {
                            if let Some((_, version_name)) = line.rsplit_once("| Android ") {
                                let version_name = version_name.trim();
                                if !version_name.is_empty() {
                                    return Some(version_name.to_string());
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
