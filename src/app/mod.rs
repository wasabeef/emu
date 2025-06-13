//! Application core module for coordinating all Emu functionality.
//!
//! This module serves as the central controller for the application, managing:
//! - Application lifecycle and main event loop
//! - Coordination between UI, state, and device managers
//! - Background task management for async operations
//! - User input handling and action dispatch

/// Event type definitions for user input and system events.
pub mod events;

/// Application state management and data structures.
pub mod state;

/// Optimized event processing for handling rapid key repeats.
pub mod event_processing;

use crate::{
    managers::common::DeviceManager,
    managers::{AndroidManager, IosManager},
    models::error::format_user_error,
    models::SdkInstallStatus,
    ui,
};
use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

// Re-export commonly used types from the state module
pub use self::state::{AppState, FocusedPanel, Mode, Panel};

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

    /// Last time navigation occurred, used for debouncing background updates
    last_navigation_time: std::time::Instant,

    /// Whether navigation updates are pending
    navigation_updates_pending: bool,

    /// Last time a navigation key was processed (for input debouncing)
    last_navigation_key_time: std::time::Instant,

    /// Navigation event accumulator for batching
    navigation_accumulator: i32,
}

impl App {
    /// Creates a new application instance with initialized managers and state.
    ///
    /// This function:
    /// 1. Initializes the application state with default values
    /// 2. Creates platform-specific device managers (Android and iOS)
    /// 3. Starts background cache loading for fast startup
    /// 4. Initiates background device discovery
    ///
    /// # Performance
    ///
    /// The function prioritizes fast startup by deferring expensive operations:
    /// - Device discovery runs in the background via `tokio::spawn`
    /// - Cache loading is non-blocking
    /// - UI renders immediately (~50ms) with loading indicators
    /// - Total startup time typically under 150ms (average: ~104ms)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Android SDK is not properly configured (missing `ANDROID_HOME`)
    /// - iOS tools are unavailable on macOS (missing Xcode)
    /// - Initial manager creation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use emu::App;
    /// # async fn example() -> anyhow::Result<()> {
    /// let app = App::new().await?;
    /// # Ok(())
    /// # }
    /// ```
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
            last_navigation_time: std::time::Instant::now(),
            navigation_updates_pending: false,
            last_navigation_key_time: std::time::Instant::now(),
            navigation_accumulator: 0,
        };

        // Start background operations for optimal startup performance
        app.start_background_cache_loading();
        app.start_background_device_loading();

        Ok(app)
    }

    /// Runs the main application event loop optimized for high-performance operation.
    ///
    /// This function implements a high-performance event loop designed for 60+ FPS operation:
    /// 1. **Consistent 125 FPS rendering** (8ms frame time) for all operations
    /// 2. **Advanced event processing** with intelligent batching and debouncing
    /// 3. **Sub-8ms input latency** through optimized polling and processing
    /// 4. **Background task coordination** with performance-aware scheduling
    /// 5. **Real-time log monitoring** with minimal overhead
    ///
    /// # High-Performance Event Processing
    ///
    /// The event system is optimized for responsiveness:
    /// - **4ms polling intervals** for sub-frame input detection
    /// - **Batch processing**: Up to 50 events per frame with 4ms time limit
    /// - **Navigation optimization**: Event accumulation with 8ms debouncing
    /// - **Background task debouncing**: 50ms delay to prevent UI stuttering
    /// - **Intelligent rendering**: Event-driven with consistent frame timing
    ///
    /// # Performance Architecture
    ///
    /// The loop maintains consistent performance through:
    /// - **Frame budget management**: 4ms for events, 4ms for rendering
    /// - **Task prioritization**: UI responsiveness over background operations
    /// - **Memory efficiency**: Event queue management and log rotation
    /// - **CPU optimization**: Event-driven rendering reduces unnecessary work
    ///
    /// # Background Task Management
    ///
    /// Background operations are coordinated to avoid blocking the UI:
    /// - **Auto-refresh**: Device status updates every 1000ms
    /// - **Notification cleanup**: Expired message removal every 500ms
    /// - **Log streaming**: Real-time updates with intelligent cancellation
    /// - **Device details**: Debounced updates after navigation pauses
    ///
    /// # Performance
    ///
    /// - Event polling: 10ms for instant key response
    /// - Minimum render interval: 8ms (125 FPS max)
    /// - Navigation keys trigger immediate rendering
    /// - Background operations never block the UI
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
        const AUTO_REFRESH_CHECK_INTERVAL: Duration = Duration::from_millis(1000);
        const NOTIFICATION_CHECK_INTERVAL: Duration = Duration::from_millis(500);
        let mut last_notification_check = std::time::Instant::now();

        // Event-driven rendering: track when rendering is needed
        let mut needs_render = true;
        let mut last_render = std::time::Instant::now();
        let mut last_log_count = 0;
        const MIN_RENDER_INTERVAL: Duration = Duration::from_millis(8); // ~125 FPS for smooth operation
        const ANIMATION_RENDER_INTERVAL: Duration = Duration::from_millis(8); // Same as normal render for consistent performance

        // Initial render with full redraw
        {
            let mut state = self.state.lock().await;
            terminal.draw(|f| ui::render::draw_app(f, &mut state, &ui::Theme::dark()))?;
        }

        loop {
            // Check for auto-refresh less frequently (skip if initial loading)
            if last_auto_refresh_check.elapsed() >= AUTO_REFRESH_CHECK_INTERVAL {
                let state = self.state.lock().await;
                let should_refresh = state.should_auto_refresh();
                let has_devices =
                    !state.android_devices.is_empty() || !state.ios_devices.is_empty();
                let has_background_install = state.background_install_status.is_some();
                drop(state);

                // Only refresh if we have devices loaded (not during initial loading)
                if should_refresh && has_devices {
                    self.refresh_devices().await?;
                    needs_render = true;
                }

                // Also trigger render if background installation is in progress
                if has_background_install {
                    needs_render = true;
                }

                last_auto_refresh_check = std::time::Instant::now();
            }

            // Dismiss expired notifications less frequently
            if last_notification_check.elapsed() >= NOTIFICATION_CHECK_INTERVAL {
                let mut state = self.state.lock().await;
                let dismissed_count = state.notifications.len();
                state.dismiss_expired_notifications();
                if state.notifications.len() != dismissed_count {
                    needs_render = true;
                }
                drop(state);
                last_notification_check = std::time::Instant::now();
            }

            // Check for new logs (for real-time log display)
            {
                let state = self.state.lock().await;
                let current_log_count = state.device_logs.len();
                if current_log_count != last_log_count {
                    needs_render = true;
                    last_log_count = current_log_count;
                }
            }

            // Check if we need animation
            let should_animate = {
                let state = self.state.lock().await;
                state.is_loading
                    || state.device_operation_status.is_some()
                    || state.api_level_install.as_ref().is_some_and(|d| {
                        d.is_loading
                            || d.install_status.is_in_progress()
                            || d.uninstall_status
                                .as_ref()
                                .is_some_and(|s| s.is_in_progress())
                    })
            };

            // If animating, always render at animation interval
            if should_animate {
                needs_render = true;
            }

            // Process any accumulated navigation events immediately for responsiveness
            if self.navigation_accumulator != 0
                && self.last_navigation_key_time.elapsed() >= Duration::from_millis(8)
            {
                // Process at 125 FPS
                let mut state = self.state.lock().await;
                state.move_by_steps(self.navigation_accumulator);
                drop(state);
                self.navigation_accumulator = 0;
                needs_render = true;
            }

            // Check if we should process pending navigation updates
            // Process after 50ms of no navigation for faster updates
            if self.navigation_updates_pending
                && self.last_navigation_time.elapsed() >= Duration::from_millis(50)
            {
                self.navigation_updates_pending = false;
                self.schedule_log_stream_update().await;
                self.schedule_device_details_update().await;
            }

            // Only render when needed and respecting minimum interval
            let render_interval = if should_animate {
                ANIMATION_RENDER_INTERVAL
            } else {
                MIN_RENDER_INTERVAL
            };

            if needs_render && last_render.elapsed() >= render_interval {
                let mut state = self.state.lock().await;
                terminal.draw(|f| ui::render::draw_app(f, &mut state, &ui::Theme::dark()))?;
                drop(state);

                // Reset needs_render only if not animating
                needs_render = !should_animate;
                last_render = std::time::Instant::now();
            }

            // Process events with limits to prevent flooding
            let mut events_processed = false;
            let mut events_in_batch = 0;
            const MAX_EVENTS_PER_FRAME: usize = 50; // Process more events for high refresh rates
            let batch_start = std::time::Instant::now();
            const MAX_BATCH_TIME: Duration = Duration::from_millis(4); // 4ms to leave time for rendering in 8ms frame

            while event::poll(Duration::from_millis(0))? && events_in_batch < MAX_EVENTS_PER_FRAME {
                // Don't spend too long processing events
                if batch_start.elapsed() > MAX_BATCH_TIME {
                    break;
                }

                events_processed = true;
                events_in_batch += 1;
                match event::read()? {
                    CrosstermEvent::Key(key) => {
                        // Any key press should trigger a render
                        needs_render = true;
                        let mut state = self.state.lock().await;

                        match state.mode {
                            Mode::Normal => {
                                match key.code {
                                    KeyCode::Char('q')
                                        if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                    {
                                        // Cancel log task if running
                                        if let Some(handle) = state.log_task_handle.take() {
                                            handle.abort();
                                        }
                                        return Ok(());
                                    }
                                    KeyCode::Char('q') => {
                                        // Plain 'q' also quits in Normal mode
                                        // Cancel log task if running
                                        if let Some(handle) = state.log_task_handle.take() {
                                            handle.abort();
                                        }
                                        return Ok(());
                                    }
                                    KeyCode::Char('c')
                                        if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                    {
                                        // Ctrl+C also quits
                                        // Cancel log task if running
                                        if let Some(handle) = state.log_task_handle.take() {
                                            handle.abort();
                                        }
                                        return Ok(());
                                    }
                                    KeyCode::Esc => {
                                        // Dismiss all notifications
                                        state.dismiss_all_notifications();
                                    }
                                    KeyCode::Char('r') => {
                                        drop(state);
                                        self.refresh_devices().await?;
                                    }
                                    KeyCode::Tab => {
                                        // Tab: Switch focus between panels (android -> ios -> android)
                                        let new_panel = state.active_panel.toggle();
                                        state.smart_clear_cached_device_details(new_panel); // Smart cache clearing
                                        state.active_panel = new_panel;
                                        drop(state);
                                        // パネル切り替えを高速化：ログストリームとデバイス詳細を遅延更新
                                        self.schedule_log_stream_update().await;
                                        self.schedule_device_details_update().await;
                                    }
                                    KeyCode::BackTab => {
                                        // Shift+Tab: Switch focus in reverse order (android -> ios -> android)
                                        let new_panel = state.active_panel.toggle();
                                        state.smart_clear_cached_device_details(new_panel); // Smart cache clearing
                                        state.active_panel = new_panel;
                                        drop(state);
                                        // パネル切り替えを高速化：ログストリームとデバイス詳細を遅延更新
                                        self.schedule_log_stream_update().await;
                                        self.schedule_device_details_update().await;
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
                                        // パネル切り替えを高速化：ログストリームとデバイス詳細を遅延更新
                                        self.schedule_log_stream_update().await;
                                        self.schedule_device_details_update().await;
                                    }
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        // Debounce rapid navigation keys
                                        let now = std::time::Instant::now();
                                        if now.duration_since(self.last_navigation_key_time)
                                            >= Duration::from_millis(8)
                                        {
                                            // 8ms for 125 FPS operation
                                            // Apply any accumulated moves plus this one
                                            state.move_by_steps(self.navigation_accumulator - 1);
                                            self.navigation_accumulator = 0;
                                            self.last_navigation_key_time = now;
                                            self.last_navigation_time = now;
                                            self.navigation_updates_pending = true;
                                        } else {
                                            // Accumulate the move
                                            self.navigation_accumulator -= 1;
                                        }
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        // Debounce rapid navigation keys
                                        let now = std::time::Instant::now();
                                        if now.duration_since(self.last_navigation_key_time)
                                            >= Duration::from_millis(8)
                                        {
                                            // 8ms for 125 FPS operation
                                            // Apply any accumulated moves plus this one
                                            state.move_by_steps(self.navigation_accumulator + 1);
                                            self.navigation_accumulator = 0;
                                            self.last_navigation_key_time = now;
                                            self.last_navigation_time = now;
                                            self.navigation_updates_pending = true;
                                        } else {
                                            // Accumulate the move
                                            self.navigation_accumulator += 1;
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
                                    // Removed PageUp/PageDown log scrolling
                                    // Removed auto-scroll toggle
                                    KeyCode::Char('L')
                                        if key.modifiers.contains(KeyModifiers::SHIFT) =>
                                    {
                                        // Shift+L: Clear logs (global shortcut)
                                        state.clear_logs();
                                        state.add_info_notification("Logs cleared".to_string());
                                    }
                                    // Removed all log area specific controls
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
                                                                    .available_versions = targets;
                                                                state
                                                                    .create_device_form
                                                                    .available_device_types =
                                                                    devices;

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
                                                    Panel::Ios => {
                                                        if let Some(ref ios_manager) = ios_manager {
                                                            if let Ok(device_types) = ios_manager
                                                                .list_device_types_with_names()
                                                                .await
                                                            {
                                                                if let Ok(runtimes) = ios_manager
                                                                    .list_runtimes()
                                                                    .await
                                                                {
                                                                    let mut state =
                                                                        state_clone.lock().await;
                                                                    state
                                                                        .create_device_form
                                                                        .available_device_types =
                                                                        device_types;
                                                                    state
                                                                        .create_device_form
                                                                        .available_versions =
                                                                        runtimes;

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
                                                                        state.create_device_form.selected_device_type_index = 0;
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
                                                                        state.create_device_form.selected_api_level_index = 0;
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
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    KeyCode::Char('i') => {
                                        // Only available for Android panel
                                        if state.active_panel == Panel::Android {
                                            state.mode = Mode::ApiLevelInstall;
                                            let dialog = state::ApiLevelInstallDialog::new();
                                            state.api_level_install = Some(dialog);
                                            // UI will immediately show loading state
                                            // Start background loading without awaiting
                                            let android_manager = self.android_manager.clone();
                                            let state_clone = Arc::clone(&self.state);
                                            tokio::spawn(async move {
                                                match android_manager
                                                    .list_available_api_levels()
                                                    .await
                                                {
                                                    Ok(api_levels) => {
                                                        let mut state = state_clone.lock().await;
                                                        if let Some(dialog) =
                                                            &mut state.api_level_install
                                                        {
                                                            dialog.update_api_levels(api_levels);
                                                        }
                                                    }
                                                    Err(e) => {
                                                        let mut state = state_clone.lock().await;
                                                        if let Some(dialog) =
                                                            &mut state.api_level_install
                                                        {
                                                            dialog.set_error(format!(
                                                                "Failed to load API levels: {}",
                                                                e
                                                            ));
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    KeyCode::Char('d') => {
                                        // d: Delete device
                                        let (device_name, device_id) = match state.active_panel {
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
                                        let (device_name, device_id) = match state.active_panel {
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
                                    KeyCode::Char('g') => {
                                        // Go to top of device list
                                        match state.active_panel {
                                            Panel::Android => {
                                                if !state.android_devices.is_empty() {
                                                    state.selected_android = 0;
                                                }
                                            }
                                            Panel::Ios => {
                                                if !state.ios_devices.is_empty() {
                                                    state.selected_ios = 0;
                                                }
                                            }
                                        }
                                        // No need to force render, let the main loop handle it
                                    }
                                    KeyCode::Char('G') => {
                                        // Go to bottom of device list
                                        match state.active_panel {
                                            Panel::Android => {
                                                if !state.android_devices.is_empty() {
                                                    state.selected_android =
                                                        state.android_devices.len() - 1;
                                                }
                                            }
                                            Panel::Ios => {
                                                if !state.ios_devices.is_empty() {
                                                    state.selected_ios =
                                                        state.ios_devices.len() - 1;
                                                }
                                            }
                                        }
                                        // No need to force render, let the main loop handle it
                                    }
                                    _ => {}
                                }
                            }
                            Mode::CreateDevice => match key.code {
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
                                            Panel::Android => state.create_device_form.next_field(),
                                            Panel::Ios => state.create_device_form.next_field_ios(),
                                        }
                                    }
                                }
                                KeyCode::BackTab => {
                                    // Ignore navigation if currently creating
                                    if !state.create_device_form.is_creating {
                                        match state.active_panel {
                                            Panel::Android => state.create_device_form.prev_field(),
                                            Panel::Ios => state.create_device_form.prev_field_ios(),
                                        }
                                    }
                                }
                                KeyCode::Down => {
                                    // Ignore navigation if currently creating
                                    if !state.create_device_form.is_creating {
                                        match state.active_panel {
                                            Panel::Android => state.create_device_form.next_field(),
                                            Panel::Ios => state.create_device_form.next_field_ios(),
                                        }
                                    }
                                }
                                KeyCode::Up => {
                                    // Ignore navigation if currently creating
                                    if !state.create_device_form.is_creating {
                                        match state.active_panel {
                                            Panel::Android => state.create_device_form.prev_field(),
                                            Panel::Ios => state.create_device_form.prev_field_ios(),
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
                                                // カテゴリが変更された場合、デバイスリストを再読み込み
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
                                                        let mut state = self.state.lock().await;
                                                        state.create_device_form.error_message =
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
                                                // カテゴリが変更された場合、デバイスリストを再読み込み
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
                                                        let mut state = self.state.lock().await;
                                                        state.create_device_form.error_message =
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
                                                            self.handle_create_device_left(
                                                                &mut state,
                                                            );
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
                                                            self.handle_create_device_left(
                                                                &mut state,
                                                            );
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
                                                            self.handle_create_device_right(
                                                                &mut state,
                                                            );
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
                                                            self.handle_create_device_right(
                                                                &mut state,
                                                            );
                                                        }
                                                    }
                                                }
                                                'j' => {
                                                    // Ctrl+j = Down arrow (field navigation)
                                                    match state.active_panel {
                                                        Panel::Android => {
                                                            state.create_device_form.next_field()
                                                        }
                                                        Panel::Ios => state
                                                            .create_device_form
                                                            .next_field_ios(),
                                                    }
                                                }
                                                'k' => {
                                                    // Ctrl+k = Up arrow (field navigation)
                                                    match state.active_panel {
                                                        Panel::Android => {
                                                            state.create_device_form.prev_field()
                                                        }
                                                        Panel::Ios => state
                                                            .create_device_form
                                                            .prev_field_ios(),
                                                    }
                                                }
                                                _ => {
                                                    // Other Ctrl+char combinations - treat as regular char input
                                                    self.handle_create_device_char(&mut state, c);
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
                            },
                            Mode::ConfirmDelete => match key.code {
                                KeyCode::Char('y') | KeyCode::Char('Y') => {
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
                                    drop(state);
                                    self.execute_wipe_device().await?;
                                }
                                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                    state.mode = Mode::Normal;
                                    state.confirm_wipe_dialog = None;
                                }
                                _ => {}
                            },
                            Mode::ApiLevelInstall => match key.code {
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if let Some(dialog) = &mut state.api_level_install {
                                        // Don't allow navigation during installation
                                        if !dialog.install_status.is_in_progress() {
                                            // Use faster movement without debounce for smoother scrolling
                                            dialog.move_selection_up_with_debounce(false);
                                        }
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if let Some(dialog) = &mut state.api_level_install {
                                        // Don't allow navigation during installation
                                        if !dialog.install_status.is_in_progress() {
                                            // Use faster movement without debounce for smoother scrolling
                                            dialog.move_selection_down_with_debounce(false);
                                        }
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Some(dialog) = &state.api_level_install {
                                        // Prevent Enter during installation or uninstallation
                                        if dialog.install_status.is_in_progress() {
                                            state.add_warning_notification(
                                                "Installation in progress, please wait..."
                                                    .to_string(),
                                            );
                                        } else if dialog
                                            .uninstall_status
                                            .as_ref()
                                            .is_some_and(|s| s.is_in_progress())
                                        {
                                            state.add_warning_notification(
                                                "Uninstallation in progress, please wait..."
                                                    .to_string(),
                                            );
                                        } else if let Some(selected) = dialog.selected_api_level() {
                                            if dialog.uninstall_mode {
                                                // Uninstall mode
                                                if !selected.installed {
                                                    let api_level = selected.level;
                                                    state.add_info_notification(format!(
                                                        "API {} is not installed",
                                                        api_level
                                                    ));
                                                } else {
                                                    let api_level = selected.level;
                                                    drop(state);
                                                    self.uninstall_selected_api_level(api_level)
                                                        .await?;
                                                }
                                            } else {
                                                // Install mode
                                                if selected.installed {
                                                    let api_level = selected.level;
                                                    state.add_info_notification(format!(
                                                        "API {} is already installed",
                                                        api_level
                                                    ));
                                                } else {
                                                    let api_level = selected.level;
                                                    drop(state);
                                                    self.install_selected_api_level(api_level)
                                                        .await?;
                                                }
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char('d') => {
                                    if let Some(dialog) = &mut state.api_level_install {
                                        // Toggle uninstall mode
                                        if !dialog.install_status.is_in_progress()
                                            && dialog
                                                .uninstall_status
                                                .as_ref()
                                                .is_none_or(|s| !s.is_in_progress())
                                        {
                                            dialog.uninstall_mode = !dialog.uninstall_mode;
                                            let mode_msg = if dialog.uninstall_mode {
                                                "Uninstall mode enabled - Select API to uninstall"
                                            } else {
                                                "Install mode enabled - Select API to install"
                                            };
                                            state.add_info_notification(mode_msg.to_string());
                                        }
                                    }
                                }
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    // Check if installation is in progress and move to background
                                    let (is_in_progress, api_level_opt, install_status_opt) =
                                        if let Some(dialog) = &state.api_level_install {
                                            let in_progress =
                                                dialog.install_status.is_in_progress();
                                            let api_level =
                                                dialog.selected_api_level().map(|api| api.level);
                                            let status = if in_progress {
                                                Some(dialog.install_status.clone())
                                            } else {
                                                None
                                            };
                                            (in_progress, api_level, status)
                                        } else {
                                            (false, None, None)
                                        };

                                    if is_in_progress {
                                        state.add_info_notification(
                                            "Installation continues in background".to_string(),
                                        );
                                        // Move install status to background
                                        if let (Some(api_level), Some(status)) =
                                            (api_level_opt, install_status_opt)
                                        {
                                            state.background_install_status =
                                                Some((api_level, status));
                                        }
                                    }

                                    state.mode = Mode::Normal;
                                    state.api_level_install = None;
                                }
                                _ => {}
                            },
                            Mode::Help => match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    state.mode = Mode::Normal;
                                }
                                _ => {}
                            },
                        }
                    }
                    _ => {
                        // Ignore other events (mouse, resize, etc.)
                    }
                }
            }

            // If no events were processed, poll with shorter timeout for higher responsiveness
            if !events_processed {
                let _ = event::poll(Duration::from_millis(4))?;
            }
        }
    }

    async fn refresh_devices(&mut self) -> Result<()> {
        let mut state = self.state.lock().await;
        state.is_loading = true;
        let pending_device = state.get_pending_device_start().cloned();
        drop(state);

        // Run Android and iOS device listing concurrently
        let android_future = self.android_manager.list_devices();
        let ios_future = async {
            if let Some(ref ios_manager) = self.ios_manager {
                ios_manager.list_devices().await
            } else {
                Ok(Vec::new())
            }
        };

        let (android_result, ios_result) = tokio::join!(android_future, ios_future);
        let android_devices = android_result?;
        let ios_devices = ios_result?;

        let mut state = self.state.lock().await;

        // Check if pending device is now running
        let mut device_started = None;
        if let Some(ref pending_name) = pending_device {
            let device_running = android_devices
                .iter()
                .any(|d| &d.name == pending_name && d.is_running)
                || ios_devices
                    .iter()
                    .any(|d| &d.name == pending_name && d.is_running);

            if device_running {
                state
                    .add_success_notification(format!("Device '{}' is now running!", pending_name));
                state.clear_pending_device_start();
                device_started = Some(pending_name.clone());
            }
        }

        state.android_devices = android_devices;
        state.ios_devices = ios_devices;
        state.is_loading = false;
        state.mark_refreshed();

        // Mark panels for re-render after device refresh

        // Check if we need to update device details for started device
        let need_detail_update = if let Some(ref started_name) = device_started {
            // Check if the started device is currently selected and displayed
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

        // Update log stream for currently selected device
        self.update_log_stream().await?;

        // Update device details if the started device is currently selected
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
                                "Stopping device '{}'...",
                                name
                            ));
                        }

                        match self.android_manager.stop_device(&name).await {
                            Ok(()) => {
                                let mut state = self.state.lock().await;
                                state.clear_device_operation_status();
                                state
                                    .add_success_notification(format!("Device '{}' stopped", name));
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
                                    "Failed to stop device '{}': {}",
                                    name,
                                    format_user_error(&e)
                                ));
                                Err(e)
                            }
                        }
                    } else {
                        let mut state = self.state.lock().await;
                        state.set_pending_device_start(name.clone());
                        state.set_device_operation_status(format!("Starting device '{}'...", name));
                        drop(state);

                        match self.android_manager.start_device(&name).await {
                            Ok(()) => {
                                let mut state = self.state.lock().await;
                                state.clear_device_operation_status();
                                state.add_info_notification(format!(
                                    "Starting device '{}'...",
                                    name
                                ));
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
                                    "Failed to start device '{}': {}",
                                    name,
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
                                    "Stopping device '{}'...",
                                    name
                                ));
                            }

                            match ios_manager.stop_device(&udid).await {
                                Ok(()) => {
                                    let mut state = self.state.lock().await;
                                    state.clear_device_operation_status();
                                    state.add_success_notification(format!(
                                        "Device '{}' stopped",
                                        name
                                    ));
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
                                        "Failed to stop device '{}': {}",
                                        name, e
                                    ));
                                    Err(e)
                                }
                            }
                        } else {
                            let mut state = self.state.lock().await;
                            state.set_pending_device_start(name.clone());
                            state.set_device_operation_status(format!(
                                "Starting device '{}'...",
                                name
                            ));
                            drop(state);

                            match ios_manager.start_device(&udid).await {
                                Ok(()) => {
                                    let mut state = self.state.lock().await;
                                    state.clear_device_operation_status();
                                    state.add_info_notification(format!(
                                        "Starting device '{}'...",
                                        name
                                    ));
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
                                        "Failed to start device '{}': {}",
                                        name, e
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
            self.refresh_devices().await?;
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

    /// Schedule log stream update with delay (for panel switching)
    async fn schedule_log_stream_update(&mut self) {
        // Cancel any pending log update
        if let Some(handle) = self.log_update_handle.take() {
            handle.abort();
        }

        // First, immediately cancel the current log stream
        {
            let mut state = self.state.lock().await;
            if let Some(handle) = state.log_task_handle.take() {
                handle.abort();
            }
        }

        // Update logs in background
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        // Spawn as a task to avoid blocking
        let update_handle = tokio::spawn(async move {
            Self::update_log_stream_internal(state_clone, android_manager, ios_manager).await;
        });

        self.log_update_handle = Some(update_handle);
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
            current_log_device,
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

        // Check if we're already streaming logs for this device
        if let Some((panel, name)) = &current_log_device {
            if panel == &active_panel && name == &device_name {
                // Already streaming logs for this device, no need to update
                return;
            }
        }

        if !device_is_running {
            // Clear current log device
            let mut state_lock = state.lock().await;
            state_lock.current_log_device = None;
            return;
        }

        // Cancel existing log task before starting new one
        {
            let mut state_lock = state.lock().await;
            if let Some(handle) = state_lock.log_task_handle.take() {
                handle.abort();
                // No need to wait - the task will stop on its own
            }
            // Update current log device
            state_lock.current_log_device = Some((active_panel, device_name.clone()));
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
            .args(["-s", &emulator_serial, "logcat", "-v", "time", "-T", "0"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null()) // Suppress stderr output
            .stdin(std::process::Stdio::null()) // No stdin needed
            .spawn();

        match result {
            Ok(mut child) => {
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::with_capacity(256, stdout); // Small buffer for low latency
                    let mut lines = reader.lines();

                    while let Ok(Some(line)) = lines.next_line().await {
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
                        // Check if we're still streaming for the correct device
                        if let Some((Panel::Android, ref current_device)) = state.current_log_device
                        {
                            if current_device == &device_name {
                                state.add_log(level.to_string(), message);
                            } else {
                                // Different device selected, stop streaming
                                break;
                            }
                        } else {
                            // No device selected or wrong platform, stop streaming
                            break;
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
        device_name: String,
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
                        let reader = BufReader::with_capacity(256, stdout); // Small buffer for low latency
                        let mut lines = reader.lines();

                        // Start streaming immediately without timeout
                        while let Ok(Some(line_content)) = lines.next_line().await {
                            if line_content.trim().is_empty() {
                                continue;
                            }

                            let level = if line_content.contains("error")
                                || line_content.contains("Error")
                            {
                                "ERROR"
                            } else if line_content.contains("warning")
                                || line_content.contains("Warning")
                            {
                                "WARN"
                            } else {
                                "INFO"
                            };

                            let mut app_state = state.lock().await;
                            // Check if we're still streaming for the correct device
                            if let Some((Panel::Ios, ref current_device)) =
                                app_state.current_log_device
                            {
                                if current_device == &device_name {
                                    app_state.add_log(level.to_string(), line_content);
                                } else {
                                    // Different device selected, stop streaming
                                    break;
                                }
                            } else {
                                // No device selected or wrong platform, stop streaming
                                break;
                            }
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
                    Some(format!("Creating device '{}'...", device_name_for_display));
            }

            // Perform the actual creation
            let result = match active_panel {
                Panel::Android => {
                    // Add a small delay to show the progress
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    android_manager.create_device(&config).await
                }
                Panel::Ios => {
                    if let Some(ref ios_manager) = ios_manager {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
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
                                    "Device '{}' created successfully",
                                    device_name_for_display
                                ));
                            } else {
                                let mut state = state_clone.lock().await;
                                state.mode = Mode::Normal;
                                state.create_device_form.is_creating = false;
                                state.create_device_form.creation_status = None;
                                state.add_success_notification(format!(
                                    "Device '{}' created successfully",
                                    device_name_for_display
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
                                        "Device '{}' created successfully",
                                        device_name_for_display
                                    ));
                                } else {
                                    let mut state = state_clone.lock().await;
                                    state.mode = Mode::Normal;
                                    state.create_device_form.is_creating = false;
                                    state.create_device_form.creation_status = None;
                                    state.add_success_notification(format!(
                                        "Device '{}' created successfully",
                                        device_name_for_display
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
            let dialog = state.confirm_delete_dialog.take();
            state.mode = Mode::Normal;
            dialog
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

                    state.add_success_notification(format!(
                        "Device '{}' deleted successfully",
                        dialog.device_name
                    ));
                }
                Err(e) => {
                    let mut state = self.state.lock().await;
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
            let dialog = state.confirm_wipe_dialog.take();
            state.mode = Mode::Normal;
            dialog
        };

        if let Some(dialog) = dialog_info {
            // Set wiping status
            {
                let mut state = self.state.lock().await;
                state.set_device_operation_status(format!(
                    "Wiping device '{}'...",
                    dialog.device_name
                ));
            }

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

                    // Refresh devices to update status
                    drop(state);
                    self.refresh_devices().await?;
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

    /// カテゴリフィルター変更時にデバイスタイプを再読み込み
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

                    // デバイスタイプ選択をリセット
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

                    // デバイスタイプ選択をリセット
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

    /// Installs the selected API level.
    async fn install_selected_api_level(&mut self, api_level: u32) -> Result<()> {
        let android_manager = self.android_manager.clone();
        let state = Arc::clone(&self.state);

        // Cancel any existing background installation
        {
            let mut state_guard = self.state.lock().await;
            if let Some(handle) = state_guard.background_install_handle.take() {
                handle.abort();
            }
        }

        // Start installation in background with progress reporting
        let handle = tokio::spawn(async move {
            let result = android_manager
                .install_api_level_with_progress(api_level, |status| {
                    if let Ok(mut state) = state.try_lock() {
                        // Update dialog if it's still open
                        if let Some(dialog) = &mut state.api_level_install {
                            dialog.update_install_status(status.clone());
                        }
                        // Also update background status
                        state.background_install_status = Some((api_level, status));
                    }
                })
                .await;

            // Handle result
            let mut state = state.lock().await;
            match result {
                Ok(()) => {
                    state.add_success_notification(format!(
                        "Successfully installed API Level {}",
                        api_level
                    ));
                    // Invalidate device cache to force refresh of available API levels
                    {
                        let mut cache = state.device_cache.write().await;
                        cache.last_updated = std::time::Instant::now()
                            .checked_sub(std::time::Duration::from_secs(301))
                            .unwrap_or_else(std::time::Instant::now);
                        // Clear cached data to force reload
                        cache.android_api_levels.clear();
                        cache.android_device_types.clear();
                        cache.android_device_cache = None;
                    }
                    // Return to normal mode after successful installation
                    state.mode = Mode::Normal;
                    state.api_level_install = None;
                }
                Err(e) => {
                    state.add_error_notification(format!(
                        "Failed to install API Level {}: {}",
                        api_level, e
                    ));
                    if let Some(dialog) = &mut state.api_level_install {
                        dialog.update_install_status(crate::models::SdkInstallStatus::Failed {
                            error: e.to_string(),
                        });
                    }
                }
            }
            // Clear background status when done
            state.background_install_status = None;
            state.background_install_handle = None;
        });

        // Store the handle
        {
            let mut state = self.state.lock().await;
            state.background_install_handle = Some(handle);
        }

        Ok(())
    }

    async fn uninstall_selected_api_level(&mut self, api_level: u32) -> Result<()> {
        // Update dialog state to show uninstalling
        {
            let mut state = self.state.lock().await;
            if let Some(dialog) = &mut state.api_level_install {
                dialog.uninstall_status = Some(SdkInstallStatus::Installing {
                    progress: 0,
                    message: format!("Uninstalling API Level {}...", api_level),
                });
            }
        }

        let android_manager = self.android_manager.clone();
        let state_clone = Arc::clone(&self.state);

        // Spawn uninstallation task
        tokio::spawn(async move {
            match android_manager.uninstall_api_level(api_level).await {
                Ok(()) => {
                    let mut state = state_clone.lock().await;
                    if let Some(dialog) = &mut state.api_level_install {
                        dialog.uninstall_status = Some(SdkInstallStatus::Completed);
                        // Refresh API levels
                        if let Ok(api_levels) = android_manager.list_available_api_levels().await {
                            dialog.available_api_levels = api_levels;
                        }
                    }
                    state.add_success_notification(format!(
                        "Successfully uninstalled API Level {}",
                        api_level
                    ));
                    // Invalidate device cache to force refresh of available API levels
                    {
                        let mut cache = state.device_cache.write().await;
                        cache.last_updated = std::time::Instant::now()
                            .checked_sub(std::time::Duration::from_secs(301))
                            .unwrap_or_else(std::time::Instant::now);
                        // Clear cached data to force reload
                        cache.android_api_levels.clear();
                        cache.android_device_types.clear();
                        cache.android_device_cache = None;
                    }
                }
                Err(e) => {
                    let mut state = state_clone.lock().await;
                    if let Some(dialog) = &mut state.api_level_install {
                        dialog.uninstall_status = Some(SdkInstallStatus::Failed {
                            error: format_user_error(&e),
                        });
                    }
                    state.add_error_notification(format!(
                        "Failed to uninstall API Level {}: {}",
                        api_level,
                        format_user_error(&e)
                    ));
                }
            }
        });

        Ok(())
    }

    /// バックグラウンドでデバイス情報キャッシュを開始
    fn start_background_cache_loading(&mut self) {
        let state_clone = Arc::clone(&self.state);

        // 真のバックグラウンドタスクとして実行（UIブロックなし）
        tokio::spawn(async move {
            // 即座に開始（待機なし）

            // ここで新しいマネージャーインスタンスを作成（起動時のブロッキングを回避）
            if let Ok(android_manager) = crate::managers::AndroidManager::new() {
                // Android キャッシュを更新
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

            // iOS キャッシュを更新 (macOS のみ)
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

    /// バックグラウンドでデバイス一覧を読み込み（起動速度を向上）
    fn start_background_device_loading(&mut self) {
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        // 真のバックグラウンドタスクとして実行（UIブロックなし）
        tokio::spawn(async move {
            // 短時間待機してUIが表示されてから読み込み
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // Android と iOS デバイスを並行して読み込み
            let android_future = android_manager.list_devices();
            let ios_future = async {
                if let Some(ref ios_mgr) = ios_manager {
                    Some(ios_mgr.list_devices().await)
                } else {
                    None
                }
            };

            let (android_result, ios_result) = tokio::join!(android_future, ios_future);

            // Androidデバイス処理
            match android_result {
                Ok(android_devices) => {
                    let mut state = state_clone.lock().await;
                    state.android_devices = android_devices;
                    state.is_loading = false;
                    state.mark_refreshed();

                    // Mark Android panel for re-render

                    // 最初のデバイスが選択されている場合、その詳細を取得
                    let should_update_details = state.active_panel == Panel::Android
                        && !state.android_devices.is_empty()
                        && state.cached_device_details.is_none();
                    drop(state);

                    if should_update_details {
                        // デバイス詳細を非同期で更新
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
                    state.add_error_notification(format!("Failed to load Android devices: {}", e));
                    drop(state);
                }
            }

            // iOS結果を処理
            if let Some(ios_res) = ios_result {
                match ios_res {
                    Ok(ios_devices) => {
                        let mut state = state_clone.lock().await;
                        state.ios_devices = ios_devices;

                        // Mark iOS panel for re-render

                        // 最初のデバイスが選択されている場合、その詳細を取得
                        let should_update_details = state.active_panel == Panel::Ios
                            && !state.ios_devices.is_empty()
                            && state.cached_device_details.is_none();
                        drop(state);

                        if should_update_details {
                            // デバイス詳細を非同期で更新
                            let state_clone2 = Arc::clone(&state_clone);
                            let _ios_manager_clone = ios_manager.clone();
                            tokio::spawn(async move {
                                {
                                    let state = state_clone2.lock().await;
                                    if let Some(device) = state.ios_devices.get(state.selected_ios)
                                    {
                                        // iOS デバイスの詳細は基本情報から生成
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
                        state.add_error_notification(format!("Failed to load iOS devices: {}", e));
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

        // Clone necessary data for the background task
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();
        let ios_manager = self.ios_manager.clone();

        // Update device details immediately in background
        let update_handle = tokio::spawn(async move {
            Self::update_device_details_internal(state_clone, android_manager, ios_manager).await;
        });

        self.detail_update_handle = Some(update_handle);
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
