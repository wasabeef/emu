//! Application core module for coordinating all Emu functionality.
//!
//! This module serves as the central controller for the application, managing:
//! - Application lifecycle and ultra-responsive main event loop
//! - Coordination between UI, state, and device managers
//! - Background task management for async operations
//! - Direct user input handling for 120fps responsiveness without debouncing

/// Event type definitions for user input and system events.
pub mod events;

/// Application state management and data structures.
pub mod state;

/// Event processing optimizations for improved key input handling.
pub mod event_processing;

mod api_levels;
mod background;
mod create_device;
mod create_device_form;
mod details;
mod device_actions;
mod input;
mod logs;
mod refresh;

use crate::{
    constants::{
        performance::{FULL_DEVICE_REFRESH_INTERVAL, INPUT_BATCH_DELAY, MAX_CONTINUOUS_EVENTS},
        timeouts::{AUTO_REFRESH_CHECK_INTERVAL, EVENT_POLL_TIMEOUT, NOTIFICATION_CHECK_INTERVAL},
    },
    managers::{AndroidManager, IosManager},
    ui,
};
use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(test)]
use crate::models::AndroidDevice;

// Removed EventBatcher import for more responsive input handling

// Re-export commonly used types from the state module
pub use self::state::{ApiLevelManagementState, AppState, FocusedPanel, Mode, Panel};

/// Main application controller that coordinates all components.
///
/// The `App` struct is responsible for:
/// - Managing application state through an `Arc<Mutex<AppState>>`
/// - Coordinating platform-specific device managers
/// - Handling background tasks for non-blocking operations
/// - Processing user input and updating the UI
///
/// # Architecture
///
/// The application uses a shared state pattern with async mutex protection
/// to allow safe concurrent access from multiple tasks. Background operations
/// are managed through join handles that can be cancelled when needed.
pub struct App {
    /// Shared application state protected by an async mutex.
    /// This allows multiple tasks to safely access and modify state.
    state: Arc<Mutex<AppState>>,

    /// Android device manager for AVD operations.
    /// Always present as Android is supported on all platforms.
    android_manager: AndroidManager,

    /// iOS device manager for simulator operations.
    /// Only present on macOS where Xcode tools are available.
    ios_manager: Option<IosManager>,

    /// Join handle for background log streaming task.
    /// Cancelled and recreated when switching devices or panels.
    log_update_handle: Option<tokio::task::JoinHandle<()>>,

    /// Join handle for background device detail fetching.
    /// Used to debounce detail updates during rapid navigation.
    detail_update_handle: Option<tokio::task::JoinHandle<()>>,

    /// Timestamp of the last full device metadata refresh.
    /// Auto-refresh can use lighter status-only checks between these refreshes.
    last_full_device_refresh: std::time::Instant,
}

impl App {
    /// Creates a new application instance with initialized managers and state.
    ///
    /// This function:
    /// 1. Initializes the application state
    /// 2. Creates platform-specific device managers
    /// 3. Starts background cache loading for fast startup
    /// 4. Initiates background device discovery
    ///
    /// # Performance
    ///
    /// The function prioritizes fast startup by deferring expensive operations:
    /// - Device discovery runs in the background
    /// - Cache loading is non-blocking
    /// - UI renders immediately with loading indicators
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Android SDK is not properly configured
    /// - iOS tools are unavailable on macOS
    /// - Initial manager creation fails
    pub async fn new() -> Result<Self> {
        let state = Arc::new(Mutex::new(AppState::new()));
        let android_manager = AndroidManager::new()?;
        let ios_manager = if cfg!(target_os = "macos") {
            Some(IosManager::new()?)
        } else {
            None
        };

        let mut app = Self {
            state,
            android_manager,
            ios_manager,
            log_update_handle: None,
            detail_update_handle: None,
            last_full_device_refresh: std::time::Instant::now() - FULL_DEVICE_REFRESH_INTERVAL,
        };

        // Start background operations for optimal startup performance
        app.start_background_cache_loading();
        app.start_background_device_loading();

        Ok(app)
    }

    /// Runs the ultra-responsive main application event loop.
    ///
    /// This function implements the core application loop optimized for 120fps input responsiveness:
    /// 1. Renders the UI with immediate state updates
    /// 2. Processes user input events directly without batching or debouncing
    /// 3. Manages background refresh cycles
    /// 4. Handles notification timeouts
    ///
    /// # Event Processing
    ///
    /// The loop uses direct event processing with 8ms polling for maximum responsiveness:
    /// - No event batching that could cause input lag
    /// - No debouncing that could ignore rapid key presses
    /// - Immediate processing of all keyboard input including rapid input and long holds
    /// - Supports continuous navigation without key press ignoring
    ///
    /// # Performance Optimizations
    ///
    /// - 8ms polling timeout for 120fps responsiveness target
    /// - Direct event processing eliminates all input lag
    /// - Proper state lock scoping prevents deadlocks
    /// - Background operations don't block input processing
    ///
    /// # Background Tasks
    ///
    /// Two background timers run during the event loop:
    /// - Auto-refresh: Checks every 1000ms for device status updates
    /// - Notification cleanup: Runs every 500ms to dismiss expired messages
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Terminal operations fail
    /// - Device refresh encounters an error
    /// - Critical system errors occur
    pub async fn run(
        mut self,
        mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        let mut last_auto_refresh_check = std::time::Instant::now();
        // Use constants from performance module instead of hardcoding
        let mut last_notification_check = std::time::Instant::now();

        loop {
            // Priority 1: Process multiple events in batch for ultra-responsive handling
            let mut events_processed = 0;
            while events_processed < MAX_CONTINUOUS_EVENTS && event::poll(INPUT_BATCH_DELAY)? {
                if let Ok(event) = event::read() {
                    events_processed += 1;
                    match event {
                        CrosstermEvent::Key(key) => {
                            if self.process_key_event(key).await? {
                                return Ok(());
                            }
                        }
                        CrosstermEvent::Resize(_, _) => {
                            // Handle resize if needed
                        }
                        _ => {
                            // Ignore other events
                        }
                    }
                }
            }

            // If no events available, poll with longer timeout for efficiency
            if events_processed == 0 && event::poll(EVENT_POLL_TIMEOUT)? {
                // Process single event with longer timeout
                continue;
            }

            // Priority 2: Render UI after processing input for immediate visual feedback
            {
                let mut state = self.state.lock().await;
                terminal.draw(|f| ui::render::draw_app(f, &mut state, &ui::Theme::dark()))?;
            }

            // Priority 3: Handle background tasks (less frequently to avoid blocking input)
            if last_auto_refresh_check.elapsed() >= AUTO_REFRESH_CHECK_INTERVAL {
                let state = self.state.lock().await;
                let should_refresh = state.should_auto_refresh();
                let has_devices =
                    !state.android_devices.is_empty() || !state.ios_devices.is_empty();
                drop(state);

                // Only refresh if we have devices loaded (not during initial loading)
                if should_refresh && has_devices {
                    self.refresh_devices_smart().await?;
                }
                last_auto_refresh_check = std::time::Instant::now();
            }

            // Handle notification cleanup
            if last_notification_check.elapsed() >= NOTIFICATION_CHECK_INTERVAL {
                let mut state = self.state.lock().await;
                state.dismiss_expired_notifications();
                drop(state);
                last_notification_check = std::time::Instant::now();
            }
        }
    }
}

#[cfg(test)]
mod tests;
