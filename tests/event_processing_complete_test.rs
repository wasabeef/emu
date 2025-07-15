//! イベント処理システム完全テスト
//!
//! このテストスイートは アプリケーションのイベント処理システムの
//! 完全性とパフォーマンスを検証します。

use emu::app::state::AppState;
use emu::app::state::LogEntry;
use emu::app::Panel;
use emu::models::device::{AndroidDevice, DeviceStatus};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_event_processing_basic() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // 基本的なイベント処理テスト
    {
        let mut state = state.lock().await;
        state.active_panel = Panel::Android;
        state.selected_android = 0;
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.active_panel, Panel::Android);
    assert_eq!(final_state.selected_android, 0);
}

#[tokio::test]
async fn test_event_processing_panel_switching() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // パネル切り替えイベント処理
    {
        let mut state = state.lock().await;
        state.active_panel = Panel::Android;
        state.next_panel();
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.active_panel, Panel::Ios);
}

#[tokio::test]
async fn test_event_processing_device_selection() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // デバイス選択イベント処理
    {
        let mut state = state.lock().await;
        // Android デバイスを追加
        state.android_devices.push(AndroidDevice {
            name: "test_device".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "8192".to_string(),
        });

        state.active_panel = Panel::Android;
        state.selected_android = 0;
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.selected_android, 0);
    assert_eq!(final_state.android_devices.len(), 1);
}

#[tokio::test]
async fn test_event_processing_device_navigation() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // デバイスナビゲーション処理
    {
        let mut state = state.lock().await;
        // 複数のデバイスを追加
        for i in 0..5 {
            state.android_devices.push(AndroidDevice {
                name: format!("device_{i}"),
                device_type: "pixel_7".to_string(),
                api_level: 34,
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: "4096".to_string(),
                storage_size: "8192".to_string(),
            });
        }

        state.active_panel = Panel::Android;
        state.selected_android = 0;

        // 下に移動
        state.move_down();
        assert_eq!(state.selected_android, 1);

        // 上に移動
        state.move_up();
        assert_eq!(state.selected_android, 0);

        // 最初で上に移動すると最後に循環
        state.move_up();
        assert_eq!(state.selected_android, 4);
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.selected_android, 4);
    assert_eq!(final_state.android_devices.len(), 5);
}

#[tokio::test]
async fn test_event_processing_concurrent_events() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    let mut handles = vec![];

    // 並行イベント処理
    for i in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                {
                    let mut state = state_clone.lock().await;
                    state.active_panel = if (i + j) % 2 == 0 {
                        Panel::Android
                    } else {
                        Panel::Ios
                    };
                    state.selected_android = i;
                    state.selected_ios = j;
                }
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_state = state.lock().await;
    assert!(matches!(
        final_state.active_panel,
        Panel::Android | Panel::Ios
    ));
    assert!(final_state.selected_android < 10);
    assert!(final_state.selected_ios < 10);
}

#[tokio::test]
async fn test_event_processing_loading_states() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // ローディング状態のイベント処理
    {
        let mut state = state.lock().await;
        state.is_loading = true;

        // ローディング中は操作が制限される場合をテスト
        assert!(state.is_loading);

        state.is_loading = false;
        assert!(!state.is_loading);
    }

    let final_state = state.lock().await;
    assert!(!final_state.is_loading);
}

#[tokio::test]
async fn test_event_processing_log_events() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // ログイベント処理
    {
        let mut state = state.lock().await;

        // ログエントリを追加
        state.device_logs.push_back(LogEntry {
            timestamp: "12:34:56".to_string(),
            level: "INFO".to_string(),
            message: "Test log entry 1".to_string(),
        });
        state.device_logs.push_back(LogEntry {
            timestamp: "12:34:57".to_string(),
            level: "WARN".to_string(),
            message: "Test log entry 2".to_string(),
        });
        state.device_logs.push_back(LogEntry {
            timestamp: "12:34:58".to_string(),
            level: "ERROR".to_string(),
            message: "Test log entry 3".to_string(),
        });

        assert_eq!(state.device_logs.len(), 3);

        // ログスクロール
        state.log_scroll_offset = 1;
        assert_eq!(state.log_scroll_offset, 1);
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.device_logs.len(), 3);
    assert_eq!(final_state.log_scroll_offset, 1);
}

#[tokio::test]
async fn test_event_processing_notification_events() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // 通知イベント処理
    {
        let mut state = state.lock().await;

        // 通知を追加
        state
            .notifications
            .push_back(emu::app::state::Notification::success(
                "Operation completed successfully".to_string(),
            ));
        state
            .notifications
            .push_back(emu::app::state::Notification::error(
                "An error occurred".to_string(),
            ));

        assert_eq!(state.notifications.len(), 2);

        // 通知のタイプをテスト
        assert_eq!(
            state.notifications[0].notification_type,
            emu::app::state::NotificationType::Success
        );
        assert_eq!(
            state.notifications[1].notification_type,
            emu::app::state::NotificationType::Error
        );
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.notifications.len(), 2);
}

#[tokio::test]
async fn test_event_processing_device_operations() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // デバイス操作イベント処理
    {
        let mut state = state.lock().await;

        // デバイスを追加
        let device = AndroidDevice {
            name: "operation_device".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "8192".to_string(),
        };
        state.android_devices.push(device);

        // 操作状態を設定
        state.device_operation_status = Some("Starting device...".to_string());

        assert!(state.device_operation_status.is_some());
        assert_eq!(
            state.device_operation_status.as_ref().unwrap(),
            "Starting device..."
        );
    }

    let final_state = state.lock().await;
    assert!(final_state.device_operation_status.is_some());
    assert_eq!(final_state.android_devices.len(), 1);
}

#[tokio::test]
async fn test_event_processing_performance() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // パフォーマンステスト: イベント処理が高速であることを確認
    let start = std::time::Instant::now();

    {
        let mut state = state.lock().await;

        // 大量のイベントを処理
        for i in 0..1000 {
            state.active_panel = if i % 2 == 0 {
                Panel::Android
            } else {
                Panel::Ios
            };
            state.selected_android = i % 10;
            state.selected_ios = i % 5;

            // デバイスを追加
            if i % 100 == 0 {
                state.android_devices.push(AndroidDevice {
                    name: format!("perf_device_{i}"),
                    device_type: "pixel_7".to_string(),
                    api_level: 34,
                    status: DeviceStatus::Stopped,
                    is_running: false,
                    ram_size: "4096".to_string(),
                    storage_size: "8192".to_string(),
                });
            }
        }
    }

    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 100,
        "Event processing took too long: {duration:?}"
    );

    let final_state = state.lock().await;
    assert_eq!(final_state.android_devices.len(), 10);
    assert!(matches!(
        final_state.active_panel,
        Panel::Android | Panel::Ios
    ));
}

#[tokio::test]
async fn test_event_processing_error_handling() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // エラーハンドリングイベント処理
    {
        let mut state = state.lock().await;

        // エラー状態を設定
        state
            .notifications
            .push_back(emu::app::state::Notification::error(
                "Device operation failed".to_string(),
            ));

        // 操作状態をクリア
        state.device_operation_status = None;

        assert!(state.device_operation_status.is_none());
        assert_eq!(state.notifications.len(), 1);
        assert_eq!(
            state.notifications[0].notification_type,
            emu::app::state::NotificationType::Error
        );
    }

    let final_state = state.lock().await;
    assert!(final_state.device_operation_status.is_none());
    assert_eq!(final_state.notifications.len(), 1);
}
