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

use crate::{
    constants::{
        keywords::{LOG_LEVEL_ERROR, LOG_LEVEL_WARNING},
        performance::{
            API_INSTALLATION_COMPLETION_DELAY, DETAIL_UPDATE_DEBOUNCE, FAST_DETAIL_UPDATE_DEBOUNCE,
            FAST_LOG_UPDATE_DEBOUNCE, LOG_UPDATE_DEBOUNCE,
        },
        timeouts::{
            AUTO_REFRESH_CHECK_INTERVAL, DEVICE_STOP_WAIT_TIME, EVENT_POLL_TIMEOUT,
            NOTIFICATION_CHECK_INTERVAL,
        },
    },
    managers::common::DeviceManager,
    managers::{AndroidManager, IosManager},
    models::{error::format_user_error, AndroidDevice, IosDevice},
    ui,
};
use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

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
}

impl App {
    /// Refresh devices using incremental update for optimal performance
    async fn refresh_devices_smart(&mut self) -> Result<()> {
        self.refresh_devices_incremental().await
    }

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
            // Check for auto-refresh less frequently (skip if initial loading)
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

            // Dismiss expired notifications less frequently
            if last_notification_check.elapsed() >= NOTIFICATION_CHECK_INTERVAL {
                let mut state = self.state.lock().await;
                state.dismiss_expired_notifications();
                drop(state);
                last_notification_check = std::time::Instant::now();
            }

            // Draw UI
            {
                let mut state = self.state.lock().await;
                terminal.draw(|f| ui::render::draw_app(f, &mut state, &ui::Theme::dark()))?;
            }

            // Direct event processing - wait for input with reasonable timeout
            if event::poll(EVENT_POLL_TIMEOUT)? {
                if let Ok(event) = event::read() {
                    match event {
                        CrosstermEvent::Key(key) => {
                            let mut state = self.state.lock().await;

                            // Quick quit check
                            if let KeyCode::Char('q') = key.code {
                                if key.modifiers.contains(KeyModifiers::CONTROL)
                                    || key.modifiers.is_empty()
                                {
                                    // Cancel any running tasks
                                    if let Some(handle) = state.log_task_handle.take() {
                                        handle.abort();
                                    }
                                    drop(state);
                                    return Ok(());
                                }
                            }

                            match state.mode {
                                Mode::Normal => {
                                    match key.code {
                                        KeyCode::Char('c')
                                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                        {
                                            // Ctrl+C also quits
                                            if let Some(handle) = state.log_task_handle.take() {
                                                handle.abort();
                                            }
                                            drop(state);
                                            return Ok(());
                                        }
                                        KeyCode::Esc => {
                                            // Dismiss all notifications
                                            state.dismiss_all_notifications();
                                        }
                                        KeyCode::Char('r') => {
                                            drop(state);
                                            self.refresh_devices_smart().await?;
                                        }
                                        KeyCode::Tab => {
                                            // Tab: Switch focus between panels (android -> ios -> android)
                                            let new_panel = state.active_panel.toggle();
                                            state.smart_clear_cached_device_details(new_panel); // Smart cache clearing
                                            state.active_panel = new_panel;
                                            drop(state);

                                            // Optimized panel switching with parallel execution
                                            self.schedule_panel_switch_updates_parallel().await;
                                        }
                                        KeyCode::BackTab => {
                                            // Shift+Tab: Switch focus in reverse order (android -> ios -> android)
                                            let new_panel = state.active_panel.toggle();
                                            state.smart_clear_cached_device_details(new_panel); // Smart cache clearing
                                            state.active_panel = new_panel;
                                            drop(state);

                                            // Optimized panel switching with parallel execution
                                            self.schedule_panel_switch_updates_parallel().await;
                                        }
                                        KeyCode::Char('h')
                                        | KeyCode::Char('l')
                                        | KeyCode::Left
                                        | KeyCode::Right => {
                                            // Switch panels
                                            let new_panel = state.active_panel.toggle();
                                            state.smart_clear_cached_device_details(new_panel); // Smart cache clearing
                                            state.active_panel = new_panel;
                                            drop(state);

                                            // Optimized panel switching with parallel execution
                                            self.schedule_panel_switch_updates_parallel().await;
                                        }
                                        KeyCode::Up | KeyCode::Char('k') => {
                                            state.move_up();
                                            // Always clear logs and stop streaming when device changes
                                            state.clear_logs();

                                            // Stop current log task
                                            if let Some(handle) = state.log_task_handle.take() {
                                                handle.abort();
                                            }
                                            state.current_log_device = None;

                                            // Only clear cache if device actually changed
                                            let need_update = {
                                                let current_device = match state.active_panel {
                                                    Panel::Android => state
                                                        .android_devices
                                                        .get(state.selected_android)
                                                        .map(|d| &d.name),
                                                    Panel::Ios => state
                                                        .ios_devices
                                                        .get(state.selected_ios)
                                                        .map(|d| &d.udid),
                                                };

                                                if let Some(ref cached) =
                                                    state.cached_device_details
                                                {
                                                    current_device.map(String::as_str)
                                                        != Some(&cached.identifier)
                                                } else {
                                                    true
                                                }
                                            };

                                            if need_update {
                                                state.clear_cached_device_details();
                                                drop(state);
                                                self.schedule_device_details_update().await;
                                                self.update_log_stream().await?;
                                            }
                                        }
                                        KeyCode::Down | KeyCode::Char('j') => {
                                            state.move_down();
                                            // Always clear logs and stop streaming when device changes
                                            state.clear_logs();

                                            // Stop current log task
                                            if let Some(handle) = state.log_task_handle.take() {
                                                handle.abort();
                                            }
                                            state.current_log_device = None;

                                            // Only clear cache if device actually changed
                                            let need_update = {
                                                let current_device = match state.active_panel {
                                                    Panel::Android => state
                                                        .android_devices
                                                        .get(state.selected_android)
                                                        .map(|d| &d.name),
                                                    Panel::Ios => state
                                                        .ios_devices
                                                        .get(state.selected_ios)
                                                        .map(|d| &d.udid),
                                                };

                                                if let Some(ref cached) =
                                                    state.cached_device_details
                                                {
                                                    current_device.map(String::as_str)
                                                        != Some(&cached.identifier)
                                                } else {
                                                    true
                                                }
                                            };

                                            if need_update {
                                                state.clear_cached_device_details();
                                                drop(state);
                                                self.schedule_device_details_update().await;
                                                self.update_log_stream().await?;
                                            }
                                        }
                                        KeyCode::Enter => {
                                            drop(state);
                                            self.toggle_device().await?;
                                        }
                                        KeyCode::Char('f') => {
                                            // f: Toggle log filter (global shortcut)
                                            let next_filter = match &state.log_filter_level {
                                                None => Some("ERROR".to_string()),
                                                Some(level) if level == "ERROR" => {
                                                    Some("WARN".to_string())
                                                }
                                                Some(level) if level == "WARN" => {
                                                    Some("INFO".to_string())
                                                }
                                                Some(level) if level == "INFO" => {
                                                    Some("DEBUG".to_string())
                                                }
                                                _ => None,
                                            };
                                            state.toggle_log_filter(next_filter);
                                        }
                                        KeyCode::Char('F')
                                            if key.modifiers.contains(KeyModifiers::SHIFT) =>
                                        {
                                            // Shift+F: Toggle fullscreen logs
                                            state.toggle_fullscreen_logs();
                                        }
                                        KeyCode::Char('L')
                                            if key.modifiers.contains(KeyModifiers::SHIFT) =>
                                        {
                                            // Shift+L: Clear logs (global shortcut)
                                            state.clear_logs();
                                            state.add_info_notification("Logs cleared".to_string());
                                        }
                                        KeyCode::Char('c') => {
                                            let active_panel = state.active_panel;
                                            state.mode = Mode::CreateDevice;
                                            // Initialize form based on active panel
                                            state.create_device_form = match state.active_panel {
                                                Panel::Android => {
                                                    state::CreateDeviceForm::for_android()
                                                }
                                                Panel::Ios => state::CreateDeviceForm::for_ios(),
                                            };

                                            // Show form immediately with loading state
                                            state.create_device_form.is_loading_cache = true;

                                            // Try to populate from cache first
                                            let cache_available =
                                                state.is_cache_available(active_panel).await;
                                            if cache_available {
                                                state.populate_form_from_cache(active_panel).await;
                                                state.create_device_form.is_loading_cache = false;
                                            } else {
                                                // Show UI immediately, load data in background
                                                drop(state);
                                                // Clone necessary data for async operation
                                                let state_clone = Arc::clone(&self.state);
                                                let android_manager = self.android_manager.clone();
                                                let ios_manager = self.ios_manager.clone();
                                                let panel = active_panel;

                                                // Load data asynchronously without blocking UI
                                                tokio::spawn(async move {
                                                    match panel {
                                                        Panel::Android => {
                                                            if let Ok(targets) = android_manager
                                                                .list_available_targets()
                                                                .await
                                                            {
                                                                if let Ok(devices) = android_manager
                                                                    .list_devices_by_category(Some(
                                                                        "all",
                                                                    ))
                                                                    .await
                                                                {
                                                                    let mut state =
                                                                        state_clone.lock().await;
                                                                    state
                                                                        .create_device_form
                                                                        .available_versions =
                                                                        targets.clone();
                                                                    state
                                                                        .create_device_form
                                                                        .available_device_types =
                                                                        devices.clone();

                                                                    // Update cache with the fetched data
                                                                    {
                                                                        let mut cache = state
                                                                            .device_cache
                                                                            .write()
                                                                            .await;
                                                                        cache.update_android_cache(
                                                                            devices, targets,
                                                                        );
                                                                    }

                                                                    // Set defaults if not empty
                                                                    if !state
                                                                        .create_device_form
                                                                        .available_device_types
                                                                        .is_empty()
                                                                    {
                                                                        let (id, display) = state
                                                                            .create_device_form
                                                                            .available_device_types
                                                                            [0]
                                                                        .clone();
                                                                        state
                                                                            .create_device_form
                                                                            .device_type_id = id;
                                                                        state
                                                                            .create_device_form
                                                                            .device_type = display;
                                                                        state
                                                                            .create_device_form
                                                                            .selected_device_type_index = 0;
                                                                    }

                                                                    if !state
                                                                        .create_device_form
                                                                        .available_versions
                                                                        .is_empty()
                                                                    {
                                                                        let (value, display) =
                                                                            state
                                                                                .create_device_form
                                                                                .available_versions
                                                                                [0]
                                                                            .clone();
                                                                        state
                                                                            .create_device_form
                                                                            .version = value;
                                                                        state
                                                                            .create_device_form
                                                                            .version_display =
                                                                            display;
                                                                        state
                                                                            .create_device_form
                                                                            .selected_api_level_index = 0;
                                                                    }

                                                                    state
                                                                        .create_device_form
                                                                        .generate_placeholder_name(
                                                                        );
                                                                    state
                                                                        .create_device_form
                                                                        .is_loading_cache = false;
                                                                }
                                                            }
                                                        }
                                                        Panel::Ios => {
                                                            if let Some(ref ios_manager) =
                                                                ios_manager
                                                            {
                                                                if let Ok(device_types) = ios_manager
                                                                    .list_device_types_with_names()
                                                                    .await
                                                                {
                                                                    if let Ok(runtimes) =
                                                                        ios_manager.list_runtimes().await
                                                                    {
                                                                        let mut state =
                                                                            state_clone.lock().await;
                                                                        state
                                                                            .create_device_form
                                                                            .available_device_types =
                                                                            device_types.clone();
                                                                        state
                                                                            .create_device_form
                                                                            .available_versions =
                                                                            runtimes.clone();

                                                                        // Update cache with the fetched data
                                                                        {
                                                                            let mut cache = state
                                                                                .device_cache
                                                                                .write()
                                                                                .await;
                                                                            cache.update_ios_cache(
                                                                                device_types,
                                                                                runtimes,
                                                                            );
                                                                        }

                                                                        // Set defaults if not empty
                                                                        if !state
                                                                            .create_device_form
                                                                            .available_device_types
                                                                            .is_empty()
                                                                        {
                                                                            let (id, display) = state
                                                                                .create_device_form
                                                                                .available_device_types[0]
                                                                                .clone();
                                                                            state
                                                                                .create_device_form
                                                                                .device_type_id = id;
                                                                            state
                                                                                .create_device_form
                                                                                .device_type = display;
                                                                            state.create_device_form.selected_device_type_index = 0;
                                                                        }

                                                                        if !state
                                                                            .create_device_form
                                                                            .available_versions
                                                                            .is_empty()
                                                                        {
                                                                            let (value, display) = state
                                                                                .create_device_form
                                                                                .available_versions[0]
                                                                                .clone();
                                                                            state
                                                                                .create_device_form
                                                                                .version = value;
                                                                            state
                                                                                .create_device_form
                                                                                .version_display = display;
                                                                            state
                                                                                .create_device_form
                                                                                .selected_api_level_index =
                                                                                0;
                                                                        }

                                                                        state
                                                                            .create_device_form
                                                                            .generate_placeholder_name();
                                                                        state
                                                                            .create_device_form
                                                                            .is_loading_cache = false;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                        KeyCode::Char('d') => {
                                            // d: Delete device
                                            let (device_name, device_id) = match state.active_panel
                                            {
                                                Panel::Android => {
                                                    if let Some(device) = state
                                                        .android_devices
                                                        .get(state.selected_android)
                                                    {
                                                        (device.name.clone(), device.name.clone())
                                                    } else {
                                                        return Ok(());
                                                    }
                                                }
                                                Panel::Ios => {
                                                    if let Some(device) =
                                                        state.ios_devices.get(state.selected_ios)
                                                    {
                                                        (device.name.clone(), device.udid.clone())
                                                    } else {
                                                        return Ok(());
                                                    }
                                                }
                                            };

                                            state.mode = Mode::ConfirmDelete;
                                            state.confirm_delete_dialog =
                                                Some(state::ConfirmDeleteDialog {
                                                    device_name,
                                                    device_identifier: device_id,
                                                    platform: state.active_panel,
                                                });
                                        }
                                        KeyCode::Char('w') => {
                                            let (device_name, device_id) = match state.active_panel
                                            {
                                                Panel::Android => {
                                                    if let Some(device) = state
                                                        .android_devices
                                                        .get(state.selected_android)
                                                    {
                                                        (device.name.clone(), device.name.clone())
                                                    } else {
                                                        return Ok(());
                                                    }
                                                }
                                                Panel::Ios => {
                                                    if let Some(device) =
                                                        state.ios_devices.get(state.selected_ios)
                                                    {
                                                        (device.name.clone(), device.udid.clone())
                                                    } else {
                                                        return Ok(());
                                                    }
                                                }
                                            };

                                            state.mode = Mode::ConfirmWipe;
                                            state.confirm_wipe_dialog =
                                                Some(state::ConfirmWipeDialog {
                                                    device_name,
                                                    device_identifier: device_id,
                                                    platform: state.active_panel,
                                                });
                                        }
                                        KeyCode::Char('i') => {
                                            // i: Install API level (Android only)
                                            if state.active_panel == Panel::Android {
                                                state.mode = Mode::ManageApiLevels;
                                                state.api_level_management =
                                                    Some(state::ApiLevelManagementState::new());

                                                // Load API levels in background
                                                let android_manager = self.android_manager.clone();
                                                let state_clone = self.state.clone();

                                                tokio::spawn(async move {
                                                    if let Ok(api_levels) =
                                                        android_manager.list_api_levels().await
                                                    {
                                                        let mut state = state_clone.lock().await;
                                                        if let Some(ref mut api_state) =
                                                            state.api_level_management
                                                        {
                                                            api_state.api_levels = api_levels;
                                                            api_state.is_loading = false;
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                Mode::CreateDevice => {
                                    match key.code {
                                        KeyCode::Esc => {
                                            // Ignore ESC if currently creating
                                            if !state.create_device_form.is_creating {
                                                state.mode = Mode::Normal;
                                                state.create_device_form.error_message = None;
                                            }
                                        }
                                        KeyCode::Tab => {
                                            // Ignore navigation if currently creating
                                            if !state.create_device_form.is_creating {
                                                match state.active_panel {
                                                    Panel::Android => {
                                                        state.create_device_form.next_field()
                                                    }
                                                    Panel::Ios => {
                                                        state.create_device_form.next_field_ios()
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::BackTab => {
                                            // Ignore navigation if currently creating
                                            if !state.create_device_form.is_creating {
                                                match state.active_panel {
                                                    Panel::Android => {
                                                        state.create_device_form.prev_field()
                                                    }
                                                    Panel::Ios => {
                                                        state.create_device_form.prev_field_ios()
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::Down => {
                                            // Ignore navigation if currently creating
                                            if !state.create_device_form.is_creating {
                                                // Up/Down keys always move between fields
                                                match state.active_panel {
                                                    Panel::Android => {
                                                        state.create_device_form.next_field()
                                                    }
                                                    Panel::Ios => {
                                                        state.create_device_form.next_field_ios()
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::Up => {
                                            // Ignore navigation if currently creating
                                            if !state.create_device_form.is_creating {
                                                // Up/Down keys always move between fields
                                                match state.active_panel {
                                                    Panel::Android => {
                                                        state.create_device_form.prev_field()
                                                    }
                                                    Panel::Ios => {
                                                        state.create_device_form.prev_field_ios()
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::Enter => {
                                            // Ignore Enter key if already creating
                                            if !state.create_device_form.is_creating {
                                                drop(state);
                                                self.submit_create_device().await?;
                                            }
                                        }
                                        KeyCode::Left => {
                                            // Ignore if currently creating
                                            if !state.create_device_form.is_creating {
                                                match state.create_device_form.active_field {
                                                    state::CreateDeviceField::Category => {
                                                        let old_category = state
                                                            .create_device_form
                                                            .device_category_filter
                                                            .clone();
                                                        self.handle_create_device_left(&mut state);
                                                        // Reload device list if category changed
                                                        if old_category
                                                            != state
                                                                .create_device_form
                                                                .device_category_filter
                                                        {
                                                            drop(state);
                                                            if let Err(e) = self
                                                                .reload_device_types_for_category()
                                                                .await
                                                            {
                                                                let mut state =
                                                                    self.state.lock().await;
                                                                state
                                                                    .create_device_form
                                                                    .error_message =
                                                                    Some(format_user_error(&e));
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        self.handle_create_device_left(&mut state);
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::Right => {
                                            // Ignore if currently creating
                                            if !state.create_device_form.is_creating {
                                                match state.create_device_form.active_field {
                                                    state::CreateDeviceField::Category => {
                                                        let old_category = state
                                                            .create_device_form
                                                            .device_category_filter
                                                            .clone();
                                                        self.handle_create_device_right(&mut state);
                                                        // Reload device list if category changed
                                                        if old_category
                                                            != state
                                                                .create_device_form
                                                                .device_category_filter
                                                        {
                                                            drop(state);
                                                            if let Err(e) = self
                                                                .reload_device_types_for_category()
                                                                .await
                                                            {
                                                                let mut state =
                                                                    self.state.lock().await;
                                                                state
                                                                    .create_device_form
                                                                    .error_message =
                                                                    Some(format_user_error(&e));
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        self.handle_create_device_right(&mut state);
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::Char(c) => {
                                            // Handle Ctrl+hjkl for navigation
                                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                                if !state.create_device_form.is_creating {
                                                    match c {
                                                        'h' => {
                                                            // Ctrl+h = Left arrow (selection change)
                                                            match state.create_device_form.active_field {
                                                            state::CreateDeviceField::Category => {
                                                                let old_category = state
                                                                    .create_device_form
                                                                    .device_category_filter
                                                                    .clone();
                                                                self.handle_create_device_left(&mut state);
                                                                if old_category
                                                                    != state
                                                                        .create_device_form
                                                                        .device_category_filter
                                                                {
                                                                    drop(state);
                                                                    if let Err(e) = self
                                                                        .reload_device_types_for_category()
                                                                        .await
                                                                    {
                                                                        let mut state =
                                                                            self.state.lock().await;
                                                                        state
                                                                            .create_device_form
                                                                            .error_message =
                                                                            Some(format_user_error(&e));
                                                                    }
                                                                }
                                                            }
                                                            _ => {
                                                                self.handle_create_device_left(&mut state);
                                                            }
                                                        }
                                                        }
                                                        'l' => {
                                                            // Ctrl+l = Right arrow (selection change)
                                                            match state.create_device_form.active_field {
                                                            state::CreateDeviceField::Category => {
                                                                let old_category = state
                                                                    .create_device_form
                                                                    .device_category_filter
                                                                    .clone();
                                                                self.handle_create_device_right(&mut state);
                                                                if old_category
                                                                    != state
                                                                        .create_device_form
                                                                        .device_category_filter
                                                                {
                                                                    drop(state);
                                                                    if let Err(e) = self
                                                                        .reload_device_types_for_category()
                                                                        .await
                                                                    {
                                                                        let mut state =
                                                                            self.state.lock().await;
                                                                        state
                                                                            .create_device_form
                                                                            .error_message =
                                                                            Some(format_user_error(&e));
                                                                    }
                                                                }
                                                            }
                                                            _ => {
                                                                self.handle_create_device_right(&mut state);
                                                            }
                                                        }
                                                        }
                                                        'j' => {
                                                            // Ctrl+j = Down arrow (field navigation)
                                                            match state.active_panel {
                                                                Panel::Android => state
                                                                    .create_device_form
                                                                    .next_field(),
                                                                Panel::Ios => state
                                                                    .create_device_form
                                                                    .next_field_ios(),
                                                            }
                                                        }
                                                        'k' => {
                                                            // Ctrl+k = Up arrow (field navigation)
                                                            match state.active_panel {
                                                                Panel::Android => state
                                                                    .create_device_form
                                                                    .prev_field(),
                                                                Panel::Ios => state
                                                                    .create_device_form
                                                                    .prev_field_ios(),
                                                            }
                                                        }
                                                        _ => {
                                                            // Other Ctrl+char combinations - treat as regular char input
                                                            self.handle_create_device_char(
                                                                &mut state, c,
                                                            );
                                                        }
                                                    }
                                                }
                                            } else {
                                                // Regular character input (no Ctrl modifier)
                                                if !state.create_device_form.is_creating {
                                                    self.handle_create_device_char(&mut state, c);
                                                }
                                            }
                                        }
                                        KeyCode::Backspace => {
                                            // Ignore if currently creating
                                            if !state.create_device_form.is_creating {
                                                self.handle_create_device_backspace(&mut state);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                Mode::ConfirmDelete => match key.code {
                                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                                        // Immediately close dialog for instant feedback
                                        state.mode = Mode::Normal;
                                        if let Some(dialog) = state.confirm_delete_dialog.clone() {
                                            state.set_device_operation_status(format!(
                                                "Deleting device '{}'...",
                                                dialog.device_name
                                            ));
                                        }
                                        drop(state);
                                        self.execute_delete_device().await?;
                                    }
                                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                        state.mode = Mode::Normal;
                                        state.confirm_delete_dialog = None;
                                    }
                                    _ => {}
                                },
                                Mode::ConfirmWipe => match key.code {
                                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                                        // Immediately close dialog and show status for instant feedback
                                        state.mode = Mode::Normal;
                                        if let Some(dialog) = state.confirm_wipe_dialog.clone() {
                                            state.set_device_operation_status(format!(
                                                "Wiping device '{}'...",
                                                dialog.device_name
                                            ));
                                        }
                                        drop(state);
                                        self.execute_wipe_device().await?;
                                    }
                                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                        state.mode = Mode::Normal;
                                        state.confirm_wipe_dialog = None;
                                    }
                                    _ => {}
                                },
                                Mode::ManageApiLevels => match key.code {
                                    KeyCode::Esc => {
                                        // Only allow closing if not currently installing/uninstalling
                                        if let Some(ref api_mgmt) = state.api_level_management {
                                            if api_mgmt.install_progress.is_none()
                                                && api_mgmt.installing_package.is_none()
                                            {
                                                state.mode = Mode::Normal;
                                                state.api_level_management = None;
                                            }
                                        }
                                    }
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        if let Some(ref mut api_state) = state.api_level_management
                                        {
                                            api_state.move_up();
                                        }
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        if let Some(ref mut api_state) = state.api_level_management
                                        {
                                            api_state.move_down();
                                        }
                                    }
                                    KeyCode::Enter => {
                                        // Enter key - Install only
                                        if let Some(ref api_state) = state.api_level_management {
                                            if let Some(api_level) =
                                                api_state.get_selected_api_level()
                                            {
                                                if let Some(variant) =
                                                    api_level.get_recommended_variant()
                                                {
                                                    if !variant.is_installed {
                                                        let package_id = variant.package_id.clone();

                                                        // Set installing package indicator
                                                        if let Some(ref mut api_mgmt) =
                                                            state.api_level_management
                                                        {
                                                            api_mgmt.installing_package =
                                                                Some(package_id.clone());
                                                            api_mgmt.error_message = None;
                                                        }

                                                        let android_manager =
                                                            self.android_manager.clone();
                                                        let state_clone = self.state.clone();
                                                        let state_clone_for_progress =
                                                            state_clone.clone();

                                                        tokio::spawn(async move {
                                                            let result = android_manager
                                                                .install_system_image(
                                                                    &package_id,
                                                                    move |progress| {
                                                                        let state_clone =
                                                                            state_clone_for_progress
                                                                                .clone();
                                                                        tokio::spawn(async move {
                                                                            let mut state =
                                                                                state_clone.lock().await;
                                                                            if let Some(ref mut api_mgmt) =
                                                                                state.api_level_management
                                                                            {
                                                                                api_mgmt.install_progress =
                                                                                    Some(progress);
                                                                            }
                                                                        });
                                                                    },
                                                                )
                                                                .await;

                                                            // Small delay to ensure final progress update is shown
                                                            tokio::time::sleep(
                                                                API_INSTALLATION_COMPLETION_DELAY,
                                                            )
                                                            .await;

                                                            let mut state =
                                                                state_clone.lock().await;
                                                            if let Some(ref mut api_mgmt) =
                                                                state.api_level_management
                                                            {
                                                                api_mgmt.installing_package = None;
                                                                api_mgmt.install_progress = None;
                                                            }

                                                            if let Err(e) = result {
                                                                if let Some(ref mut api_mgmt) =
                                                                    state.api_level_management
                                                                {
                                                                    api_mgmt.error_message =
                                                                        Some(format!(
                                                                            "Failed to install: {e}"
                                                                        ));
                                                                }
                                                            } else {
                                                                state.add_success_notification(
                                                                    "System image installed successfully"
                                                                        .to_string(),
                                                                );
                                                                // Invalidate device creation cache to ensure new API levels appear
                                                                {
                                                                    let mut cache = state
                                                                        .device_cache
                                                                        .write()
                                                                        .await;
                                                                    cache
                                                                        .invalidate_android_cache();
                                                                }

                                                                // Refresh API levels list to show new installation status
                                                                if let Some(ref mut api_mgmt) =
                                                                    state.api_level_management
                                                                {
                                                                    api_mgmt.is_loading = true;
                                                                }

                                                                // Trigger background refresh
                                                                let android_manager_refresh =
                                                                    android_manager.clone();
                                                                let state_refresh =
                                                                    state_clone.clone();
                                                                tokio::spawn(async move {
                                                                    if let Ok(new_levels) =
                                                                        android_manager_refresh
                                                                            .list_api_levels()
                                                                            .await
                                                                    {
                                                                        let mut state =
                                                                            state_refresh
                                                                                .lock()
                                                                                .await;
                                                                        if let Some(
                                                                            ref mut api_mgmt,
                                                                        ) = state
                                                                            .api_level_management
                                                                        {
                                                                            api_mgmt.api_levels =
                                                                                new_levels;
                                                                            api_mgmt.is_loading =
                                                                                false;
                                                                        }
                                                                    }
                                                                });
                                                                // Don't auto-close dialog - user should close manually
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    KeyCode::Char('d') => {
                                        // 'd' key - Uninstall only
                                        if let Some(ref api_state) = state.api_level_management {
                                            if let Some(api_level) =
                                                api_state.get_selected_api_level()
                                            {
                                                // Get all installed variants for this API level
                                                let installed_variants: Vec<_> = api_level
                                                    .variants
                                                    .iter()
                                                    .filter(|v| v.is_installed)
                                                    .map(|v| v.package_id.clone())
                                                    .collect();

                                                if !installed_variants.is_empty() {
                                                    // Display first package ID in UI (for progress indicator)
                                                    let display_package_id =
                                                        installed_variants[0].clone();

                                                    // Set installing package indicator
                                                    if let Some(ref mut api_mgmt) =
                                                        state.api_level_management
                                                    {
                                                        api_mgmt.installing_package =
                                                            Some(display_package_id.clone());
                                                        api_mgmt.error_message = None;
                                                    }

                                                    let android_manager =
                                                        self.android_manager.clone();
                                                    let state_clone = self.state.clone();

                                                    tokio::spawn(async move {
                                                        let mut success = true;
                                                        let mut last_error = None;

                                                        // Uninstall all installed variants
                                                        for package_id in &installed_variants {
                                                            if let Err(e) = android_manager
                                                                .uninstall_system_image(package_id)
                                                                .await
                                                            {
                                                                success = false;
                                                                last_error = Some(e);
                                                            }
                                                        }

                                                        let mut state = state_clone.lock().await;

                                                        if success {
                                                            // Immediately update the UI state before background refresh
                                                            if let Some(ref mut api_mgmt) =
                                                                state.api_level_management
                                                            {
                                                                // Update the installed status of all uninstalled variants
                                                                for api_level in
                                                                    &mut api_mgmt.api_levels
                                                                {
                                                                    for variant in
                                                                        &mut api_level.variants
                                                                    {
                                                                        if installed_variants
                                                                            .contains(
                                                                                &variant.package_id,
                                                                            )
                                                                        {
                                                                            variant.is_installed =
                                                                                false;
                                                                        }
                                                                    }
                                                                    // Update overall API level installation status
                                                                    api_level.is_installed =
                                                                        api_level
                                                                            .variants
                                                                            .iter()
                                                                            .any(|v| {
                                                                                v.is_installed
                                                                            });
                                                                }

                                                                api_mgmt.installing_package = None;
                                                            }

                                                            state.add_success_notification(
                                                                "System image(s) uninstalled successfully"
                                                                    .to_string(),
                                                            );
                                                        } else {
                                                            // Handle error
                                                            if let Some(ref mut api_mgmt) =
                                                                state.api_level_management
                                                            {
                                                                api_mgmt.installing_package = None;
                                                                api_mgmt.error_message =
                                                                    Some(format!(
                                                                        "Failed to uninstall: {}",
                                                                        last_error.unwrap_or_else(
                                                                            || anyhow::anyhow!(
                                                                                "Unknown error"
                                                                            )
                                                                        )
                                                                    ));
                                                            }
                                                        }

                                                        // Invalidate device creation cache
                                                        {
                                                            let mut cache =
                                                                state.device_cache.write().await;
                                                            cache.invalidate_android_cache();
                                                        }

                                                        // Trigger background refresh
                                                        let android_manager_refresh =
                                                            android_manager.clone();
                                                        let state_refresh = state_clone.clone();
                                                        tokio::spawn(async move {
                                                            if let Ok(new_levels) =
                                                                android_manager_refresh
                                                                    .list_api_levels()
                                                                    .await
                                                            {
                                                                let mut state =
                                                                    state_refresh.lock().await;
                                                                if let Some(ref mut api_mgmt) =
                                                                    state.api_level_management
                                                                {
                                                                    api_mgmt.api_levels =
                                                                        new_levels;
                                                                    api_mgmt.is_loading = false;
                                                                }
                                                            }
                                                        });
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                },
                                Mode::Help => {
                                    // Help mode - any key to exit
                                    match key.code {
                                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') => {
                                            state.mode = Mode::Normal;
                                        }
                                        _ => {}
                                    }
                                }
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
        }
    }

    /// Incrementally refresh device lists by only updating changed devices
    /// This is more efficient than full refresh for large device counts
    async fn refresh_devices_incremental(&mut self) -> Result<()> {
        use std::collections::HashMap;

        let mut state = self.state.lock().await;
        state.is_loading = true;
        let pending_device = state.get_pending_device_start().cloned();

        // Create maps of existing devices for efficient lookup
        let existing_android: HashMap<String, AndroidDevice> = state
            .android_devices
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect();
        let existing_ios: HashMap<String, IosDevice> = state
            .ios_devices
            .iter()
            .map(|d| (d.name.clone(), d.clone()))
            .collect();

        drop(state);

        // Fetch new device lists
        let new_android_devices = self.android_manager.list_devices().await?;
        let new_ios_devices = if let Some(ref ios_manager) = self.ios_manager {
            ios_manager.list_devices().await?
        } else {
            Vec::new()
        };

        let mut state = self.state.lock().await;

        // Update Android devices incrementally
        let mut updated_android = Vec::with_capacity(new_android_devices.len());
        for new_device in new_android_devices {
            if let Some(existing) = existing_android.get(&new_device.name) {
                // Check if only status changed
                if existing.status != new_device.status
                    || existing.is_running != new_device.is_running
                {
                    // Keep existing device but update status
                    let mut updated = existing.clone();
                    updated.status = new_device.status;
                    updated.is_running = new_device.is_running;
                    updated_android.push(updated);
                } else {
                    // No change, keep existing
                    updated_android.push(existing.clone());
                }
            } else {
                // New device
                updated_android.push(new_device);
            }
        }

        // Update iOS devices incrementally
        let mut updated_ios = Vec::with_capacity(new_ios_devices.len());
        for new_device in new_ios_devices {
            if let Some(existing) = existing_ios.get(&new_device.name) {
                // Check if only status changed
                if existing.status != new_device.status
                    || existing.is_running != new_device.is_running
                {
                    // Keep existing device but update status
                    let mut updated = existing.clone();
                    updated.status = new_device.status;
                    updated.is_running = new_device.is_running;
                    updated_ios.push(updated);
                } else {
                    // No change, keep existing
                    updated_ios.push(existing.clone());
                }
            } else {
                // New device
                updated_ios.push(new_device);
            }
        }

        // Check if pending device is now running
        let mut device_started = None;
        if let Some(ref pending_name) = pending_device {
            let device_running = updated_android
                .iter()
                .any(|d| &d.name == pending_name && d.is_running)
                || updated_ios
                    .iter()
                    .any(|d| &d.name == pending_name && d.is_running);

            if device_running {
                state.add_success_notification(format!("Device '{pending_name}' is now running!"));
                state.clear_pending_device_start();
                device_started = Some(pending_name.clone());
            }
        }

        state.android_devices = updated_android;
        state.ios_devices = updated_ios;
        state.is_loading = false;
        state.mark_refreshed();

        // Check if we need to update device details for started device
        let need_detail_update = if let Some(ref started_name) = device_started {
            match state.active_panel {
                Panel::Android => state
                    .android_devices
                    .get(state.selected_android)
                    .map(|d| &d.name == started_name)
                    .unwrap_or(false),
                Panel::Ios => state
                    .ios_devices
                    .get(state.selected_ios)
                    .map(|d| &d.name == started_name)
                    .unwrap_or(false),
            }
        } else {
            false
        };

        drop(state);

        if need_detail_update {
            self.update_device_details().await;
        }

        Ok(())
    }

    async fn toggle_device(&mut self) -> Result<()> {
        let (active_panel, selected_android, selected_ios, android_devices, ios_devices) = {
            let state = self.state.lock().await;
            (
                state.active_panel,
                state.selected_android,
                state.selected_ios,
                state.android_devices.clone(),
                state.ios_devices.clone(),
            )
        };

        let result = match active_panel {
            Panel::Android => {
                if let Some(device) = android_devices.get(selected_android) {
                    let name = device.name.clone();
                    let is_running = device.is_running;

                    if is_running {
                        // Set stopping status
                        {
                            let mut state = self.state.lock().await;
                            state.set_device_operation_status(format!(
                                "Stopping device '{name}'..."
                            ));
                        }

                        match self.android_manager.stop_device(&name).await {
                            Ok(()) => {
                                let mut state = self.state.lock().await;
                                state.clear_device_operation_status();
                                state.add_success_notification(format!("Device '{name}' stopped"));

                                // Immediate UI update for optimal responsiveness
                                state.update_single_android_device_status(&name, false);

                                // Clear device details cache since device status changed
                                if let Some(ref cached) = state.cached_device_details {
                                    if cached.identifier == name {
                                        state.clear_cached_device_details();
                                    }
                                }
                                Ok(())
                            }
                            Err(e) => {
                                let mut state = self.state.lock().await;
                                state.clear_device_operation_status();
                                state.add_error_notification(format!(
                                    "Failed to stop device '{name}': {}",
                                    format_user_error(&e)
                                ));
                                Err(e)
                            }
                        }
                    } else {
                        let mut state = self.state.lock().await;
                        state.set_pending_device_start(name.clone());
                        state.set_device_operation_status(format!("Starting device '{name}'..."));
                        drop(state);

                        match self.android_manager.start_device(&name).await {
                            Ok(()) => {
                                let mut state = self.state.lock().await;
                                state.clear_device_operation_status();
                                state.add_info_notification(format!("Starting device '{name}'..."));

                                // Immediate UI update for optimal responsiveness
                                state.update_single_android_device_status(&name, true);

                                // Clear device details cache since device status is changing
                                if let Some(ref cached) = state.cached_device_details {
                                    if cached.identifier == name {
                                        state.clear_cached_device_details();
                                    }
                                }
                                Ok(())
                            }
                            Err(e) => {
                                let mut state = self.state.lock().await;
                                state.clear_pending_device_start();
                                state.clear_device_operation_status();
                                state.add_error_notification(format!(
                                    "Failed to start device '{name}': {}",
                                    format_user_error(&e)
                                ));
                                Err(e)
                            }
                        }
                    }
                } else {
                    Ok(())
                }
            }
            Panel::Ios => {
                if let Some(ref ios_manager) = self.ios_manager {
                    if let Some(device) = ios_devices.get(selected_ios) {
                        let name = device.name.clone();
                        let udid = device.udid.clone();
                        let is_running = device.is_running;

                        if is_running {
                            // Set stopping status
                            {
                                let mut state = self.state.lock().await;
                                state.set_device_operation_status(format!(
                                    "Stopping device '{name}'..."
                                ));
                            }

                            match ios_manager.stop_device(&udid).await {
                                Ok(()) => {
                                    let mut state = self.state.lock().await;
                                    state.clear_device_operation_status();
                                    state.add_success_notification(format!(
                                        "Device '{name}' stopped"
                                    ));

                                    // Immediate UI update for optimal responsiveness
                                    state.update_single_ios_device_status(&udid, false);

                                    // Clear device details cache since device status changed
                                    if let Some(ref cached) = state.cached_device_details {
                                        if cached.identifier == udid {
                                            state.clear_cached_device_details();
                                        }
                                    }
                                    Ok(())
                                }
                                Err(e) => {
                                    let mut state = self.state.lock().await;
                                    state.clear_device_operation_status();
                                    state.add_error_notification(format!(
                                        "Failed to stop device '{name}': {e}"
                                    ));
                                    Err(e)
                                }
                            }
                        } else {
                            let mut state = self.state.lock().await;
                            state.set_pending_device_start(name.clone());
                            state.set_device_operation_status(format!(
                                "Starting device '{name}'..."
                            ));
                            drop(state);

                            match ios_manager.start_device(&udid).await {
                                Ok(()) => {
                                    let mut state = self.state.lock().await;
                                    state.clear_device_operation_status();
                                    state.add_info_notification(format!(
                                        "Starting device '{name}'..."
                                    ));

                                    // Immediate UI update for optimal responsiveness
                                    state.update_single_ios_device_status(&udid, true);

                                    // Clear device details cache since device status is changing
                                    if let Some(ref cached) = state.cached_device_details {
                                        if cached.identifier == udid {
                                            state.clear_cached_device_details();
                                        }
                                    }
                                    Ok(())
                                }
                                Err(e) => {
                                    let mut state = self.state.lock().await;
                                    state.clear_pending_device_start();
                                    state.clear_device_operation_status();
                                    state.add_error_notification(format!(
                                        "Failed to start device '{name}': {e}"
                                    ));
                                    Err(e)
                                }
                            }
                        }
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            }
        };

        if result.is_ok() {
            // Optimized refresh strategy: immediate UI update with background verification
            // We already updated the device status in the operation,
            // so we only need to schedule a background status check for accuracy
            self.schedule_background_device_status_check().await;
        }
        Ok(())
    }

    async fn update_log_stream(&mut self) -> Result<()> {
        // Clone necessary data for the delayed task
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        // Execute the log update immediately (no delay)
        Self::update_log_stream_internal(state_clone, android_manager, ios_manager).await;
        Ok(())
    }

    async fn update_log_stream_internal(
        state: Arc<Mutex<AppState>>,
        android_manager: AndroidManager,
        _ios_manager: Option<IosManager>,
    ) {
        let (
            active_panel,
            selected_android,
            selected_ios,
            android_devices,
            ios_devices,
            _current_log_device,
        ) = {
            let state_lock = state.lock().await;
            (
                state_lock.active_panel,
                state_lock.selected_android,
                state_lock.selected_ios,
                state_lock.android_devices.clone(),
                state_lock.ios_devices.clone(),
                state_lock.current_log_device.clone(),
            )
        };

        // Get current device info
        let (device_name, device_is_running) = match active_panel {
            Panel::Android => {
                if let Some(device) = android_devices.get(selected_android) {
                    (device.name.clone(), device.is_running)
                } else {
                    return;
                }
            }
            Panel::Ios => {
                if let Some(device) = ios_devices.get(selected_ios) {
                    (device.name.clone(), device.is_running)
                } else {
                    return;
                }
            }
        };

        // Always update log stream to ensure correct device selection
        // Note: Commented out early return to force log stream update
        // if let Some((panel, name)) = &current_log_device {
        //     if panel == &active_panel && name == &device_name {
        //         // Already streaming logs for this device, no need to update
        //         return;
        //     }
        // }

        if !device_is_running {
            // Clear current log device
            let mut state_lock = state.lock().await;
            state_lock.current_log_device = None;
            return;
        }

        // Update current log device
        {
            let mut state_lock = state.lock().await;
            state_lock.current_log_device = Some((active_panel, device_name.clone()));

            // Cancel existing log task before starting new one
            if let Some(handle) = state_lock.log_task_handle.take() {
                handle.abort();
            }
        }

        match active_panel {
            Panel::Android => {
                if let Some(device) = android_devices.get(selected_android) {
                    if device.is_running {
                        // Clear logs only when switching to a running device
                        {
                            let mut state_lock = state.lock().await;
                            state_lock.clear_logs();
                            state_lock.reset_log_scroll();
                        }

                        let device_name = device.name.clone();
                        let state_clone = Arc::clone(&state);

                        // Get the emulator serial for this AVD
                        if let Ok(running_avds) = android_manager.get_running_avd_names().await {
                            if let Some(emulator_serial) = running_avds.get(&device_name) {
                                let serial = emulator_serial.clone();
                                let handle = tokio::spawn(async move {
                                    Self::stream_android_logs(state_clone, device_name, serial)
                                        .await;
                                });
                                let mut state_lock = state.lock().await;
                                state_lock.log_task_handle = Some(handle);
                            } else {
                                // Try with normalized name (spaces to underscores)
                                let normalized_name = device_name.replace(' ', "_");
                                if let Some(emulator_serial) = running_avds.get(&normalized_name) {
                                    let serial = emulator_serial.clone();
                                    let handle = tokio::spawn(async move {
                                        Self::stream_android_logs(state_clone, device_name, serial)
                                            .await;
                                    });
                                    let mut state_lock = state.lock().await;
                                    state_lock.log_task_handle = Some(handle);
                                } else {
                                    // As a fallback, try to match any running emulator
                                    if device.is_running && !running_avds.is_empty() {
                                        if let Some((_, serial)) = running_avds.iter().next() {
                                            let serial = serial.clone();
                                            let handle = tokio::spawn(async move {
                                                Self::stream_android_logs(
                                                    state_clone,
                                                    device_name,
                                                    serial,
                                                )
                                                .await;
                                            });
                                            let mut state_lock = state.lock().await;
                                            state_lock.log_task_handle = Some(handle);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Panel::Ios => {
                if let Some(device) = ios_devices.get(selected_ios) {
                    if device.is_running {
                        // Clear logs only when switching to a running device
                        {
                            let mut state_lock = state.lock().await;
                            state_lock.clear_logs();
                            state_lock.reset_log_scroll();
                        }

                        let device_udid = device.udid.clone();
                        let device_name = device.name.clone();
                        let state_clone = Arc::clone(&state);
                        let handle = tokio::spawn(async move {
                            Self::stream_ios_logs(state_clone, device_udid, device_name).await;
                        });
                        let mut state_lock = state.lock().await;
                        state_lock.log_task_handle = Some(handle);
                    }
                }
            }
        }
    }

    async fn stream_android_logs(
        state: Arc<Mutex<AppState>>,
        device_name: String,
        emulator_serial: String,
    ) {
        let result = Command::new("adb")
            .args(["-s", &emulator_serial, "logcat", "-v", "time"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null()) // Suppress stderr output
            .stdin(std::process::Stdio::null()) // No stdin needed
            .spawn();

        match result {
            Ok(mut child) => {
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();

                    loop {
                        tokio::select! {
                            result = lines.next_line() => {
                                match result {
                                    Ok(Some(line)) => {
                                        // No filtering - show all Android logs
                                        if line.trim().is_empty() {
                                            continue;
                                        }

                                        let level = if line.contains(" E ") || line.contains("ERROR") {
                                            "ERROR"
                                        } else if line.contains(" W ") || line.contains("WARN") {
                                            "WARN"
                                        } else if line.contains(" I ") || line.contains("INFO") {
                                            "INFO"
                                        } else if line.contains(" D ") || line.contains("DEBUG") {
                                            "DEBUG"
                                        } else {
                                            "INFO"
                                        };

                                        let message = line;

                                        let mut state = state.lock().await;
                                        state.add_log(level.to_string(), message);
                                    }
                                    Ok(None) => break, // Stream ended
                                    Err(_) => break,   // Error occurred
                                }
                            }
                            _ = tokio::time::sleep(DETAIL_UPDATE_DEBOUNCE) => {
                                // Check if task should be cancelled by checking if we're still the active log device
                                let should_continue = {
                                    let state_lock = state.lock().await;
                                    if let Some((panel, name)) = &state_lock.current_log_device {
                                        panel == &crate::app::Panel::Android && name == &device_name
                                    } else {
                                        false
                                    }
                                };
                                if !should_continue {
                                    break;
                                }
                            }
                        }
                    }
                }

                let _ = child.kill().await;
            }
            Err(_) => {
                // Silently fail - no error messages in logs
            }
        }
    }

    async fn stream_ios_logs(
        state: Arc<Mutex<AppState>>,
        device_udid: String,
        _device_name: String,
    ) {
        // Try different log streaming methods - start with most reliable
        let log_commands = [
            // Method 1: Use simctl with log command (most reliable)
            (
                "xcrun",
                vec!["simctl", "spawn", &device_udid, "log", "stream"],
            ),
            // Method 2: System log stream without filtering
            ("log", vec!["stream", "--style", "compact"]),
            // Method 3: Simple log stream
            ("log", vec!["stream"]),
        ];

        for (command, args) in log_commands.iter() {
            let result = tokio::process::Command::new(command)
                .args(args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();

            match result {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        use tokio::io::{AsyncBufReadExt, BufReader};
                        let reader = BufReader::new(stdout);
                        let mut lines = reader.lines();

                        // Start streaming immediately without timeout
                        while let Ok(Some(line_content)) = lines.next_line().await {
                            if line_content.trim().is_empty() {
                                continue;
                            }

                            let level = if line_content.contains("error")
                                || line_content.contains(LOG_LEVEL_ERROR)
                            {
                                "ERROR"
                            } else if line_content.contains("warning")
                                || line_content.contains(LOG_LEVEL_WARNING)
                            {
                                "WARN"
                            } else {
                                "INFO"
                            };

                            let mut app_state = state.lock().await;
                            app_state.add_log(level.to_string(), line_content);
                        }
                        break;
                    }

                    let _ = child.kill().await;
                }
                Err(_) => {
                    // Try next method
                    continue;
                }
            }
        }

        // If no log streaming worked, just wait silently
        // No status messages - logs will appear naturally when events occur
    }

    #[allow(dead_code)]
    async fn load_available_versions(&mut self) -> Result<()> {
        let state = self.state.lock().await;

        match state.active_panel {
            Panel::Android => {
                // Get available targets (API levels) and device types
                drop(state);

                // Load available device types with category filter
                let available_devices = {
                    let state = self.state.lock().await;
                    let category_filter =
                        if state.create_device_form.device_category_filter == "all" {
                            None
                        } else {
                            Some(state.create_device_form.device_category_filter.clone())
                        };
                    drop(state);
                    self.android_manager
                        .list_devices_by_category(category_filter.as_deref())
                        .await?
                };
                if available_devices.is_empty() {
                    let mut state = self.state.lock().await;
                    state.create_device_form.error_message = Some(
                        "No Android device definitions found. Check your Android SDK installation."
                            .to_string(),
                    );
                    return Ok(());
                }

                // Load available API levels
                let available_targets = self.android_manager.list_available_targets().await?;
                if available_targets.is_empty() {
                    let mut state = self.state.lock().await;
                    state.create_device_form.error_message = Some("No Android targets found. Use Android Studio SDK Manager to install system images.".to_string());
                    return Ok(());
                }

                let mut state = self.state.lock().await;
                state.create_device_form.available_device_types = available_devices;
                state.create_device_form.available_versions = available_targets;

                // Set defaults
                if !state.create_device_form.available_device_types.is_empty() {
                    let (id, display) = state.create_device_form.available_device_types[0].clone();
                    state.create_device_form.device_type_id = id;
                    state.create_device_form.device_type = display;
                    state.create_device_form.selected_device_type_index = 0;
                }

                if !state.create_device_form.available_versions.is_empty() {
                    let (value, display) = state.create_device_form.available_versions[0].clone();
                    state.create_device_form.version = value;
                    state.create_device_form.version_display = display;
                    state.create_device_form.selected_api_level_index = 0;
                }

                // Generate placeholder name
                state.create_device_form.generate_placeholder_name();
            }
            Panel::Ios => {
                // Get actually available iOS runtimes and device types
                if let Some(ref ios_manager) = self.ios_manager {
                    drop(state);

                    // Load device types with display names
                    let available_device_types = ios_manager.list_device_types_with_names().await?;
                    if available_device_types.is_empty() {
                        let mut state = self.state.lock().await;
                        state.create_device_form.error_message =
                            Some("No iOS device types available.".to_string());
                        return Ok(());
                    }

                    // Load runtimes with display names
                    let available_runtimes = ios_manager.list_runtimes().await?;
                    if available_runtimes.is_empty() {
                        let mut state = self.state.lock().await;
                        state.create_device_form.error_message = Some(
                            "No iOS runtimes available. Install iOS runtimes using Xcode."
                                .to_string(),
                        );
                        return Ok(());
                    }

                    let mut state = self.state.lock().await;
                    state.create_device_form.available_device_types = available_device_types;
                    state.create_device_form.available_versions = available_runtimes;

                    // Set defaults
                    if !state.create_device_form.available_device_types.is_empty() {
                        let (id, display) =
                            state.create_device_form.available_device_types[0].clone();
                        state.create_device_form.device_type_id = id;
                        state.create_device_form.device_type = display;
                        state.create_device_form.selected_device_type_index = 0;
                    }

                    if !state.create_device_form.available_versions.is_empty() {
                        let (value, display) =
                            state.create_device_form.available_versions[0].clone();
                        state.create_device_form.version = value;
                        state.create_device_form.version_display = display;
                        state.create_device_form.selected_api_level_index = 0;
                    }

                    // Generate placeholder name
                    state.create_device_form.generate_placeholder_name();
                } else {
                    let mut state = self.state.lock().await;
                    state.create_device_form.error_message =
                        Some("iOS simulator not available on this platform.".to_string());
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn handle_create_device_char(&self, state: &mut AppState, c: char) {
        use crate::app::state::CreateDeviceField;

        match state.create_device_form.active_field {
            CreateDeviceField::Name => {
                state.create_device_form.name.push(c);
            }
            CreateDeviceField::Category => {
                // Category is selected via up/down arrows
            }
            CreateDeviceField::DeviceType => {
                // Device type is selected via left/right arrows
            }
            CreateDeviceField::ApiLevel => {
                // API Level is selected via up/down arrows when options are available
            }
            CreateDeviceField::RamSize => {
                if c.is_ascii_digit() {
                    state.create_device_form.ram_size.push(c);
                }
            }
            CreateDeviceField::StorageSize => {
                if c.is_ascii_digit() {
                    state.create_device_form.storage_size.push(c);
                }
            }
        }
        state.create_device_form.error_message = None;
    }

    fn handle_create_device_backspace(&self, state: &mut AppState) {
        use crate::app::state::CreateDeviceField;

        match state.create_device_form.active_field {
            CreateDeviceField::Name => {
                state.create_device_form.name.pop();
            }
            CreateDeviceField::Category => {
                // Category is selected via up/down arrows
            }
            CreateDeviceField::DeviceType => {
                // Device type is selected via left/right arrows
            }
            CreateDeviceField::ApiLevel => {
                // API Level is selected via up/down arrows when options are available
            }
            CreateDeviceField::RamSize => {
                state.create_device_form.ram_size.pop();
            }
            CreateDeviceField::StorageSize => {
                state.create_device_form.storage_size.pop();
            }
        }
        state.create_device_form.error_message = None;
    }

    fn handle_create_device_left(&self, state: &mut AppState) {
        use crate::app::state::CreateDeviceField;

        match state.create_device_form.active_field {
            CreateDeviceField::Category => {
                if state.create_device_form.selected_category_index > 0 {
                    state.create_device_form.selected_category_index -= 1;
                } else {
                    state.create_device_form.selected_category_index =
                        state.create_device_form.available_categories.len() - 1;
                }
                state.create_device_form.update_selected_category();
            }
            CreateDeviceField::DeviceType => {
                let options = &state.create_device_form.available_device_types;
                if let Some(current_index) = options
                    .iter()
                    .position(|(id, _)| id == &state.create_device_form.device_type_id)
                {
                    let new_index = if current_index == 0 {
                        options.len() - 1
                    } else {
                        current_index - 1
                    };
                    let (id, display) = options[new_index].clone();
                    state.create_device_form.device_type_id = id;
                    state.create_device_form.device_type = display;
                    state.create_device_form.selected_device_type_index = new_index;
                    state.create_device_form.generate_placeholder_name();
                }
            }
            CreateDeviceField::ApiLevel => {
                let options = &state.create_device_form.available_versions;
                if !options.is_empty() {
                    if let Some(current_index) = options
                        .iter()
                        .position(|(value, _)| value == &state.create_device_form.version)
                    {
                        let new_index = if current_index == 0 {
                            options.len() - 1
                        } else {
                            current_index - 1
                        };
                        let (value, display) = options[new_index].clone();
                        state.create_device_form.version = value;
                        state.create_device_form.version_display = display;
                        state.create_device_form.selected_api_level_index = new_index;
                        state.create_device_form.generate_placeholder_name();
                    }
                }
            }
            _ => {}
        }
        state.create_device_form.error_message = None;
    }

    fn handle_create_device_right(&self, state: &mut AppState) {
        use crate::app::state::CreateDeviceField;

        match state.create_device_form.active_field {
            CreateDeviceField::Category => {
                let len = state.create_device_form.available_categories.len();
                state.create_device_form.selected_category_index =
                    (state.create_device_form.selected_category_index + 1) % len;
                state.create_device_form.update_selected_category();
            }
            CreateDeviceField::DeviceType => {
                let options = &state.create_device_form.available_device_types;
                if let Some(current_index) = options
                    .iter()
                    .position(|(id, _)| id == &state.create_device_form.device_type_id)
                {
                    let new_index = (current_index + 1) % options.len();
                    let (id, display) = options[new_index].clone();
                    state.create_device_form.device_type_id = id;
                    state.create_device_form.device_type = display;
                    state.create_device_form.selected_device_type_index = new_index;
                    state.create_device_form.generate_placeholder_name();
                }
            }
            CreateDeviceField::ApiLevel => {
                let options = &state.create_device_form.available_versions;
                if !options.is_empty() {
                    if let Some(current_index) = options
                        .iter()
                        .position(|(value, _)| value == &state.create_device_form.version)
                    {
                        let new_index = (current_index + 1) % options.len();
                        let (value, display) = options[new_index].clone();
                        state.create_device_form.version = value;
                        state.create_device_form.version_display = display;
                        state.create_device_form.selected_api_level_index = new_index;
                        state.create_device_form.generate_placeholder_name();
                    }
                }
            }
            _ => {}
        }
        state.create_device_form.error_message = None;
    }

    async fn submit_create_device(&mut self) -> Result<()> {
        use crate::managers::common::{DeviceConfig, DeviceManager};

        // Get form data and validate
        let (active_panel, form_data, config) = {
            let state = self.state.lock().await;
            let form_data = state.create_device_form.clone();

            // Validate form
            if form_data.name.trim().is_empty() {
                drop(state);
                let mut state = self.state.lock().await;
                state.create_device_form.error_message =
                    Some("Device name is required".to_string());
                return Ok(());
            }

            if form_data.version.trim().is_empty() {
                drop(state);
                let mut state = self.state.lock().await;
                state.create_device_form.error_message = Some("Version is required".to_string());
                return Ok(());
            }

            // Create device config
            let device_name = form_data.name.clone();
            let device_type = form_data.device_type_id.clone();
            let mut config = DeviceConfig::new(device_name, device_type, form_data.version.clone());

            // Add Android-specific options
            if matches!(state.active_panel, Panel::Android) {
                if !form_data.ram_size.is_empty() {
                    config = config.with_ram(form_data.ram_size.clone());
                }
                if !form_data.storage_size.is_empty() {
                    config = config.with_storage(form_data.storage_size.clone());
                }
            }

            (state.active_panel, form_data, config)
        };

        // Set creating state
        {
            let mut state = self.state.lock().await;
            state.create_device_form.is_creating = true;
            state.create_device_form.creation_status =
                Some("Initializing device creation...".to_string());
            state.create_device_form.error_message = None;
        }

        // Clone necessary data for async operation
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();
        let device_name_for_display = form_data.name.clone();

        // Create device asynchronously
        tokio::spawn(async move {
            // Update status
            {
                let mut state = state_clone.lock().await;
                state.create_device_form.creation_status =
                    Some(format!("Creating device '{device_name_for_display}'..."));
            }

            // Perform the actual creation
            let result = match active_panel {
                Panel::Android => {
                    // Add a small delay to show the progress
                    tokio::time::sleep(DETAIL_UPDATE_DEBOUNCE).await;
                    android_manager.create_device(&config).await
                }
                Panel::Ios => {
                    if let Some(ref ios_manager) = ios_manager {
                        tokio::time::sleep(DETAIL_UPDATE_DEBOUNCE).await;
                        ios_manager.create_device(&config).await
                    } else {
                        Err(anyhow::anyhow!("iOS manager not available"))
                    }
                }
            };

            // Handle result
            match result {
                Ok(()) => {
                    // Update status
                    {
                        let mut state = state_clone.lock().await;
                        state.create_device_form.creation_status =
                            Some("Finalizing...".to_string());
                    }

                    // Refresh devices for the affected platform and update state
                    match active_panel {
                        Panel::Android => {
                            if let Ok(devices) = android_manager.list_devices().await {
                                let mut state = state_clone.lock().await;
                                state.android_devices = devices;
                                state.mode = Mode::Normal;
                                state.create_device_form.is_creating = false;
                                state.create_device_form.creation_status = None;
                                state.add_success_notification(format!(
                                    "Device '{device_name_for_display}' created successfully"
                                ));
                            } else {
                                let mut state = state_clone.lock().await;
                                state.mode = Mode::Normal;
                                state.create_device_form.is_creating = false;
                                state.create_device_form.creation_status = None;
                                state.add_success_notification(format!(
                                    "Device '{device_name_for_display}' created successfully"
                                ));
                            }
                        }
                        Panel::Ios => {
                            if let Some(ref ios_manager) = ios_manager {
                                if let Ok(devices) = ios_manager.list_devices().await {
                                    let mut state = state_clone.lock().await;
                                    state.ios_devices = devices;
                                    state.mode = Mode::Normal;
                                    state.create_device_form.is_creating = false;
                                    state.create_device_form.creation_status = None;
                                    state.add_success_notification(format!(
                                        "Device '{device_name_for_display}' created successfully"
                                    ));
                                } else {
                                    let mut state = state_clone.lock().await;
                                    state.mode = Mode::Normal;
                                    state.create_device_form.is_creating = false;
                                    state.create_device_form.creation_status = None;
                                    state.add_success_notification(format!(
                                        "Device '{device_name_for_display}' created successfully"
                                    ));
                                }
                            } else {
                                let mut state = state_clone.lock().await;
                                state.mode = Mode::Normal;
                                state.create_device_form.is_creating = false;
                                state.create_device_form.creation_status = None;
                                state.add_error_notification(
                                    "iOS manager not available (only available on macOS)"
                                        .to_string(),
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    // Update state with error
                    let mut state = state_clone.lock().await;
                    state.create_device_form.is_creating = false;
                    state.create_device_form.creation_status = None;
                    state.add_error_notification(format!(
                        "Device creation error: {}",
                        format_user_error(&e)
                    ));
                    state.create_device_form.error_message = Some(format_user_error(&e));
                }
            }
        });

        Ok(())
    }

    async fn execute_delete_device(&mut self) -> Result<()> {
        use crate::managers::common::DeviceManager;

        let dialog_info = {
            let mut state = self.state.lock().await;
            state.confirm_delete_dialog.take()
        };

        if let Some(dialog) = dialog_info {
            let result = match dialog.platform {
                Panel::Android => {
                    self.android_manager
                        .delete_device(&dialog.device_identifier)
                        .await
                }
                Panel::Ios => {
                    if let Some(ref ios_manager) = self.ios_manager {
                        ios_manager.delete_device(&dialog.device_identifier).await
                    } else {
                        return Err(anyhow::anyhow!("iOS manager not available"));
                    }
                }
            };

            match result {
                Ok(()) => {
                    let mut state = self.state.lock().await;

                    // Optimized: Remove the device from local state instead of full refresh
                    match dialog.platform {
                        Panel::Android => {
                            state
                                .android_devices
                                .retain(|device| device.name != dialog.device_identifier);
                            // Adjust selection if needed
                            if state.selected_android >= state.android_devices.len() {
                                state.selected_android =
                                    state.android_devices.len().saturating_sub(1);
                            }
                        }
                        Panel::Ios => {
                            state
                                .ios_devices
                                .retain(|device| device.udid != dialog.device_identifier);
                            // Adjust selection if needed
                            if state.selected_ios >= state.ios_devices.len() {
                                state.selected_ios = state.ios_devices.len().saturating_sub(1);
                            }
                        }
                    }

                    state.clear_device_operation_status();
                    state.add_success_notification(format!(
                        "Device '{}' deleted successfully",
                        dialog.device_name
                    ));
                }
                Err(e) => {
                    let mut state = self.state.lock().await;
                    state.clear_device_operation_status();
                    state.add_error_notification(format!(
                        "Failed to delete device '{}': {}",
                        dialog.device_name, e
                    ));
                }
            }
        }

        Ok(())
    }

    async fn execute_wipe_device(&mut self) -> Result<()> {
        use crate::managers::common::DeviceManager;

        let dialog_info = {
            let mut state = self.state.lock().await;
            state.confirm_wipe_dialog.take()
        };

        if let Some(dialog) = dialog_info {
            let result = match dialog.platform {
                Panel::Android => {
                    self.android_manager
                        .wipe_device(&dialog.device_identifier)
                        .await
                }
                Panel::Ios => {
                    if let Some(ref ios_manager) = self.ios_manager {
                        ios_manager.wipe_device(&dialog.device_identifier).await
                    } else {
                        let mut state = self.state.lock().await;
                        state.clear_device_operation_status();
                        return Err(anyhow::anyhow!("iOS manager not available"));
                    }
                }
            };

            match result {
                Ok(()) => {
                    let mut state = self.state.lock().await;
                    state.clear_device_operation_status();
                    state.add_success_notification(format!(
                        "Device '{}' wiped successfully",
                        dialog.device_name
                    ));

                    // Update only the specific device status instead of full refresh
                    match dialog.platform {
                        Panel::Android => {
                            drop(state);
                            self.update_single_android_device_status(&dialog.device_identifier)
                                .await;
                        }
                        Panel::Ios => {
                            drop(state);
                            self.update_single_ios_device_status(&dialog.device_identifier)
                                .await;
                        }
                    }
                    self.update_device_details().await;
                }
                Err(e) => {
                    let mut state = self.state.lock().await;
                    state.clear_device_operation_status();
                    state.add_error_notification(format!(
                        "Failed to wipe device '{}': {}",
                        dialog.device_name,
                        format_user_error(&e)
                    ));
                }
            }
        }

        Ok(())
    }

    /// Reload device types when category filter changes
    async fn reload_device_types_for_category(&mut self) -> Result<()> {
        // First, get the necessary info and check cache
        let (current_panel, category_filter, device_cache_clone) = {
            let state = self.state.lock().await;
            let device_cache_clone = Arc::clone(&state.device_cache);
            (
                state.active_panel,
                state.create_device_form.device_category_filter.clone(),
                device_cache_clone,
            )
        };

        match current_panel {
            Panel::Android => {
                // Check cache availability without holding state lock
                let cache = device_cache_clone.read().await;
                let cached_devices = cache.android_device_cache.clone();
                drop(cache);

                if let Some(all_devices) = cached_devices {
                    // Filter in memory instead of making new API call
                    let filtered_devices = if category_filter == "all" {
                        all_devices
                    } else {
                        all_devices
                            .into_iter()
                            .filter(|(id, display)| {
                                let device_category =
                                    self.android_manager.get_device_category(id, display);
                                device_category == category_filter
                            })
                            .collect()
                    };

                    // Update state with filtered devices
                    let mut state = self.state.lock().await;
                    state.create_device_form.available_device_types = filtered_devices;

                    // Reset device type selection
                    state.create_device_form.selected_device_type_index = 0;
                    if !state.create_device_form.available_device_types.is_empty() {
                        let (id, display) =
                            state.create_device_form.available_device_types[0].clone();
                        state.create_device_form.device_type_id = id;
                        state.create_device_form.device_type = display;
                        state.create_device_form.generate_placeholder_name();
                    }
                } else {
                    // Fallback to API call if cache not available
                    let filtered_devices = self
                        .android_manager
                        .list_devices_by_category(if category_filter == "all" {
                            None
                        } else {
                            Some(&category_filter)
                        })
                        .await?;

                    let mut state = self.state.lock().await;
                    state.create_device_form.available_device_types = filtered_devices;

                    // Reset device type selection
                    state.create_device_form.selected_device_type_index = 0;
                    if !state.create_device_form.available_device_types.is_empty() {
                        let (id, display) =
                            state.create_device_form.available_device_types[0].clone();
                        state.create_device_form.device_type_id = id;
                        state.create_device_form.device_type = display;
                        state.create_device_form.generate_placeholder_name();
                    }
                }
            }
            Panel::Ios => {
                // iOS doesn't need category filtering for now
            }
        }

        Ok(())
    }

    /// Start background device info cache loading
    fn start_background_cache_loading(&mut self) {
        let state_clone = Arc::clone(&self.state);

        // Run as true background task (non-blocking UI)
        tokio::spawn(async move {
            // Start immediately (no waiting)

            // Create new manager instance here (avoid startup blocking)
            if let Ok(android_manager) = crate::managers::AndroidManager::new() {
                // Update Android cache
                if let Ok(device_types) = android_manager.list_available_devices().await {
                    if let Ok(api_levels) = android_manager.list_available_targets().await {
                        let state = state_clone.lock().await;
                        let mut cache = state.device_cache.write().await;
                        cache.android_device_cache = Some(device_types.clone());
                        cache.update_android_cache(device_types, api_levels);
                        log::info!("Android device cache updated successfully");
                    }
                }
            }

            // Update iOS cache (macOS only)
            #[cfg(target_os = "macos")]
            if let Ok(ios_manager) = crate::managers::IosManager::new() {
                if let Ok(device_types) = ios_manager.list_device_types_with_names().await {
                    if let Ok(runtimes) = ios_manager.list_runtimes().await {
                        let state = state_clone.lock().await;
                        let mut cache = state.device_cache.write().await;
                        cache.update_ios_cache(device_types, runtimes);
                        log::info!("iOS device cache updated successfully");
                    }
                }
            }
        });
    }

    /// Load device list in background (improve startup speed)
    fn start_background_device_loading(&mut self) {
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        // 真のバックグラウンドタスクとして実行（UI ブロックなし）
        tokio::spawn(async move {
            // 短時間待機して UI が表示されてから読み込み
            tokio::time::sleep(LOG_UPDATE_DEBOUNCE).await;

            // Android デバイス一覧を読み込み
            match android_manager.list_devices().await {
                Ok(android_devices) => {
                    let mut state = state_clone.lock().await;
                    state.android_devices = android_devices;
                    state.is_loading = false;
                    state.mark_refreshed();

                    // Get details for first device if selected
                    let should_update_details = state.active_panel == Panel::Android
                        && !state.android_devices.is_empty()
                        && state.cached_device_details.is_none();
                    drop(state);

                    if should_update_details {
                        // Update device details asynchronously
                        let state_clone2 = Arc::clone(&state_clone);
                        let android_manager_clone = android_manager.clone();
                        tokio::spawn(async move {
                            {
                                let state = state_clone2.lock().await;
                                if let Some(device) =
                                    state.android_devices.get(state.selected_android)
                                {
                                    let device_name = device.name.clone();
                                    drop(state);

                                    if let Ok(details) =
                                        android_manager_clone.get_device_details(&device_name).await
                                    {
                                        let mut state = state_clone2.lock().await;
                                        state.update_cached_device_details(details);
                                    }
                                }
                            }
                        });
                    }
                }
                Err(e) => {
                    let mut state = state_clone.lock().await;
                    state.is_loading = false;
                    state.add_error_notification(format!("Failed to load Android devices: {e}"));
                    drop(state);
                }
            }

            // Load iOS device list (macOS only)
            if let Some(ios_manager) = ios_manager {
                match ios_manager.list_devices().await {
                    Ok(ios_devices) => {
                        let mut state = state_clone.lock().await;
                        state.ios_devices = ios_devices;

                        // Get details for first device if selected
                        let should_update_details = state.active_panel == Panel::Ios
                            && !state.ios_devices.is_empty()
                            && state.cached_device_details.is_none();
                        drop(state);

                        if should_update_details {
                            // Update device details asynchronously
                            let state_clone2 = Arc::clone(&state_clone);
                            let _ios_manager_clone = ios_manager.clone();
                            tokio::spawn(async move {
                                {
                                    let state = state_clone2.lock().await;
                                    if let Some(device) = state.ios_devices.get(state.selected_ios)
                                    {
                                        // Generate iOS device details from basic info
                                        let details = crate::app::state::DeviceDetails {
                                            name: device.name.clone(),
                                            status: if device.is_running {
                                                "Running".to_string()
                                            } else {
                                                "Stopped".to_string()
                                            },
                                            platform: Panel::Ios,
                                            device_type: device.device_type.clone(),
                                            api_level_or_version: format!(
                                                "iOS {}",
                                                device.ios_version
                                            ),
                                            ram_size: None,
                                            storage_size: None,
                                            resolution: None,
                                            dpi: None,
                                            device_path: None,
                                            system_image: None,
                                            identifier: device.udid.clone(),
                                        };
                                        drop(state);

                                        let mut state = state_clone2.lock().await;
                                        state.update_cached_device_details(details);
                                    }
                                }
                            });
                        }
                    }
                    Err(e) => {
                        let mut state = state_clone.lock().await;
                        state.add_error_notification(format!("Failed to load iOS devices: {e}"));
                        drop(state);
                    }
                }
            }
        });
    }

    /// Update device details for the currently selected device
    async fn update_device_details(&mut self) {
        let (active_panel, device_identifier) = {
            let state = self.state.lock().await;
            let identifier = match state.active_panel {
                Panel::Android => state
                    .android_devices
                    .get(state.selected_android)
                    .map(|d| d.name.clone()),
                Panel::Ios => state
                    .ios_devices
                    .get(state.selected_ios)
                    .map(|d| d.udid.clone()),
            };
            (state.active_panel, identifier)
        };

        if let Some(identifier) = device_identifier {
            // Get detailed information asynchronously
            match active_panel {
                Panel::Android => {
                    if let Ok(details) = self.android_manager.get_device_details(&identifier).await
                    {
                        let mut state = self.state.lock().await;
                        state.update_cached_device_details(details);
                    }
                }
                Panel::Ios => {
                    // TODO: Implement iOS device details
                    // For now, just use basic details
                }
            }
        }
    }

    /// Schedule device details update with delay to avoid performance issues
    async fn schedule_device_details_update(&mut self) {
        // Cancel any pending detail update
        if let Some(handle) = self.detail_update_handle.take() {
            handle.abort();
        }

        // Clone necessary data for the delayed task
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        // Use optimized delay for fast panel switching
        let delay = FAST_DETAIL_UPDATE_DEBOUNCE;

        // Create a delayed task to update device details
        let update_handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            Self::update_device_details_internal(state_clone, android_manager, ios_manager).await;
        });

        self.detail_update_handle = Some(update_handle);
    }

    /// Schedule both log stream and device details updates in parallel for fast panel switching
    async fn schedule_panel_switch_updates_parallel(&mut self) {
        // Cancel any pending updates
        if let Some(handle) = self.log_update_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.detail_update_handle.take() {
            handle.abort();
        }

        // Clone necessary data for the parallel task
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        // Use optimized delays for fast switching
        let log_delay = FAST_LOG_UPDATE_DEBOUNCE;
        let detail_delay = FAST_DETAIL_UPDATE_DEBOUNCE;

        // Launch parallel tasks
        let state_clone_log = Arc::clone(&state_clone);
        let android_manager_log = android_manager.clone();
        let ios_manager_log = ios_manager.clone();

        let log_handle = tokio::spawn(async move {
            tokio::time::sleep(log_delay).await;
            Self::update_log_stream_internal(state_clone_log, android_manager_log, ios_manager_log)
                .await;
        });

        let detail_handle = tokio::spawn(async move {
            tokio::time::sleep(detail_delay).await;
            Self::update_device_details_internal(state_clone, android_manager, ios_manager).await;
        });

        // Store handles for potential cancellation
        self.log_update_handle = Some(log_handle);
        self.detail_update_handle = Some(detail_handle);
    }

    /// Schedule background device status check for smart device start mode.
    /// This performs a lightweight status check after a delay to ensure accuracy.
    async fn update_single_android_device_status(&mut self, device_name: &str) {
        // Get actual device status from manager
        if let Ok(devices) = self.android_manager.list_devices().await {
            if let Some(device) = devices.iter().find(|d| d.name == device_name) {
                let mut state = self.state.lock().await;
                state.update_single_android_device_status(device_name, device.is_running);
            }
        }
    }

    async fn update_single_ios_device_status(&mut self, device_udid: &str) {
        // Get actual device status from manager
        if let Some(ref ios_manager) = self.ios_manager {
            if let Ok(devices) = ios_manager.list_devices().await {
                if let Some(device) = devices.iter().find(|d| d.udid == device_udid) {
                    let mut state = self.state.lock().await;
                    state.update_single_ios_device_status(device_udid, device.is_running);
                }
            }
        }
    }

    async fn schedule_background_device_status_check(&mut self) {
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        tokio::spawn(async move {
            // Wait a bit for device to fully start/stop
            tokio::time::sleep(DEVICE_STOP_WAIT_TIME).await;

            // Get current active device
            let (active_panel, device_identifier) = {
                let state = state_clone.lock().await;
                let identifier = match state.active_panel {
                    Panel::Android => state
                        .android_devices
                        .get(state.selected_android)
                        .map(|d| d.name.clone()),
                    Panel::Ios => state
                        .ios_devices
                        .get(state.selected_ios)
                        .map(|d| d.udid.clone()),
                };
                (state.active_panel, identifier)
            };

            if let Some(identifier) = device_identifier {
                match active_panel {
                    Panel::Android => {
                        // Check actual Android device status
                        if let Ok(devices) = android_manager.list_devices().await {
                            if let Some(device) = devices.iter().find(|d| d.name == identifier) {
                                let mut state = state_clone.lock().await;
                                state.update_single_android_device_status(
                                    &identifier,
                                    device.is_running,
                                );
                            }
                        }
                    }
                    Panel::Ios => {
                        // Check actual iOS device status
                        if let Some(ios_manager) = ios_manager {
                            if let Ok(devices) = ios_manager.list_devices().await {
                                if let Some(device) = devices.iter().find(|d| d.udid == identifier)
                                {
                                    let mut state = state_clone.lock().await;
                                    state.update_single_ios_device_status(
                                        &identifier,
                                        device.is_running,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    async fn update_device_details_internal(
        state: Arc<Mutex<AppState>>,
        android_manager: AndroidManager,
        ios_manager: Option<IosManager>,
    ) {
        let (active_panel, device_identifier) = {
            let state_lock = state.lock().await;
            let identifier = match state_lock.active_panel {
                Panel::Android => state_lock
                    .android_devices
                    .get(state_lock.selected_android)
                    .map(|d| d.name.clone()),
                Panel::Ios => state_lock
                    .ios_devices
                    .get(state_lock.selected_ios)
                    .map(|d| d.udid.clone()),
            };
            (state_lock.active_panel, identifier)
        };

        if let Some(identifier) = device_identifier {
            // Get detailed information asynchronously
            match active_panel {
                Panel::Android => {
                    if let Ok(details) = android_manager.get_device_details(&identifier).await {
                        let mut state_lock = state.lock().await;
                        state_lock.update_cached_device_details(details);
                    }
                }
                Panel::Ios => {
                    // TODO: Implement iOS device details
                    if let Some(_ios_manager) = ios_manager {
                        // For now, just use basic details from state
                    }
                }
            }
        }
    }
}
