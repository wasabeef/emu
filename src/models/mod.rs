//! Data models for the Emu application.
//!
//! This module contains all the core data structures used throughout the application,
//! including device representations, error types, and platform definitions.
//!
//! # Module Organization
//!
//! - `device` - Device structures for Android and iOS virtual devices
//! - `device_info` - Dynamic device information and discovery system
//! - `error` - Custom error types and error handling utilities
//! - `platform` - Platform definitions and platform-specific information

pub mod api_level;
pub mod device;
pub mod device_info;
pub mod error;
pub mod platform;

// Re-export commonly used types for convenience
pub use api_level::{ApiLevel, InstallProgress, SystemImageVariant};
pub use device::{AndroidDevice, DeviceStatus, IosDevice};
pub use error::DeviceError;
pub use platform::Platform;
