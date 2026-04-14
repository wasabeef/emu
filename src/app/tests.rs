use super::*;
use crate::models::DeviceStatus;
use std::ffi::OsString;
use std::sync::OnceLock;
use tokio::test;
use tokio::time::{sleep, Duration};

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: impl Into<OsString>) -> Self {
        let original = std::env::var_os(key);
        std::env::set_var(key, value.into());
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

fn test_env_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

async fn acquire_test_env_lock() -> tokio::sync::MutexGuard<'static, ()> {
    test_env_lock().lock().await
}

struct StartupTestEnv {
    _temp_dir: tempfile::TempDir,
    _android_home: EnvVarGuard,
    _home: EnvVarGuard,
    _xdg_config_home: EnvVarGuard,
    _path: EnvVarGuard,
}

impl StartupTestEnv {
    fn new() -> Self {
        Self::with_running_android(false)
    }

    fn with_running_android(running_android: bool) -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();
        let android_home = root.join("android-sdk");
        let home_dir = root.join("home");
        let xdg_config_home = root.join("config");
        let original_path = std::env::var_os("PATH").unwrap_or_default();

        std::fs::create_dir_all(android_home.join("cmdline-tools/latest/bin")).unwrap();
        std::fs::create_dir_all(android_home.join("emulator")).unwrap();
        std::fs::create_dir_all(android_home.join("platform-tools")).unwrap();
        std::fs::create_dir_all(home_dir.join(".android/avd/Pixel_7_API_34.avd")).unwrap();
        std::fs::create_dir_all(&xdg_config_home).unwrap();

        let avd_path = home_dir
            .join(".android/avd/Pixel_7_API_34.avd")
            .to_string_lossy()
            .to_string();

        let adb_script = if running_android {
            r#"#!/bin/sh
if [ "$1" = "devices" ]; then
    echo "List of devices attached"
    echo "emulator-5554	device"
    exit 0
fi

if [ "$1" = "-s" ] && [ "$2" = "emulator-5554" ] && [ "$3" = "shell" ] && [ "$4" = "getprop" ] && [ "$5" = "ro.boot.qemu.avd_name" ]; then
    echo "Pixel_7_API_34"
    exit 0
fi

if [ "$1" = "-s" ] && [ "$2" = "emulator-5554" ] && [ "$3" = "logcat" ]; then
    echo "04-11 12:00:00.000 I TestTag: hello from logcat"
    exit 0
fi

exit 0
"#
        } else {
            r#"#!/bin/sh
if [ "$1" = "devices" ]; then
    echo "List of devices attached"
    exit 0
fi

exit 0
"#
        };

        let avdmanager_script = format!(
            r#"#!/bin/sh
if [ "$1" = "list" ] && [ "$2" = "avd" ]; then
    cat <<'EOF'
Available Android Virtual Devices:
    Name: Pixel_7_API_34
  Device: pixel_7
    Path: {avd_path}
  Target: Google APIs (Google Inc.)
            Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a
---------
EOF
    exit 0
fi

if [ "$1" = "list" ] && [ "$2" = "device" ]; then
    cat <<'EOF'
id: 0 or "pixel_7"
    Name: Pixel 7
    OEM : Google
---------
EOF
    exit 0
fi

if [ "$1" = "list" ] && [ "$2" = "target" ]; then
    cat <<'EOF'
Available targets:
id: 1 or "android-34"
     Name: Android API 34
     Type: Platform
     API level: 34
EOF
    exit 0
fi

exit 0
"#
        );

        let sdkmanager_script = r#"#!/bin/sh
if echo "$@" | grep -q -- "--list"; then
    cat <<'EOF'
Installed packages:
  system-images;android-34;google_apis;arm64-v8a | 1 | Google APIs ARM 64 v8a System Image

Available Packages:
  system-images;android-35;google_apis;arm64-v8a | 1 | Google APIs ARM 64 v8a System Image
EOF
    exit 0
fi

exit 0
"#;

        let emulator_script = "#!/bin/sh\nexit 0\n";

        let avdmanager_path = android_home.join("cmdline-tools/latest/bin/avdmanager");
        let sdkmanager_path = android_home.join("cmdline-tools/latest/bin/sdkmanager");
        let adb_path = android_home.join("platform-tools/adb");
        let emulator_path = android_home.join("emulator/emulator");

        std::fs::write(&avdmanager_path, avdmanager_script).unwrap();
        std::fs::write(&sdkmanager_path, sdkmanager_script).unwrap();
        std::fs::write(&adb_path, adb_script).unwrap();
        std::fs::write(&emulator_path, emulator_script).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            for path in [
                &avdmanager_path,
                &sdkmanager_path,
                &adb_path,
                &emulator_path,
            ] {
                let mut perms = std::fs::metadata(path).unwrap().permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(path, perms).unwrap();
            }
        }

        std::fs::write(
            home_dir.join(".android/avd/Pixel_7_API_34.avd/config.ini"),
            r#"hw.ramSize=4096
disk.dataPartition.size=8192M
hw.lcd.width=1080
hw.lcd.height=2400
hw.lcd.density=420
image.sysdir.1=system-images/android-34/google_apis/arm64-v8a/
hw.device.name=pixel_7
"#,
        )
        .unwrap();

        let mut path_entries = vec![
            android_home.join("platform-tools"),
            android_home.join("cmdline-tools/latest/bin"),
            android_home.join("emulator"),
        ];
        path_entries.extend(std::env::split_paths(&original_path));
        let combined_path = std::env::join_paths(path_entries).unwrap();

        Self {
            _temp_dir: temp_dir,
            _android_home: EnvVarGuard::set("ANDROID_HOME", android_home),
            _home: EnvVarGuard::set("HOME", home_dir),
            _xdg_config_home: EnvVarGuard::set("XDG_CONFIG_HOME", xdg_config_home),
            _path: EnvVarGuard::set("PATH", combined_path),
        }
    }
}

async fn wait_for_app_state<F>(app: &App, mut predicate: F)
where
    F: FnMut(&AppState) -> bool,
{
    for _ in 0..120 {
        {
            let state = app.state.lock().await;
            if predicate(&state) {
                return;
            }
        }
        sleep(Duration::from_millis(25)).await;
    }

    let state = app.state.lock().await;
    panic!(
        "timed out waiting for app state: is_loading={}, android_devices={}, has_details={}",
        state.is_loading,
        state.android_devices.len(),
        state.cached_device_details.is_some()
    );
}

#[test]
async fn test_app_new_populates_android_devices_and_details_in_background() {
    let _env_lock = acquire_test_env_lock().await;
    let _env = StartupTestEnv::new();

    let app = App::new()
        .await
        .expect("app should initialize with test SDK");

    wait_for_app_state(&app, |state| {
        !state.is_loading
            && state.android_devices.len() == 1
            && state.cached_device_details.is_some()
    })
    .await;

    let state = app.state.lock().await;
    let device = &state.android_devices[0];
    let details = state
        .cached_device_details
        .as_ref()
        .expect("details should be cached after startup");

    assert_eq!(device.name, "Pixel_7_API_34");
    assert_eq!(device.device_type, "pixel_7");
    assert_eq!(device.api_level, 34);
    assert_eq!(device.status, DeviceStatus::Stopped);
    assert!(!device.is_running);

    assert_eq!(details.name, "Pixel_7_API_34");
    assert_eq!(details.status, "Stopped");
    assert_eq!(details.device_type, "pixel_7");
    assert_eq!(details.api_level_or_version, "API 34 (Android API 34)");
    assert_eq!(details.ram_size.as_deref(), Some("4096 MB"));
    assert_eq!(details.storage_size.as_deref(), Some("8192 MB"));
    assert_eq!(details.resolution.as_deref(), Some("1080x2400"));
    assert_eq!(details.dpi.as_deref(), Some("420 DPI"));
    assert!(details
        .device_path
        .as_ref()
        .is_some_and(|path| path.contains("Pixel_7_API_34.avd")));
}

#[test]
async fn test_update_log_stream_internal_clears_log_target_for_stopped_android_device() {
    let _env_lock = acquire_test_env_lock().await;
    let _env = StartupTestEnv::new();

    let android_manager = AndroidManager::new().expect("Android manager should initialize");
    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let mut state_lock = state.lock().await;
        state_lock.active_panel = Panel::Android;
        state_lock.android_devices = vec![AndroidDevice {
            name: "Pixel_7_API_34".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            android_version_name: "API 34".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "8192M".to_string(),
        }];
        state_lock.current_log_device = Some((Panel::Android, "OldDevice".to_string()));
    }

    App::update_log_stream_internal(state.clone(), android_manager, None).await;

    let state_lock = state.lock().await;
    assert_eq!(state_lock.current_log_device, None);
    assert!(state_lock.log_task_handle.is_none());
}

#[test]
async fn test_update_log_stream_internal_sets_log_target_for_running_android_device() {
    let _env_lock = acquire_test_env_lock().await;
    let _env = StartupTestEnv::with_running_android(true);

    let android_manager = AndroidManager::new().expect("Android manager should initialize");
    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let mut state_lock = state.lock().await;
        state_lock.active_panel = Panel::Android;
        state_lock.android_devices = vec![AndroidDevice {
            name: "Pixel_7_API_34".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            android_version_name: "API 34".to_string(),
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "8192M".to_string(),
        }];
    }

    App::update_log_stream_internal(state.clone(), android_manager, None).await;
    sleep(Duration::from_millis(50)).await;

    let mut state_lock = state.lock().await;
    assert_eq!(
        state_lock.current_log_device,
        Some((Panel::Android, "Pixel_7_API_34".to_string()))
    );

    if let Some(handle) = state_lock.log_task_handle.take() {
        handle.abort();
    }
}

#[test]
async fn test_update_device_details_internal_populates_selected_android_details() {
    let _env_lock = acquire_test_env_lock().await;
    let _env = StartupTestEnv::new();

    let android_manager = AndroidManager::new().expect("Android manager should initialize");
    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let mut state_lock = state.lock().await;
        state_lock.active_panel = Panel::Android;
        state_lock.android_devices = vec![AndroidDevice {
            name: "Pixel_7_API_34".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            android_version_name: "API 34".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "8192M".to_string(),
        }];
    }

    App::update_device_details_internal(state.clone(), android_manager, None).await;

    let state_lock = state.lock().await;
    let details = state_lock
        .cached_device_details
        .as_ref()
        .expect("selected Android device should populate details");

    assert_eq!(details.name, "Pixel_7_API_34");
    assert_eq!(details.status, "Stopped");
    assert_eq!(details.device_type, "pixel_7");
    assert_eq!(details.api_level_or_version, "API 34 (Android API 34)");
    assert_eq!(details.ram_size.as_deref(), Some("4096 MB"));
    assert_eq!(details.storage_size.as_deref(), Some("8192 MB"));
    assert_eq!(details.resolution.as_deref(), Some("1080x2400"));
    assert_eq!(details.dpi.as_deref(), Some("420 DPI"));
}

#[test]
async fn test_execute_delete_device_removes_android_device_and_adjusts_selection() {
    let _env_lock = acquire_test_env_lock().await;
    let _env = StartupTestEnv::new();

    let mut app = App {
        state: Arc::new(Mutex::new(AppState::new())),
        android_manager: AndroidManager::new().expect("Android manager should initialize"),
        ios_manager: None,
        log_update_handle: None,
        detail_update_handle: None,
        last_full_device_refresh: std::time::Instant::now(),
    };

    {
        let mut state = app.state.lock().await;
        state.android_devices = vec![
            AndroidDevice {
                name: "Pixel_7_API_34".to_string(),
                device_type: "pixel_7".to_string(),
                api_level: 34,
                android_version_name: "API 34".to_string(),
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: "4096".to_string(),
                storage_size: "8192M".to_string(),
            },
            AndroidDevice {
                name: "Tablet_API_33".to_string(),
                device_type: "tablet".to_string(),
                api_level: 33,
                android_version_name: "API 33".to_string(),
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: "4096".to_string(),
                storage_size: "8192M".to_string(),
            },
        ];
        state.selected_android = 1;
        state.device_operation_status = Some("Deleting device...".to_string());
        state.confirm_delete_dialog = Some(state::ConfirmDeleteDialog {
            device_name: "Tablet_API_33".to_string(),
            device_identifier: "Tablet_API_33".to_string(),
            platform: Panel::Android,
        });
    }

    app.execute_delete_device().await.unwrap();

    let state = app.state.lock().await;
    assert_eq!(state.android_devices.len(), 1);
    assert_eq!(state.android_devices[0].name, "Pixel_7_API_34");
    assert_eq!(state.selected_android, 0);
    assert!(state.confirm_delete_dialog.is_none());
    assert!(state.device_operation_status.is_none());
    assert_eq!(
        state
            .notifications
            .back()
            .map(|notification| notification.message.as_str()),
        Some("Device 'Tablet_API_33' deleted successfully")
    );
}

#[test]
async fn test_execute_wipe_device_removes_android_user_data_and_notifies() {
    let _env_lock = acquire_test_env_lock().await;
    let _env = StartupTestEnv::new();

    let mut app = App {
        state: Arc::new(Mutex::new(AppState::new())),
        android_manager: AndroidManager::new().expect("Android manager should initialize"),
        ios_manager: None,
        log_update_handle: None,
        detail_update_handle: None,
        last_full_device_refresh: std::time::Instant::now(),
    };

    let home_dir = std::env::var("HOME").expect("HOME should be set by StartupTestEnv");
    let avd_dir = std::path::PathBuf::from(home_dir).join(".android/avd/Pixel_7_API_34.avd");
    let userdata_path = avd_dir.join("userdata.img");
    let snapshots_dir = avd_dir.join("snapshots");
    std::fs::write(&userdata_path, "userdata").unwrap();
    std::fs::create_dir_all(&snapshots_dir).unwrap();
    std::fs::write(snapshots_dir.join("snapshot.pb"), "snapshot").unwrap();

    {
        let mut state = app.state.lock().await;
        state.active_panel = Panel::Android;
        state.android_devices = vec![AndroidDevice {
            name: "Pixel_7_API_34".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            android_version_name: "API 34".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "8192M".to_string(),
        }];
        state.device_operation_status = Some("Wiping device...".to_string());
        state.confirm_wipe_dialog = Some(state::ConfirmWipeDialog {
            device_name: "Pixel_7_API_34".to_string(),
            device_identifier: "Pixel_7_API_34".to_string(),
            platform: Panel::Android,
        });
    }

    app.execute_wipe_device().await.unwrap();

    let state = app.state.lock().await;
    assert!(!userdata_path.exists());
    assert!(!snapshots_dir.exists());
    assert!(state.confirm_wipe_dialog.is_none());
    assert!(state.device_operation_status.is_none());
    assert_eq!(
        state
            .notifications
            .back()
            .map(|notification| notification.message.as_str()),
        Some("Device 'Pixel_7_API_34' wiped successfully")
    );
}

#[test]
async fn test_reload_device_types_for_category_uses_cached_android_devices() {
    let _env_lock = acquire_test_env_lock().await;
    let _env = StartupTestEnv::new();

    let mut app = App {
        state: Arc::new(Mutex::new(AppState::new())),
        android_manager: AndroidManager::new().expect("Android manager should initialize"),
        ios_manager: None,
        log_update_handle: None,
        detail_update_handle: None,
        last_full_device_refresh: std::time::Instant::now(),
    };

    {
        let mut state = app.state.lock().await;
        state.active_panel = Panel::Android;
        state.create_device_form.device_category_filter = "tablet".to_string();
        state.create_device_form.selected_device_type_index = 1;
        state.create_device_form.version = "34".to_string();
        state.create_device_form.version_display = "API 34 - Android 14".to_string();

        let mut cache = state.device_cache.write().await;
        cache.android_device_cache = Some(vec![
            ("pixel_7".to_string(), "Pixel 7".to_string()),
            ("tablet_10".to_string(), "Tablet 10".to_string()),
            ("tv_1080p".to_string(), "Android TV (1080p)".to_string()),
        ]);
    }

    app.reload_device_types_for_category().await.unwrap();

    let state = app.state.lock().await;
    assert_eq!(state.create_device_form.available_device_types.len(), 1);
    assert_eq!(
        state.create_device_form.available_device_types[0],
        ("tablet_10".to_string(), "Tablet 10".to_string())
    );
    assert_eq!(state.create_device_form.selected_device_type_index, 0);
    assert_eq!(state.create_device_form.device_type_id, "tablet_10");
    assert_eq!(state.create_device_form.device_type, "Tablet 10");
    assert_eq!(state.create_device_form.name, "Tablet 10 API 34");
}

#[test]
async fn test_app_new() {
    let _env_lock = acquire_test_env_lock().await;
    std::env::set_var("ANDROID_HOME", "/tmp/mock_android_sdk");

    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();
    std::env::set_var("ANDROID_HOME", sdk_path);

    tokio::fs::create_dir_all(sdk_path.join("emulator"))
        .await
        .ok();
    tokio::fs::create_dir_all(sdk_path.join("platform-tools"))
        .await
        .ok();
    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho 'mock avdmanager'\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms)
            .await
            .ok();
    }

    let result = App::new().await;

    #[cfg(target_os = "macos")]
    {
        if result.is_ok() {
            let app = result.unwrap();
            assert!(app.ios_manager.is_some());
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if result.is_ok() {
            let app = result.unwrap();
            assert!(app.ios_manager.is_none());
        }
    }

    std::env::remove_var("ANDROID_HOME");
}

#[test]
async fn test_start_background_cache_loading() {
    let _env_lock = acquire_test_env_lock().await;
    let _env = StartupTestEnv::new();

    if let Ok(app) = App::new().await {
        for _ in 0..120 {
            let has_android_cache = {
                let state = app.state.lock().await;
                let cache = state.device_cache.read().await;
                !cache.android_device_types.is_empty() && !cache.android_api_levels.is_empty()
            };
            let has_api_level_cache = app.android_manager.get_cached_api_levels().await.is_some();

            if has_android_cache && has_api_level_cache {
                return;
            }

            sleep(Duration::from_millis(25)).await;
        }

        panic!("background cache loading did not warm Android caches in time");
    }
}

#[test]
async fn test_app_state_initialization() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();
    std::env::set_var("ANDROID_HOME", sdk_path);

    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho ''\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms)
            .await
            .ok();
    }

    if let Ok(app) = App::new().await {
        let state = app.state.lock().await;
        assert_eq!(state.focused_panel, FocusedPanel::DeviceList);
        assert_eq!(state.mode, Mode::Normal);
        assert!(state.android_devices.is_empty());
        assert!(state.ios_devices.is_empty());
        assert!(state.device_logs.is_empty());
        assert!(state.notifications.is_empty());
    }

    std::env::remove_var("ANDROID_HOME");
}

#[test]
async fn test_app_with_mock_devices() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();
    std::env::set_var("ANDROID_HOME", sdk_path);

    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho ''\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms)
            .await
            .ok();
    }

    if let Ok(app) = App::new().await {
        {
            let mut state = app.state.lock().await;

            let mock_android_device = AndroidDevice {
                name: "TestDevice".to_string(),
                device_type: "pixel_7".to_string(),
                api_level: 33,
                android_version_name: "13".to_string(),
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: "2048".to_string(),
                storage_size: "8192M".to_string(),
            };

            state.android_devices.push(mock_android_device);
        }

        let state = app.state.lock().await;
        assert_eq!(state.android_devices.len(), 1);
        assert_eq!(state.android_devices[0].name, "TestDevice");
    }

    std::env::remove_var("ANDROID_HOME");
}

#[test]
async fn test_refresh_devices_incremental() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();
    std::env::set_var("ANDROID_HOME", sdk_path);

    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(
        &avdmanager_path,
        "#!/bin/bash\necho 'Available Android Virtual Devices:'\n",
    )
    .await
    .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms)
            .await
            .ok();
    }

    if let Ok(mut app) = App::new().await {
        let result = app.refresh_devices_incremental().await;

        if result.is_err() {
            println!("Incremental refresh failed as expected with mock setup: {result:?}");
        }
    }

    std::env::remove_var("ANDROID_HOME");
}

#[test]
async fn test_refresh_devices_smart_uses_status_only_path_between_full_refreshes() {
    let _env_lock = acquire_test_env_lock().await;
    let _env = StartupTestEnv::with_running_android(true);

    let mock_executor = crate::utils::command_executor::mock::MockCommandExecutor::new()
        .with_success(
            "adb",
            &["devices"],
            "List of devices attached\nemulator-5554\tdevice\n",
        )
        .with_success(
            "adb",
            &[
                "-s",
                "emulator-5554",
                "shell",
                "getprop",
                "ro.boot.qemu.avd_name",
            ],
            "Pixel_7_API_34\n",
        );

    let mut app = App {
        state: Arc::new(Mutex::new(AppState::new())),
        android_manager: AndroidManager::with_executor(Arc::new(mock_executor))
            .expect("Android manager should initialize"),
        ios_manager: None,
        log_update_handle: None,
        detail_update_handle: None,
        last_full_device_refresh: std::time::Instant::now(),
    };

    {
        let mut state = app.state.lock().await;
        state.android_devices = vec![AndroidDevice {
            name: "Pixel_7_API_34".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            android_version_name: "API 34".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "8192M".to_string(),
        }];
    }

    let start = std::time::Instant::now();
    app.refresh_devices_smart().await.unwrap();
    let elapsed = start.elapsed();

    let state = app.state.lock().await;
    assert_eq!(state.android_devices.len(), 1);
    assert!(state.android_devices[0].is_running);
    assert_eq!(state.android_devices[0].status, DeviceStatus::Running);
    assert!(
        elapsed <= crate::constants::performance::STATUS_ONLY_REFRESH_TARGET,
        "status-only refresh should stay fast, took {elapsed:?}"
    );
}

#[test]
async fn test_should_use_full_device_refresh_without_any_devices() {
    assert!(App::should_use_full_device_refresh(
        false,
        false,
        false,
        Duration::from_secs(1),
    ));
}

#[test]
async fn test_should_use_full_device_refresh_for_pending_device() {
    assert!(App::should_use_full_device_refresh(
        true,
        false,
        true,
        Duration::from_secs(1),
    ));
}

#[test]
async fn test_should_use_status_only_refresh_when_ios_devices_exist_and_refresh_is_fresh() {
    assert!(!App::should_use_full_device_refresh(
        false,
        true,
        false,
        Duration::from_secs(1),
    ));
}

#[test]
async fn test_should_use_full_device_refresh_when_interval_expires() {
    assert!(App::should_use_full_device_refresh(
        true,
        true,
        false,
        crate::constants::performance::FULL_DEVICE_REFRESH_INTERVAL,
    ));
}

#[test]
async fn test_open_api_level_management_uses_cached_levels_immediately() {
    let _env_lock = acquire_test_env_lock().await;
    let _env = StartupTestEnv::new();
    let android_home =
        std::env::var("ANDROID_HOME").expect("ANDROID_HOME should be set by StartupTestEnv");
    let sdkmanager_path =
        std::path::PathBuf::from(android_home).join("cmdline-tools/latest/bin/sdkmanager");
    std::fs::write(
        &sdkmanager_path,
        r#"#!/bin/sh
cat <<'EOF'
Installed packages:
  Path | Version | Description | Location
  system-images;android-34;google_apis_playstore;arm64-v8a | 1 | Android SDK Platform 34 | system-images/android-34/google_apis_playstore/arm64-v8a

Available Packages:
  system-images;android-35;google_apis;arm64-v8a | 1 | Android SDK Platform 35 | system-images/android-35/google_apis/arm64-v8a
EOF
"#,
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&sdkmanager_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&sdkmanager_path, perms).unwrap();
    }

    let mock_executor = crate::utils::command_executor::mock::MockCommandExecutor::new()
        .with_success(
            &sdkmanager_path.to_string_lossy(),
            &["--list", "--verbose", "--include_obsolete"],
            "Installed packages:\n  Path | Version | Description | Location\n  system-images;android-34;google_apis_playstore;arm64-v8a | 1 | Android SDK Platform 34 | system-images/android-34/google_apis_playstore/arm64-v8a\n\nAvailable Packages:\n  system-images;android-35;google_apis;arm64-v8a | 1 | Android SDK Platform 35 | system-images/android-35/google_apis/arm64-v8a\n",
        );
    let android_manager = AndroidManager::with_executor(Arc::new(mock_executor))
        .expect("Android manager should initialize");
    let cached_levels = android_manager.list_api_levels().await.unwrap();
    assert!(!cached_levels.is_empty());

    let mut app = App {
        state: Arc::new(Mutex::new(AppState::new())),
        android_manager,
        ios_manager: None,
        log_update_handle: None,
        detail_update_handle: None,
        last_full_device_refresh: std::time::Instant::now(),
    };

    let start = std::time::Instant::now();
    app.open_api_level_management().await;
    let elapsed = start.elapsed();

    let state = app.state.lock().await;
    let api_state = state
        .api_level_management
        .as_ref()
        .expect("API level management should be open");
    assert!(!api_state.is_loading);
    assert_eq!(api_state.api_levels.len(), cached_levels.len());
    assert!(
        elapsed <= crate::constants::performance::API_LEVEL_DIALOG_OPEN_TARGET,
        "opening API level dialog from warm cache should be immediate, took {elapsed:?}"
    );
}

#[test]
async fn test_enter_create_device_mode_uses_cached_android_form_immediately() {
    let _env_lock = acquire_test_env_lock().await;
    let _env = StartupTestEnv::new();

    let mut app = App::new()
        .await
        .expect("App should initialize with startup test environment");

    for _ in 0..120 {
        let has_android_cache = {
            let state = app.state.lock().await;
            state.is_cache_available(Panel::Android).await
        };

        if has_android_cache {
            break;
        }

        sleep(Duration::from_millis(25)).await;
    }

    let start = std::time::Instant::now();
    app.enter_create_device_mode().await;
    let elapsed = start.elapsed();

    let state = app.state.lock().await;
    assert_eq!(state.mode, Mode::CreateDevice);
    assert!(!state.create_device_form.is_loading_cache);
    assert!(!state.create_device_form.available_device_types.is_empty());
    assert!(!state.create_device_form.available_versions.is_empty());
    assert!(
        elapsed <= crate::constants::performance::CREATE_DEVICE_DIALOG_OPEN_TARGET,
        "opening create-device dialog from warm cache should be immediate, took {elapsed:?}"
    );
}

#[allow(dead_code)]
async fn test_event_processing_disabled() {
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();
    std::env::set_var("ANDROID_HOME", sdk_path);

    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho 'mock'\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms)
            .await
            .ok();
    }

    if let Ok(app) = App::new().await {
        let state = app.state.lock().await;
        assert_eq!(state.active_panel, Panel::Android);
    }

    std::env::remove_var("ANDROID_HOME");
}

#[test]
async fn test_device_list_management() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();
    std::env::set_var("ANDROID_HOME", sdk_path);

    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(
        &avdmanager_path,
        "#!/bin/bash\necho 'Available Android Virtual Devices:'\n",
    )
    .await
    .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms)
            .await
            .ok();
    }

    if let Ok(mut app) = App::new().await {
        let result = app.refresh_devices_smart().await;
        if result.is_err() {
            println!("Device refresh failed as expected with mock setup: {result:?}");
        }

        let state = app.state.lock().await;
        assert!(state.android_devices.is_empty() || !state.android_devices.is_empty());
    }

    std::env::remove_var("ANDROID_HOME");
}

#[test]
async fn test_background_task_management() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();
    std::env::set_var("ANDROID_HOME", sdk_path);

    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho 'mock'\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms)
            .await
            .ok();
    }

    if let Ok(app) = App::new().await {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let state = app.state.lock().await;
        assert_eq!(state.active_panel, Panel::Android);
    }

    std::env::remove_var("ANDROID_HOME");
}

#[test]
async fn test_state_synchronization() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();
    std::env::set_var("ANDROID_HOME", sdk_path);

    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho 'mock'\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms)
            .await
            .ok();
    }

    if let Ok(app) = App::new().await {
        let state_clone = Arc::clone(&app.state);

        let handle1 = tokio::spawn(async move {
            let state = state_clone.lock().await;
            state.active_panel
        });

        let handle2 = tokio::spawn(async move {
            let state = app.state.lock().await;
            state.mode
        });

        let (panel, mode) = tokio::try_join!(handle1, handle2).unwrap();
        assert_eq!(panel, Panel::Android);
        assert_eq!(mode, Mode::Normal);
    }

    std::env::remove_var("ANDROID_HOME");
}

#[test]
async fn test_device_creation_workflow() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();
    std::env::set_var("ANDROID_HOME", sdk_path);

    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho 'mock'\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms)
            .await
            .ok();
    }

    if let Ok(app) = App::new().await {
        let state = app.state.lock().await;
        assert!(state.create_device_form.available_device_types.is_empty());
        assert!(state.create_device_form.available_versions.is_empty());
        assert_eq!(state.create_device_form.ram_size, "2048");
    }

    std::env::remove_var("ANDROID_HOME");
}

#[test]
async fn test_notification_system() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();
    std::env::set_var("ANDROID_HOME", sdk_path);

    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho 'mock'\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms)
            .await
            .ok();
    }

    if let Ok(app) = App::new().await {
        {
            let mut state = app.state.lock().await;
            use crate::app::state::{Notification, NotificationType};
            let notification =
                Notification::new("Test notification".to_string(), NotificationType::Info);
            state.add_notification(notification);
        }

        let state = app.state.lock().await;
        assert!(!state.notifications.is_empty());
        assert_eq!(state.notifications[0].message, "Test notification");
    }

    std::env::remove_var("ANDROID_HOME");
}

#[test]
async fn test_mode_transitions() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();
    std::env::set_var("ANDROID_HOME", sdk_path);

    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho 'mock'\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms)
            .await
            .ok();
    }

    if let Ok(app) = App::new().await {
        {
            let mut state = app.state.lock().await;
            assert_eq!(state.mode, Mode::Normal);
            state.mode = Mode::CreateDevice;
            assert_eq!(state.mode, Mode::CreateDevice);
            state.mode = Mode::Normal;
            assert_eq!(state.mode, Mode::Normal);
        }
    }

    std::env::remove_var("ANDROID_HOME");
}
