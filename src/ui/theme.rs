//! UI theme definitions and styling configuration.
//!
//! This module provides color themes for the terminal interface, supporting
//! both dark and light modes with carefully chosen colors for accessibility
//! and visual clarity.

use crate::constants::colors::*;
use ratatui::style::{Color, Modifier, Style};

/// Theme configuration for the terminal user interface.
///
/// Contains all color and style definitions used throughout the application.
/// Supports both dark and light themes with consistent visual hierarchy.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary accent color (used for highlights and selections)
    pub primary: Color,
    /// Base background color
    pub background: Color,
    /// Default text color
    pub text: Color,
    /// Color for selected items
    pub selected: Color,
    /// Color for running/active devices
    pub running: Color,
    /// Color for stopped/inactive devices
    pub stopped: Color,
    /// Color for errors and unavailable devices
    pub error: Color,
    /// Border color for panels and widgets
    pub border: Color,
    /// Background color for focused panels
    pub focused_bg: Color,
    /// Background color for unfocused panels
    pub unfocused_bg: Color,
    /// Style for header text
    pub header: Style,
    /// Style for status information
    pub status: Style,
}

impl Theme {
    /// Creates a dark theme suitable for dark terminal backgrounds.
    ///
    /// Uses yellow as the primary accent color with high contrast colors
    /// for good visibility on dark backgrounds.
    pub fn dark() -> Self {
        Self {
            primary: UI_COLOR_HIGHLIGHT,
            background: UI_COLOR_BACKGROUND,
            text: UI_COLOR_TEXT_BRIGHT,
            selected: UI_COLOR_HIGHLIGHT,
            running: STATUS_COLOR_ACTIVE,
            stopped: STATUS_COLOR_INACTIVE,
            error: STATUS_COLOR_ERROR,
            border: UI_COLOR_BORDER,
            focused_bg: DARK_THEME_BG_PRIMARY,
            unfocused_bg: DARK_THEME_BG_SECONDARY,
            header: Style::default()
                .fg(UI_COLOR_HIGHLIGHT)
                .add_modifier(Modifier::BOLD),
            status: Style::default().fg(STATUS_COLOR_DEBUG),
        }
    }

    /// Creates a light theme suitable for light terminal backgrounds.
    ///
    /// Uses blue as the primary accent color with colors optimized
    /// for visibility on light backgrounds.
    pub fn light() -> Self {
        Self {
            primary: STATUS_COLOR_INFO,
            background: UI_COLOR_TEXT_BRIGHT,
            text: UI_COLOR_BACKGROUND,
            selected: STATUS_COLOR_INFO,
            running: STATUS_COLOR_ACTIVE,
            stopped: STATUS_COLOR_INACTIVE,
            error: STATUS_COLOR_ERROR,
            border: UI_COLOR_BACKGROUND,
            focused_bg: LIGHT_THEME_BG_PRIMARY,
            unfocused_bg: LIGHT_THEME_BG_SECONDARY,
            header: Style::default()
                .fg(STATUS_COLOR_INFO)
                .add_modifier(Modifier::BOLD),
            status: Style::default().fg(UI_COLOR_TEXT_DIM),
        }
    }

    /// Determines the appropriate color for a device based on its status.
    ///
    /// # Arguments
    /// * `is_running` - Whether the device is currently running
    /// * `is_available` - Whether the device is available (not corrupted)
    ///
    /// # Returns
    /// - Error color if device is unavailable
    /// - Running color (green) if device is running
    /// - Stopped color (gray) if device is stopped
    pub fn device_status_color(&self, is_running: bool, is_available: bool) -> Color {
        if !is_available {
            self.error
        } else if is_running {
            self.running
        } else {
            self.stopped
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test dark theme creation
    #[test]
    fn test_dark_theme() {
        let theme = Theme::dark();

        assert_eq!(theme.primary, UI_COLOR_HIGHLIGHT);
        assert_eq!(theme.background, UI_COLOR_BACKGROUND);
        assert_eq!(theme.text, UI_COLOR_TEXT_BRIGHT);
        assert_eq!(theme.selected, UI_COLOR_HIGHLIGHT);
        assert_eq!(theme.running, STATUS_COLOR_ACTIVE);
        assert_eq!(theme.stopped, STATUS_COLOR_INACTIVE);
        assert_eq!(theme.error, STATUS_COLOR_ERROR);
        assert_eq!(theme.border, UI_COLOR_BORDER);
        assert_eq!(theme.focused_bg, DARK_THEME_BG_PRIMARY);
        assert_eq!(theme.unfocused_bg, DARK_THEME_BG_SECONDARY);
    }

    /// Test light theme creation
    #[test]
    fn test_light_theme() {
        let theme = Theme::light();

        assert_eq!(theme.primary, STATUS_COLOR_INFO);
        assert_eq!(theme.background, UI_COLOR_TEXT_BRIGHT);
        assert_eq!(theme.text, UI_COLOR_BACKGROUND);
        assert_eq!(theme.selected, STATUS_COLOR_INFO);
        assert_eq!(theme.running, STATUS_COLOR_ACTIVE);
        assert_eq!(theme.stopped, STATUS_COLOR_INACTIVE);
        assert_eq!(theme.error, STATUS_COLOR_ERROR);
        assert_eq!(theme.border, UI_COLOR_BACKGROUND);
        assert_eq!(theme.focused_bg, LIGHT_THEME_BG_PRIMARY);
        assert_eq!(theme.unfocused_bg, LIGHT_THEME_BG_SECONDARY);
    }

    /// Test device status color logic
    #[test]
    fn test_device_status_color() {
        let theme = Theme::dark();

        // Test unavailable device
        let color = theme.device_status_color(true, false);
        assert_eq!(color, STATUS_COLOR_ERROR);

        // Test running device
        let color = theme.device_status_color(true, true);
        assert_eq!(color, STATUS_COLOR_ACTIVE);

        // Test stopped device
        let color = theme.device_status_color(false, true);
        assert_eq!(color, STATUS_COLOR_INACTIVE);
    }

    /// Test header style configuration
    #[test]
    fn test_header_style() {
        let dark_theme = Theme::dark();
        let light_theme = Theme::light();

        // Check that header styles have bold modifier
        assert!(dark_theme.header.add_modifier.contains(Modifier::BOLD));
        assert!(light_theme.header.add_modifier.contains(Modifier::BOLD));

        // Check header colors differ between themes
        assert_ne!(dark_theme.header.fg, light_theme.header.fg);
    }

    /// Test theme contrast and visibility
    #[test]
    fn test_theme_contrast() {
        let dark_theme = Theme::dark();
        let light_theme = Theme::light();

        // Test that themes have different backgrounds
        assert_ne!(dark_theme.background, light_theme.background);

        // Test that text colors contrast with backgrounds
        assert_ne!(dark_theme.text, dark_theme.background);
        assert_ne!(light_theme.text, light_theme.background);
    }

    /// Test theme clone functionality
    #[test]
    fn test_theme_clone() {
        let original = Theme::dark();
        let cloned = original.clone();

        assert_eq!(original.primary, cloned.primary);
        assert_eq!(original.background, cloned.background);
        assert_eq!(original.text, cloned.text);
    }

    /// Test status colors are distinct
    #[test]
    fn test_status_colors_distinct() {
        let theme = Theme::dark();

        // Ensure all status colors are different for clear differentiation
        assert_ne!(theme.running, theme.stopped);
        assert_ne!(theme.running, theme.error);
        assert_ne!(theme.stopped, theme.error);
    }

    /// Test theme debug output
    #[test]
    fn test_theme_debug() {
        let theme = Theme::dark();
        let debug_output = format!("{theme:?}");

        // Should contain "Theme" in debug output
        assert!(debug_output.contains("Theme"));
    }
}
