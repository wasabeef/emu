//! Emu - Core library for the terminal UI mobile device emulator manager.
//!
//! This library provides the complete implementation for managing Android emulators
//! and iOS simulators through a unified terminal interface. It is designed with
//! a modular architecture that separates concerns across different domains.
//!
//! # Architecture Overview
//!
//! The library is organized into the following modules:
//!
//! - [`app`] - Main application logic, state management, and event handling
//! - [`managers`] - Platform-specific device management implementations
//! - [`models`] - Core data structures and domain models
//! - [`ui`] - Terminal UI rendering and widget components
//! - [`utils`] - Shared utilities for command execution and logging
//! - [`constants`] - Application-wide constants and configuration values
//!
//! # Key Features
//!
//! - **Cross-platform support**: Works on macOS, Linux, and Windows
//! - **Async-first design**: Non-blocking operations using Tokio
//! - **Platform abstraction**: Unified interface for different device types
//! - **Real-time updates**: Live device status monitoring and log streaming
//! - **Performance optimized**: Fast startup with background loading
//!
//! # Usage
//!
//! The primary entry point is the [`App`] struct which coordinates all functionality:
//!
//! ```no_run
//! use emu::App;
//! use ratatui::Terminal;
//! use ratatui::backend::CrosstermBackend;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let app = App::new().await?;
//! let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
//! app.run(terminal).await?;
//! # Ok(())
//! # }
//! ```

/// Application core functionality including state management and event handling.
///
/// This module contains the main application controller, state management,
/// and coordination between different components.
pub mod app;

/// Application-wide constants and configuration values.
///
/// Includes Android SDK paths, command names, environment variables,
/// and version mappings used throughout the application.
pub mod constants;

/// Platform-specific device management implementations.
///
/// Contains the trait-based abstraction layer and concrete implementations
/// for Android (via Android SDK) and iOS (via Xcode simctl) device management.
pub mod managers;

/// Core data structures and domain models.
///
/// Defines the primary types used throughout the application including
/// device representations, error types, and configuration structures.
pub mod models;

/// Terminal user interface components.
///
/// Provides the rendering logic, themes, and custom widgets for the
/// three-panel terminal interface.
pub mod ui;

/// Shared utility functions and helpers.
///
/// Contains common functionality for command execution, error handling,
/// and logging that is used across multiple modules.
pub mod utils;

// Re-export the main application entry point for convenience
pub use app::App;
