//! Emu - Terminal UI application for managing Android emulators and iOS simulators.
//!
//! This is the main entry point for the Emu application. It provides a lazygit-inspired
//! terminal user interface for unified management of mobile device emulators across platforms.
//!
//! # Features
//!
//! - Android Virtual Device (AVD) management through Android SDK tools
//! - iOS Simulator management through Xcode command line tools (macOS only)
//! - Real-time device status monitoring and log streaming
//! - Keyboard-driven interface with vim-like keybindings
//! - Fast startup with background device loading (< 150ms typical)
//!
//! # Usage
//!
//! ```bash
//! emu                    # Start the TUI in normal mode
//! emu --debug           # Enable debug logging to console
//! emu --log-level trace # Set custom log level (debug mode only)
//! ```

use anyhow::Result;
use clap::Parser;
use emu::app::App;

/// Command line arguments for the Emu application.
///
/// Provides configuration options for debugging and logging levels.
#[derive(Parser)]
#[command(
    name = "emu",
    version,
    about = "A lazygit-inspired TUI for managing Android emulators and iOS simulators"
)]
struct Cli {
    /// Log level for debug mode.
    ///
    /// Valid values: trace, debug, info, warn, error
    /// Only applies when --debug flag is set.
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Enable debug mode with console logging.
    ///
    /// When enabled:
    /// - Console logging is activated with the specified log level
    /// - Android emulator verbose output is enabled
    /// - Terminal UI may be affected by log output
    ///
    /// When disabled (default):
    /// - No console logging to preserve TUI display
    /// - Android emulator output is suppressed
    #[arg(long)]
    debug: bool,
}

/// Main entry point for the Emu application.
///
/// Initializes the Tokio async runtime, parses command line arguments,
/// sets up error handling, and launches the terminal UI.
///
/// # Errors
///
/// Returns an error if:
/// - Terminal initialization fails
/// - Application startup fails
/// - Runtime errors occur during execution
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Install color-eyre for enhanced error reporting with colored output
    color_eyre::install().map_err(|e| anyhow::anyhow!("Failed to install color_eyre: {}", e))?;

    // Configure logging based on debug mode
    if cli.debug {
        // Initialize env_logger with user-specified or default log level
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&cli.log_level))
            .format_timestamp_secs()
            .init();
    } else {
        // Suppress Android emulator verbose output in normal TUI mode
        // These environment variables control Android SDK tool verbosity
        std::env::set_var("ANDROID_EMULATOR_LOG_ENABLE", "0");
        std::env::set_var("ANDROID_AVD_VERBOSE", "0");
        std::env::set_var("ANDROID_VERBOSE", "0");
    }

    run_tui().await
}

/// Initializes and runs the terminal user interface.
///
/// This function:
/// 1. Configures the terminal for raw mode and alternate screen
/// 2. Creates a crossterm backend for the ratatui Terminal
/// 3. Initializes and runs the main application
/// 4. Restores terminal state on exit (success or error)
///
/// # Terminal State Management
///
/// The function ensures proper terminal cleanup even if the application
/// panics or encounters an error. It uses crossterm for cross-platform
/// terminal manipulation.
///
/// # Errors
///
/// Returns an error if:
/// - Terminal mode changes fail
/// - Terminal backend creation fails
/// - Application initialization or execution fails
async fn run_tui() -> Result<()> {
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io;

    // Configure terminal for TUI mode
    // Raw mode disables line buffering and echoing for immediate key input
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Switch to alternate screen buffer to preserve terminal history
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Initialize and run the main application
    let app = App::new().await?;
    let result = app.run(terminal).await;

    // Restore terminal to original state
    // This cleanup runs even if the app returns an error
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}
