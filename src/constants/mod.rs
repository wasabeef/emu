//! Application-wide constants and configuration values.
//!
//! This module centralizes all constant values used throughout the application,
//! including command names, file paths, default values, performance parameters,
//! and user-facing messages.
//!
//! # Module Organization
//!
//! Constants are organized into domain-specific modules:
//!
//! - [`android`] - Android-specific constants
//! - [`colors`] - Color definitions for UI theming
//! - [`commands`] - CLI command names and arguments
//! - [`defaults`] - Default configuration values
//! - [`env_vars`] - Environment variable names
//! - [`errors`] - Error message constants
//! - [`files`] - File paths and extensions
//! - [`ios`] - iOS-specific constants
//! - [`ios_devices`] - iOS device type definitions
//! - [`keywords`] - Keywords for device detection and parsing
//! - [`limits`] - Size limits, validation ranges, and array indices
//! - [`messages`] - User-facing message strings
//! - [`numeric`] - Numeric constants for calculations
//! - [`patterns`] - Regular expression patterns
//! - [`performance`] - Performance tuning parameters
//! - [`priorities`] - Device priority values for sorting
//! - [`progress`] - Progress tracking and phase increments
//! - [`resolutions`] - Screen resolution definitions
//! - [`timeouts`] - Operation timeout values
//! - [`ui_layout`] - UI layout dimensions and spacing
//!
//! # Usage
//!
//! Most constants are re-exported at the module level for convenience:
//!
//! ```rust
//! use emu::constants::{MAX_DEVICE_NAME_LENGTH, HEADER_HEIGHT, ANDROID_PIXEL_PRIORITY};
//! ```

pub mod android;
pub mod colors;
pub mod commands;
pub mod defaults;
pub mod env_vars;
pub mod errors;
pub mod files;
pub mod ios;
pub mod ios_devices;
pub mod keywords;
pub mod limits;
pub mod messages;
pub mod numeric;
pub mod patterns;
pub mod performance;
pub mod priorities;
pub mod progress;
pub mod resolutions;
pub mod timeouts;
pub mod ui_layout;

// Re-export commonly used constants for convenience
pub use colors::*;
pub use commands::*;
pub use defaults::*;
pub use env_vars::*;
pub use files::*;
pub use ios_devices::*;
pub use keywords::*;
pub use limits::*;
pub use numeric::*;
pub use performance::*;
pub use priorities::*;
pub use progress::*;
pub use resolutions::*;
pub use timeouts::*;
pub use ui_layout::*;
