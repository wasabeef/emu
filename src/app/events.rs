//! Application events
//!
//! This module defines the event system for the application, handling keyboard inputs,
//! navigation, and device management actions. Events are processed through a centralized
//! event handler that translates raw keyboard input into structured application actions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Represents all possible events that can occur in the application.
///
/// Events are categorized into:
/// - Navigation: Moving between panels and items
/// - Device actions: Operations on virtual devices
/// - Mode changes: Switching between application modes
/// - Input: Text input and special keys
/// - System: Application lifecycle and terminal events
#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    // Navigation events
    /// Exit the application
    Quit,
    /// Refresh device lists and status
    Refresh,
    /// Switch to the next panel (Android -> iOS)
    NextPanel,
    /// Switch to the previous panel (iOS -> Android)
    PreviousPanel,
    /// Move selection up in current list
    MoveUp,
    /// Move selection down in current list
    MoveDown,
    /// Jump to top of current list
    PageUp,
    /// Jump to bottom of current list
    PageDown,
    /// Move to first item in list
    Home,
    /// Move to last item in list
    End,

    // Device operation events
    /// Toggle device state (start if stopped, stop if running)
    ToggleDevice,
    /// Start the selected device
    StartDevice,
    /// Stop the selected device
    StopDevice,
    /// Create a new device
    CreateDevice,
    /// Delete the selected device
    DeleteDevice,
    /// Wipe/reset the selected device
    WipeDevice,

    // Application mode events
    /// Enter device creation mode
    EnterCreateMode,
    /// Enter device deletion confirmation mode
    EnterDeleteMode,
    /// Enter help/usage mode
    EnterHelpMode,
    /// Exit current mode and return to normal mode
    ExitMode,

    // Input events
    /// Character input for text fields
    Input(char),
    /// Backspace key pressed
    Backspace,
    /// Enter key pressed
    Enter,
    /// Escape key pressed
    Escape,

    // System events
    /// Regular tick for application updates
    Tick,
    /// Terminal resize event with new dimensions
    Resize(u16, u16),
}

impl AppEvent {
    /// Converts a keyboard event into an application event.
    ///
    /// This method maps raw keyboard input to structured application events,
    /// handling key combinations and modifiers appropriately.
    ///
    /// # Arguments
    /// * `key` - The keyboard event from crossterm
    ///
    /// # Returns
    /// * `Some(AppEvent)` - If the key corresponds to a known action
    /// * `None` - If the key is not mapped to any action
    ///
    /// # Key Mappings
    /// * `Ctrl+Q`, `Ctrl+C` - Quit application
    /// * `Esc` - Exit current mode
    /// * `Tab` - Switch to next panel
    /// * `Shift+Tab` - Switch to previous panel
    /// * `↑/↓` - Navigate items in list
    /// * `Enter`, `Space` - Toggle device state
    /// * `r` - Refresh device lists
    /// * `c` - Create new device
    /// * `d` - Delete device
    /// * `w` - Wipe device
    /// * `h`, `?` - Show help
    /// * `q` - Quit (Normal mode only)
    /// * `Ctrl+q` - Quit
    pub fn from_key(key: KeyEvent) -> Option<Self> {
        match (key.code, key.modifiers) {
            // Application control
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Self::Quit),
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Self::Quit),
            (KeyCode::Esc, _) => Some(Self::ExitMode),

            // Panel navigation
            (KeyCode::Tab, _) => Some(Self::NextPanel),
            (KeyCode::BackTab, _) => Some(Self::PreviousPanel),

            // List navigation
            (KeyCode::Up, _) => Some(Self::MoveUp),
            (KeyCode::Down, _) => Some(Self::MoveDown),
            (KeyCode::PageUp, _) => Some(Self::PageUp),
            (KeyCode::PageDown, _) => Some(Self::PageDown),
            (KeyCode::Home, _) => Some(Self::Home),
            (KeyCode::End, _) => Some(Self::End),

            // Device operations
            (KeyCode::Char('r'), _) => Some(Self::Refresh),
            (KeyCode::Enter, _) => Some(Self::ToggleDevice),
            (KeyCode::Char(' '), _) => Some(Self::ToggleDevice),
            (KeyCode::Char('s'), _) => Some(Self::StartDevice),
            (KeyCode::Char('S'), _) => Some(Self::StopDevice),
            (KeyCode::Char('c'), _) => Some(Self::EnterCreateMode),
            (KeyCode::Char('d'), _) => Some(Self::EnterDeleteMode),
            (KeyCode::Char('w'), _) => Some(Self::WipeDevice),

            // Help and information
            (KeyCode::Char('h'), _) => Some(Self::EnterHelpMode),
            (KeyCode::Char('?'), _) => Some(Self::EnterHelpMode),

            // Text input (fallback for unmapped characters)
            (KeyCode::Char(c), _) => Some(Self::Input(c)),
            (KeyCode::Backspace, _) => Some(Self::Backspace),

            // Unmapped keys
            _ => None,
        }
    }

    /// Checks if this event is a navigation event.
    ///
    /// Navigation events include moving between panels, scrolling lists,
    /// and changing focus within the user interface.
    ///
    /// # Returns
    /// * `true` - If the event moves focus or changes selection
    /// * `false` - If the event performs an action or changes mode
    pub fn is_navigation(&self) -> bool {
        matches!(
            self,
            Self::MoveUp
                | Self::MoveDown
                | Self::PageUp
                | Self::PageDown
                | Self::Home
                | Self::End
                | Self::NextPanel
                | Self::PreviousPanel
        )
    }

    /// Checks if this event performs a device action.
    ///
    /// Action events include operations that modify device state or
    /// trigger external commands like starting, stopping, or creating devices.
    ///
    /// # Returns
    /// * `true` - If the event performs a device operation
    /// * `false` - If the event is navigation or mode change
    pub fn is_action(&self) -> bool {
        matches!(
            self,
            Self::ToggleDevice
                | Self::StartDevice
                | Self::StopDevice
                | Self::CreateDevice
                | Self::DeleteDevice
                | Self::WipeDevice
                | Self::Refresh
        )
    }

    /// Checks if this event changes the application mode.
    ///
    /// Mode change events switch between different application states
    /// such as normal view, device creation dialog, or help screen.
    ///
    /// # Returns
    /// * `true` - If the event changes the application mode
    /// * `false` - If the event is navigation or action
    pub fn is_mode_change(&self) -> bool {
        matches!(
            self,
            Self::EnterCreateMode | Self::EnterDeleteMode | Self::EnterHelpMode | Self::ExitMode
        )
    }
}

/// Central event handler for processing application events.
///
/// The EventHandler manages the application's event loop state and processes
/// events that need special handling, such as quit events that affect the
/// main application loop.
#[derive(Debug, Clone)]
pub struct EventHandler {
    /// Flag indicating whether the application should exit
    should_quit: bool,
}

impl EventHandler {
    /// Creates a new event handler with default state.
    ///
    /// # Returns
    /// A new EventHandler instance ready to process events
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    /// Checks if the application should exit.
    ///
    /// This flag is set when a quit event is processed and signals
    /// the main event loop to terminate gracefully.
    ///
    /// # Returns
    /// * `true` - If the application should exit
    /// * `false` - If the application should continue running
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Processes an application event and returns it for further handling.
    ///
    /// Some events (like Quit) are handled internally and consumed,
    /// while others are passed through for processing by other components.
    ///
    /// # Arguments
    /// * `event` - The application event to process
    ///
    /// # Returns
    /// * `Some(AppEvent)` - If the event should be processed further
    /// * `None` - If the event was consumed and needs no further processing
    pub fn handle_event(&mut self, event: AppEvent) -> Option<AppEvent> {
        match event {
            AppEvent::Quit => {
                self.should_quit = true;
                None // Consume the quit event
            }
            _ => Some(event), // Pass other events through
        }
    }
}

impl Default for EventHandler {
    /// Creates a default event handler.
    ///
    /// This is equivalent to calling `EventHandler::new()`.
    fn default() -> Self {
        Self::new()
    }
}
