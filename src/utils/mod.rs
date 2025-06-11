//! Utility functions and helpers.
//!
//! This module provides common utilities used throughout the application,
//! including command execution and logging configuration.
//!
//! # Module Organization
//!
//! - `command` - Command execution wrapper with consistent error handling
//! - `logger` - Application logging setup and configuration

pub mod command;
pub mod logger;

// Re-export commonly used utilities
pub use command::CommandRunner;
pub use logger::setup_logger;
