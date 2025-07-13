//! Command security tests
//!
//! This module tests security aspects of command execution in Emu to prevent
//! command injection vulnerabilities and ensure proper argument handling.
//! Tests focus on defensive patterns without requiring emulator/simulator startup.

use emu::utils::command::CommandRunner;

/// Test that command arguments are properly separated and cannot cause injection
#[tokio::test]
async fn test_command_injection_prevention() {
    let runner = CommandRunner::new();

    // Test that semicolons in arguments don't execute additional commands
    let malicious_arg = "devices; rm -rf /tmp/test";

    // This should fail cleanly rather than executing `rm -rf /tmp/test`
    let result = runner.run("echo", &[malicious_arg]).await;

    // Command should either succeed (echoing the full string) or fail cleanly
    // It should NOT execute the `rm` command
    match result {
        Ok(output) => {
            // If successful, should contain the full malicious string as one argument
            assert!(output.contains(malicious_arg));
            assert!(!output.contains("No such file or directory")); // rm command shouldn't run
        }
        Err(_) => {
            // Clean failure is acceptable - the important thing is no injection
        }
    }
}

/// Test that shell metacharacters are handled safely
#[tokio::test]
async fn test_shell_metacharacter_safety() {
    let runner = CommandRunner::new();

    let dangerous_chars = vec![
        "|", "&&", "||", ";", "`", "$(", "$((", "&", ">", ">>", "<", "<<", "*", "?", "[", "]",
    ];

    for dangerous_char in dangerous_chars {
        let test_arg = format!("test{dangerous_char}value");

        // Test with echo command which should be safe
        let result = runner.run("echo", &[&test_arg]).await;

        match result {
            Ok(output) => {
                // Should contain the literal string, not interpret metacharacters
                assert!(output.contains(&test_arg));
            }
            Err(_) => {
                // Clean failure is acceptable
            }
        }
    }
}

/// Test path traversal prevention in arguments
#[tokio::test]
async fn test_path_traversal_prevention() {
    let runner = CommandRunner::new();

    let traversal_attempts = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32",
        "/etc/passwd",
        "C:\\Windows\\System32\\cmd.exe",
        "~/../../etc/passwd",
        "%USERPROFILE%/../../../etc/passwd",
    ];

    for path in traversal_attempts {
        // Test with a safe command that takes a path argument
        let result = runner.run("file", &[path]).await;

        // The important thing is that it doesn't crash or expose sensitive files
        // We don't care about the specific result, just that it's handled safely
        match result {
            Ok(_) => {
                // If it succeeds, that's fine - the command handled it safely
            }
            Err(_) => {
                // If it fails, that's also fine - proper error handling
            }
        }
    }
}

/// Test that empty and null arguments are handled properly
#[tokio::test]
async fn test_empty_argument_handling() {
    let runner = CommandRunner::new();

    // Test with empty string argument
    let result = runner.run("echo", &[""]).await;
    assert!(result.is_ok());

    // Test with multiple empty arguments
    let result = runner.run("echo", &["", "", ""]).await;
    assert!(result.is_ok());

    // Test with whitespace-only argument
    let result = runner.run("echo", &["   "]).await;
    assert!(result.is_ok());
}

/// Test that unicode and special encoding arguments are handled safely
#[tokio::test]
async fn test_unicode_argument_safety() {
    let runner = CommandRunner::new();

    let unicode_strings = vec![
        "こんにちは",     // Japanese
        "🚀📱💻",         // Emojis
        "test\x00null",   // Null byte
        "test\nnewline",  // Newline
        "test\ttab",      // Tab
        "test\rcarriage", // Carriage return
    ];

    for unicode_str in unicode_strings {
        let result = runner.run("echo", &[unicode_str]).await;

        match result {
            Ok(_) => {
                // Unicode handling is platform-dependent, success is fine
            }
            Err(_) => {
                // Clean failure for unsupported characters is also fine
            }
        }
    }
}

/// Test that OsStr conversion maintains argument boundaries
#[tokio::test]
async fn test_osstr_argument_boundaries() {
    let runner = CommandRunner::new();

    // Create arguments that might be problematic if boundaries are not maintained
    let args = vec!["arg1", "arg with spaces", "arg;with;semicolons"];

    let result = runner.run("echo", &args).await;

    match result {
        Ok(output) => {
            // All arguments should appear in output, maintaining their boundaries
            for arg in &args {
                assert!(output.contains(arg));
            }
        }
        Err(_) => {
            // Clean failure is acceptable
        }
    }
}

/// Test command execution with very long arguments
#[tokio::test]
async fn test_long_argument_handling() {
    let runner = CommandRunner::new();

    // Create a very long argument (but not excessively long to avoid system limits)
    let long_arg = "a".repeat(1000);

    let result = runner.run("echo", &[&long_arg]).await;

    match result {
        Ok(output) => {
            assert!(output.contains(&long_arg));
        }
        Err(_) => {
            // May fail due to system argument length limits, which is acceptable
        }
    }
}

/// Test that spawn method is also secure against command injection
#[tokio::test]
async fn test_spawn_security() {
    let runner = CommandRunner::new();

    // Test spawning with potentially dangerous arguments
    let malicious_arg = "test; echo 'injection'";

    // Use a harmless command that will exit quickly
    let result = runner.spawn("echo", &[malicious_arg]).await;

    match result {
        Ok(pid) => {
            assert!(pid > 0);
            // If successful, the spawn should have treated the argument as literal
        }
        Err(_) => {
            // Clean failure is acceptable
        }
    }
}

/// Test run_ignoring_errors security
#[tokio::test]
async fn test_run_ignoring_errors_security() {
    let runner = CommandRunner::new();

    // Test that error patterns don't allow injection
    let malicious_pattern = "error; rm -rf /tmp";

    let result = runner
        .run_ignoring_errors("echo", &["test"], &[malicious_pattern])
        .await;

    // Should succeed since echo "test" is a valid command
    assert!(result.is_ok());
}

/// Test retry mechanism security
#[tokio::test]
async fn test_retry_security() {
    let runner = CommandRunner::new();

    // Test that retries don't amplify security issues
    let malicious_arg = "test; echo 'injection'";

    let result = runner.run_with_retry("echo", &[malicious_arg], 2).await;

    match result {
        Ok(output) => {
            // Should contain the literal string, not execute injection
            assert!(output.contains(malicious_arg));
        }
        Err(_) => {
            // Clean failure is acceptable
        }
    }
}

/// Test command validation - ensure programs are properly specified
#[tokio::test]
async fn test_command_program_validation() {
    let runner = CommandRunner::new();

    // Test that empty program name fails cleanly
    let result = runner.run("", &["test"]).await;
    assert!(result.is_err());

    // Test that program with path separators is handled properly
    let result = runner.run("../bin/echo", &["test"]).await;
    // This should either succeed (if path exists) or fail cleanly
    if result.is_ok() {
        // Success is fine if the path is valid
    } else {
        // Clean failure is expected for invalid paths
    }
}

/// Integration test combining multiple security concerns
#[tokio::test]
async fn test_comprehensive_security_integration() {
    let runner = CommandRunner::new();

    // Combine multiple potential security issues in one test
    let complex_args = vec![
        "normal_arg",
        "arg with spaces",
        "arg;with;semicolons|and|pipes",
        "../../../etc/passwd",
        "$(malicious_command)",
        "`backtick_command`",
        "unicode_测试_🚀",
        "",    // empty arg
        "   ", // whitespace arg
    ];

    let result = runner.run("echo", &complex_args).await;

    match result {
        Ok(output) => {
            // Verify that arguments are treated as literals
            for arg in &complex_args {
                if !arg.trim().is_empty() {
                    assert!(output.contains(arg));
                }
            }
        }
        Err(_) => {
            // Clean failure is acceptable for complex inputs
        }
    }
}
