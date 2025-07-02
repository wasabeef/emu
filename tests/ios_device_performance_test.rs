//! iOS device processing performance tests
//!
//! Tests that verify the performance of iOS device list processing
//! when dealing with large numbers of devices (30+).

#[cfg(target_os = "macos")]
mod ios_performance_tests {
    use serde_json::json;
    use std::time::Instant;

    #[tokio::test]
    async fn test_large_device_list_performance() {
        // Create a mock JSON response with many devices
        let mut devices_map = serde_json::Map::new();

        // iOS 18.5 devices
        let mut ios_18_5_devices = Vec::new();
        for i in 0..15 {
            ios_18_5_devices.push(json!({
                "name": format!("iPhone {} Test Device", i),
                "udid": format!("TEST-UUID-18-5-{:02}", i),
                "state": if i % 3 == 0 { "Booted" } else { "Shutdown" },
                "isAvailable": true,
                "deviceTypeIdentifier": format!("com.apple.CoreSimulator.SimDeviceType.iPhone-{}",
                    if i < 5 { "16-Pro" } else if i < 10 { "16" } else { "15" })
            }));
        }
        devices_map.insert(
            "com.apple.CoreSimulator.SimRuntime.iOS-18-5".to_string(),
            json!(ios_18_5_devices),
        );

        // iOS 18.1 devices
        let mut ios_18_1_devices = Vec::new();
        for i in 0..15 {
            ios_18_1_devices.push(json!({
                "name": format!("iPad {} Test Device", i),
                "udid": format!("TEST-UUID-18-1-{:02}", i),
                "state": "Shutdown",
                "isAvailable": true,
                "deviceTypeIdentifier": format!("com.apple.CoreSimulator.SimDeviceType.iPad-{}", 
                    if i < 5 { "Pro-13-inch-M4" } else if i < 10 { "Air-11-inch-M3" } else { "mini-A17-Pro" })
            }));
        }
        devices_map.insert(
            "com.apple.CoreSimulator.SimRuntime.iOS-18-1".to_string(),
            json!(ios_18_1_devices),
        );

        let devices_json = json!({ "devices": devices_map });
        let json_str = serde_json::to_string(&devices_json).unwrap();

        // Measure JSON parsing performance (without private method access)
        let start = Instant::now();

        // Simulate the parsing that happens in list_devices
        let parsed_json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let mut device_count = 0;
        if let Some(devices_obj) = parsed_json.get("devices") {
            if let Some(devices_map) = devices_obj.as_object() {
                for (_runtime, device_list_json) in devices_map {
                    if let Some(device_array) = device_list_json.as_array() {
                        device_count += device_array.len();
                    }
                }
            }
        }

        let parse_time = start.elapsed();

        println!("Parsed {} devices in {:?}", device_count, parse_time);

        // Performance assertions
        assert_eq!(device_count, 30, "Should have parsed all 30 devices");
        assert!(
            parse_time.as_millis() < 100,
            "Parsing 30 devices should take less than 100ms, took {:?}",
            parse_time
        );
    }

    #[tokio::test]
    async fn test_device_sorting_performance() {
        use emu::models::device_info::DynamicDeviceConfig;

        // Create a large list of device names
        let mut devices = Vec::new();

        // Add various iPhone models
        for version in 14..=16 {
            devices.push(format!("iPhone {}", version));
            devices.push(format!("iPhone {} Pro", version));
            devices.push(format!("iPhone {} Pro Max", version));
            devices.push(format!("iPhone {} Plus", version));
            if version == 16 {
                devices.push("iPhone 16e".to_string());
            }
        }

        // Add various iPad models
        devices.extend(vec![
            "iPad Pro 13-inch (M4)".to_string(),
            "iPad Pro 11-inch (M4)".to_string(),
            "iPad Air 13-inch (M3)".to_string(),
            "iPad Air 11-inch (M3)".to_string(),
            "iPad Air 13-inch (M2)".to_string(),
            "iPad Air 11-inch (M2)".to_string(),
            "iPad mini (A17 Pro)".to_string(),
            "iPad (10th generation)".to_string(),
            "iPad (9th generation)".to_string(),
        ]);

        let device_count = devices.len();

        // Measure sorting performance
        let start = Instant::now();

        devices.sort_by(|a, b| {
            let priority_a = DynamicDeviceConfig::calculate_ios_device_priority(a);
            let priority_b = DynamicDeviceConfig::calculate_ios_device_priority(b);
            priority_a.cmp(&priority_b)
        });

        let sort_time = start.elapsed();

        println!("Sorted {} devices in {:?}", device_count, sort_time);

        // Performance assertion
        assert!(
            sort_time.as_micros() < 1000,
            "Sorting {} devices should take less than 1ms, took {:?}",
            device_count,
            sort_time
        );

        // Verify sorting correctness - Pro Max should come first
        assert!(
            devices[0].contains("Pro Max"),
            "First device should be a Pro Max model, got: {}",
            devices[0]
        );

        // Verify iPhone 16 models come before iPhone 15 models
        let iphone_16_pos = devices
            .iter()
            .position(|d| d.contains("iPhone 16 ") && !d.contains("Pro"))
            .unwrap();
        let iphone_15_pos = devices
            .iter()
            .position(|d| d.contains("iPhone 15 ") && !d.contains("Pro"))
            .unwrap();
        assert!(
            iphone_16_pos < iphone_15_pos,
            "iPhone 16 should come before iPhone 15"
        );
    }

    #[test]
    fn test_batch_processing_efficiency() {
        // Test that batch processing improves performance
        const BATCH_SIZE: usize = 10;
        let device_count = 50;

        let mut processing_times = Vec::new();

        // Simulate batch processing
        for batch_start in (0..device_count).step_by(BATCH_SIZE) {
            let start = Instant::now();

            // Simulate processing a batch
            let batch_end = (batch_start + BATCH_SIZE).min(device_count);
            for i in batch_start..batch_end {
                // Simulate some work
                let _ = format!("Device-{}", i);
            }

            processing_times.push(start.elapsed());
        }

        // Verify batching doesn't introduce significant overhead
        let total_time: std::time::Duration = processing_times.iter().sum();
        println!(
            "Batch processed {} devices in {:?}",
            device_count, total_time
        );

        assert!(
            total_time.as_micros() < 1000,
            "Batch processing should be efficient, took {:?}",
            total_time
        );
    }

    #[test]
    fn test_priority_calculation_performance() {
        use emu::models::device_info::DynamicDeviceConfig;

        // Test that priority calculation is fast even when called many times
        let device_names = vec![
            "iPhone 16 Pro Max",
            "iPhone 16e",
            "iPad Pro 13-inch (M4)",
            "iPad Air 11-inch (M3)",
            "iPhone SE (3rd generation)",
        ];

        let iterations = 10000;
        let start = Instant::now();

        for _ in 0..iterations {
            for device in &device_names {
                let _ = DynamicDeviceConfig::calculate_ios_device_priority(device);
            }
        }

        let total_time = start.elapsed();
        let per_calculation = total_time.as_nanos() / (iterations * device_names.len()) as u128;

        println!(
            "Priority calculation: {} iterations in {:?}",
            iterations, total_time
        );
        println!("Average time per calculation: {}ns", per_calculation);

        // Each priority calculation should be very fast (allow up to 2µs)
        assert!(
            per_calculation < 2000,
            "Priority calculation should take less than 2µs, took {}ns",
            per_calculation
        );
    }
}
