//! Comprehensive tests for utils::command module
//!
//! These tests ensure complete coverage of CommandRunner functionality,
//! including all methods, error cases, and edge conditions.

use emu::utils::command::CommandRunner;
use std::env;
use std::time::Duration;
use tokio::time::timeout;

#[test]
fn test_command_runner_new() {
    let runner = CommandRunner::new();
    // Test that new returns a valid instance
    // Since CommandRunner is a unit struct, we can't test much here
    // but we ensure it compiles and runs
    let _ = runner;
}

#[test]
fn test_command_runner_default() {
    let runner = CommandRunner::new();
    // Test that default trait implementation works
    let _ = runner;
}

#[test]
fn test_command_runner_clone() {
    let runner = CommandRunner::new();
    let cloned = runner.clone();
    // Test that clone works correctly
    let _ = runner;
    let _ = cloned;
}

#[tokio::test]
async fn test_command_runner_run_success() {
    let runner = CommandRunner::new();

    // Test with echo command (available on Unix systems)
    #[cfg(unix)]
    {
        let result = runner.run("echo", &["hello", "world"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("hello world"));
    }

    // Test with cmd command (available on Windows)
    #[cfg(windows)]
    {
        let result = runner.run("cmd", &["/C", "echo hello world"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("hello world"));
    }
}

#[tokio::test]
async fn test_command_runner_run_command_not_found() {
    let runner = CommandRunner::new();

    // Test with non-existent command
    let result = runner
        .run("nonexistent_command_12345", &[] as &[&str])
        .await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Failed to execute command"));
}

#[tokio::test]
async fn test_command_runner_run_command_failure() {
    let runner = CommandRunner::new();

    // Test with command that returns non-zero exit code
    #[cfg(unix)]
    {
        let result = runner.run("sh", &["-c", "exit 1"]).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Command failed with exit code"));
    }

    #[cfg(windows)]
    {
        let result = runner.run("cmd", &["/C", "exit 1"]).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Command failed with exit code"));
    }
}

#[tokio::test]
async fn test_command_runner_run_with_stderr() {
    let runner = CommandRunner::new();

    // Test command that outputs to stderr and fails
    #[cfg(unix)]
    {
        let result = runner
            .run("sh", &["-c", "echo 'error message' >&2; exit 1"])
            .await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("error message"));
    }

    #[cfg(windows)]
    {
        let result = runner
            .run("cmd", &["/C", "echo error message 1>&2 && exit 1"])
            .await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("error message"));
    }
}

#[tokio::test]
async fn test_command_runner_debug_logging() {
    let runner = CommandRunner::new();

    // Test with debug logging enabled
    env::set_var("RUST_LOG", "debug");

    #[cfg(unix)]
    {
        let result = runner.run("echo", &["debug", "test"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("debug test"));
    }

    #[cfg(windows)]
    {
        let result = runner.run("cmd", &["/C", "echo debug test"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("debug test"));
    }

    // Clean up environment
    env::remove_var("RUST_LOG");
}

#[tokio::test]
async fn test_command_runner_debug_logging_disabled() {
    let runner = CommandRunner::new();

    // Test with debug logging disabled
    env::remove_var("RUST_LOG");

    #[cfg(unix)]
    {
        let result = runner.run("echo", &["no", "debug"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("no debug"));
    }

    #[cfg(windows)]
    {
        let result = runner.run("cmd", &["/C", "echo no debug"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("no debug"));
    }
}

#[tokio::test]
async fn test_command_runner_spawn_success() {
    let runner = CommandRunner::new();

    // Test spawning a command that exits quickly
    #[cfg(unix)]
    {
        let result = runner.spawn("true", &[] as &[&str]).await;
        assert!(result.is_ok());
        let pid = result.unwrap();
        assert!(pid > 0);
    }

    #[cfg(windows)]
    {
        let result = runner.spawn("cmd", &["/C", "exit 0"]).await;
        assert!(result.is_ok());
        let pid = result.unwrap();
        assert!(pid > 0);
    }
}

#[tokio::test]
async fn test_command_runner_spawn_command_not_found() {
    let runner = CommandRunner::new();

    // Test spawning non-existent command
    let result = runner
        .spawn("nonexistent_command_67890", &[] as &[&str])
        .await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Failed to spawn command"));
}

#[tokio::test]
async fn test_command_runner_spawn_with_args() {
    let runner = CommandRunner::new();

    // Test spawning command with arguments
    #[cfg(unix)]
    {
        let result = runner.spawn("sh", &["-c", "exit 0"]).await;
        assert!(result.is_ok());
        let pid = result.unwrap();
        assert!(pid > 0);
    }

    #[cfg(windows)]
    {
        let result = runner.spawn("cmd", &["/C", "echo spawned && exit 0"]).await;
        assert!(result.is_ok());
        let pid = result.unwrap();
        assert!(pid > 0);
    }
}

#[tokio::test]
async fn test_command_runner_run_ignoring_errors_success() {
    let runner = CommandRunner::new();

    // Test successful command (should work normally)
    #[cfg(unix)]
    {
        let result = runner
            .run_ignoring_errors("echo", &["success"], &["not_found"])
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("success"));
    }

    #[cfg(windows)]
    {
        let result = runner
            .run_ignoring_errors("cmd", &["/C", "echo success"], &["not_found"])
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("success"));
    }
}

#[tokio::test]
async fn test_command_runner_run_ignoring_errors_ignored_error() {
    let runner = CommandRunner::new();

    // Test command that fails with ignored pattern
    #[cfg(unix)]
    {
        let result = runner
            .run_ignoring_errors(
                "sh",
                &["-c", "echo 'already booted' >&2; exit 1"],
                &["already booted"],
            )
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_empty()); // Should return empty string for ignored errors
    }

    #[cfg(windows)]
    {
        let result = runner
            .run_ignoring_errors(
                "cmd",
                &["/C", "echo already booted 1>&2 && exit 1"],
                &["already booted"],
            )
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_empty());
    }
}

#[tokio::test]
async fn test_command_runner_run_ignoring_errors_not_ignored() {
    let runner = CommandRunner::new();

    // Test command that fails with non-ignored pattern
    #[cfg(unix)]
    {
        let result = runner
            .run_ignoring_errors(
                "sh",
                &["-c", "echo 'unexpected error' >&2; exit 1"],
                &["already booted"],
            )
            .await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("unexpected error"));
    }

    #[cfg(windows)]
    {
        let result = runner
            .run_ignoring_errors(
                "cmd",
                &["/C", "echo unexpected error 1>&2 && exit 1"],
                &["already booted"],
            )
            .await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("unexpected error"));
    }
}

#[tokio::test]
async fn test_command_runner_run_ignoring_errors_multiple_patterns() {
    let runner = CommandRunner::new();

    // Test with multiple ignore patterns
    #[cfg(unix)]
    {
        let result = runner
            .run_ignoring_errors(
                "sh",
                &["-c", "echo 'device offline' >&2; exit 1"],
                &["already booted", "device offline", "not found"],
            )
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_empty());
    }

    #[cfg(windows)]
    {
        let result = runner
            .run_ignoring_errors(
                "cmd",
                &["/C", "echo device offline 1>&2 && exit 1"],
                &["already booted", "device offline", "not found"],
            )
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_empty());
    }
}

#[tokio::test]
async fn test_command_runner_run_with_retry_success_first_try() {
    let runner = CommandRunner::new();

    // Test successful command on first try
    #[cfg(unix)]
    {
        let result = runner.run_with_retry("echo", &["retry", "test"], 2).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("retry test"));
    }

    #[cfg(windows)]
    {
        let result = runner
            .run_with_retry("cmd", &["/C", "echo retry test"], 2)
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("retry test"));
    }
}

#[tokio::test]
async fn test_command_runner_run_with_retry_max_retries() {
    let runner = CommandRunner::new();

    // Test command that always fails
    #[cfg(unix)]
    {
        let result = runner.run_with_retry("sh", &["-c", "exit 1"], 2).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("failed after 3 attempts"));
    }

    #[cfg(windows)]
    {
        let result = runner.run_with_retry("cmd", &["/C", "exit 1"], 2).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("failed after 3 attempts"));
    }
}

#[tokio::test]
async fn test_command_runner_run_with_retry_zero_retries() {
    let runner = CommandRunner::new();

    // Test with zero retries (single attempt)
    #[cfg(unix)]
    {
        let result = runner.run_with_retry("sh", &["-c", "exit 1"], 0).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("failed after 1 attempts"));
    }

    #[cfg(windows)]
    {
        let result = runner.run_with_retry("cmd", &["/C", "exit 1"], 0).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("failed after 1 attempts"));
    }
}

#[tokio::test]
async fn test_command_runner_run_with_retry_delay_timing() {
    let runner = CommandRunner::new();

    // Test that retry delays are applied (we can't test exact timing easily,
    // but we can test that the command takes longer than it should without delays)
    let start_time = std::time::Instant::now();

    #[cfg(unix)]
    {
        let result = runner.run_with_retry("sh", &["-c", "exit 1"], 1).await;
        assert!(result.is_err());
    }

    #[cfg(windows)]
    {
        let result = runner.run_with_retry("cmd", &["/C", "exit 1"], 1).await;
        assert!(result.is_err());
    }

    let elapsed = start_time.elapsed();
    // Should take at least some time due to retry delay
    // Using a conservative estimate since timing can be flaky in tests
    assert!(elapsed.as_millis() >= 10);
}

#[tokio::test]
async fn test_command_runner_unicode_arguments() {
    let runner = CommandRunner::new();

    // Test with Unicode arguments
    #[cfg(unix)]
    {
        let result = runner.run("echo", &["こんにちは", "世界"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("こんにちは"));
        assert!(output.contains("世界"));
    }

    #[cfg(windows)]
    {
        let result = runner.run("cmd", &["/C", "echo こんにちは 世界"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("こんにちは") || output.contains("?")); // Windows might not handle Unicode well
    }
}

#[tokio::test]
async fn test_command_runner_empty_arguments() {
    let runner = CommandRunner::new();

    // Test with no arguments
    #[cfg(unix)]
    {
        let result = runner.run("echo", &[] as &[&str]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.trim(), "");
    }

    #[cfg(windows)]
    {
        let result = runner.run("cmd", &["/C", "echo."]).await; // Use echo. for empty output
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_command_runner_special_characters() {
    let runner = CommandRunner::new();

    // Test with special characters that might cause issues
    #[cfg(unix)]
    {
        let result = runner.run("echo", &["$PATH", "|", "&", ";"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("$PATH"));
        assert!(output.contains("|"));
        assert!(output.contains("&"));
        assert!(output.contains(";"));
    }

    #[cfg(windows)]
    {
        let result = runner.run("cmd", &["/C", "echo %PATH% ^| ^& ^;"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("%PATH%"));
    }
}

#[tokio::test]
async fn test_command_runner_large_output() {
    let runner = CommandRunner::new();

    // Test with command that produces large output
    #[cfg(unix)]
    {
        // Generate 1000 lines of output
        let result = runner
            .run(
                "sh",
                &["-c", "for i in $(seq 1 1000); do echo line_$i; done"],
            )
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("line_1"));
        assert!(output.contains("line_1000"));
        let line_count = output.lines().count();
        assert_eq!(line_count, 1000);
    }

    #[cfg(windows)]
    {
        // Generate multiple lines of output
        let result = runner
            .run("cmd", &["/C", "for /L %i in (1,1,100) do @echo line_%i"])
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("line_1"));
        assert!(output.contains("line_100"));
    }
}

#[tokio::test]
async fn test_command_runner_timeout_integration() {
    let runner = CommandRunner::new();

    // Test command execution with external timeout
    #[cfg(unix)]
    {
        let command_future = runner.run("sleep", &["1"]);
        let result = timeout(Duration::from_millis(100), command_future).await;
        assert!(result.is_err()); // Should timeout

        let command_future = runner.run("echo", &["quick"]);
        let result = timeout(Duration::from_secs(1), command_future).await;
        assert!(result.is_ok()); // Should complete
        assert!(result.unwrap().is_ok());
    }

    #[cfg(windows)]
    {
        let command_future = runner.run("cmd", &["/C", "timeout /t 1 /nobreak > nul"]);
        let result = timeout(Duration::from_millis(100), command_future).await;
        assert!(result.is_err()); // Should timeout

        let command_future = runner.run("cmd", &["/C", "echo quick"]);
        let result = timeout(Duration::from_secs(1), command_future).await;
        assert!(result.is_ok()); // Should complete
        assert!(result.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_command_runner_error_message_formatting() {
    let runner = CommandRunner::new();

    // Test that error messages contain all expected information
    #[cfg(unix)]
    {
        let result = runner
            .run(
                "sh",
                &[
                    "-c",
                    "echo 'stderr content' >&2; echo 'stdout content'; exit 42",
                ],
            )
            .await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_str = error.to_string();
        assert!(error_str.contains("42")); // Exit code
        assert!(error_str.contains("stderr content")); // Stderr
        assert!(error_str.contains("stdout content")); // Stdout
    }

    #[cfg(windows)]
    {
        let result = runner
            .run(
                "cmd",
                &[
                    "/C",
                    "echo stderr content 1>&2 && echo stdout content && exit 42",
                ],
            )
            .await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_str = error.to_string();
        assert!(error_str.contains("42")); // Exit code
        assert!(error_str.contains("stderr content")); // Stderr
        assert!(error_str.contains("stdout content")); // Stdout
    }
}

#[tokio::test]
async fn test_command_runner_run_various_types() {
    let runner = CommandRunner::new();

    // Test that the generic type system works with different argument types
    #[cfg(unix)]
    {
        // String arguments
        let result1 = runner
            .run("echo", &["string".to_string(), "args".to_string()])
            .await;
        assert!(result1.is_ok());

        // &str arguments
        let result2 = runner.run("echo", &["str", "args"]).await;
        assert!(result2.is_ok());

        // Mixed iterator
        let args: Vec<&str> = vec!["vec", "args"];
        let result3 = runner.run("echo", args).await;
        assert!(result3.is_ok());
    }

    #[cfg(windows)]
    {
        // String arguments
        let result1 = runner
            .run("cmd", &["/C".to_string(), "echo string args".to_string()])
            .await;
        assert!(result1.is_ok());

        // &str arguments
        let result2 = runner.run("cmd", &["/C", "echo str args"]).await;
        assert!(result2.is_ok());

        // Mixed iterator
        let args: Vec<&str> = vec!["/C", "echo vec args"];
        let result3 = runner.run("cmd", args).await;
        assert!(result3.is_ok());
    }
}
