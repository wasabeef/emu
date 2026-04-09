# テストリファクタリング計画

## 背景

現在のテストスイートには **構造** と **結合** の 2 つの問題がある。
構造だけ整理しても結合問題が残り、リファクタリング耐性が得られない。
**結合問題を先に解決し、その後に構造を整理する。**

---

## 1. 現状分析

### 数値

| 項目                        | 値       |
| --------------------------- | -------- |
| テストファイル数            | 109      |
| 空ファイル (0-1 行)         | 25 (23%) |
| テスト関数 (tests/)         | 818      |
| インラインテスト関数 (src/) | 211      |
| テスト総行数                | 31,174   |
| ソース総行数                | ~20,600  |
| テスト/ソース比率           | 1.5:1    |

### 構造の問題

| ID  | 問題                   | 詳細                                                                                |
| --- | ---------------------- | ----------------------------------------------------------------------------------- |
| P1  | 構造的肥大化           | 存在しない機能 (telemetry, analytics, plugin system 等) 用の空ファイルが 25 個      |
| P2  | 重複カバレッジ         | Android manager が 6 ファイル、iOS が 4、AppState が 4、UI render が 4 で重複テスト |
| P3  | 基盤の二重化           | `tests/common/` と `tests/unit/common/` に同じヘルパーが重複                        |
| P4  | ルートレベルのレガシー | `tests/` 直下の 5 ファイルが `tests/unit/` `tests/integration/` と重複              |
| P5  | テスト境界の曖昧さ     | unit/integration/performance の区別基準がない                                       |
| P6  | 定数の過剰テスト       | 静的 `const` 値に対して 163 テスト。大半は `assert_eq!(X, 5)` レベル                |
| P7  | 不安定な性能テスト     | 絶対時間閾値が CI 負荷で失敗する                                                    |

### 結合の問題

テストが実装詳細に密結合しており、動作が正しくてもリファクタリングでテストが壊れる。

| パターン                       | 出現数   | リスク | 例                                        |
| ------------------------------ | -------- | ------ | ----------------------------------------- |
| 直接フィールドアクセス         | ~250+    | 高     | `state.mode = Mode::CreateDevice`         |
| enum variant 直接比較          | 416      | 致命的 | `DeviceStatus::Stopped` が 195 箇所       |
| Mock の引数完全一致            | 176 呼出 | 高     | `.with_success("adb", &["devices"], ...)` |
| インデックスベースの assertion | ~50+     | 高     | `state.android_devices[0].status`         |
| trait contract テスト          | 0        | 高     | 別の DeviceManager 実装を検証不可         |

### リファクタリングシナリオの影響

| シナリオ                                       | 壊れるテスト数 | 原因                        |
| ---------------------------------------------- | -------------- | --------------------------- |
| field → getter (`state.mode` → `state.mode()`) | 40-50          | テスト内の直接代入          |
| App を generic 化                              | 12-15          | concrete type の import     |
| DeviceStatus::Stopped → Offline にリネーム     | 416+           | enum variant のハードコード |
| android.rs をサブモジュールに分割              | 7-10           | モジュールパスの直接 import |
| MockCommandExecutor API 変更                   | 50-80          | builder パターンの結合      |

---

## 2. 設計原則

1. **1 ソースモジュール = 1 テストファイル**。4-6 ファイルに分散させない
2. **存在しないものは削除**。未実装機能のプレースホルダーは不要
3. **純粋ロジックのテストは src/ に inline**。`#[cfg(test)]` でコードの隣に配置
4. **tests/ は integration テスト専用**。複数モジュールの結合や複雑な fixture が必要なもの
5. **テスト基盤は 1 箇所に統合**。`tests/support/` のみ
6. **実装ではなく振る舞いをテスト**。呼び出し側が観察できる結果を検証
7. **絶対的な時間閾値は使わない**。相対比較か `#[ignore]` で手動実行

---

## 3. 制約 (Rust 固有)

### App の concrete type 制約

`App` は `AndroidManager`/`IosManager` を concrete type で保持。
App インスタンスが必要なテストは `setup_mock_android_sdk()` + 環境変数操作が必要。
→ **inline 化不可**、`tests/` に残す。

### 環境変数の race condition

`ANDROID_HOME` を `std::env::set_var` するテストはスレッドセーフでない。
inline 化すると同一バイナリ内で並列実行され race が発生。
→ env var 操作テストは `tests/` に残し `RUST_TEST_THREADS=1` で実行。
将来的には `serial_test` crate の `#[serial]` 導入を検討。

### fixture ファイルパス

`include_str!("../../tests/fixtures/...")` はテストを `src/` に移動すると壊れる。
→ `std::fs::read_to_string` でランタイム読み込みに変更するか、`tests/` に残す。

### コンパイル時間のトレードオフ

inline テスト増加 → `cargo test --lib` のバイナリが大きくなり incremental recompile が遅くなる。
一方でバイナリ総数は減り full build は速くなる。20K LOC のクレートでは許容範囲。

### テストバイナリ構成

integration テストは **top-level `tests/*.rs`** (個別バイナリ、並列実行) 。
`tests/lib.rs` のサブモジュールにはしない。

---

## 4. 目標構造

```
src/
├── app/
│   ├── mod.rs              # inline: 純粋ヘルパーのみ (App instance テストは不可)
│   ├── state.rs            # inline: state 遷移, is_busy(), ナビゲーション, 通知
│   ├── events.rs           # inline: キーマッピング, イベント分類
│   └── event_processing.rs # inline: batcher, debouncer
├── managers/
│   ├── android.rs          # inline: パース (AVD リスト, system image, API level)
│   ├── ios.rs              # inline: パース (simctl JSON, runtime, device type)
│   ├── common.rs           # inline: DeviceConfig builder, 名前サニタイズ
│   └── mock.rs             # inline: mock 動作検証
├── models/
│   ├── device.rs           # inline: trait impl, ステータス遷移, Display
│   ├── api_level.rs        # inline: variant 選択, アーキテクチャ検出
│   ├── device_info.rs      # inline: 優先度計算, カテゴリマッピング
│   ├── error.rs            # inline: エラーフォーマット, ユーザーメッセージ
│   └── platform.rs         # inline: プラットフォーム検出
├── ui/
│   ├── render.rs           # (inline テストなし — MockBackend で integration テスト)
│   ├── theme.rs            # inline: 色選択, ステータス色
│   └── widgets.rs          # inline: builder パターン
├── utils/
│   ├── validation.rs       # inline: 全バリデータ, 合成
│   ├── command.rs          # inline: リトライロジック, エラーハンドリング
│   └── command_executor.rs # inline: mock executor builder
└── constants/
    └── defaults.rs         # inline: ~10 件の関係性不変条件のみ (MIN < MAX 等)

tests/
├── support/
│   ├── mod.rs              # re-exports
│   ├── devices.rs          # ファクトリ: android_device(), ios_device()
│   ├── managers.rs         # ファクトリ: mock_android_manager(), mock_ios_manager()
│   ├── state.rs            # TestStateBuilder
│   ├── fixtures.rs         # JSON fixture ローダー (ランタイム読込、include_str! 不使用)
│   ├── assertions.rs       # カスタム assert マクロ
│   └── contract.rs         # DeviceManager trait contract テスト
├── android_manager.rs      # integration: Android デバイスライフサイクル全体
├── ios_manager.rs          # integration: iOS デバイスライフサイクル全体
├── app_lifecycle.rs        # integration: App 初期化 → イベント処理 → 状態変更
├── ui_rendering.rs         # integration: draw_app() + MockBackend
├── device_creation.rs      # integration: デバイス作成ウィザードフロー
├── api_level_mgmt.rs       # integration: install/uninstall, progress, is_busy()
├── error_recovery.rs       # integration: 横断的エラー回復パターン
├── benchmarks.rs           # #[ignore] 性能テスト (手動実行、CI では実行しない)
└── fixtures/
    ├── android_outputs.json
    ├── ios_outputs.json
    ├── error_scenarios.json
    └── android_outputs/
        └── *.txt
```

### ファイル数比較

|                            | 現在 | 目標                         |
| -------------------------- | ---- | ---------------------------- |
| テストファイル (tests/)    | 109  | ~18                          |
| 空ファイル                 | 25   | 0                            |
| integration テストファイル | 45   | 7                            |
| performance テストファイル | 16   | 1 (benchmarks.rs, #[ignore]) |
| 共通基盤                   | 分散 | support/ に統一              |

---

## 5. リファクタリング耐性ルール

テスト移動・書き換え時に必ず適用する 5 つのルール。

### Rule 1: public メソッド経由でテスト、フィールド直接アクセス禁止

```rust
// ❌ リファクタリングで壊れる
state.mode = Mode::CreateDevice;
assert_eq!(state.active_panel, Panel::Android);

// ✅ 内部構造変更に耐える
state.enter_create_mode();
assert!(state.is_create_mode());
assert!(state.is_android_panel());
```

### Rule 2: テスト状態の setup は TestStateBuilder 経由

```rust
// ❌ 全フィールド名に結合
let mut state = AppState::new();
state.android_devices = vec![create_device()];
state.mode = Mode::Normal;
state.active_panel = Panel::Android;

// ✅ builder 内部のみ更新すればよい
let state = TestStateBuilder::new()
    .with_android_devices(vec![android_device("Pixel")])
    .in_mode(Mode::Normal)
    .on_panel(Panel::Android)
    .build();
```

### Rule 3: DeviceStatus は predicate メソッドで検証

```rust
// ❌ Stopped → Offline リネームで 195 箇所壊れる
assert_eq!(device.status, DeviceStatus::Stopped);

// ✅ predicate impl のみ更新で済む
assert!(device.status().is_stopped());
```

### Rule 4: Mock は結果を assert、call_history() は原則使わない

```rust
// Mock setup は実装結合だが環境モデリングとして許容
let mock = MockCommandExecutor::new()
    .with_success("adb", &["devices"], "device-1\tdevice\n");

// ✅ 振る舞いを検証
let devices = manager.list_devices().await?;
assert_eq!(devices.len(), 1);
assert_eq!(devices[0].name(), "device-1");

// ❌ どのコマンドが呼ばれたかは検証しない (実装詳細)
// mock.call_history() は mock 自体のテスト以外では使わない
```

### Rule 5: trait contract テストで振る舞い契約を検証

```rust
// 任意の DeviceManager 実装が満たすべき契約
async fn verify_device_manager_contract(manager: &impl DeviceManager) {
    let devices = manager.list_devices().await.unwrap();
    // 存在しないデバイスの削除 → エラー
    assert!(manager.delete_device("nonexistent").await.is_err());
}
```

`tests/support/contract.rs` に配置し、AndroidManager, IosManager, MockDeviceManager すべてに対して実行。

### ルール適用後の影響

| シナリオ              | 適用前             | 適用後                         |
| --------------------- | ------------------ | ------------------------------ |
| field → getter        | 40-50 テスト壊れる | 0 (predicate が吸収)           |
| DeviceStatus リネーム | 416+ 壊れる        | ~5 (predicate impl のみ)       |
| App generic 化        | 12-15 壊れる       | ~3 (builder + factory 更新)    |
| android.rs 分割       | 7-10 壊れる        | 0 (mod.rs の re-export)        |
| Mock 戦略変更         | 50-80 壊れる       | ~10 (support/managers.rs のみ) |

---

## 6. 全ファイル統合マップ

空ファイル以外のすべてのテストファイルの移行先。記載のないファイルは空プレースホルダー（削除）。

### ルートレベルのレガシー

| 移行元                    | 移行先                                                          | 操作                                          |
| ------------------------- | --------------------------------------------------------------- | --------------------------------------------- |
| `android_manager_test.rs` | `src/managers/android.rs` (inline) + `tests/android_manager.rs` | パースは inline、ライフサイクルは integration |
| `ios_manager_test.rs`     | `src/managers/ios.rs` (inline) + `tests/ios_manager.rs`         | 同上                                          |
| `app_state_test.rs`       | `src/app/state.rs` (inline)                                     | 純粋な state ロジック                         |
| `app_test.rs`             | `tests/app_lifecycle.rs`                                        | App instance 必要                             |
| `ui_render_test.rs`       | `tests/ui_rendering.rs`                                         | MockBackend 必要                              |

### tests/unit/ → src/ inline

| 移行元                                  | 移行先                                                         | 操作               |
| --------------------------------------- | -------------------------------------------------------------- | ------------------ |
| `unit/app/mod_test.rs`                  | `tests/app_lifecycle.rs`                                       | App instance 必要  |
| `unit/app/state_test.rs`                | `src/app/state.rs` (inline)                                    | 純粋ロジック       |
| `unit/app/events_test.rs`               | `src/app/events.rs` (inline)                                   | キーマッピング     |
| `unit/app/event_processing_test.rs`     | `src/app/event_processing.rs` (inline)                         | 純粋ロジック       |
| `unit/managers/android_test.rs`         | `src/managers/android.rs` (inline)                             | パーステスト       |
| `unit/managers/ios_test.rs`             | `src/managers/ios.rs` (inline)                                 | パーステスト       |
| `unit/models/device_test.rs`            | `src/models/device.rs` (inline)                                | モデルロジック     |
| `unit/models/device_validation_test.rs` | `src/models/device.rs` (inline)                                | バリデーション     |
| `unit/models/error_test.rs`             | `src/models/error.rs` (inline)                                 | エラーフォーマット |
| `unit/ui/render_test.rs`                | `tests/ui_rendering.rs`                                        | MockBackend 必要   |
| `unit/ui/render_helper_test.rs`         | `tests/ui_rendering.rs`                                        | MockBackend 必要   |
| `unit/utils/command_test.rs`            | `src/utils/command.rs` (inline)                                | 純粋ロジック       |
| `unit/utils/validation_test.rs`         | `src/utils/validation.rs` (inline)                             | 純粋ロジック       |
| `unit/utils/non_utf8_path_test.rs`      | `src/utils/command.rs` (inline)                                | エッジケース       |
| `unit/constants/*.rs` (8 files)         | ~10 件を `src/constants/defaults.rs` に残し **153 テスト削除** | 関係性不変条件のみ |

### tests/integration/ → 統合

| 移行元                                | 移行先                                              | 操作                     |
| ------------------------------------- | --------------------------------------------------- | ------------------------ |
| `android_manager_integration_test.rs` | `tests/android_manager.rs`                          | 統合                     |
| `android_error_handling_test.rs`      | `tests/error_recovery.rs`                           | 横断的エラー処理         |
| `android_parsing_test.rs`             | `src/managers/android.rs` (inline)                  | 純粋パース               |
| `ios_manager_integration_test.rs`     | `tests/ios_manager.rs`                              | 統合                     |
| `app_mod_integration_test.rs`         | `tests/app_lifecycle.rs`                            | 統合                     |
| `app_main_logic_test.rs`              | `tests/app_lifecycle.rs`                            | 統合                     |
| `app_realistic_test.rs`               | `tests/app_lifecycle.rs`                            | 統合                     |
| `app_state_concurrency_test.rs`       | `tests/app_lifecycle.rs` (並行セクション)           | 統合                     |
| `app_fixture_test.rs`                 | `tests/app_lifecycle.rs`                            | 統合                     |
| `ui_render_test.rs`                   | `tests/ui_rendering.rs`                             | 統合                     |
| `ui_focus_theme_test.rs`              | `tests/ui_rendering.rs`                             | 統合                     |
| `panel_switching_test.rs`             | `tests/app_lifecycle.rs`                            | パネルロジック           |
| `platform_switching_test.rs`          | `tests/app_lifecycle.rs`                            | プラットフォーム切替     |
| `device_creation_test.rs`             | `tests/device_creation.rs`                          | 統合                     |
| `device_creation_navigation_test.rs`  | `tests/device_creation.rs`                          | 統合                     |
| `device_lifecycle_test.rs`            | `tests/android_manager.rs` / `tests/ios_manager.rs` | プラットフォーム別に分割 |
| `device_lifecycle_models_test.rs`     | `src/models/device.rs` (inline)                     | 純粋モデルロジック       |
| `device_operations_test.rs`           | `tests/android_manager.rs` / `tests/ios_manager.rs` | プラットフォーム別に分割 |
| `device_sync_test.rs`                 | `tests/app_lifecycle.rs`                            | 状態同期                 |
| `cache_background_test.rs`            | `tests/app_lifecycle.rs` (キャッシュセクション)     | 統合                     |
| `error_recovery_test.rs`              | `tests/error_recovery.rs`                           | 統合                     |
| `log_streaming_test.rs`               | `tests/app_lifecycle.rs` (ログセクション)           | 統合                     |
| `notification_test.rs`                | `src/app/state.rs` (inline)                         | 純粋な通知ロジック       |
| `navigation_circular_test.rs`         | `src/app/state.rs` (inline)                         | 純粋なナビゲーション計算 |
| `models_test.rs`                      | `src/models/` (inline、ファイル別に分割)            | 純粋モデルロジック       |
| `utils_command_test.rs`               | `src/utils/command.rs` (inline)                     | 純粋コマンドロジック     |
| `comprehensive_test.rs`               | `tests/app_lifecycle.rs`                            | E2E シナリオ             |
| `constants_test.rs`                   | **削除**                                            | unit/constants/ と重複   |

### tests/performance/ → benchmarks.rs または削除

| 移行元                              | 移行先                               | 操作                                |
| ----------------------------------- | ------------------------------------ | ----------------------------------- |
| `startup_benchmark_test.rs`         | `tests/benchmarks.rs` (#[ignore])    | 保持、App instance 必要             |
| `memory_usage_test.rs`              | `tests/benchmarks.rs` (#[ignore])    | 保持 (相対比較)                     |
| `input_responsiveness_test.rs`      | `tests/benchmarks.rs` (#[ignore])    | 保持                                |
| `responsiveness_validation_test.rs` | `tests/benchmarks.rs` (#[ignore])    | 保持                                |
| `app_mod_test.rs`                   | **削除**                             | unit テストの timing ラッパー重複   |
| `app_state_test.rs`                 | **削除**                             | 同上                                |
| `ui_responsiveness_test.rs`         | **削除**                             | render テストの timing ラッパー重複 |
| `models_device_info_test.rs`        | `src/models/device_info.rs` (inline) | 純粋な優先度計算                    |

### tests/fixtures/ → データ保持、テスト統合

| 移行元                            | 移行先                          | 操作                 |
| --------------------------------- | ------------------------------- | -------------------- |
| `android_manager_fixture_test.rs` | `tests/android_manager.rs`      | fixture テストを統合 |
| `ios_manager_fixture_test.rs`     | `tests/ios_manager.rs`          | fixture テストを統合 |
| `command_utility_fixture_test.rs` | `src/utils/command.rs` (inline) | 純粋ロジック         |
| `fixture_loader.rs`               | `tests/support/fixtures.rs`     | 基盤                 |
| `device_fixtures.rs`              | `tests/support/devices.rs`      | 基盤                 |
| `*.json`                          | `tests/fixtures/*.json`         | **そのまま保持**     |

---

## 7. integration テスト仕様

### tests/android_manager.rs

```
# デバイスライフサイクル
- test_list_devices_with_mixed_states
- test_list_devices_empty
- test_list_devices_malformed_output
- test_start_device_success
- test_start_device_not_found
- test_stop_device_success
- test_stop_device_not_running
- test_create_device_success
- test_create_device_duplicate_name
- test_delete_device_success
- test_wipe_device_success

# 詳細 & クエリ
- test_get_device_details
- test_list_api_levels
- test_parallel_device_listing

# インストールフロー
- test_install_system_image_with_progress
- test_install_system_image_failure
- test_install_system_image_cancellation

# fixture ベース
- test_parse_avd_list_fixture
- test_parse_device_details_fixture
```

### tests/ios_manager.rs

```
# デバイスライフサイクル
- test_list_devices_with_mixed_states
- test_list_devices_empty
- test_start_device_success
- test_stop_device_success
- test_create_device_success
- test_delete_device_success

# 詳細 & クエリ
- test_list_runtimes
- test_list_device_types
- test_get_device_details
- test_simulator_app_lifecycle

# エラーハンドリング (Android とのパリティ)
- test_xcode_not_installed
- test_simctl_command_failure
- test_invalid_device_udid

# fixture ベース
- test_parse_simctl_list_fixture
```

### tests/app_lifecycle.rs

```
# 初期化
- test_app_initialization

# ナビゲーション
- test_panel_switching
- test_platform_switching_android_to_ios
- test_device_navigation_up_down
- test_device_navigation_page_up_down

# モード遷移
- test_mode_transitions_normal_to_create
- test_mode_transitions_normal_to_help

# 通知
- test_notification_creation_and_display
- test_notification_auto_dismiss

# ログ
- test_log_streaming_and_filtering
- test_log_clear

# バックグラウンド操作
- test_background_device_refresh
- test_cache_staleness_detection
- test_concurrent_cache_updates

# 並行処理
- test_concurrent_panel_switching
- test_concurrent_device_selection

# E2E シナリオ
- test_full_session_workflow
```

### tests/ui_rendering.rs

```
- test_render_normal_mode
- test_render_with_devices
- test_render_create_device_dialog
- test_render_confirm_delete_dialog
- test_render_api_level_dialog
- test_render_api_level_dialog_installing
- test_render_help_dialog
- test_render_notification
- test_render_minimal_terminal
- test_render_device_details_panel
- test_render_focused_vs_unfocused_panels
- test_render_theme_colors
```

### tests/device_creation.rs

```
- test_create_android_device_full_flow
- test_create_ios_device_full_flow
- test_create_device_validation_errors
- test_create_device_cancel
- test_field_navigation
```

### tests/api_level_mgmt.rs

```
- test_open_api_level_dialog
- test_install_progress_reaches_100
- test_install_prevents_stale_callbacks
- test_is_busy_during_install
- test_close_blocked_while_busy
- test_uninstall_flow
```

### tests/error_recovery.rs

```
- test_sdk_not_found_graceful_degradation
- test_command_timeout_retry
- test_partial_device_list_failure
- test_concurrent_operation_failure_isolation
- test_error_notification_on_operation_failure
- test_recovery_after_transient_error
```

### tests/benchmarks.rs

すべて `#[ignore]` — 手動実行用。CI では実行しない。

```
#[ignore] test_startup_time
#[ignore] test_memory_usage_under_load
#[ignore] test_input_responsiveness
#[ignore] test_cache_performance_relative  # 相対比較、絶対閾値なし
```

---

## 8. inline テストのガイドライン

### inline にするもの (`#[cfg(test)]`)

- **パース**: 文字列 → 構造体変換 (AVD リスト、simctl JSON、API level)
- **バリデーション**: 名前サニタイズ、数値範囲、フォームバリデータ
- **状態遷移**: Mode 切替、Panel toggling、ナビゲーション計算、通知ロジック
- **純粋関数**: 優先度計算、エラーフォーマット、デバイスカテゴリ分類
- **builder パターン**: DeviceConfig、MockCommandExecutor、バリデータ
- **定数の不変条件**: ~10 件の関係性テスト (MIN < MAX、デフォルト値が範囲内)

### inline にしないもの

- **App instance テスト**: concrete type → mock SDK setup 必要 → `tests/`
- **UI レンダリング**: MockBackend 必要 → `tests/`
- **非同期タスク連携**: マルチタスクパターン → `tests/`
- **環境変数操作テスト**: スレッドセーフティリスク → `tests/` + `RUST_TEST_THREADS=1`
- **fixture ファイル読込テスト**: `include_str!` パス問題 → `fs::read_to_string` 使用

---

## 9. 実装手順

### Phase 1: Predicate Methods 追加 (src/ のみ変更)

テストが直接参照するフィールド/enum を predicate で抽象化。既存コードは壊れない（追加のみ）。

**1a. AppState predicates** (`src/app/state.rs`)

```rust
// Mode predicates (49 箇所の直接比較を置換)
pub fn is_normal_mode(&self) -> bool { self.mode == Mode::Normal }
pub fn is_create_mode(&self) -> bool { self.mode == Mode::CreateDevice }
pub fn is_help_mode(&self) -> bool { self.mode == Mode::Help }
pub fn is_confirm_delete_mode(&self) -> bool { self.mode == Mode::ConfirmDelete }
pub fn is_confirm_wipe_mode(&self) -> bool { self.mode == Mode::ConfirmWipe }
pub fn is_api_level_mode(&self) -> bool { self.mode == Mode::ManageApiLevels }

// Panel predicates
pub fn is_android_panel(&self) -> bool { self.active_panel == Panel::Android }
pub fn is_ios_panel(&self) -> bool { self.active_panel == Panel::Ios }

// Device accessors (74+ 箇所の直接ベクタ操作を置換)
pub fn android_device_count(&self) -> usize { self.android_devices.len() }
pub fn ios_device_count(&self) -> usize { self.ios_devices.len() }
pub fn selected_android_device(&self) -> Option<&AndroidDevice> {
    self.android_devices.get(self.selected_android)
}
pub fn selected_ios_device(&self) -> Option<&IosDevice> {
    self.ios_devices.get(self.selected_ios)
}
```

**1b. DeviceStatus predicates** (`src/models/device.rs`)

```rust
impl DeviceStatus {
    pub fn is_running(&self) -> bool { matches!(self, Self::Running) }
    pub fn is_stopped(&self) -> bool { matches!(self, Self::Stopped) }
    pub fn is_error(&self) -> bool { matches!(self, Self::Error) }
    pub fn is_transitioning(&self) -> bool {
        matches!(self, Self::Starting | Self::Stopping | Self::Creating)
    }
}
```

**確認**: `cargo test` + `cargo clippy`

### Phase 2: テスト基盤作成 (`tests/support/`)

**2a.** `tests/support/` ディレクトリ + 全モジュール作成

**2b. TestStateBuilder** (`tests/support/state.rs`)

```rust
pub struct TestStateBuilder { state: AppState }

impl TestStateBuilder {
    pub fn new() -> Self { ... }
    pub fn with_android_devices(mut self, devices: Vec<AndroidDevice>) -> Self { ... }
    pub fn with_ios_devices(mut self, devices: Vec<IosDevice>) -> Self { ... }
    pub fn in_mode(mut self, mode: Mode) -> Self { ... }
    pub fn on_panel(mut self, panel: Panel) -> Self { ... }
    pub fn selecting_android(mut self, index: usize) -> Self { ... }
    pub fn selecting_ios(mut self, index: usize) -> Self { ... }
    pub fn with_api_level_management(mut self, mgmt: ApiLevelManagementState) -> Self { ... }
    /// 低頻度フィールド用エスケープハッチ (device_cache, log_task_handle 等)
    pub fn with_raw(mut self, f: impl FnOnce(&mut AppState)) -> Self { f(&mut self.state); self }
    pub fn build(self) -> AppState { self.state }
}
```

**2c. Device factories** (`tests/support/devices.rs`) — 重複する 2 モジュールを統合

**2d. Trait contract tests** (`tests/support/contract.rs`)

**確認**: `cargo test`

### Phase 3: Coverage Baseline 取得

**Phase 4 の前に必ず完了させること。**

```bash
cargo tarpaulin --features test-utils --out Stdout --ignore-tests \
  --exclude-files 'src/main.rs' --exclude-files 'src/bin/*' \
  --exclude-files 'src/app/test_helpers.rs' --exclude-files 'src/managers/mock.rs' \
  --exclude-files '*/tests/*' > docs/coverage_baseline.txt
```

### Phase 4: 空プレースホルダー削除 (25 files + .bak)

```bash
rm tests/integration/device_lifecycle_test.rs.bak
# unit/ 8 files, integration/ 10 files, performance/ 7 files
```

mod.rs の対応する `mod` 宣言も削除。**確認**: `cargo test`

### Phase 5: Constants テスト削除

163 → ~10 件。関係性不変条件のみ `src/constants/defaults.rs` に inline 移動。

```bash
rm -r tests/unit/constants/
```

**確認**: `cargo test` + coverage check

### Phase 6: Unit tests → inline 移動

1 ファイルずつ移動 → テスト実行 → 次へ。
assertions を predicate 方式に書き換え。env var テストは inline 化しない。

完了後: `rm -r tests/unit/`

### Phase 7: Integration tests 統合

7 files + benchmarks.rs を作成。Complete Consolidation Map に従い 1 カテゴリずつ。
TestStateBuilder + predicate assertions 使用、重複除去。

完了後: 旧ディレクトリ・ファイル削除

### Phase 8: 残骸削除 & lib.rs 整理

`tests/lib.rs` は Phase 7 で段階的に `mod` 宣言を削除。最終的に空になった時点で削除。
`tests/fixtures/` の `.rs` ファイル削除、`tests/README.md` 更新/削除。

### Phase 9: CI 修正 & 最終検証

#### CI 影響分析 (`.github/workflows/ci.yml`)

| Job                | コマンド                                              | 影響                                                | 対応         |
| ------------------ | ----------------------------------------------------- | --------------------------------------------------- | ------------ |
| check              | `cargo check`, `cargo fmt`, `cargo clippy`            | なし                                                | 不要         |
| test (unit)        | `cargo test --bins --lib` (L84)                       | なし。inline テスト増加で自動的にカバー範囲拡大     | 不要         |
| test (integration) | `cargo test --features test-utils --tests` (L92)      | なし。`--tests` は tests/ 直下の全 `.rs` を自動検出 | 不要         |
| build              | `cargo build --release`                               | なし                                                | 不要         |
| coverage           | `cargo llvm-cov ... --features test-utils` (L178)     | なし。`tests/` を除外しており構造変更の影響なし     | 不要         |
| **security**       | `cargo test --test lib comprehensive_test` **(L218)** | **🚨 壊れる**                                       | **修正必要** |

#### security job が壊れる理由

```yaml
# 現在の CI (L218)
cargo test --test lib comprehensive_test --verbose --features test-utils
```

- `--test lib` → `tests/lib.rs` をテストバイナリとして指定
- `comprehensive_test` → そのバイナリ内のテスト名フィルタ
- パス: `tests/lib.rs` → `mod integration` → `mod comprehensive_test` (5 テスト)

Phase 7 で `comprehensive_test.rs` を `tests/app_lifecycle.rs` に統合、
Phase 8 で `tests/lib.rs` を削除するため、このコマンドは壊れる。

#### 修正内容

```yaml
# BEFORE (壊れる)
cargo test --test lib comprehensive_test --verbose --features test-utils

# AFTER (修正)
cargo test --test app_lifecycle comprehensive --verbose --features test-utils
```

`comprehensive_test` 内のテスト関数名に `comprehensive` を含めることで、
`--test app_lifecycle comprehensive` のフィルタで同じテストを実行可能。

#### tests/lib.rs 削除時の注意

`tests/lib.rs` は現在全テストを 1 バイナリにまとめている。
Phase 7 で新しい top-level `tests/*.rs` を追加しつつ `lib.rs` の `mod` を段階的に削除しないと、
同じテストが 2 重実行される（lib.rs 経由 + top-level 経由）。
**Phase 7 の各コミットで対応する `mod` 宣言を必ず削除する。**

#### 最終検証チェックリスト

```bash
cargo test --bins --tests --features test-utils
cargo clippy --all-targets --all-features -- -D warnings
cargo tarpaulin ... > docs/coverage_after.txt
```

- **各モジュールの line coverage が 2% 以上低下しないこと** (現在: 41.45%)
- `CLAUDE.md` Testing セクション更新 (`tests/support/` 追記)
- `docs/TESTING.md` 全面更新
- `test-utils` feature flag が integration tests から引き続きアクセス可能であること確認
- `.github/workflows/ci.yml` の security job コマンド修正済みであること確認

---

## 10. 作業見積もり

| Phase | 内容                   | 規模                | 依存           |
| ----- | ---------------------- | ------------------- | -------------- |
| 1     | Predicate methods 追加 | 小 (~50 行)         | なし           |
| 2     | tests/support/ 作成    | 中 (~500 行)        | Phase 1        |
| 3     | Coverage baseline      | 小                  | なし           |
| 4     | 空プレースホルダー削除 | 小                  | Phase 3 完了後 |
| 5     | Constants テスト削除   | 小                  | Phase 2        |
| 6     | Unit → inline 移動     | **大** (~150 tests) | Phase 1, 2     |
| 7     | Integration 統合       | **大** (~280 tests) | Phase 2, 6     |
| 8     | 残骸削除               | 小                  | Phase 7        |
| 9     | 最終検証               | 小                  | Phase 8        |

**クリティカルパス**: 1 → 2 → 6 → 7 → 8 → 9
**並行可能**: Phase 3, 4 は他と並行

---

## 11. ブランチ戦略

```
main
 └─ feat/test-refactoring
     ├─ "refactor: add predicate methods to AppState and DeviceStatus"
     ├─ "test: create tests/support/ infrastructure"
     ├─ "test: remove 25 empty placeholder test files"
     ├─ "test: remove constants tests, keep 10 invariants inline"
     ├─ "test: move unit/app/ tests inline to src/app/"
     ├─ "test: move unit/managers/ tests inline to src/managers/"
     ├─ "test: move unit/models/ tests inline to src/models/"
     ├─ "test: move unit/utils/ tests inline to src/utils/"
     ├─ "test: consolidate android manager integration tests"
     ├─ "test: consolidate ios manager integration tests"
     ├─ "test: consolidate app lifecycle integration tests"
     ├─ "test: consolidate UI rendering integration tests"
     ├─ "test: consolidate device creation & api level tests"
     ├─ "test: add error recovery and benchmark tests"
     ├─ "test: remove old test directories and lib.rs"
     ├─ "ci: update security job for new test structure"
     └─ "docs: update TESTING.md and CLAUDE.md for new test structure"
```

各コミット後に `cargo test --bins --tests --features test-utils` pass を確認。

---

## 12. 期待される成果

| 指標                    | 現在   | 目標        | 変化     |
| ----------------------- | ------ | ----------- | -------- |
| テストファイル (tests/) | 109    | ~18         | -83%     |
| 空ファイル              | 25     | 0           | -100%    |
| テスト関数 (推定)       | 1,029  | ~400-500    | -50%     |
| テスト行数 (推定)       | 31,174 | ~12,000     | -60%     |
| 実質的カバレッジ        | 維持   | 維持        | 変化なし |
| CI テスト時間           | ~90s   | ~50s (推定) | -44%     |
| リファクタリング耐性    | 低     | 高          | 大幅改善 |

テスト数を減らすこと自体が目的ではない。重複とノイズを除去し、残る各テストが実際のバグクラスを捕捉できる状態にすることが目的。

---

## 13. Codex レビュー結果

### TEST_REDESIGN.md (v2): APPROVE

10 件の required changes をすべて反映済み:

1. 未マッピング 14 files (115 tests) → 完全統合マップ追加
2. App concrete type 制約 → 文書化、event loop テストは tests/ に残す
3. iOS error tests 欠落 → xcode_not_installed, simctl_failure, invalid_udid 追加
4. Coverage baseline なし → Phase 0 追加
5. Env var race condition → RUST_TEST_THREADS=1 要件を記載
6. include_str! パス破損 → runtime fs::read_to_string 使用
7. テストバイナリ構成 → top-level tests/\*.rs (並列バイナリ) に決定
8. コンパイル時間トレードオフ → 制約セクションに記載
9. Constants 関係性テスト → ~10 件保持、153 件削除
10. error_recovery.rs 欠落 → 7 番目の integration file として追加

### TEST_IMPLEMENTATION_PLAN.md: APPROVE (6 件の required changes 反映済み)

1. `device_lifecycle_test.rs.bak` → Phase 4 削除リストに追加
2. `with_raw()` エスケープハッチ → TestStateBuilder に追加
3. Env var テスト → inline 移動禁止を明記、`serial_test` crate 推奨
4. `tests/README.md` + `CLAUDE.md` → 更新チェックリストに追加
5. Coverage regression threshold → モジュール単位 2% 以内
6. `tests/lib.rs` → 段階的 refactor、big-bang 削除はしない
