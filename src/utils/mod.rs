//! Utility functions and helpers.
//!
//! This module provides common utilities used throughout the application,
//! including command execution, logging configuration, and input validation.
//!
//! # Module Organization
//!
//! - `command` - Command execution wrapper with consistent error handling
//! - `logger` - Application logging setup and configuration
//! - `validation` - Form field validation framework

pub mod cache_manager;
pub mod command;
pub mod logger;
pub mod validation;

// Re-export commonly used utilities
pub use command::CommandRunner;
pub use logger::setup_logger;
pub use validation::{DeviceNameValidator, FieldValidator, NumericRangeValidator};
