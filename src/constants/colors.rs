use ratatui::style::Color;

/// Color constants for theming and UI elements
// Dark theme background colors
pub const DARK_THEME_BG_PRIMARY: Color = Color::Rgb(25, 25, 35);
pub const DARK_THEME_BG_SECONDARY: Color = Color::Rgb(20, 20, 25);

// Light theme background colors
pub const LIGHT_THEME_BG_PRIMARY: Color = Color::Rgb(240, 245, 250);
pub const LIGHT_THEME_BG_SECONDARY: Color = Color::Rgb(250, 250, 255);

// Status colors
pub const STATUS_COLOR_SUCCESS: Color = Color::Green;
pub const STATUS_COLOR_WARNING: Color = Color::Yellow;
pub const STATUS_COLOR_ERROR: Color = Color::Red;
pub const STATUS_COLOR_INFO: Color = Color::Blue;
pub const STATUS_COLOR_DEBUG: Color = Color::Cyan;
pub const STATUS_COLOR_INACTIVE: Color = Color::Gray;
pub const STATUS_COLOR_ACTIVE: Color = Color::Green;

// UI element colors
pub const UI_COLOR_HIGHLIGHT: Color = Color::Yellow;
pub const UI_COLOR_BORDER: Color = Color::Gray;
pub const UI_COLOR_BORDER_FOCUSED: Color = Color::Green;
pub const UI_COLOR_TEXT_DIM: Color = Color::DarkGray;
pub const UI_COLOR_TEXT_NORMAL: Color = Color::Gray;
pub const UI_COLOR_TEXT_BRIGHT: Color = Color::White;
pub const UI_COLOR_BACKGROUND: Color = Color::Black;

// Log level colors
pub const LOG_COLOR_ERROR: Color = Color::Red;
pub const LOG_COLOR_WARN: Color = Color::Yellow;
pub const LOG_COLOR_INFO: Color = Color::Blue;
pub const LOG_COLOR_DEBUG: Color = Color::Cyan;
pub const LOG_COLOR_VERBOSE: Color = Color::Magenta;
pub const LOG_COLOR_DEFAULT: Color = Color::Gray;
