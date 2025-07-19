//! Unit tests for constants/mod.rs module
//!
//! This module contains comprehensive tests for the constants module's re-export functionality
//! and module organization.

use emu::constants::*;
use std::time::Duration;

#[cfg(test)]
mod re_export_tests {
    use super::*;

    #[test]
    fn test_colors_re_export() {
        // Test that color constants are re-exported and accessible
        use ratatui::style::Color;
        
        // Test that we can use re-exported color constants
        assert_eq!(DARK_THEME_BG_PRIMARY, Color::Rgb(25, 25, 35));
        assert_eq!(LIGHT_THEME_BG_PRIMARY, Color::Rgb(240, 245, 250));
        assert_eq!(STATUS_COLOR_SUCCESS, Color::Green);
        assert_eq!(STATUS_COLOR_ERROR, Color::Red);
        assert_eq!(UI_COLOR_HIGHLIGHT, Color::Yellow);
        assert_eq!(LOG_COLOR_ERROR, Color::Red);
        
        // Test that color constants are of correct type
        let _: Color = DARK_THEME_BG_PRIMARY;
        let _: Color = LIGHT_THEME_BG_PRIMARY;
        let _: Color = STATUS_COLOR_SUCCESS;
        let _: Color = STATUS_COLOR_ERROR;
        let _: Color = UI_COLOR_HIGHLIGHT;
        let _: Color = LOG_COLOR_ERROR;
    }

    #[test]
    fn test_commands_re_export() {
        // Test that command constants are re-exported and accessible
        assert_eq!(ADB, "adb");
        assert_eq!(AVDMANAGER, "avdmanager");
        assert_eq!(EMULATOR, "emulator");
        assert_eq!(XCRUN, "xcrun");
        assert_eq!(SIMCTL, "simctl");
        
        // Test that command constants are of correct type
        let _: &str = ADB;
        let _: &str = AVDMANAGER;
        let _: &str = EMULATOR;
        let _: &str = XCRUN;
        let _: &str = SIMCTL;
    }

    #[test]
    fn test_defaults_re_export() {
        // Test that default constants are re-exported and accessible
        assert_eq!(DEFAULT_RAM_MB, 2048);
        assert_eq!(DEFAULT_STORAGE_MB, 8192);
        assert_eq!(DEFAULT_GPU_MODE, "auto");
        assert_eq!(DEFAULT_SKIN, "pixel_5");
        assert_eq!(CACHE_EXPIRATION, Duration::from_secs(300));
        assert_eq!(DEVICE_REFRESH_INTERVAL, Duration::from_secs(5));
        assert_eq!(DEFAULT_API_LEVELS, &[35, 34, 33, 32, 31, 30, 29, 28]);
        
        // Test that default constants are of correct type
        let _: u32 = DEFAULT_RAM_MB;
        let _: u32 = DEFAULT_STORAGE_MB;
        let _: &str = DEFAULT_GPU_MODE;
        let _: &str = DEFAULT_SKIN;
        let _: Duration = CACHE_EXPIRATION;
        let _: Duration = DEVICE_REFRESH_INTERVAL;
        let _: &[u32] = DEFAULT_API_LEVELS;
    }

    #[test]
    fn test_env_vars_re_export() {
        // Test that environment variable constants are re-exported and accessible
        assert_eq!(ANDROID_HOME, "ANDROID_HOME");
        assert_eq!(ANDROID_SDK_ROOT, "ANDROID_SDK_ROOT");
        assert_eq!(HOME, "HOME");
        assert_eq!(PATH, "PATH");
        assert_eq!(RUST_LOG, "RUST_LOG");
        
        // Test that env var constants are of correct type
        let _: &str = ANDROID_HOME;
        let _: &str = ANDROID_SDK_ROOT;
        let _: &str = HOME;
        let _: &str = PATH;
        let _: &str = RUST_LOG;
    }

    #[test]
    fn test_files_re_export() {
        // Test that file constants are re-exported and accessible
        assert_eq!(AVD_EXTENSION, ".avd");
        assert_eq!(INI_EXTENSION, ".ini");
        assert_eq!(LOG_EXTENSION, ".log");
        assert_eq!(CONFIG_FILE, "config.ini");
        assert_eq!(HARDWARE_FILE, "hardware-qemu.ini");
        
        // Test that file constants are of correct type
        let _: &str = AVD_EXTENSION;
        let _: &str = INI_EXTENSION;
        let _: &str = LOG_EXTENSION;
        let _: &str = CONFIG_FILE;
        let _: &str = HARDWARE_FILE;
    }

    #[test]
    fn test_ios_devices_re_export() {
        // Test that iOS device constants are re-exported and accessible
        assert_eq!(DISPLAY_SIZE_13_INCH, "13 inch");
        assert_eq!(DISPLAY_SIZE_11_INCH, "11 inch");
        assert_eq!(MEMORY_8GB_INDICATOR, "8GB");
        assert_eq!(MEMORY_16GB_INDICATOR, "16GB");
        assert_eq!(IOS_DEVICE_BATCH_SIZE, 10);
        
        // Test that iOS device constants are of correct type
        let _: &str = DISPLAY_SIZE_13_INCH;
        let _: &str = DISPLAY_SIZE_11_INCH;
        let _: &str = MEMORY_8GB_INDICATOR;
        let _: &str = MEMORY_16GB_INDICATOR;
        let _: usize = IOS_DEVICE_BATCH_SIZE;
    }

    #[test]
    fn test_keywords_re_export() {
        // Test that keyword constants are re-exported and accessible
        assert_eq!(LOG_LEVEL_ERROR, "Error");
        assert_eq!(LOG_LEVEL_WARNING, "Warning");
        assert_eq!(DEVICE_KEYWORD_PIXEL, "pixel");
        assert_eq!(DEVICE_KEYWORD_IPHONE, "iphone");
        assert_eq!(DEVICE_KEYWORD_IPAD, "ipad");
        assert_eq!(DEVICE_KEYWORD_PRO, "pro");
        
        // Test that keyword constants are of correct type
        let _: &str = LOG_LEVEL_ERROR;
        let _: &str = LOG_LEVEL_WARNING;
        let _: &str = DEVICE_KEYWORD_PIXEL;
        let _: &str = DEVICE_KEYWORD_IPHONE;
        let _: &str = DEVICE_KEYWORD_IPAD;
        let _: &str = DEVICE_KEYWORD_PRO;
    }

    #[test]
    fn test_limits_re_export() {
        // Test that limit constants are re-exported and accessible
        assert_eq!(MIN_RAM_MB, 512);
        assert_eq!(MAX_RAM_MB, 8192);
        assert_eq!(MIN_STORAGE_MB, 1024);
        assert_eq!(MAX_STORAGE_MB, 65536);
        assert_eq!(MAX_DEVICE_NAME_LENGTH, 50);
        assert_eq!(MAX_LOG_ENTRIES, 1000);
        
        // Test that limit constants are of correct type
        let _: u32 = MIN_RAM_MB;
        let _: u32 = MAX_RAM_MB;
        let _: u32 = MIN_STORAGE_MB;
        let _: u32 = MAX_STORAGE_MB;
        let _: usize = MAX_DEVICE_NAME_LENGTH;
        let _: usize = MAX_LOG_ENTRIES;
    }

    #[test]
    fn test_numeric_re_export() {
        // Test that numeric constants are re-exported and accessible
        assert_eq!(BYTES_PER_KB, 1024);
        assert_eq!(BYTES_PER_MB, 1024 * 1024);
        assert_eq!(BYTES_PER_GB, 1024 * 1024 * 1024);
        assert_eq!(VERSION_MAJOR_DIVISOR, 10.0);
        assert_eq!(VERSION_DEFAULT, 0.0);
        assert_eq!(IOS_DEVICE_PARSE_BATCH_SIZE, 10);
        
        // Test that numeric constants are of correct type
        let _: u64 = BYTES_PER_KB;
        let _: u64 = BYTES_PER_MB;
        let _: u64 = BYTES_PER_GB;
        let _: f32 = VERSION_MAJOR_DIVISOR;
        let _: f32 = VERSION_DEFAULT;
        let _: usize = IOS_DEVICE_PARSE_BATCH_SIZE;
    }

    #[test]
    fn test_performance_re_export() {
        // Test that performance constants are re-exported and accessible
        assert_eq!(MAX_EVENTS_PER_FRAME, 50);
        assert_eq!(UI_UPDATE_INTERVAL, Duration::from_millis(17));
        assert_eq!(TARGET_FPS, 125);
        assert_eq!(FRAME_TIME_MS, 8);
        assert_eq!(FRAME_DURATION, Duration::from_millis(8));
        
        // Test that performance constants are of correct type
        let _: usize = MAX_EVENTS_PER_FRAME;
        let _: Duration = UI_UPDATE_INTERVAL;
        let _: u32 = TARGET_FPS;
        let _: u64 = FRAME_TIME_MS;
        let _: Duration = FRAME_DURATION;
    }

    #[test]
    fn test_priorities_re_export() {
        // Test that priority constants are re-exported and accessible
        assert_eq!(ANDROID_PIXEL_PRIORITY, 30);
        assert_eq!(ANDROID_NEXUS_PRIORITY, 40);
        assert_eq!(ANDROID_ONEPLUS_PRIORITY, 50);
        assert_eq!(IOS_IPHONE_MINI_PRIORITY, 30);
        assert_eq!(IOS_IPHONE_PRO_MAX_PRIORITY, 80);
        assert_eq!(IOS_IPAD_PRO_11_PRIORITY, 130);
        
        // Test that priority constants are of correct type
        let _: u8 = ANDROID_PIXEL_PRIORITY;
        let _: u8 = ANDROID_NEXUS_PRIORITY;
        let _: u8 = ANDROID_ONEPLUS_PRIORITY;
        let _: u8 = IOS_IPHONE_MINI_PRIORITY;
        let _: u8 = IOS_IPHONE_PRO_MAX_PRIORITY;
        let _: u8 = IOS_IPAD_PRO_11_PRIORITY;
    }

    #[test]
    fn test_progress_re_export() {
        // Test that progress constants are re-exported and accessible
        assert_eq!(DOWNLOAD_PHASE_START_PERCENTAGE, 20);
        assert_eq!(EXTRACT_PHASE_START_PERCENTAGE, 70);
        assert_eq!(INSTALL_PHASE_START_PERCENTAGE, 90);
        assert_eq!(COMPLETION_THRESHOLD_PERCENTAGE, 95);
        assert_eq!(LOADING_PHASE_INCREMENT, 5);
        assert_eq!(DOWNLOAD_PHASE_INCREMENT, 3);
        
        // Test that progress constants are of correct type
        let _: u8 = DOWNLOAD_PHASE_START_PERCENTAGE;
        let _: u8 = EXTRACT_PHASE_START_PERCENTAGE;
        let _: u8 = INSTALL_PHASE_START_PERCENTAGE;
        let _: u8 = COMPLETION_THRESHOLD_PERCENTAGE;
        let _: u8 = LOADING_PHASE_INCREMENT;
        let _: u8 = DOWNLOAD_PHASE_INCREMENT;
    }

    #[test]
    fn test_resolutions_re_export() {
        // Test that resolution constants are re-exported and accessible
        assert_eq!(IPHONE_15_PRO_MAX_RESOLUTION, "1290x2796");
        assert_eq!(IPHONE_14_PRO_RESOLUTION, "1179x2556");
        assert_eq!(IPAD_PRO_12_9_RESOLUTION, "2048x2732");
        assert_eq!(DEFAULT_IPHONE_RESOLUTION, "1170x2532");
        assert_eq!(DEFAULT_IPAD_RESOLUTION, "1620x2160");
        assert_eq!(RETINA_DISPLAY, "Retina");
        
        // Test that resolution constants are of correct type
        let _: &str = IPHONE_15_PRO_MAX_RESOLUTION;
        let _: &str = IPHONE_14_PRO_RESOLUTION;
        let _: &str = IPAD_PRO_12_9_RESOLUTION;
        let _: &str = DEFAULT_IPHONE_RESOLUTION;
        let _: &str = DEFAULT_IPAD_RESOLUTION;
        let _: &str = RETINA_DISPLAY;
    }

    #[test]
    fn test_timeouts_re_export() {
        // Test that timeout constants are re-exported and accessible
        assert_eq!(INITIAL_RETRY_DELAY, Duration::from_millis(100));
        assert_eq!(MAX_RETRY_DELAY, Duration::from_secs(2));
        assert_eq!(EVENT_PROCESSING_SLEEP, Duration::from_millis(15));
        assert_eq!(CACHE_EXPIRATION_TIME, Duration::from_secs(300));
        assert_eq!(NOTIFICATION_AUTO_DISMISS_TIME, Duration::from_secs(5));
        
        // Test that timeout constants are of correct type
        let _: Duration = INITIAL_RETRY_DELAY;
        let _: Duration = MAX_RETRY_DELAY;
        let _: Duration = EVENT_PROCESSING_SLEEP;
        let _: Duration = CACHE_EXPIRATION_TIME;
        let _: Duration = NOTIFICATION_AUTO_DISMISS_TIME;
    }

    #[test]
    fn test_ui_layout_re_export() {
        // Test that UI layout constants are re-exported and accessible
        assert_eq!(ANDROID_PANEL_PERCENTAGE, 30);
        assert_eq!(IOS_PANEL_PERCENTAGE, 30);
        assert_eq!(DEVICE_DETAILS_PANEL_PERCENTAGE, 40);
        assert_eq!(DIALOG_WIDTH_SMALL, 60);
        assert_eq!(DIALOG_HEIGHT_SMALL, 10);
        assert_eq!(MIN_TERMINAL_WIDTH, 40);
        assert_eq!(MIN_TERMINAL_HEIGHT, 10);
        assert_eq!(HEADER_HEIGHT, 3);
        
        // Test that UI layout constants are of correct type
        let _: u16 = ANDROID_PANEL_PERCENTAGE;
        let _: u16 = IOS_PANEL_PERCENTAGE;
        let _: u16 = DEVICE_DETAILS_PANEL_PERCENTAGE;
        let _: u16 = DIALOG_WIDTH_SMALL;
        let _: u16 = DIALOG_HEIGHT_SMALL;
        let _: u16 = MIN_TERMINAL_WIDTH;
        let _: u16 = MIN_TERMINAL_HEIGHT;
        let _: u16 = HEADER_HEIGHT;
    }
}

#[cfg(test)]
mod module_accessibility_tests {
    use super::*;

    #[test]
    fn test_module_specific_imports() {
        // Test that we can import specific modules directly
        use emu::constants::android;
        use emu::constants::colors;
        use emu::constants::commands;
        use emu::constants::defaults;
        use emu::constants::env_vars;
        use emu::constants::errors;
        use emu::constants::files;
        use emu::constants::ios;
        use emu::constants::keywords;
        use emu::constants::limits;
        use emu::constants::messages;
        use emu::constants::numeric;
        use emu::constants::patterns;
        use emu::constants::performance;
        use emu::constants::priorities;
        use emu::constants::progress;
        use emu::constants::resolutions;
        use emu::constants::timeouts;
        use emu::constants::ui_layout;
        
        // Test that module-specific constants are accessible
        assert_eq!(android::EMULATOR_SERIAL_PREFIX, "emulator-");
        assert_eq!(colors::STATUS_COLOR_SUCCESS, ratatui::style::Color::Green);
        assert_eq!(commands::ADB, "adb");
        assert_eq!(defaults::DEFAULT_RAM_MB, 2048);
        assert_eq!(env_vars::ANDROID_HOME, "ANDROID_HOME");
        assert_eq!(errors::ERR_LIST_ANDROID_DEVICES, "Failed to list Android devices");
        assert_eq!(files::AVD_EXTENSION, ".avd");
        assert_eq!(ios::IOS_DEVICE_STATUS_BOOTED, "Booted");
        assert_eq!(keywords::LOG_LEVEL_ERROR, "Error");
        assert_eq!(limits::MIN_RAM_MB, 512);
        assert_eq!(messages::status::RUNNING, "Running");
        assert_eq!(numeric::BYTES_PER_KB, 1024);
        assert_eq!(patterns::DEVICE_NAME_PATTERN, r"^[a-zA-Z0-9_.-]+$");
        assert_eq!(performance::MAX_EVENTS_PER_FRAME, 50);
        assert_eq!(priorities::ANDROID_PIXEL_PRIORITY, 30);
        assert_eq!(progress::DOWNLOAD_PHASE_START_PERCENTAGE, 20);
        assert_eq!(resolutions::DEFAULT_IPHONE_RESOLUTION, "1170x2532");
        assert_eq!(timeouts::INITIAL_RETRY_DELAY, Duration::from_millis(100));
        assert_eq!(ui_layout::ANDROID_PANEL_PERCENTAGE, 30);
    }

    #[test]
    fn test_nested_module_imports() {
        // Test that nested modules are accessible
        use emu::constants::commands::adb;
        use emu::constants::commands::ios;
        use emu::constants::commands::avdmanager;
        use emu::constants::commands::emulator;
        use emu::constants::files::android;
        use emu::constants::patterns::errors;
        use emu::constants::messages::errors as msg_errors;
        use emu::constants::messages::notifications;
        use emu::constants::messages::ui;
        
        // Test nested module constants
        assert_eq!(adb::DEVICES, "devices");
        assert_eq!(ios::LIST, "list");
        assert_eq!(avdmanager::LIST, "list");
        assert_eq!(emulator::AVD_ARG, "-avd");
        assert_eq!(android::CMDLINE_TOOLS_LATEST_BIN, "cmdline-tools/latest/bin");
        assert_eq!(errors::ERROR_PREFIX, "Error:");
        assert_eq!(msg_errors::ANDROID_SDK_NOT_FOUND, "Android SDK not found. Please set ANDROID_HOME or ANDROID_SDK_ROOT");
        assert_eq!(notifications::DEVICE_STARTING, "Starting device '{}'...");
        assert_eq!(ui::ANDROID_DEVICES_TITLE, "🤖 Android Devices");
    }
}

#[cfg(test)]
mod re_export_completeness_tests {
    use super::*;

    #[test]
    fn test_all_re_exports_accessible() {
        // Test that all re-exported constants are accessible without module prefix
        // This is a sampling test - we test a few from each module to ensure re-exports work
        
        // From colors
        let _: ratatui::style::Color = DARK_THEME_BG_PRIMARY;
        let _: ratatui::style::Color = STATUS_COLOR_SUCCESS;
        
        // From commands
        let _: &str = ADB;
        let _: &str = EMULATOR;
        
        // From defaults
        let _: u32 = DEFAULT_RAM_MB;
        let _: &str = DEFAULT_GPU_MODE;
        
        // From env_vars
        let _: &str = ANDROID_HOME;
        let _: &str = RUST_LOG;
        
        // From files
        let _: &str = AVD_EXTENSION;
        let _: &str = CONFIG_FILE;
        
        // From ios_devices
        let _: &str = DISPLAY_SIZE_13_INCH;
        let _: usize = IOS_DEVICE_BATCH_SIZE;
        
        // From keywords
        let _: &str = LOG_LEVEL_ERROR;
        let _: &str = DEVICE_KEYWORD_PIXEL;
        
        // From limits
        let _: u32 = MIN_RAM_MB;
        let _: usize = MAX_DEVICE_NAME_LENGTH;
        
        // From numeric
        let _: u64 = BYTES_PER_KB;
        let _: f32 = VERSION_MAJOR_DIVISOR;
        
        // From performance
        let _: usize = MAX_EVENTS_PER_FRAME;
        let _: Duration = UI_UPDATE_INTERVAL;
        
        // From priorities
        let _: u8 = ANDROID_PIXEL_PRIORITY;
        let _: u8 = IOS_IPHONE_PRO_MAX_PRIORITY;
        
        // From progress
        let _: u8 = DOWNLOAD_PHASE_START_PERCENTAGE;
        let _: u8 = LOADING_PHASE_INCREMENT;
        
        // From resolutions
        let _: &str = IPHONE_15_PRO_MAX_RESOLUTION;
        let _: &str = DEFAULT_IPHONE_RESOLUTION;
        
        // From timeouts
        let _: Duration = INITIAL_RETRY_DELAY;
        let _: Duration = CACHE_EXPIRATION_TIME;
        
        // From ui_layout
        let _: u16 = ANDROID_PANEL_PERCENTAGE;
        let _: u16 = DIALOG_WIDTH_SMALL;
    }
}

#[cfg(test)]
mod constants_module_documentation_tests {
    use super::*;

    #[test]
    fn test_module_organization() {
        // Test that the module organization matches the documentation
        // This ensures the module structure is maintained correctly
        
        // Test that we can access both through module path and re-export
        use emu::constants;
        
        // Test module-specific access
        assert_eq!(constants::android::EMULATOR_SERIAL_PREFIX, "emulator-");
        assert_eq!(constants::defaults::DEFAULT_RAM_MB, 2048);
        assert_eq!(constants::limits::MIN_RAM_MB, 512);
        
        // Test re-exported access
        assert_eq!(constants::DEFAULT_RAM_MB, 2048);
        assert_eq!(constants::MIN_RAM_MB, 512);
        
        // Test that both access methods give the same result
        assert_eq!(constants::defaults::DEFAULT_RAM_MB, constants::DEFAULT_RAM_MB);
        assert_eq!(constants::limits::MIN_RAM_MB, constants::MIN_RAM_MB);
    }
}