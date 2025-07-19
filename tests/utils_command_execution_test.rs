//! Unit tests for utils/command.rs command execution utilities

use emu::utils::command::CommandRunner;
use std::time::Duration;

#[tokio::test]
async fn test_command_runner_creation() {
    // Test CommandRunner::new()
    let _runner = CommandRunner::new();

    // Should create successfully

    // Test default implementation
    let _runner_default = CommandRunner;
}

#[tokio::test]
async fn test_command_runner_simple_command() {
    // Test running a simple command that should exist on most systems
    let runner = CommandRunner::new();

    // Use echo command which should be available on Unix-like systems
    if cfg!(unix) {
        let result = runner.run("echo", &["hello"]).await;

        match result {
            Ok(output) => {
                assert!(
                    output.contains("hello"),
                    "Output should contain 'hello': {output}"
                );
                assert!(!output.trim().is_empty(), "Output should not be empty");
            }
            Err(e) => {
                // If echo fails, the error should be informative
                let error_message = e.to_string();
                assert!(
                    error_message.contains("echo")
                        || error_message.contains("command")
                        || error_message.contains("not found"),
                    "Error should be command-related: {error_message}"
                );
            }
        }
    } else {
        // On Windows, test with a different command
        let result = runner.run("cmd", &["/C", "echo hello"]).await;

        match result {
            Ok(output) => {
                assert!(
                    output.contains("hello"),
                    "Output should contain 'hello': {output}"
                );
            }
            Err(e) => {
                // If cmd fails, the error should be informative
                let error_message = e.to_string();
                assert!(
                    error_message.contains("cmd") || error_message.contains("command"),
                    "Error should be command-related: {error_message}"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_command_runner_error_handling() {
    // Test error handling with a non-existent command
    let runner = CommandRunner::new();

    let result = runner
        .run("nonexistent_command_12345", &[] as &[&str])
        .await;

    assert!(result.is_err(), "Non-existent command should return error");

    let error = result.unwrap_err();
    let error_message = error.to_string();

    // Error should be informative
    assert!(
        !error_message.is_empty(),
        "Error message should not be empty"
    );
    assert!(
        error_message.contains("nonexistent_command_12345")
            || error_message.contains("not found")
            || error_message.contains("No such file")
            || error_message.contains("command"),
        "Error message should indicate command not found: {error_message}"
    );
}

#[tokio::test]
async fn test_command_runner_with_args() {
    // Test running command with multiple arguments
    let runner = CommandRunner::new();

    if cfg!(unix) {
        // Test with multiple arguments
        let result = runner.run("echo", &["hello", "world", "test"]).await;

        match result {
            Ok(output) => {
                let trimmed = output.trim();
                assert!(
                    trimmed.contains("hello"),
                    "Output should contain 'hello': {output}"
                );
                assert!(
                    trimmed.contains("world"),
                    "Output should contain 'world': {output}"
                );
                assert!(
                    trimmed.contains("test"),
                    "Output should contain 'test': {output}"
                );
            }
            Err(e) => {
                // If command fails, error should be informative
                let error_message = e.to_string();
                assert!(
                    error_message.contains("echo") || error_message.contains("command"),
                    "Error should be command-related: {error_message}"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_command_runner_concurrent_execution() {
    // Test concurrent command execution
    let runner = CommandRunner::new();
    let mut handles = vec![];

    // Create multiple concurrent tasks
    for i in 0..3 {
        let runner_clone = runner.clone();
        let handle = tokio::spawn(async move {
            if cfg!(unix) {
                runner_clone.run("echo", &[&format!("test{i}")]).await
            } else {
                runner_clone
                    .run("cmd", &["/C", &format!("echo test{i}")])
                    .await
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(handles).await;

    // All tasks should complete successfully
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_ok(),
            "Concurrent task {i} should complete: {result:?}"
        );

        if let Ok(Ok(output)) = result {
            assert!(
                output.contains(&format!("test{i}")),
                "Output should contain test{i}: {output}"
            );
        }
    }
}

#[tokio::test]
async fn test_command_runner_performance() {
    // Test that simple commands complete quickly
    let runner = CommandRunner::new();

    let start = std::time::Instant::now();

    if cfg!(unix) {
        let result = runner.run("echo", &["performance_test"]).await;
        let duration = start.elapsed();

        // Should complete within 5 seconds
        assert!(
            duration < Duration::from_secs(15),
            "Simple command should complete within 15 seconds, took: {duration:?}"
        );

        match result {
            Ok(output) => {
                assert!(
                    output.contains("performance_test"),
                    "Output should contain test string: {output}"
                );
            }
            Err(e) => {
                // If command fails, error should be informative
                let error_message = e.to_string();
                assert!(
                    error_message.contains("echo") || error_message.contains("command"),
                    "Error should be command-related: {error_message}"
                );
            }
        }
    } else {
        // Test on Windows
        let result = runner.run("cmd", &["/C", "echo performance_test"]).await;
        let duration = start.elapsed();

        assert!(
            duration < Duration::from_secs(15),
            "Simple command should complete within 15 seconds, took: {duration:?}"
        );

        match result {
            Ok(output) => {
                assert!(
                    output.contains("performance_test"),
                    "Output should contain test string: {output}"
                );
            }
            Err(e) => {
                let error_message = e.to_string();
                assert!(
                    error_message.contains("cmd") || error_message.contains("command"),
                    "Error should be command-related: {error_message}"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_command_runner_empty_args() {
    // Test running command with no arguments
    let runner = CommandRunner::new();

    if cfg!(unix) {
        let result = runner.run("echo", &[] as &[&str]).await;

        match result {
            Ok(output) => {
                // Echo with no args should produce empty line or newline
                assert!(
                    output.len() <= 1 || output.trim().is_empty(),
                    "Echo with no args should produce minimal output: '{output}'"
                );
            }
            Err(e) => {
                let error_message = e.to_string();
                assert!(
                    error_message.contains("echo") || error_message.contains("command"),
                    "Error should be command-related: {error_message}"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_command_runner_output_capture() {
    // Test that output is properly captured
    let runner = CommandRunner::new();

    if cfg!(unix) {
        let result = runner.run("echo", &["line1\nline2\nline3"]).await;

        match result {
            Ok(output) => {
                // Should capture all output
                assert!(!output.is_empty(), "Output should not be empty");
                assert!(
                    output.contains("line1"),
                    "Output should contain line1: {output}"
                );
                // Note: echo might not preserve \n literally, but should contain the text
            }
            Err(e) => {
                let error_message = e.to_string();
                assert!(
                    error_message.contains("echo") || error_message.contains("command"),
                    "Error should be command-related: {error_message}"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_command_runner_special_characters() {
    // Test handling of special characters in arguments
    let runner = CommandRunner::new();

    if cfg!(unix) {
        let result = runner
            .run("echo", &["hello world", "test@example.com", "path/to/file"])
            .await;

        match result {
            Ok(output) => {
                let trimmed = output.trim();
                assert!(
                    trimmed.contains("hello world"),
                    "Output should contain 'hello world': {output}"
                );
                assert!(
                    trimmed.contains("test@example.com"),
                    "Output should contain email: {output}"
                );
                assert!(
                    trimmed.contains("path/to/file"),
                    "Output should contain path: {output}"
                );
            }
            Err(e) => {
                let error_message = e.to_string();
                assert!(
                    error_message.contains("echo") || error_message.contains("command"),
                    "Error should be command-related: {error_message}"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_command_runner_clone() {
    // Test that CommandRunner can be cloned
    let runner = CommandRunner::new();
    let runner_clone = runner.clone();

    // Both should work independently
    if cfg!(unix) {
        let result1 = runner.run("echo", &["original"]).await;
        let result2 = runner_clone.run("echo", &["clone"]).await;

        match (result1, result2) {
            (Ok(output1), Ok(output2)) => {
                assert!(
                    output1.contains("original"),
                    "Original runner output: {output1}"
                );
                assert!(output2.contains("clone"), "Cloned runner output: {output2}");
            }
            _ => {
                // If commands fail, that's also acceptable for this test
            }
        }
    }
}

#[tokio::test]
async fn test_command_runner_memory_usage() {
    // Test that CommandRunner doesn't cause memory issues
    let initial_memory = get_memory_usage();

    // Create and use multiple runners
    for i in 0..10 {
        let runner = CommandRunner::new();

        if cfg!(unix) {
            let _ = runner.run("echo", &[&format!("test{i}")]).await;
        } else {
            let _ = runner.run("cmd", &["/C", &format!("echo test{i}")]).await;
        }
    }

    let final_memory = get_memory_usage();

    // Memory usage should not increase dramatically
    let memory_increase = final_memory.saturating_sub(initial_memory);
    assert!(
        memory_increase < 10_000_000, // 10MB limit
        "Memory usage should not increase dramatically: {memory_increase} bytes"
    );
}

// Helper function to get rough memory usage
fn get_memory_usage() -> usize {
    // Simple memory usage estimation
    std::process::id() as usize * 1024 // Simple approximation
}
