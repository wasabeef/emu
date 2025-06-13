//! Tests for Android API Level installation functionality.

use emu::app::state::{ApiLevelInstallDialog, AppState, Mode};
use emu::models::{ApiLevel, SdkInstallStatus};

#[test]
fn test_api_level_install_dialog_creation() {
    let dialog = ApiLevelInstallDialog::new();

    assert!(dialog.available_api_levels.is_empty());
    assert_eq!(dialog.selected_index, 0);
    assert!(dialog.is_loading);
    assert!(dialog.error_message.is_none());
    assert!(matches!(dialog.install_status, SdkInstallStatus::Pending));

    println!("✅ API Level install dialog created successfully");
}

#[test]
fn test_api_level_dialog_navigation() {
    let mut dialog = ApiLevelInstallDialog::new();

    // Add some test API levels
    let api_levels = vec![
        ApiLevel::new(34, "Android 14".to_string(), false),
        ApiLevel::new(33, "Android 13".to_string(), true),
        ApiLevel::new(32, "Android 12L".to_string(), false),
    ];

    dialog.update_api_levels(api_levels);

    assert_eq!(dialog.selected_index, 0);
    assert_eq!(dialog.available_api_levels.len(), 3);
    assert!(!dialog.is_loading);

    // Test navigation down (disable debouncing for tests)
    dialog.move_selection_down_with_debounce(false);
    assert_eq!(dialog.selected_index, 1);

    dialog.move_selection_down_with_debounce(false);
    assert_eq!(dialog.selected_index, 2);

    // Test circular navigation (wrap to beginning)
    dialog.move_selection_down_with_debounce(false);
    assert_eq!(dialog.selected_index, 0);

    // Test navigation up
    dialog.move_selection_up_with_debounce(false);
    assert_eq!(dialog.selected_index, 2); // Wrap to end

    dialog.move_selection_up_with_debounce(false);
    assert_eq!(dialog.selected_index, 1);

    // Test selected API level
    let selected = dialog.selected_api_level().unwrap();
    assert_eq!(selected.level, 33);
    assert_eq!(selected.version_name, "Android 13");
    assert!(selected.installed);

    println!("✅ API Level dialog navigation works correctly");
}

#[test]
fn test_api_level_status_updates() {
    let mut dialog = ApiLevelInstallDialog::new();

    // Test status update
    dialog.update_install_status(SdkInstallStatus::Installing {
        progress: 50,
        message: "Downloading API 34...".to_string(),
    });

    assert!(dialog.install_status.is_in_progress());
    assert_eq!(dialog.install_status.progress(), Some(50));
    assert_eq!(
        dialog.install_status.message(),
        Some("Downloading API 34...")
    );

    // Test completion
    dialog.update_install_status(SdkInstallStatus::Completed);
    assert!(dialog.install_status.is_completed());
    assert!(!dialog.install_status.is_in_progress());

    // Test failure
    dialog.update_install_status(SdkInstallStatus::Failed {
        error: "Network timeout".to_string(),
    });
    assert!(dialog.install_status.is_failed());
    assert_eq!(dialog.install_status.message(), Some("Network timeout"));

    println!("✅ API Level status updates work correctly");
}

#[test]
fn test_api_level_error_handling() {
    let mut dialog = ApiLevelInstallDialog::new();

    assert!(dialog.is_loading);
    assert!(dialog.error_message.is_none());

    // Set error
    dialog.set_error("Failed to load API levels".to_string());

    assert!(!dialog.is_loading);
    assert_eq!(
        dialog.error_message,
        Some("Failed to load API levels".to_string())
    );

    println!("✅ API Level error handling works correctly");
}

#[test]
fn test_app_state_api_level_integration() {
    let mut app_state = AppState::new();

    // Initially no API level dialog
    assert!(app_state.api_level_install.is_none());
    assert_eq!(app_state.mode, Mode::Normal);

    // Create and set API level dialog
    let dialog = ApiLevelInstallDialog::new();
    app_state.api_level_install = Some(dialog);
    app_state.mode = Mode::ApiLevelInstall;

    assert!(app_state.api_level_install.is_some());
    assert_eq!(app_state.mode, Mode::ApiLevelInstall);

    // Test dialog access
    let dialog = app_state.api_level_install.as_mut().unwrap();
    assert!(dialog.is_loading);

    println!("✅ App state API level integration works correctly");
}

#[test]
fn test_api_level_display_format() {
    let api_level = ApiLevel::new(34, "Android 14".to_string(), false);

    assert_eq!(api_level.display_name(), "API 34 - Android 14");
    assert_eq!(api_level.package_name, "platforms;android-34");
    assert!(api_level.is_modern());
    assert!(!api_level.installed);

    let old_api = ApiLevel::new(19, "Android 4.4".to_string(), true);
    assert!(!old_api.is_modern());
    assert!(old_api.installed);

    println!("✅ API Level display format works correctly");
}

#[tokio::test]
async fn test_api_level_dialog_lifecycle() {
    let mut dialog = ApiLevelInstallDialog::new();

    // 1. Initial loading state
    assert!(dialog.is_loading);
    assert!(dialog.available_api_levels.is_empty());

    // 2. Load API levels
    let api_levels = vec![
        ApiLevel::new(35, "Android 15".to_string(), false),
        ApiLevel::new(34, "Android 14".to_string(), true),
    ];
    dialog.update_api_levels(api_levels);

    assert!(!dialog.is_loading);
    assert_eq!(dialog.available_api_levels.len(), 2);

    // 3. Select API level to install
    dialog.move_selection_down_with_debounce(false); // Select API 34
    let selected = dialog.selected_api_level().unwrap();
    assert_eq!(selected.level, 34);

    // 4. Start installation
    dialog.update_install_status(SdkInstallStatus::Installing {
        progress: 0,
        message: "Starting installation...".to_string(),
    });
    assert!(dialog.install_status.is_in_progress());

    // 5. Update progress
    dialog.update_install_status(SdkInstallStatus::Installing {
        progress: 75,
        message: "Extracting files...".to_string(),
    });
    assert_eq!(dialog.install_status.progress(), Some(75));

    // 6. Complete installation
    dialog.update_install_status(SdkInstallStatus::Completed);
    assert!(dialog.install_status.is_completed());

    println!("✅ API Level dialog lifecycle works correctly");
}
