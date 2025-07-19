# Test Fixtures

このディレクトリには、Emu プロジェクトのテストで使用される実際のコマンド出力データが含まれています。

## ファイル構成

### android_outputs.json

Android SDK コマンドの出力パターンを格納します。

**主要なセクション:**

- `avdmanager_list_avd` - AVD 一覧の出力形式
- `avdmanager_list_device` - デバイス一覧の出力形式
- `adb_devices` - ADB デバイス一覧の出力形式
- `adb_shell_getprop` - Android プロパティの出力形式
- `adb_emu_avd_name` - エミュレータ AVD 名の出力形式
- `sdkmanager_list` - SDK マネージャーの出力形式
- `errors` - エラーケースの出力形式

### ios_outputs.json

iOS Simulator コマンドの出力パターンを格納します。

**主要なセクション:**

- `xcrun_simctl_list_devices` - デバイス一覧の JSON 出力
- `xcrun_simctl_list_devicetypes` - デバイスタイプ一覧の JSON 出力
- `xcrun_simctl_list_runtimes` - ランタイム一覧の JSON 出力
- `xcrun_simctl_boot/shutdown/create/delete/erase` - 各操作の出力形式
- `log_stream` - ログストリームのサンプル出力

### state_transitions.json

デバイスの状態遷移パターンを格納します。

**主要なセクション:**

- `android_device_states` - Android デバイスの状態変化
- `ios_device_states` - iOS デバイスの状態変化
- 各状態遷移の前後のコマンド出力

### error_scenarios.json

エラーシナリオとその出力を格納します。

**主要なセクション:**

- `android_errors` - Android SDK 関連のエラー
- `ios_errors` - iOS Simulator 関連のエラー
- `network_errors` - ネットワーク関連のエラー

### environment_variations.json

環境の違いによる出力の変化を格納します。

**主要なセクション:**

- `android_environments` - Android SDK 環境の違い
- `ios_environments` - iOS 環境の違い
- `system_variations` - システム環境の違い

## 使用方法

### テストでの活用例

```rust
use serde_json::Value;
use std::fs;

// フィクスチャーデータの読み込み
fn load_fixture(filename: &str) -> Value {
    let content = fs::read_to_string(format!("tests/fixtures/{}", filename))
        .expect("Failed to read fixture file");
    serde_json::from_str(&content).expect("Failed to parse fixture JSON")
}

// Android AVD 一覧の解析テスト
#[test]
fn test_parse_avd_list() {
    let fixtures = load_fixture("android_outputs.json");
    let avd_output = fixtures["avdmanager_list_avd"]["single_device"]
        .as_str()
        .unwrap();

    // パース処理のテスト
    let parsed = parse_avd_list(avd_output);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].name, "Pixel_7_API_34");
}

// iOS デバイス一覧の解析テスト
#[test]
fn test_parse_ios_devices() {
    let fixtures = load_fixture("ios_outputs.json");
    let device_output = fixtures["xcrun_simctl_list_devices"]["single_runtime"]
        .as_str()
        .unwrap();

    // JSON パース処理のテスト
    let parsed: Value = serde_json::from_str(device_output).unwrap();
    assert!(parsed["devices"].is_object());
}
```

### MockDeviceManager での活用

```rust
use crate::managers::mock::MockDeviceManager;
use crate::managers::mock::MockScenario;

// フィクスチャーデータを使用したモックシナリオの構築
fn create_android_scenario() -> MockScenario {
    let fixtures = load_fixture("android_outputs.json");

    MockScenario::new()
        .with_command_output(
            "avdmanager",
            vec!["list", "avd"],
            fixtures["avdmanager_list_avd"]["multiple_devices"]
                .as_str()
                .unwrap()
                .to_string(),
        )
        .with_command_output(
            "adb",
            vec!["devices"],
            fixtures["adb_devices"]["multiple_devices"]
                .as_str()
                .unwrap()
                .to_string(),
        )
}
```

## データ収集方法

### Android データ収集

```bash
# AVD 一覧の取得
avdmanager list avd > avd_list.txt

# デバイス一覧の取得
avdmanager list device > device_list.txt

# ADB デバイス一覧の取得
adb devices > adb_devices.txt

# プロパティの取得
adb -s emulator-5554 shell getprop ro.boot.qemu.avd_name > avd_name.txt

# システムイメージ一覧の取得
sdkmanager --list --verbose > sdkmanager_list.txt
```

### iOS データ収集

```bash
# デバイス一覧の取得
xcrun simctl list devices --json > ios_devices.json

# デバイスタイプ一覧の取得
xcrun simctl list devicetypes --json > ios_devicetypes.json

# ランタイム一覧の取得
xcrun simctl list runtimes --json > ios_runtimes.json

# ログストリームのサンプル
xcrun simctl spawn booted log stream --level debug > ios_logs.txt
```

## データ管理ポリシー

### バージョン管理

- 新しい Android SDK や iOS バージョンがリリースされた際は、対応する出力データを更新
- 古いバージョンのデータも互換性テストのために保持

### 環境差異の管理

- 異なる OS バージョンやハードウェア構成での出力差異を `environment_variations.json` で管理
- 地域やロケール設定による出力の違いも考慮

### セキュリティ考慮事項

- 実際のデバイス ID や個人情報を含むパスは匿名化
- UUIDs や実際のユーザー名は汎用的なプレースホルダーに置換

## 更新頻度

- **Android**: 四半期ごとに Android SDK の更新に合わせて更新
- **iOS**: Xcode の新バージョンリリースに合わせて更新
- **エラーシナリオ**: 新しいエラーケースが発見された際に随時更新

## 品質保証

### 検証方法

1. **実環境での検証**: 実際の Android SDK と iOS Simulator で出力を確認
2. **パース処理の検証**: フィクスチャーデータが正しくパースされることを確認
3. **状態遷移の検証**: 状態遷移データが実際の挙動と一致することを確認

### 自動テスト

- CI/CD パイプラインでフィクスチャーデータの構文チェック
- 定期的な実環境との整合性チェック
- パフォーマンステストでの使用量監視

このフィクスチャーシステムにより、実際のエミュレータや SDK に依存しない高速で安定したテストが可能になります。
