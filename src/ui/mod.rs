//! Terminal User Interface (TUI) components.
//!
//! This module contains all UI-related functionality for rendering the terminal
//! interface, including layout management, theming, and custom widgets.
//!
//! # Module Organization
//!
//! - `render` - Main rendering logic and layout management
//! - `theme` - Color themes and styling configuration
//! - `widgets` - Custom UI widgets and components
//!
//! # Architecture
//!
//! The UI is built using the `ratatui` library and follows a immediate-mode
//! rendering pattern. The application state is rendered to the terminal on
//! each frame, with optimizations for minimal redraws.

pub mod render;
pub mod theme;
pub mod widgets;

// Testing infrastructure
#[cfg(any(test, feature = "test-utils"))]
pub mod mock_backend;

// Re-export commonly used UI types
pub use theme::Theme;
pub use widgets::*;

// Re-export testing utilities when available
#[cfg(any(test, feature = "test-utils"))]
pub use mock_backend::MockBackend;
