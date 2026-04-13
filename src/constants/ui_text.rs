//! UI text and symbols used throughout the interface.
//!
//! This module contains UI symbols, navigation instructions, and interface text
//! that are not part of user messages or device categories.

/// Status indicators for devices
pub mod status_indicators {
    /// Active/running device indicator
    pub const ACTIVE_INDICATOR: &str = "●";

    /// Inactive/stopped device indicator
    pub const INACTIVE_INDICATOR: &str = "○";
}

/// Navigation arrows and scroll indicators
pub mod navigation {
    /// Bidirectional scroll indicator
    pub const SCROLL_BOTH: &str = " [↕]";

    /// Upward scroll indicator
    pub const SCROLL_UP: &str = " [↑]";

    /// Downward scroll indicator
    pub const SCROLL_DOWN: &str = " [↓]";

    /// No scroll indicator (empty)
    pub const SCROLL_NONE: &str = "";
}

/// Keyboard shortcuts and navigation hints
pub mod shortcuts {
    /// Refresh shortcut
    pub const REFRESH: &str = "🔄 [r]efresh";

    /// Panel switching shortcut
    pub const SWITCH_PANELS: &str = "🔀 [Tab]switch panels";

    /// Horizontal navigation shortcut
    pub const HORIZONTAL_NAV: &str = "🔁 [h/l/←/→]switch";

    /// Start/stop device shortcut
    pub const START_STOP: &str = "🚀 [Enter]start/stop";

    /// Vertical movement shortcut
    pub const VERTICAL_NAV: &str = "🔃 [k/j/↑/↓]move";

    /// Create device shortcut
    pub const CREATE: &str = "➕ [c]reate";

    /// Delete device shortcut
    pub const DELETE: &str = "❌ [d]elete";

    /// Wipe device shortcut
    pub const WIPE: &str = "🧹 [w]ipe";

    /// Install packages shortcut
    pub const INSTALL: &str = "📦 [i]nstall";

    /// Complete shortcut text for Android normal mode
    pub const ANDROID_NORMAL_MODE_SHORTCUTS: &str = "🔄 [r]efresh  🔀 [Tab]switch panels  🔁 [h/l/←/→]switch  🚀 [Enter]start/stop  🔃 [k/j/↑/↓]move  ➕ [c]reate  ❌ [d]elete  🧹 [w]ipe  📦 [i]nstall";

    /// Complete shortcut text for iOS normal mode
    pub const IOS_NORMAL_MODE_SHORTCUTS: &str = "🔄 [r]efresh  🔀 [Tab]switch panels  🔁 [h/l/←/→]switch  🚀 [Enter]start/stop  🔃 [k/j/↑/↓]move  ➕ [c]reate  ❌ [d]elete  🧹 [w]ipe";
}

/// Architecture identifiers
pub mod architectures {
    /// ARM 64-bit architecture
    pub const ARM64: &str = "arm64-v8a";

    /// x86 64-bit architecture
    pub const X86_64: &str = "x86_64";

    /// x86 32-bit architecture
    pub const X86: &str = "x86";

    /// Unknown architecture fallback
    pub const UNKNOWN: &str = "unknown";
}

/// Character patterns for text formatting
pub mod text_formatting {
    /// Underscore replacement for display names
    pub const UNDERSCORE: char = '_';

    /// Space replacement for display names
    pub const SPACE: char = ' ';

    /// Underscore as string for string operations
    pub const UNDERSCORE_STR: &str = "_";

    /// Space as string for string operations  
    pub const SPACE_STR_SINGLE: &str = " ";

    /// Unicode separator for device details
    pub const SEPARATOR_CHAR: &str = "━";

    /// Empty line spacer
    pub const EMPTY_LINE: &str = "";

    /// Single space
    pub const SPACE_STR: &str = " ";

    /// Continuation indicator for truncated text
    pub const TRUNCATE_SUFFIX: &str = "...";

    /// Input cursor indicator
    pub const INPUT_CURSOR: &str = "_";
}

/// Status suffixes for device states
pub mod device_states {
    /// Device not streaming logs
    pub const NOT_STREAMING: &str = " (not streaming)";

    /// Device is stopped
    pub const STOPPED: &str = " (stopped)";

    /// iOS unavailable suffix
    pub const IOS_UNAVAILABLE: &str = " (unavailable)";
}

/// Progress and loading text
pub mod progress {
    /// Generic loading text
    pub const LOADING: &str = "Loading";

    /// Loading devices text
    pub const LOADING_DEVICES: &str = "Loading devices...";

    /// Loading device information text
    pub const LOADING_DEVICE_INFO: &str = "Loading device information...";

    /// Loading API levels text
    pub const LOADING_API_LEVELS: &str = "Loading API levels...";

    /// Creating device text
    pub const CREATING_DEVICE: &str = "Creating device... Please wait...";

    /// Processing text
    pub const PROCESSING: &str = "Processing... Please wait...";

    /// Processing with waiting indicator
    pub const PROCESSING_WAIT: &str = "⏳ Processing... Please wait...";
}

/// Notification icons
pub mod notification_icons {
    /// Success icon
    pub const SUCCESS: &str = "✓";

    /// Error icon
    pub const ERROR: &str = "✗";

    /// Warning icon
    pub const WARNING: &str = "⚠";

    /// Info icon
    pub const INFO: &str = "ℹ";

    /// Installation success icon
    pub const INSTALL_SUCCESS: &str = "✅";

    /// Available package icon
    pub const PACKAGE_AVAILABLE: &str = "📦";
}

/// Installation and API level management text
pub mod api_management {
    /// Installation completed successfully
    pub const INSTALL_COMPLETED: &str = "✅ Installation completed successfully!";

    /// API level instructions
    pub const API_INSTRUCTIONS: &str =
        "✅ Green = Installed  📦 Gray = Available  Select and press Enter/d";

    /// API management navigation (installed packages)
    pub const NAV_UNINSTALL: &str = "[↑/↓/j/k] Navigate  [d] Uninstall Selected  [Esc] Cancel";

    /// API management navigation (available packages)
    pub const NAV_INSTALL: &str = "[↑/↓/j/k] Navigate  [Enter] Install Selected  [Esc] Cancel";

    /// API management navigation (general)
    pub const NAV_GENERAL: &str =
        "[↑/↓/j/k] Navigate  [Enter] Install  [d] Uninstall  [Esc] Cancel";
}

/// Log management shortcuts
pub mod log_shortcuts {
    /// Clear logs shortcut
    pub const CLEAR_LOGS: &str = "🗑️ [Shift+L]clear logs";

    /// Filter logs shortcut
    pub const FILTER_LOGS: &str = "🔍 [f]ilter";

    /// Fullscreen logs shortcut
    pub const FULLSCREEN_LOGS: &str = "🖥️ [Shift+F]ullscreen";

    /// Complete log shortcuts text
    pub const LOG_MODE_SHORTCUTS: &str =
        "🗑️ [Shift+L]clear logs  🔍 [f]filter  🖥️ [Shift+F]ullscreen";
}

/// Mode indicators for status text
pub mod mode_indicators {
    /// Normal mode indicator
    pub const NORMAL_MODE: &str = "🚪";

    /// Create device mode indicator
    pub const CREATE_MODE: &str = "📝";
}

/// Form field navigation
pub mod form_navigation {
    /// Field navigation instructions
    pub const FIELD_NAV: &str = "[Tab]next field [Shift+Tab]prev field [Enter]submit [Esc]cancel";
}

/// Quit instructions for different modes
pub mod quit_instructions {
    /// Standard quit instruction
    pub const QUIT: &str = "[q/Ctrl+q]:Quit";
}
