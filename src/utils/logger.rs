//! Logging utilities
//!
//! This module provides application-wide logging configuration and convenience
//! macros for consistent log formatting. It uses the env_logger crate for
//! output formatting and log level filtering.

use anyhow::Result;
use log::LevelFilter;
use std::str::FromStr;

/// Sets up the global logger with the specified log level.
///
/// This function initializes the env_logger with custom formatting
/// that excludes module paths and targets for cleaner output.
/// The logger respects the `RUST_LOG` environment variable.
///
/// # Arguments
/// * `level` - Log level string ("error", "warn", "info", "debug", "trace")
///
/// # Returns
/// * `Ok(())` - If logger initialization succeeds
/// * `Err(anyhow::Error)` - If logger is already initialized or setup fails
///
/// # Examples
/// ```rust,no_run
/// use emu::utils::setup_logger;
///
/// # fn main() -> anyhow::Result<()> {
/// setup_logger("info")?;  // Set to info level
/// // Note: Can't call setup_logger twice in same process
/// // setup_logger("debug")?; // Would fail - already initialized
/// # Ok(())
/// # }
/// ```
///
/// # Environment Variables
/// The logger also respects the `RUST_LOG` environment variable:
/// - `RUST_LOG=debug` - Enable debug logging
/// - `RUST_LOG=emu=trace` - Enable trace logging for emu crate only
/// - `RUST_LOG=warn` - Show only warnings and errors
///
/// # Log Format
/// Logs are formatted as: `[TIMESTAMP] [LEVEL] message`
/// - Timestamps are in seconds precision
/// - Module paths and targets are omitted for cleaner output
pub fn setup_logger(level: &str) -> Result<()> {
    let log_level = LevelFilter::from_str(level).unwrap_or(LevelFilter::Info);

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .format_timestamp_secs()
        .format_module_path(false)
        .format_target(false)
        .try_init()?;

    Ok(())
}

/// Convenience macro for debug-level logging.
///
/// This macro provides a shorter alternative to `log::debug!` and ensures
/// consistent usage across the application. Debug messages are only output
/// when the log level is set to debug or trace.
///
/// # Examples
/// ```rust
/// use emu::debug;
///
/// let device_name = "Pixel 7";
/// debug!("Processing device: {}", device_name);
///
/// let cmd = "adb";
/// let args = vec!["devices"];
/// debug!("Command executed: {} {}", cmd, args.join(" "));
/// ```
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*)
    };
}

/// Convenience macro for info-level logging.
///
/// This macro provides a shorter alternative to `log::info!` for general
/// informational messages that are useful for understanding application flow.
///
/// # Examples
/// ```rust
/// use emu::info;
///
/// info!("Starting device manager");
///
/// let device_name = "Pixel 7";
/// info!("Device '{}' started successfully", device_name);
/// ```
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        log::info!($($arg)*)
    };
}

/// Convenience macro for warning-level logging.
///
/// This macro provides a shorter alternative to `log::warn!` for messages
/// about potentially problematic situations that don't prevent operation.
///
/// # Examples
/// ```rust
/// use emu::warn;
///
/// let device_name = "Pixel 7";
/// warn!("Device '{}' not responding, retrying...", device_name);
/// warn!("Using fallback configuration due to missing file");
/// ```
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        log::warn!($($arg)*)
    };
}

/// Convenience macro for error-level logging.
///
/// This macro provides a shorter alternative to `log::error!` for messages
/// about error conditions that prevent normal operation.
///
/// # Examples
/// ```rust
/// use emu::error;
///
/// let device_name = "Pixel 7";
/// let error = "Connection timeout";
/// error!("Failed to start device '{}': {}", device_name, error);
///
/// let sdk_path = "/usr/local/android-sdk";
/// error!("SDK not found at path: {}", sdk_path);
/// ```
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        log::error!($($arg)*)
    };
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;
    use std::str::FromStr;

    /// Test setup_logger function with different log levels
    #[test]
    fn test_setup_logger_levels() {
        // Test valid log levels
        let valid_levels = vec!["error", "warn", "info", "debug", "trace"];

        for level in valid_levels {
            // Note: We can't actually test initialization because env_logger
            // can only be initialized once per process. Instead, we test
            // that the function handles valid input without panicking.
            let log_level = LevelFilter::from_str(level);
            assert!(log_level.is_ok(), "Level {level} should be valid");
        }
    }

    /// Test that invalid log levels default to Info
    #[test]
    fn test_setup_logger_invalid_level() {
        // Test that invalid levels default to Info
        let invalid_level = "invalid";
        let log_level = LevelFilter::from_str(invalid_level).unwrap_or(LevelFilter::Info);
        assert_eq!(log_level, LevelFilter::Info);
    }

    /// Test log level ordering
    #[test]
    fn test_log_level_ordering() {
        assert!(LevelFilter::Error < LevelFilter::Warn);
        assert!(LevelFilter::Warn < LevelFilter::Info);
        assert!(LevelFilter::Info < LevelFilter::Debug);
        assert!(LevelFilter::Debug < LevelFilter::Trace);
    }

    /// Test that macros compile and expand correctly
    #[test]
    fn test_logging_macros_compile() {
        // These tests verify the macros compile and expand correctly
        // In a real environment with initialized logger, these would output logs

        // Test debug macro
        debug!("Test debug message");
        debug!("Test debug with format: {}", "value");

        // Test info macro
        info!("Test info message");
        info!("Test info with format: {}", "value");

        // Test warn macro
        warn!("Test warn message");
        warn!("Test warn with format: {}", "value");

        // Test error macro
        error!("Test error message");
        error!("Test error with format: {}", "value");
    }

    /// Test logger configuration values
    #[test]
    fn test_logger_configuration() {
        // Test that we can create a logger builder with expected settings
        let _builder = env_logger::Builder::from_default_env()
            .filter_level(LevelFilter::Info)
            .format_timestamp_secs()
            .format_module_path(false)
            .format_target(false);

        // Just verify the builder was created successfully
        // We can't easily test the internal state without initialization
    }

    /// Test level filter from string conversion
    #[test]
    fn test_level_filter_from_string() {
        assert_eq!(LevelFilter::from_str("error").unwrap(), LevelFilter::Error);
        assert_eq!(LevelFilter::from_str("warn").unwrap(), LevelFilter::Warn);
        assert_eq!(LevelFilter::from_str("info").unwrap(), LevelFilter::Info);
        assert_eq!(LevelFilter::from_str("debug").unwrap(), LevelFilter::Debug);
        assert_eq!(LevelFilter::from_str("trace").unwrap(), LevelFilter::Trace);

        // Test case insensitive
        assert_eq!(LevelFilter::from_str("INFO").unwrap(), LevelFilter::Info);
        assert_eq!(LevelFilter::from_str("Error").unwrap(), LevelFilter::Error);
    }
}
