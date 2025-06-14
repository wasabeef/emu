use emu::app::state::AppState;
use emu::models::{AndroidDevice, DeviceStatus};
use std::time::{Duration, Instant};

#[test]
fn test_rapid_navigation_performance() {
    println!("=== RAPID NAVIGATION PERFORMANCE TEST ===");

    let mut state = AppState::new();

    // Add 100 devices to test with
    for i in 0..100 {
        state.android_devices.push(AndroidDevice {
            name: format!("Device_{}", i),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
            is_physical: false,
        });
    }

    // Test rapid navigation with move_up/move_down
    let start = Instant::now();
    for _ in 0..1000 {
        state.move_down();
    }
    let duration = start.elapsed();
    println!("1000 move_down operations took: {:?}", duration);
    assert!(duration < Duration::from_millis(100), "Navigation too slow");

    // Test rapid navigation with move_by_steps
    let start = Instant::now();
    state.move_by_steps(1000);
    let duration = start.elapsed();
    println!("move_by_steps(1000) took: {:?}", duration);
    assert!(
        duration < Duration::from_millis(10),
        "Batch navigation too slow"
    );

    // Test circular wrapping
    assert_eq!(state.selected_android, 0); // Should wrap around to 0
}

#[test]
fn test_navigation_debouncing() {
    println!("=== NAVIGATION DEBOUNCING TEST ===");

    let mut state = AppState::new();

    // Add some devices
    for i in 0..10 {
        state.android_devices.push(AndroidDevice {
            name: format!("Device_{}", i),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
            is_physical: false,
        });
    }

    // Test that rapid movements are efficient
    let start = Instant::now();
    let mut last_time = start;
    let mut intervals = Vec::new();

    for _ in 0..100 {
        state.move_down();
        let now = Instant::now();
        intervals.push(now.duration_since(last_time));
        last_time = now;

        // Simulate rapid key presses
        std::thread::sleep(Duration::from_micros(100));
    }

    let total_time = start.elapsed();
    println!("100 navigation events took: {:?}", total_time);

    // Calculate average interval
    let avg_interval = intervals.iter().sum::<Duration>() / intervals.len() as u32;
    println!("Average interval between moves: {:?}", avg_interval);

    // Should handle rapid navigation efficiently
    assert!(total_time < Duration::from_millis(200));
}

#[test]
fn test_batch_navigation_efficiency() {
    println!("=== BATCH NAVIGATION EFFICIENCY TEST ===");

    let mut state = AppState::new();

    // Add 1000 devices for stress testing
    for i in 0..1000 {
        state.android_devices.push(AndroidDevice {
            name: format!("Device_{}", i),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
            is_physical: false,
        });
    }

    // Test individual moves vs batch moves
    let start = Instant::now();
    for _ in 0..100 {
        state.move_down();
    }
    let individual_time = start.elapsed();

    let start = Instant::now();
    state.move_by_steps(100);
    let batch_time = start.elapsed();

    println!("100 individual moves: {:?}", individual_time);
    println!("Batch move by 100: {:?}", batch_time);
    println!(
        "Speedup: {:.2}x",
        individual_time.as_nanos() as f64 / batch_time.as_nanos() as f64
    );

    // Batch should be significantly faster
    assert!(batch_time < individual_time / 10);
}

#[test]
fn test_move_by_steps_correctness() {
    println!("=== MOVE BY STEPS CORRECTNESS TEST ===");

    let mut state = AppState::new();

    // Add 10 devices
    for i in 0..10 {
        state.android_devices.push(AndroidDevice {
            name: format!("Device_{}", i),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
            is_physical: false,
        });
    }

    // Test positive movement
    state.selected_android = 0;
    state.move_by_steps(3);
    assert_eq!(state.selected_android, 3);

    // Test negative movement
    state.move_by_steps(-2);
    assert_eq!(state.selected_android, 1);

    // Test wrap around forward
    state.selected_android = 8;
    state.move_by_steps(5);
    assert_eq!(state.selected_android, 3); // (8 + 5) % 10 = 3

    // Test wrap around backward
    state.selected_android = 2;
    state.move_by_steps(-5);
    assert_eq!(state.selected_android, 7); // (2 - 5 + 10) % 10 = 7

    // Test large movements
    state.move_by_steps(123);
    assert_eq!(state.selected_android, 0); // (7 + 123) % 10 = 0

    println!("✅ All move_by_steps tests passed");
}
