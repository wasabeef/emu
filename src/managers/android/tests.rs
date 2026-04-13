use super::*;
use crate::managers::android::parser::AvdListParser;
use crate::managers::common::DeviceConfig;
use crate::models::device_info::DynamicDeviceProvider;
use crate::models::ApiLevel;
use crate::utils::command_executor::mock::MockCommandExecutor;
use crate::utils::ApiLevelCache;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::sync::OnceLock;
use tokio::sync::{Mutex, MutexGuard};

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarGuard {
    fn set<K, V>(key: K, value: V) -> Self
    where
        K: Into<&'static str>,
        V: Into<OsString>,
    {
        let key = key.into();
        let original = env::var_os(key);
        env::set_var(key, value.into());
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => env::set_var(self.key, value),
            None => env::remove_var(self.key),
        }
    }
}

fn test_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

async fn acquire_test_env_lock() -> MutexGuard<'static, ()> {
    test_env_lock().lock().await
}

fn setup_test_android_sdk() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();

    std::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin")).unwrap();
    std::fs::create_dir_all(sdk_path.join("tools/bin")).unwrap();
    std::fs::create_dir_all(sdk_path.join("emulator")).unwrap();
    std::fs::create_dir_all(sdk_path.join("platform-tools")).unwrap();

    let tools_to_create = [
        (
            "cmdline-tools/latest/bin/avdmanager",
            "#!/bin/sh\necho 'avdmanager mock'\n",
        ),
        (
            "tools/bin/avdmanager",
            "#!/bin/sh\necho 'avdmanager mock'\n",
        ),
        (
            "cmdline-tools/latest/bin/sdkmanager",
            "#!/bin/sh\necho 'sdkmanager mock'\n",
        ),
        (
            "tools/bin/sdkmanager",
            "#!/bin/sh\necho 'sdkmanager mock'\n",
        ),
        ("emulator/emulator", "#!/bin/sh\necho 'emulator mock'\n"),
        ("platform-tools/adb", "#!/bin/sh\necho 'adb mock'\n"),
    ];

    for (tool_path, script_content) in &tools_to_create {
        let full_path = sdk_path.join(tool_path);
        std::fs::write(&full_path, script_content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&full_path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&full_path, perms).unwrap();
        }
    }

    temp_dir
}

#[test]
fn test_parse_android_version_to_api_level() {
    assert_eq!(AndroidManager::parse_android_version_to_api_level("15"), 35);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("14"), 34);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("13"), 33);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("11"), 30);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("10"), 29);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("9"), 28);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("6"), 23);
    assert_eq!(
        AndroidManager::parse_android_version_to_api_level("8.1"),
        27
    );
    assert_eq!(
        AndroidManager::parse_android_version_to_api_level("8.0"),
        26
    );
    assert_eq!(
        AndroidManager::parse_android_version_to_api_level("7.1"),
        25
    );
    assert_eq!(
        AndroidManager::parse_android_version_to_api_level("7.0"),
        24
    );
    assert_eq!(
        AndroidManager::parse_android_version_to_api_level("5.1"),
        22
    );
    assert_eq!(
        AndroidManager::parse_android_version_to_api_level("5.0"),
        21
    );
    assert_eq!(
        AndroidManager::parse_android_version_to_api_level("4.4"),
        19
    );
    assert_eq!(
        AndroidManager::parse_android_version_to_api_level("4.1"),
        16
    );
    assert_eq!(
        AndroidManager::parse_android_version_to_api_level("14.0"),
        34
    );
    assert_eq!(AndroidManager::parse_android_version_to_api_level("12"), 0);
    assert_eq!(
        AndroidManager::parse_android_version_to_api_level("12.0"),
        0
    );
    assert_eq!(AndroidManager::parse_android_version_to_api_level("8"), 0);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("7"), 0);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("5"), 0);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("4"), 0);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("16"), 0);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("20"), 0);
    assert_eq!(AndroidManager::parse_android_version_to_api_level(""), 0);
    assert_eq!(
        AndroidManager::parse_android_version_to_api_level("invalid"),
        0
    );
    assert_eq!(AndroidManager::parse_android_version_to_api_level("abc"), 0);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("3"), 0);
    assert_eq!(AndroidManager::parse_android_version_to_api_level("2"), 0);
}

#[test]
fn test_find_android_home_with_env_var() {
    let temp_dir = setup_test_android_sdk();
    let android_home = temp_dir.path().to_path_buf();

    env::set_var("ANDROID_HOME", &android_home);

    let result = AndroidManager::find_android_home();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), android_home);

    env::remove_var("ANDROID_HOME");
}

#[test]
fn test_find_android_home_not_set() {
    env::remove_var("ANDROID_HOME");
    env::remove_var("ANDROID_SDK_ROOT");

    let result = AndroidManager::find_android_home();
    if result.is_err() {
        assert!(result.unwrap_err().to_string().contains("Android"));
    }
}

#[test]
fn test_find_tool_success() {
    let temp_dir = setup_test_android_sdk();
    let android_home = temp_dir.path();

    let tool_path = android_home.join("tools").join("bin").join("avdmanager");
    std::fs::create_dir_all(tool_path.parent().unwrap()).expect("Failed to create dirs");
    std::fs::write(&tool_path, "#!/bin/bash\necho 'mock avdmanager'")
        .expect("Failed to write tool");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&tool_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&tool_path, perms).unwrap();
    }

    let result = AndroidManager::find_tool(android_home, "avdmanager");
    assert!(result.is_ok());
    let expected_path = android_home
        .join("cmdline-tools")
        .join("latest")
        .join("bin")
        .join("avdmanager");
    assert_eq!(result.unwrap(), expected_path);
}

#[test]
fn test_find_tool_not_found() {
    let temp_dir = setup_test_android_sdk();
    let android_home = temp_dir.path();

    let result = AndroidManager::find_tool(android_home, "nonexistent_tool");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_get_device_category() {
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());

    let executor = std::sync::Arc::new(MockCommandExecutor::new());
    let manager = AndroidManager::with_executor(executor).expect("Failed to create manager");

    assert_eq!(manager.get_device_category("pixel_7", "Pixel 7"), "phone");
    assert_eq!(manager.get_device_category("pixel_6", "Pixel 6"), "phone");
    assert_eq!(manager.get_device_category("nexus_5x", "Nexus 5X"), "phone");
    assert_eq!(
        manager.get_device_category("nexus_10", "Nexus 10 inch"),
        "tablet"
    );
    assert_eq!(
        manager.get_device_category("pixel_tablet", "Pixel Tablet"),
        "tablet"
    );
    assert_eq!(
        manager.get_device_category("tv_1080p", "Android TV (1080p)"),
        "tv"
    );
    assert_eq!(
        manager.get_device_category("tv_720p", "Android TV (720p)"),
        "tv"
    );
    assert_eq!(
        manager.get_device_category("wear_round", "Android Wear Round"),
        "wear"
    );
    assert_eq!(
        manager.get_device_category("wear_square", "Android Wear Square"),
        "wear"
    );
    assert_eq!(
        manager.get_device_category("automotive_1024p", "Automotive (1024p landscape)"),
        "automotive"
    );
    assert_eq!(
        manager.get_device_category("unknown_device", "Unknown Device"),
        "phone"
    );
    assert_eq!(manager.get_device_category("", ""), "phone");

    env::remove_var("ANDROID_HOME");
}

#[test]
fn test_get_android_version_name() {
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());

    let executor = std::sync::Arc::new(MockCommandExecutor::new());
    let manager = AndroidManager::with_executor(executor).expect("Failed to create manager");

    assert_eq!(manager.get_android_version_name(34), "API 34");
    assert_eq!(manager.get_android_version_name(33), "API 33");
    assert_eq!(manager.get_android_version_name(31), "API 31");
    assert_eq!(manager.get_android_version_name(30), "API 30");
    assert_eq!(manager.get_android_version_name(29), "API 29");
    assert_eq!(manager.get_android_version_name(28), "API 28");
    assert_eq!(manager.get_android_version_name(21), "API 21");
    assert_eq!(manager.get_android_version_name(16), "API 16");
    assert_eq!(manager.get_android_version_name(40), "API 40");
    assert_eq!(manager.get_android_version_name(100), "API 100");
    assert_eq!(manager.get_android_version_name(1), "API 1");
    assert_eq!(manager.get_android_version_name(0), "API 0");

    env::remove_var("ANDROID_HOME");
}

#[test]
fn test_parse_api_level_from_package() {
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());

    let executor = std::sync::Arc::new(MockCommandExecutor::new());
    let manager = AndroidManager::with_executor(executor).expect("Failed to create manager");

    assert_eq!(
        manager.parse_api_level_from_package("system-images;android-34;google_apis;arm64-v8a"),
        Some(34)
    );
    assert_eq!(
        manager.parse_api_level_from_package(
            "system-images;android-33;google_apis_playstore;arm64-v8a"
        ),
        Some(33)
    );
    assert_eq!(
        manager.parse_api_level_from_package("system-images;android-31;default;x86_64"),
        Some(31)
    );
    assert_eq!(
        manager.parse_api_level_from_package("system-images;android-28;google_apis;x86"),
        Some(28)
    );
    assert_eq!(
        manager.parse_api_level_from_package("platforms;android-34"),
        Some(34)
    );
    assert_eq!(
        manager.parse_api_level_from_package("platforms;android-21"),
        Some(21)
    );
    assert_eq!(
        manager.parse_api_level_from_package("invalid-package"),
        None
    );
    assert_eq!(manager.parse_api_level_from_package(""), None);
    assert_eq!(
        manager.parse_api_level_from_package("system-images;invalid;google_apis;arm64-v8a"),
        None
    );
    assert_eq!(
        manager.parse_api_level_from_package("system-images;android-abc;google_apis;arm64-v8a"),
        None
    );
    assert_eq!(
        manager.parse_api_level_from_package("system-images;android-;google_apis;arm64-v8a"),
        None
    );

    env::remove_var("ANDROID_HOME");
}

#[test]
fn test_find_matching_device_id() {
    let available_devices = vec![
        ("pixel_7".to_string(), "Pixel 7".to_string()),
        ("pixel_6".to_string(), "Pixel 6".to_string()),
        ("galaxy_s22".to_string(), "Galaxy S22".to_string()),
        ("nexus_5x".to_string(), "Nexus 5X".to_string()),
        ("tv_1080p".to_string(), "Android TV (1080p)".to_string()),
        ("wear_round".to_string(), "Android Wear Round".to_string()),
        (
            "automotive_1024p".to_string(),
            "Automotive (1024p landscape)".to_string(),
        ),
    ];

    assert_eq!(
        AndroidManager::find_matching_device_id(&available_devices, "pixel_7"),
        Some("pixel_7".to_string())
    );
    assert_eq!(
        AndroidManager::find_matching_device_id(&available_devices, "Pixel 7"),
        Some("pixel_7".to_string())
    );
    assert_eq!(
        AndroidManager::find_matching_device_id(&available_devices, "Galaxy S22"),
        Some("galaxy_s22".to_string())
    );
    assert_eq!(
        AndroidManager::find_matching_device_id(&available_devices, "pixel"),
        Some("pixel_7".to_string())
    );
    assert_eq!(
        AndroidManager::find_matching_device_id(&available_devices, "unknown_device"),
        None
    );
    assert_eq!(
        AndroidManager::find_matching_device_id(&available_devices, ""),
        None
    );

    let empty_devices: Vec<(String, String)> = vec![];
    assert_eq!(
        AndroidManager::find_matching_device_id(&empty_devices, "pixel_7"),
        None
    );

    assert_eq!(
        AndroidManager::find_matching_device_id(&available_devices, "Android TV"),
        Some("tv_1080p".to_string())
    );
    assert_eq!(
        AndroidManager::find_matching_device_id(&available_devices, "Android Wear"),
        Some("wear_round".to_string())
    );
}

#[tokio::test]
async fn test_run_commands_parallel() {
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());

    let mock_executor = MockCommandExecutor::new()
        .with_success("cmd1", &[], "output1")
        .with_success("cmd2", &[], "output2")
        .with_success("cmd3", &["arg1"], "output3 with arg1")
        .with_error("cmd4", &[], "Command failed");

    let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
        Ok(manager) => manager,
        Err(_) => {
            env::remove_var("ANDROID_HOME");
            return;
        }
    };

    let commands = vec![
        ("cmd1".to_string(), vec![]),
        ("cmd2".to_string(), vec![]),
        ("cmd3".to_string(), vec!["arg1".to_string()]),
        ("cmd4".to_string(), vec![]),
    ];

    let results = manager.run_commands_parallel(commands).await;
    assert_eq!(results.len(), 4);
    assert!(results[0].is_ok());
    assert_eq!(results[0].as_ref().unwrap(), "output1");
    assert!(results[1].is_ok());
    assert_eq!(results[1].as_ref().unwrap(), "output2");
    assert!(results[2].is_ok());
    assert_eq!(results[2].as_ref().unwrap(), "output3 with arg1");
    assert!(results[3].is_err());
    assert!(results[3]
        .as_ref()
        .err()
        .unwrap()
        .to_string()
        .contains("Command failed"));

    env::remove_var("ANDROID_HOME");
}

#[tokio::test]
async fn test_list_available_targets_ignores_stale_disk_cache_when_no_images_are_installed() {
    let _env_lock = acquire_test_env_lock().await;
    let sdk_dir = setup_test_android_sdk();
    let home_dir = tempfile::tempdir().unwrap();
    let _android_home = EnvVarGuard::set("ANDROID_HOME", sdk_dir.path());
    let _home = EnvVarGuard::set("HOME", home_dir.path());

    let stale_cache = ApiLevelCache {
        api_levels: vec![
            ApiLevel {
                api: 34,
                version: "API 34".to_string(),
                display_name: "API 34 - API 34".to_string(),
                system_image_id: "android-34".to_string(),
                is_installed: true,
                variants: vec![],
            },
            ApiLevel {
                api: 33,
                version: "API 33".to_string(),
                display_name: "API 33 - API 33".to_string(),
                system_image_id: "android-33".to_string(),
                is_installed: true,
                variants: vec![],
            },
        ],
        timestamp: std::time::SystemTime::now(),
    };
    stale_cache.save_to_disk().unwrap();
    assert!(ApiLevelCache::load_from_disk().is_some());

    let sdkmanager_output = "Installed packages:\n  Path | Version | Description | Location\n\nAvailable Packages:\n  system-images;android-34;google_apis_playstore;arm64-v8a | 1 | Android SDK Platform 34 | system-images/android-34/google_apis_playstore/arm64-v8a\n";

    let sdkmanager_path = sdk_dir.path().join("cmdline-tools/latest/bin/sdkmanager");
    let mock_executor = MockCommandExecutor::new()
        .with_success(
            "sdkmanager",
            &["--list", "--verbose", "--include_obsolete"],
            sdkmanager_output,
        )
        .with_success(
            &sdkmanager_path.to_string_lossy(),
            &["--list", "--verbose", "--include_obsolete"],
            sdkmanager_output,
        );

    let manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();
    let targets = manager.list_available_targets().await.unwrap();

    assert!(targets.is_empty());
    assert!(ApiLevelCache::load_from_disk().is_none());
}

#[test]
fn test_avd_list_parser_new() {
    let output = "Sample AVD list output";
    let parser = AvdListParser::new(output);

    assert!(parser.current_device_info.is_none());
    assert!(parser.current_target_full.is_empty());
}

#[test]
fn test_avd_list_parser_parse_single_device() {
    let avd_output = r#"
    Name: Pixel_7_API_34
    Device: pixel_7 (Google)
    Path: /Users/user/.android/avd/Pixel_7_API_34.avd
    Target: Google APIs (Google Inc.)
    Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a
---------
"#;

    let mut parser = AvdListParser::new(avd_output);
    let device = parser.parse_next_device();
    assert!(device.is_some());

    let (name, path, target, abi, device_id) = device.unwrap();
    assert_eq!(name, "Pixel_7_API_34");
    assert_eq!(path, "/Users/user/.android/avd/Pixel_7_API_34.avd");
    assert_eq!(target, "Google APIs (Google Inc.)");
    assert_eq!(abi, "google_apis/arm64-v8a");
    assert_eq!(device_id, "pixel_7 (Google)");
    assert!(parser.parse_next_device().is_none());
}

#[test]
fn test_avd_list_parser_parse_multiple_devices() {
    let avd_output = r#"
    Name: Pixel_7_API_34
    Device: pixel_7 (Google)
    Path: /Users/user/.android/avd/Pixel_7_API_34.avd
    Target: Google APIs (Google Inc.)
    Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a
---------
    Name: Galaxy_S22_API_33
    Device: galaxy_s22 (Samsung)
    Path: /Users/user/.android/avd/Galaxy_S22_API_33.avd
    Target: Android API 33
    Based on: Android 13.0 (API level 33) Tag/ABI: google_apis_playstore/x86_64
---------
"#;

    let mut parser = AvdListParser::new(avd_output);
    let device1 = parser.parse_next_device();
    assert!(device1.is_some());
    let (name1, _, _, _, _) = device1.unwrap();
    assert_eq!(name1, "Pixel_7_API_34");

    let device2 = parser.parse_next_device();
    assert!(device2.is_some());
    let (name2, _, _, _, _) = device2.unwrap();
    assert_eq!(name2, "Galaxy_S22_API_33");

    assert!(parser.parse_next_device().is_none());
}

#[test]
fn test_avd_list_parser_empty_input() {
    let mut parser = AvdListParser::new("");
    assert!(parser.parse_next_device().is_none());
}

#[test]
fn test_avd_list_parser_malformed_input() {
    let malformed_output = r#"
Some random text that doesn't match any patterns
Another line without proper formatting
---------
"#;

    let mut parser = AvdListParser::new(malformed_output);
    assert!(parser.parse_next_device().is_none());
}

#[tokio::test]
async fn test_detect_api_level_for_device() {
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());

    let mock_executor = MockCommandExecutor::new();
    let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
        Ok(manager) => manager,
        Err(_) => {
            env::remove_var("ANDROID_HOME");
            return;
        }
    };

    let api_level = manager
        .detect_api_level_for_device(
            "test_device",
            "Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a",
        )
        .await;
    assert_eq!(api_level, 34);

    let api_level = manager
        .detect_api_level_for_device("test_device2", "Google APIs (API level 33)")
        .await;
    assert_eq!(api_level, 33);

    let api_level = manager
        .detect_api_level_for_device(
            "test_device3",
            "Based on: Android 13 Tag/ABI: google_apis/arm64-v8a",
        )
        .await;
    assert_eq!(api_level, 33);

    let api_level = manager
        .detect_api_level_for_device("test_device4", "Some unknown target format")
        .await;
    assert_eq!(api_level, 0);

    let api_level = manager
        .detect_api_level_for_device("test_device5", "")
        .await;
    assert_eq!(api_level, 0);

    env::remove_var("ANDROID_HOME");
}

#[tokio::test]
async fn test_get_avd_path() {
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());

    let avd_list_output = r#"
Available Android Virtual Devices:
    Name: Pixel_7_API_34
    Device: pixel_7 (Google)
    Path: /Users/test/.android/avd/Pixel_7_API_34.avd
    Target: Google APIs (Google Inc.)
    Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a
---------
    Name: Galaxy_S22_API_33
    Device: galaxy_s22 (Samsung)
    Path: /Users/test/.android/avd/Galaxy_S22_API_33.avd
    Target: Android API 33
    Based on: Android 13.0 (API level 33) Tag/ABI: google_apis_playstore/x86_64
---------
"#;

    let mock_executor =
        MockCommandExecutor::new().with_success("avdmanager", &["list", "avd"], avd_list_output);

    let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
        Ok(manager) => manager,
        Err(_) => {
            env::remove_var("ANDROID_HOME");
            return;
        }
    };

    let path = manager.get_avd_path("Pixel_7_API_34").await.unwrap();
    assert!(path.is_some());
    assert_eq!(
        path.unwrap().to_str().unwrap(),
        "/Users/test/.android/avd/Pixel_7_API_34.avd"
    );

    let path = manager.get_avd_path("Galaxy_S22_API_33").await.unwrap();
    assert!(path.is_some());
    assert_eq!(
        path.unwrap().to_str().unwrap(),
        "/Users/test/.android/avd/Galaxy_S22_API_33.avd"
    );

    let path = manager.get_avd_path("NonExistent_AVD").await.unwrap();
    assert!(path.is_none());

    let path = manager.get_avd_path("").await.unwrap();
    assert!(path.is_none());

    env::remove_var("ANDROID_HOME");
}

#[tokio::test]
async fn test_fine_tune_avd_config() {
    let original_android_home = env::var("ANDROID_HOME").ok();
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());

    let avd_dir = temp_dir.path().join("test_avd.avd");
    tokio::fs::create_dir_all(&avd_dir).await.unwrap();

    let config_path = avd_dir.join("config.ini");
    let initial_config = r#"avd.ini.encoding=UTF-8
hw.accelerometer=no
hw.audioInput=yes
hw.battery=yes
vm.heapSize=256
"#;
    tokio::fs::write(&config_path, initial_config)
        .await
        .unwrap();

    let avd_list_output = format!(
        r#"
Available Android Virtual Devices:
    Name: test_avd
    Device: pixel_7 (Google)
    Path: {}
    Target: Google APIs (Google Inc.)
    Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a
---------
"#,
        avd_dir.to_str().unwrap()
    );

    let mock_executor =
        MockCommandExecutor::new().with_success("avdmanager", &["list", "avd"], &avd_list_output);

    let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
        Ok(manager) => manager,
        Err(_) => {
            env::remove_var("ANDROID_HOME");
            return;
        }
    };

    let device_config = DeviceConfig {
        name: "Test Pixel 7".to_string(),
        device_type: "pixel_7".to_string(),
        version: "14".to_string(),
        ram_size: Some("2048".to_string()),
        storage_size: Some("4096".to_string()),
        additional_options: HashMap::new(),
    };

    manager
        .fine_tune_avd_config("test_avd", &device_config, "google_apis", "arm64-v8a")
        .await
        .expect("Failed to fine tune AVD config");

    let updated_config = tokio::fs::read_to_string(&config_path).await.unwrap();
    assert!(updated_config.contains("avd.ini.displayname=Test Pixel 7"));
    assert!(updated_config.contains("AvdId=Test_Pixel_7"));
    assert!(updated_config.contains("avd.ini.encoding=UTF-8"));
    assert!(updated_config.contains("hw.accelerometer=no"));
    assert!(updated_config.contains("hw.audioInput=yes"));

    match original_android_home {
        Some(value) => env::set_var("ANDROID_HOME", value),
        None => env::remove_var("ANDROID_HOME"),
    }
}

#[tokio::test]
async fn test_fine_tune_avd_config_avd_not_found() {
    let original_android_home = env::var("ANDROID_HOME").ok();
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());

    let mock_executor = MockCommandExecutor::new().with_success("avdmanager", &["list", "avd"], "");

    let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
        Ok(manager) => manager,
        Err(_) => {
            env::remove_var("ANDROID_HOME");
            return;
        }
    };

    let device_config = DeviceConfig {
        name: "Test Device".to_string(),
        device_type: "pixel_7".to_string(),
        version: "14".to_string(),
        ram_size: None,
        storage_size: None,
        additional_options: HashMap::new(),
    };

    let result = manager
        .fine_tune_avd_config(
            "nonexistent_avd",
            &device_config,
            "google_apis",
            "arm64-v8a",
        )
        .await;
    assert!(result.is_ok());

    match original_android_home {
        Some(value) => env::set_var("ANDROID_HOME", value),
        None => env::remove_var("ANDROID_HOME"),
    }
}

#[tokio::test]
async fn test_get_dynamic_android_version_name() {
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());
    let sdkmanager_path = temp_dir.path().join("cmdline-tools/latest/bin/sdkmanager");

    let platforms_output = r#"
Installed packages:
  Path                                        | Version | Description                    | Location
  -------                                     | ------- | -------                        | -------
  platforms;android-34                        | 3       | Android SDK Platform 34        | platforms/android-34 | Android API 34, revision 2 | Android 14
  platforms;android-33                        | 3       | Android SDK Platform 33        | platforms/android-33 | Android API 33, revision 3 | Android 13
"#;

    let mock_executor = MockCommandExecutor::new()
        .with_error(
            &sdkmanager_path.to_string_lossy(),
            &["--list", "--verbose", "--include_obsolete"],
            "verbose list failed",
        )
        .with_success(
            &sdkmanager_path.to_string_lossy(),
            &["--list"],
            platforms_output,
        );

    let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
        Ok(manager) => manager,
        Err(_) => {
            env::remove_var("ANDROID_HOME");
            return;
        }
    };

    let version_name = manager.get_dynamic_android_version_name(34).await;
    assert_eq!(version_name, Some("14".to_string()));

    let version_name = manager.get_dynamic_android_version_name(999).await;
    assert!(version_name.is_none());

    env::remove_var("ANDROID_HOME");
}

#[tokio::test]
async fn test_detect_api_level_for_device_prefers_explicit_api_level() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = setup_test_android_sdk();
    let _android_home = EnvVarGuard::set("ANDROID_HOME", temp_dir.path().as_os_str());

    let manager = AndroidManager::with_executor(Arc::new(MockCommandExecutor::new())).unwrap();

    let api_level = manager
        .detect_api_level_for_device(
            "Pixel_12_API",
            "Google APIs (Google Inc.) Based on: Android 12.0 (API level 31) Tag/ABI: google_apis/arm64-v8a",
        )
        .await;

    assert_eq!(api_level, 31);
}

#[tokio::test]
async fn test_detect_api_level_for_device_keeps_ambiguous_version_unknown() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = setup_test_android_sdk();
    let _android_home = EnvVarGuard::set("ANDROID_HOME", temp_dir.path().as_os_str());

    let manager = AndroidManager::with_executor(Arc::new(MockCommandExecutor::new())).unwrap();

    let api_level = manager
        .detect_api_level_for_device(
            "Pixel_12_API",
            "Google APIs (Google Inc.) Based on: Android 12.0 Tag/ABI: google_apis/arm64-v8a",
        )
        .await;

    assert_eq!(api_level, 0);
}

#[tokio::test]
async fn test_list_devices_sorts_by_api_and_device_priority() {
    let _env_lock = acquire_test_env_lock().await;
    let temp_dir = setup_test_android_sdk();
    let _android_home = EnvVarGuard::set("ANDROID_HOME", temp_dir.path().as_os_str());

    let avd_list_output = r#"
Available Android Virtual Devices:
    Name: Pixel_6_API_34
    Device: pixel_6 (Google)
    Path: /Users/test/.android/avd/Pixel_6_API_34.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a
---------
    Name: Pixel_7a_API_34
    Device: pixel_7a (Google)
    Path: /Users/test/.android/avd/Pixel_7a_API_34.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a
---------
    Name: Pixel_8a_API_35
    Device: pixel_8a (Google)
    Path: /Users/test/.android/avd/Pixel_8a_API_35.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 15.0 (API level 35) Tag/ABI: google_apis/arm64-v8a
---------
    Name: Pixel_8_API_35
    Device: pixel_8 (Google)
    Path: /Users/test/.android/avd/Pixel_8_API_35.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 15.0 (API level 35) Tag/ABI: google_apis/arm64-v8a
---------
    Name: Pixel_9_Pro_API_36
    Device: pixel_9_pro (Google)
    Path: /Users/test/.android/avd/Pixel_9_Pro_API_36.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 16.0 (API level 36) Tag/ABI: google_apis/arm64-v8a
---------
    Name: Pixel_9_API_36
    Device: pixel_9 (Google)
    Path: /Users/test/.android/avd/Pixel_9_API_36.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 16.0 (API level 36) Tag/ABI: google_apis/arm64-v8a
---------
"#;

    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], avd_list_output)
        .with_success("adb", &["devices"], "List of devices attached\n");

    let manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();
    let devices = manager.list_devices().await.unwrap();
    let names: Vec<&str> = devices.iter().map(|device| device.name.as_str()).collect();

    assert_eq!(
        names,
        vec![
            "Pixel_9_API_36",
            "Pixel_9_Pro_API_36",
            "Pixel_8_API_35",
            "Pixel_8a_API_35",
            "Pixel_7a_API_34",
            "Pixel_6_API_34",
        ]
    );
}

#[tokio::test]
async fn test_get_device_priority() {
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());

    let mock_executor = MockCommandExecutor::new();
    let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
        Ok(manager) => manager,
        Err(_) => {
            env::remove_var("ANDROID_HOME");
            return;
        }
    };

    let priority_phone = manager.get_device_priority("pixel_7").await.unwrap();
    let priority_tv = manager.get_device_priority("tv_1080p").await.unwrap();
    let priority_unknown = manager.get_device_priority("unknown_device").await.unwrap();

    assert!(priority_phone > 0);
    assert!(priority_tv > priority_phone);
    assert!(priority_unknown > priority_tv);
    assert!(priority_tv > 0);
    assert!(priority_unknown > 0);

    env::remove_var("ANDROID_HOME");
}

#[tokio::test]
#[cfg(feature = "test-utils")]
async fn test_get_available_devices() {
    let original_android_home = env::var("ANDROID_HOME").ok();
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());

    let fixture_content = include_str!("../../../tests/fixtures/android_outputs.json");
    let fixture: serde_json::Value =
        serde_json::from_str(fixture_content).expect("Invalid JSON in fixture");
    let device_list_output = fixture["avdmanager_list_device"]["comprehensive"]
        .as_str()
        .expect("Device list fixture not found");

    let mock_executor = MockCommandExecutor::new().with_success(
        "avdmanager",
        &["list", "device"],
        device_list_output,
    );

    let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
        Ok(manager) => manager,
        Err(_) => {
            env::remove_var("ANDROID_HOME");
            return;
        }
    };

    let result = manager.get_available_devices().await;
    assert!(result.is_ok());

    let devices = result.unwrap();
    assert!(!devices.is_empty());
    assert!(devices.iter().all(|d| !d.id.is_empty()));
    assert!(devices.iter().all(|d| !d.display_name.is_empty()));

    match original_android_home {
        Some(value) => env::set_var("ANDROID_HOME", value),
        None => env::remove_var("ANDROID_HOME"),
    }
}

#[tokio::test]
#[cfg(feature = "test-utils")]
async fn test_get_available_api_levels() {
    let original_android_home = env::var("ANDROID_HOME").ok();
    let temp_dir = setup_test_android_sdk();
    env::set_var("ANDROID_HOME", temp_dir.path());

    let sdkmanager_path = temp_dir.path().join("cmdline-tools/latest/bin/sdkmanager");
    let fixture_content = include_str!("../../../tests/fixtures/android_outputs.json");
    let fixture: serde_json::Value =
        serde_json::from_str(fixture_content).expect("Invalid JSON in fixture");
    let sdkmanager_output = fixture["sdkmanager_list"]["system_images"]
        .as_str()
        .expect("System images fixture not found");

    let mock_executor = MockCommandExecutor::new()
        .with_success(
            "sdkmanager",
            &["--list", "--verbose", "--include_obsolete"],
            sdkmanager_output,
        )
        .with_success(
            &sdkmanager_path.to_string_lossy(),
            &["--list", "--verbose", "--include_obsolete"],
            sdkmanager_output,
        );

    let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
        Ok(manager) => manager,
        Err(_) => {
            env::remove_var("ANDROID_HOME");
            return;
        }
    };

    let result = manager.get_available_api_levels().await;
    assert!(result.is_ok());

    let api_levels = result.unwrap();
    assert!(!api_levels.is_empty());
    assert!(api_levels.iter().all(|a| a.level > 0));
    assert!(api_levels.iter().all(|a| !a.version_name.is_empty()));

    match original_android_home {
        Some(value) => env::set_var("ANDROID_HOME", value),
        None => env::remove_var("ANDROID_HOME"),
    }
}
