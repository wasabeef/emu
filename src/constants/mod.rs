//! Application-wide constants and configuration values.
//!
//! This module centralizes all constant values used throughout the application,
//! including command names, file paths, default values, performance parameters,
//! and user-facing messages.

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
