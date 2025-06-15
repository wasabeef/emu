//! Application-wide constants and configuration values.
//!
//! This module centralizes all constant values used throughout the application,
//! including command names, file paths, default values, performance parameters,
//! and user-facing messages.

pub mod android;
pub mod commands;
pub mod defaults;
pub mod env_vars;
pub mod errors;
pub mod files;
pub mod ios;
pub mod limits;
pub mod messages;
pub mod patterns;
pub mod performance;
pub mod timeouts;

// Re-export commonly used constants for convenience
pub use commands::*;
pub use defaults::*;
pub use env_vars::*;
pub use files::*;
pub use limits::*;
pub use performance::*;
pub use timeouts::*;
