//! Utils/Command comprehensive tests using fixture data
//!
//! These tests validate command execution utilities using controlled outputs
//! and error scenarios, ensuring robust command handling.

use anyhow::Result;
use std::collections::HashMap;
use std::process::Output;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

mod fixtures;
use fixtures::FixtureLoader;

/// Mock command executor for testing
struct MockCommandExecutor {
    command_outputs: HashMap<String, Vec<(i32, String, String)>>, // Vec of (exit_code, stdout, stderr) for retries
    execution_count: HashMap<String, usize>,
    delays: HashMap<String, Duration>,
}

impl MockCommandExecutor {
    fn new() -> Self {
        Self {
            command_outputs: HashMap::new(),
            execution_count: HashMap::new(),
            delays: HashMap::new(),
        }
    }

    fn with_output(mut self, command: &str, exit_code: i32, stdout: &str, stderr: &str) -> Self {
        self.command_outputs.insert(
            command.to_string(),
            vec![(exit_code, stdout.to_string(), stderr.to_string())],
        );
        self
    }

    fn with_retry_outputs(mut self, command: &str, outputs: Vec<(i32, String, String)>) -> Self {
        self.command_outputs.insert(command.to_string(), outputs);
        self
    }

    fn with_delay(mut self, command: &str, delay: Duration) -> Self {
        self.delays.insert(command.to_string(), delay);
        self
    }

    async fn execute(&mut self, command: &str) -> Result<Output> {
        // Track execution count
        let count = self.execution_count.entry(command.to_string()).or_insert(0);
        *count += 1;

        // Apply delay if configured
        if let Some(delay) = self.delays.get(command) {
            tokio::time::sleep(*delay).await;
        }

        // Return configured output
        if let Some(outputs) = self.command_outputs.get(command) {
            let current_count = *count - 1; // count was already incremented
            let output_index = std::cmp::min(current_count, outputs.len() - 1);
            let (exit_code, stdout, stderr) = &outputs[output_index];
            #[cfg(unix)]
            let status = if *exit_code == 0 {
                std::process::ExitStatus::from_raw(0)
            } else {
                std::process::ExitStatus::from_raw(*exit_code << 8)
            };

            #[cfg(not(unix))]
            let status = {
                // For non-Unix platforms, we'll simulate the exit status
                // This is a simplified approach for testing
                use std::process::{Command, Stdio};
                if *exit_code == 0 {
                    Command::new("true")
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .unwrap()
                } else {
                    Command::new("false")
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .unwrap()
                }
            };

            Ok(Output {
                status,
                stdout: stdout.as_bytes().to_vec(),
                stderr: stderr.as_bytes().to_vec(),
            })
        } else {
            Err(anyhow::anyhow!("Command not found: {}", command))
        }
    }

    fn get_execution_count(&self, command: &str) -> usize {
        self.execution_count.get(command).copied().unwrap_or(0)
    }
}

#[tokio::test]
async fn test_successful_command_execution() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let avd_list_output = loader
        .get_string(
            "android_outputs.json",
            &["avdmanager_list_avd", "single_device"],
        )?
        .unwrap_or_default();

    let mut executor =
        MockCommandExecutor::new().with_output("avdmanager list avd", 0, &avd_list_output, "");

    let output = executor.execute("avdmanager list avd").await?;

    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
    assert!(String::from_utf8(output.stdout)?.contains("Pixel_7_API_34"));

    Ok(())
}

#[tokio::test]
async fn test_command_execution_with_error() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let license_error = loader
        .get_string(
            "error_scenarios.json",
            &["android_errors", "license_not_accepted", "stderr"],
        )?
        .unwrap_or_default();

    let mut executor =
        MockCommandExecutor::new().with_output("avdmanager create avd", 1, "", &license_error);

    let output = executor.execute("avdmanager create avd").await?;

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)?.contains("have not been accepted"));

    Ok(())
}

#[tokio::test]
async fn test_command_retry_mechanism() -> Result<()> {
    let mut executor = MockCommandExecutor::new().with_retry_outputs(
        "unstable_command",
        vec![
            (1, "".to_string(), "Temporary failure".to_string()),
            (1, "".to_string(), "Temporary failure".to_string()),
            (0, "Success".to_string(), "".to_string()),
        ],
    );

    // Simulate multiple executions for retry testing
    let first_result = executor.execute("unstable_command").await;
    assert!(first_result.is_ok());
    assert!(!first_result.unwrap().status.success());

    let second_result = executor.execute("unstable_command").await;
    assert!(second_result.is_ok());
    assert!(!second_result.unwrap().status.success());

    let third_result = executor.execute("unstable_command").await;
    assert!(third_result.is_ok());
    assert!(third_result.unwrap().status.success());

    assert_eq!(executor.get_execution_count("unstable_command"), 3);

    Ok(())
}

#[tokio::test]
async fn test_command_timeout_handling() -> Result<()> {
    let mut executor = MockCommandExecutor::new()
        .with_output("slow_command", 0, "Eventually succeeds", "")
        .with_delay("slow_command", Duration::from_millis(200));

    // Test with short timeout (should timeout)
    let timeout_result = timeout(Duration::from_millis(50), executor.execute("slow_command")).await;

    assert!(timeout_result.is_err()); // Should timeout

    // Test with longer timeout (should succeed)
    let success_result =
        timeout(Duration::from_millis(300), executor.execute("slow_command")).await;

    assert!(success_result.is_ok());
    assert!(success_result.unwrap().is_ok());

    Ok(())
}

#[tokio::test]
async fn test_ignore_errors_functionality() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let error_output = loader
        .get_string(
            "error_scenarios.json",
            &["android_errors", "adb_device_offline", "stderr"],
        )?
        .unwrap_or_default();

    let mut executor =
        MockCommandExecutor::new().with_output("adb shell getprop", 1, "", &error_output);

    // When ignoring errors, should not panic or fail
    let result = executor.execute("adb shell getprop").await?;

    // Even though command failed, we can still process the result
    assert!(!result.status.success());
    assert!(String::from_utf8(result.stderr)?.contains("device offline"));

    Ok(())
}

#[tokio::test]
async fn test_multiple_command_scenarios() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Load multiple fixture outputs
    let avd_list = loader
        .get_string(
            "android_outputs.json",
            &["avdmanager_list_avd", "multiple_devices"],
        )?
        .unwrap_or_default();

    let adb_devices = loader
        .get_string("android_outputs.json", &["adb_devices", "multiple_devices"])?
        .unwrap_or_default();

    let system_images = loader
        .get_string(
            "android_outputs.json",
            &["sdkmanager_list", "system_images"],
        )?
        .unwrap_or_default();

    let mut executor = MockCommandExecutor::new()
        .with_output("avdmanager list avd", 0, &avd_list, "")
        .with_output("adb devices", 0, &adb_devices, "")
        .with_output("sdkmanager --list", 0, &system_images, "");

    // Test sequence of commands
    let avd_result = executor.execute("avdmanager list avd").await?;
    assert!(avd_result.status.success());
    assert!(String::from_utf8(avd_result.stdout)?.contains("Pixel_7_API_34"));

    let adb_result = executor.execute("adb devices").await?;
    assert!(adb_result.status.success());
    assert!(String::from_utf8(adb_result.stdout)?.contains("emulator-"));

    let sdk_result = executor.execute("sdkmanager --list").await?;
    assert!(sdk_result.status.success());
    assert!(String::from_utf8(sdk_result.stdout)?.contains("system-images"));

    Ok(())
}

#[tokio::test]
async fn test_ios_command_scenarios() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let ios_devices = loader
        .get_string(
            "ios_outputs.json",
            &["xcrun_simctl_list_devices", "single_runtime"],
        )?
        .unwrap_or_default();

    let ios_runtimes = loader
        .get_string(
            "ios_outputs.json",
            &["xcrun_simctl_list_runtimes", "comprehensive"],
        )?
        .unwrap_or_default();

    let mut executor = MockCommandExecutor::new()
        .with_output("xcrun simctl list devices", 0, &ios_devices, "")
        .with_output("xcrun simctl list runtimes", 0, &ios_runtimes, "");

    // Test iOS commands
    let devices_result = executor.execute("xcrun simctl list devices").await?;
    assert!(devices_result.status.success());
    assert!(String::from_utf8(devices_result.stdout)?.contains("iPhone"));

    let runtimes_result = executor.execute("xcrun simctl list runtimes").await?;
    assert!(runtimes_result.status.success());
    assert!(String::from_utf8(runtimes_result.stdout)?.contains("iOS"));

    Ok(())
}

#[tokio::test]
async fn test_error_recovery_patterns() -> Result<()> {
    let _loader = FixtureLoader::new();

    // Test network error recovery
    let network_error = "Warning: Failed to download package-list from https://dl.google.com/android/repository/repository2-3.xml";

    let mut executor = MockCommandExecutor::new().with_output(
        "sdkmanager --list",
        0,
        "Cached content",
        network_error,
    );

    let result = executor.execute("sdkmanager --list").await?;

    // Command should succeed with cached content despite network warning
    assert!(result.status.success());
    assert!(String::from_utf8(result.stdout)?.contains("Cached content"));
    assert!(String::from_utf8(result.stderr)?.contains("Failed to download"));

    Ok(())
}

#[tokio::test]
async fn test_command_argument_parsing() -> Result<()> {
    let mut executor = MockCommandExecutor::new()
        .with_output(
            "adb -s emulator-5554 shell getprop",
            0,
            "Pixel_7_API_34",
            "",
        )
        .with_output("xcrun simctl boot device-uuid", 0, "", "")
        .with_output("emulator -avd Pixel_7_API_34 -no-window", 0, "", "");

    // Test commands with different argument patterns
    let adb_result = executor
        .execute("adb -s emulator-5554 shell getprop")
        .await?;
    assert!(adb_result.status.success());

    let ios_result = executor.execute("xcrun simctl boot device-uuid").await?;
    assert!(ios_result.status.success());

    let emulator_result = executor
        .execute("emulator -avd Pixel_7_API_34 -no-window")
        .await?;
    assert!(emulator_result.status.success());

    Ok(())
}

#[tokio::test]
async fn test_concurrent_command_execution() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let avd_list = loader
        .get_string(
            "android_outputs.json",
            &["avdmanager_list_avd", "single_device"],
        )?
        .unwrap_or_default();

    let adb_devices = loader
        .get_string("android_outputs.json", &["adb_devices", "single_device"])?
        .unwrap_or_default();

    let mut executor1 = MockCommandExecutor::new()
        .with_output("avdmanager list avd", 0, &avd_list, "")
        .with_delay("avdmanager list avd", Duration::from_millis(50));

    let mut executor2 = MockCommandExecutor::new()
        .with_output("adb devices", 0, &adb_devices, "")
        .with_delay("adb devices", Duration::from_millis(30));

    // Execute commands concurrently
    let (result1, result2) = tokio::join!(
        executor1.execute("avdmanager list avd"),
        executor2.execute("adb devices")
    );

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let output1 = result1.unwrap();
    let output2 = result2.unwrap();

    assert!(output1.status.success());
    assert!(output2.status.success());
    assert!(String::from_utf8(output1.stdout)?.contains("Pixel_7_API_34"));
    assert!(String::from_utf8(output2.stdout)?.contains("emulator-5554"));

    Ok(())
}
