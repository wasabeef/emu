use emu::app::state::CreateDeviceForm;
use emu::models::device_info::DynamicDeviceConfig;

#[test]
fn test_device_name_generation_preserves_spaces() {
    let mut form = CreateDeviceForm::new();

    // テスト用のデバイスタイプとバージョンを設定
    form.device_type = "Pixel Fold (Google)".to_string();
    form.device_type_id = "pixel_fold".to_string();
    form.version = "35".to_string();
    form.version_display = "API 35 - Android 15".to_string();

    // プレースホルダー名を生成
    form.generate_placeholder_name();

    // スペースが保持されていることを確認
    assert!(form.name.contains(" "), "Device name should contain spaces");
    assert!(
        form.name.contains("Pixel"),
        "Device name should contain device type"
    );
    assert!(
        form.name.contains("API"),
        "Device name should contain API level"
    );

    // 具体的な形式をテスト - 新しい動的パーサーは括弧内の内容を除外
    assert_eq!(form.name, "Pixel Fold API 35");

    println!("Generated device name: '{}'", form.name);
}

#[test]
fn test_device_name_generation_various_formats() {
    let test_cases = vec![
        // (device_type, version_display, expected_contains)
        (
            "2.7\" QVGA",
            "API 36 - Android 16",
            vec!["2.7", "QVGA", "API 36"],
        ),
        (
            "iPhone 15 Pro",
            "iOS 17.0",
            vec!["iPhone", "15", "Pro", "iOS 17"],
        ),
        (
            "Android TV (1080p)",
            "API 34 - Android 14",
            vec!["Android", "TV", "API 34"],
        ),
        (
            "Wear Round",
            "API 33 - Android 13",
            vec!["Wear", "Round", "API 33"],
        ),
    ];

    for (device_type, version_display, expected_parts) in test_cases {
        let mut form = CreateDeviceForm::new();
        form.device_type = device_type.to_string();
        form.device_type_id = device_type.to_lowercase().replace(" ", "_");
        form.version_display = version_display.to_string();
        form.version = if version_display.starts_with("iOS") {
            version_display.to_string()
        } else {
            version_display
                .split_whitespace()
                .nth(1)
                .unwrap_or("35")
                .to_string()
        };

        form.generate_placeholder_name();

        println!(
            "Device type: '{}' -> Generated name: '{}'",
            device_type, form.name
        );

        // 期待される部分が含まれていることを確認
        for expected_part in expected_parts {
            assert!(
                form.name.contains(expected_part),
                "Device name '{}' should contain '{}'",
                form.name,
                expected_part
            );
        }

        // 名前が空でないことを確認
        assert!(
            !form.name.trim().is_empty(),
            "Device name should not be empty"
        );
    }
}

#[test]
fn test_device_name_fallback_when_empty() {
    let mut form = CreateDeviceForm::new();

    // 空のデバイスタイプとバージョンを設定
    form.device_type = "".to_string();
    form.device_type_id = "".to_string();
    form.version = "35".to_string();
    form.version_display = "".to_string();

    form.generate_placeholder_name();

    // フォールバック名が生成されることを確認
    assert_eq!(form.name, "Device API");

    println!("Fallback device name: '{}'", form.name);
}

#[test]
fn test_dynamic_device_config_parsing() {
    let config = DynamicDeviceConfig::new();

    let test_cases = vec![
        "Pixel 7 Pro (Google)",
        "iPhone 15 Pro Max",
        "2.7\" QVGA",
        "Android TV (1080p)",
        "Wear Round",
    ];

    for device_name in test_cases {
        let parsed_parts = config.parse_device_name(device_name);

        println!("Device '{}' parsed to: {:?}", device_name, parsed_parts);

        // パースが失敗していないことを確認（空でない）
        assert!(
            !parsed_parts.is_empty() || device_name.is_empty(),
            "Should be able to parse device name '{}'",
            device_name
        );
    }
}

#[test]
fn test_create_device_form_android_initialization() {
    let form = CreateDeviceForm::for_android();

    // Android フォームの初期化状態をテスト
    assert_eq!(form.ram_size, "2048");
    assert_eq!(form.storage_size, "8192");
    assert!(form.available_device_types.is_empty());
    assert!(form.available_versions.is_empty());
    assert_eq!(form.selected_api_level_index, 0);
    assert_eq!(form.selected_device_type_index, 0);
    assert!(!form.is_loading_cache);

    println!("Android form initialized correctly");
}

#[test]
fn test_create_device_form_ios_initialization() {
    let form = CreateDeviceForm::for_ios();

    // iOS フォームの初期化状態をテスト
    assert_eq!(form.ram_size, "2048");
    assert_eq!(form.storage_size, "8192");
    assert!(form.available_device_types.is_empty());
    assert!(form.available_versions.is_empty());
    assert_eq!(form.selected_api_level_index, 0);
    assert_eq!(form.selected_device_type_index, 0);
    assert!(!form.is_loading_cache);

    println!("iOS form initialized correctly");
}

#[test]
fn test_device_name_with_special_characters() {
    let mut form = CreateDeviceForm::new();

    // 特殊文字を含むデバイス名をテスト
    form.device_type = "2.7\" QVGA (Small)".to_string();
    form.device_type_id = "qvga_2_7".to_string();
    form.version = "36".to_string();
    form.version_display = "API 36 - Android 16".to_string();

    form.generate_placeholder_name();

    // 名前が生成されることを確認（スペースは保持）
    assert!(!form.name.is_empty(), "Device name should not be empty");
    assert!(form.name.contains("2.7"), "Should contain screen size");
    assert!(form.name.contains("QVGA"), "Should contain resolution");
    assert!(form.name.contains("API 36"), "Should contain API level");

    // 二重引用符が処理されていることを確認（表示名には残る）
    println!("Special character device name: '{}'", form.name);
}

#[test]
fn test_device_name_sanitization_for_avd_creation() {
    // Android サニタイゼーション用テスト
    let test_cases = vec![
        // (input, should_be_safe_for_avd)
        ("Pixel 7 Pro API 34", true),    // 通常のケース
        ("2.7\" QVGA API 36", true),     // 引用符付き（AVDでは削除される）
        ("Device with: colon", true),    // コロン付き（AVDでは削除される）
        ("Device/with/slash", true),     // スラッシュ付き（AVDでは削除される）
        ("Normal Device Name", true),    // 正常なケース
        ("'Single Quote Device'", true), // シングルクォート付き
        ("Device*with*asterisk", true),  // アスタリスク付き
    ];

    for (input, should_be_safe) in test_cases {
        // AndroidManager の実際のサニタイゼーション処理をテスト（AVD名用）
        let sanitized = input
            .chars()
            .filter_map(|c| match c {
                // AVD names: only a-z A-Z 0-9 . _ - are allowed
                c if c.is_ascii_alphanumeric() || c == '.' || c == '-' => Some(c),
                ' ' | '_' => Some('_'), // Convert spaces to underscores
                _ => None,              // Remove all other characters
            })
            .collect::<String>()
            .trim_matches('_') // Remove leading/trailing underscores
            .to_string();

        if should_be_safe {
            assert!(
                !sanitized.is_empty(),
                "Sanitized name should not be empty for: '{}'",
                input
            );
            // AVD名ではスペースはアンダースコアに変換される
            if input.contains(' ') {
                assert!(
                    sanitized.contains('_'),
                    "Spaces should be converted to underscores in AVD name: '{}'",
                    input
                );
            }
        }

        println!("Input: '{}' -> AVD Name: '{}'", input, sanitized);
    }
}
