//! UI theme definitions and styling configuration.
//!
//! This module provides color themes for the terminal interface, supporting
//! both dark and light modes with carefully chosen colors for accessibility
//! and visual clarity.

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
            primary: Color::Yellow,
            background: Color::Black,
            text: Color::White,
            selected: Color::Yellow,
            running: Color::Green,
            stopped: Color::Gray,
            error: Color::Red,
            border: Color::White,
            focused_bg: Color::Rgb(25, 25, 35), // Subtle gray background
            unfocused_bg: Color::Rgb(20, 20, 25), // Lighter background
            header: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            status: Style::default().fg(Color::Cyan),
        }
    }

    /// Creates a light theme suitable for light terminal backgrounds.
    ///
    /// Uses blue as the primary accent color with colors optimized
    /// for visibility on light backgrounds.
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            background: Color::White,
            text: Color::Black,
            selected: Color::Blue,
            running: Color::Green,
            stopped: Color::Gray,
            error: Color::Red,
            border: Color::Black,
            focused_bg: Color::Rgb(240, 245, 250), // Subtle blue background
            unfocused_bg: Color::Rgb(250, 250, 255), // Lighter background
            header: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            status: Style::default().fg(Color::DarkGray),
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
