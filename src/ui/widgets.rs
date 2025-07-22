//! Custom UI widgets for the terminal interface.
//!
//! This module provides reusable widget components built on top of ratatui's
//! primitive widgets. These widgets encapsulate common UI patterns and styling
//! logic for consistency across the application.

use crate::{
    constants::colors::*,
    constants::limits::INVALID_API_LEVEL,
    constants::performance::*,
    constants::ui_layout::*,
    models::{AndroidDevice, DeviceStatus, IosDevice},
};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

/// A widget for displaying a list of devices with selection support.
///
/// This widget renders a bordered list with highlighting for the selected item
/// and visual indication of whether the list is currently focused.
pub struct DeviceListWidget {
    /// Title displayed in the border
    pub title: String,
    /// List of items to display
    pub items: Vec<String>,
    /// Index of the currently selected item
    pub selected: Option<usize>,
    /// Whether this widget is currently active/focused
    pub is_active: bool,
}

impl DeviceListWidget {
    pub fn new(title: String) -> Self {
        Self {
            title,
            items: Vec::new(),
            selected: None,
            is_active: false,
        }
    }

    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    pub fn render(self) -> List<'static> {
        let border_style = if self.is_active {
            Style::default().fg(UI_COLOR_HIGHLIGHT)
        } else {
            Style::default().fg(UI_COLOR_TEXT_BRIGHT)
        };

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let selected = self.selected == Some(i) && self.is_active;
                let style = if selected {
                    Style::default()
                        .bg(UI_COLOR_HIGHLIGHT)
                        .fg(UI_COLOR_BACKGROUND)
                } else {
                    Style::default()
                };
                ListItem::new(item.clone()).style(style)
            })
            .collect();

        List::new(items).block(
            Block::default()
                .title(self.title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
    }
}

/// A status bar widget for displaying application status and loading indicators.
///
/// This widget shows status text at the bottom of the screen with optional
/// loading animation and consistent styling.
pub struct StatusBar {
    /// The status text to display
    pub text: String,
    /// Whether to show a loading indicator
    pub is_loading: bool,
}

impl StatusBar {
    pub fn new(text: String) -> Self {
        Self {
            text,
            is_loading: false,
        }
    }

    pub fn loading(mut self, is_loading: bool) -> Self {
        self.is_loading = is_loading;
        self
    }

    pub fn render(self) -> Paragraph<'static> {
        let display_text = if self.is_loading {
            format!("Loading... {}", self.text)
        } else {
            self.text
        };

        Paragraph::new(display_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(STATUS_COLOR_DEBUG))
    }
}

/// Enhanced device list widget that displays detailed device information.
///
/// This widget shows device lists with rich information including status icons,
/// API levels, device types, and runtime information. It supports both Android
/// and iOS devices with platform-specific formatting.
pub struct EnhancedDeviceListWidget {
    /// Title displayed in the border
    pub title: String,
    /// List of Android devices to display
    pub android_devices: Vec<AndroidDevice>,
    /// List of iOS devices to display
    pub ios_devices: Vec<IosDevice>,
    /// Index of selected Android device
    pub selected_android: usize,
    /// Index of selected iOS device
    pub selected_ios: usize,
    /// Which platform panel is currently active
    pub active_panel: crate::app::state::Panel,
    /// Whether this widget is currently active/focused
    pub is_active: bool,
}

impl EnhancedDeviceListWidget {
    pub fn new(title: String) -> Self {
        Self {
            title,
            android_devices: Vec::new(),
            ios_devices: Vec::new(),
            selected_android: 0,
            selected_ios: 0,
            active_panel: crate::app::state::Panel::Android,
            is_active: false,
        }
    }

    pub fn android_devices(mut self, devices: Vec<AndroidDevice>) -> Self {
        self.android_devices = devices;
        self
    }

    pub fn ios_devices(mut self, devices: Vec<IosDevice>) -> Self {
        self.ios_devices = devices;
        self
    }

    pub fn selected(mut self, android: usize, ios: usize) -> Self {
        self.selected_android = android;
        self.selected_ios = ios;
        self
    }

    pub fn active_panel(mut self, panel: crate::app::state::Panel) -> Self {
        self.active_panel = panel;
        self
    }

    pub fn active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    pub fn render(self) -> List<'static> {
        let border_style = if self.is_active {
            Style::default().fg(UI_COLOR_HIGHLIGHT)
        } else {
            Style::default().fg(UI_COLOR_TEXT_BRIGHT)
        };

        let items: Vec<ListItem> = match self.active_panel {
            crate::app::state::Panel::Android => {
                self.android_devices
                    .iter()
                    .enumerate()
                    .map(|(i, device)| {
                        let selected = i == self.selected_android && self.is_active;
                        let style = if selected {
                            Style::default()
                                .bg(UI_COLOR_HIGHLIGHT)
                                .fg(UI_COLOR_BACKGROUND)
                        } else {
                            Style::default()
                        };

                        // Enhanced display with status, API level, and device type
                        let status_icon = match device.status {
                            DeviceStatus::Running => "🟢",
                            DeviceStatus::Stopped => "⚫",
                            DeviceStatus::Starting => "🟡",
                            DeviceStatus::Stopping => "🟠",
                            DeviceStatus::Creating => "🔵",
                            DeviceStatus::Error => "🔴",
                            DeviceStatus::Unknown => "⚪",
                        };

                        let api_display = if device.api_level > INVALID_API_LEVEL {
                            format!("API {}", device.api_level)
                        } else {
                            "Unknown API".to_string()
                        };

                        let device_type_short = if device.device_type.len()
                            > DEVICE_TYPE_DISPLAY_MAX_LENGTH
                        {
                            format!("{}...", &device.device_type[..DEVICE_TYPE_TRUNCATED_LENGTH])
                        } else {
                            device.device_type.clone()
                        };

                        let display_text = format!(
                            "{} {} | {} | {}",
                            status_icon, device.name, api_display, device_type_short
                        );

                        ListItem::new(display_text).style(style)
                    })
                    .collect()
            }
            crate::app::state::Panel::Ios => {
                self.ios_devices
                    .iter()
                    .enumerate()
                    .map(|(i, device)| {
                        let selected = i == self.selected_ios && self.is_active;
                        let style = if selected {
                            Style::default()
                                .bg(UI_COLOR_HIGHLIGHT)
                                .fg(UI_COLOR_BACKGROUND)
                        } else {
                            Style::default()
                        };

                        // Enhanced display with status and runtime version
                        let status_icon = match device.status {
                            DeviceStatus::Running => "🟢",
                            DeviceStatus::Stopped => "⚫",
                            DeviceStatus::Starting => "🟡",
                            DeviceStatus::Stopping => "🟠",
                            DeviceStatus::Creating => "🔵",
                            DeviceStatus::Error => "🔴",
                            DeviceStatus::Unknown => "⚪",
                        };

                        let runtime_display = if !device.runtime_version.is_empty() {
                            &device.runtime_version
                        } else {
                            "Unknown Runtime"
                        };

                        let device_type_short = if device.device_type.len()
                            > DEVICE_TYPE_DISPLAY_MAX_LENGTH
                        {
                            format!("{}...", &device.device_type[..DEVICE_TYPE_TRUNCATED_LENGTH])
                        } else {
                            device.device_type.clone()
                        };

                        let display_text = format!(
                            "{} {} | {} | {}",
                            status_icon, device.name, runtime_display, device_type_short
                        );

                        ListItem::new(display_text).style(style)
                    })
                    .collect()
            }
        };

        List::new(items).block(
            Block::default()
                .title(self.title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
    }
}

/// Progress indicator widget for long-running operations.
///
/// This widget displays a progress bar with optional determinate or
/// indeterminate states, suitable for showing progress of device operations
/// like creation, deletion, or startup.
pub struct ProgressWidget {
    /// Title displayed in the border
    pub title: String,
    /// Progress value from 0.0 to 1.0
    pub progress: f64,
    /// Message describing the current operation
    pub message: String,
    /// Whether to show indeterminate progress (unknown duration)
    pub is_indeterminate: bool,
}

impl ProgressWidget {
    pub fn new(title: String, message: String) -> Self {
        Self {
            title,
            progress: 0.0,
            message,
            is_indeterminate: true,
        }
    }

    pub fn with_progress(mut self, progress: f64) -> Self {
        self.progress = progress.clamp(PROGRESS_MIN_BOUND, PROGRESS_MAX_BOUND);
        self.is_indeterminate = false;
        self
    }

    pub fn render(self) -> Gauge<'static> {
        let ratio = if self.is_indeterminate {
            // For indeterminate progress, use a cycling pattern
            PROGRESS_ANIMATION_STEP // This could be animated in a real implementation
        } else {
            self.progress
        };

        let label = if self.is_indeterminate {
            format!("{} (Working...)", self.message)
        } else {
            format!(
                "{} ({:.0}%)",
                self.message,
                self.progress * PERCENTAGE_CONVERSION_FACTOR
            )
        };

        Gauge::default()
            .block(Block::default().title(self.title).borders(Borders::ALL))
            .gauge_style(Style::default().fg(STATUS_COLOR_DEBUG))
            .ratio(ratio)
            .label(label)
    }
}

/// Header widget for displaying application title and version.
///
/// This widget is typically displayed at the top of the application
/// and shows the application name with optional version information.
pub struct Header {
    /// Application title
    pub title: String,
    /// Optional version string
    pub version: Option<String>,
}

impl Header {
    pub fn new(title: String) -> Self {
        Self {
            title,
            version: None,
        }
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn render(self) -> Paragraph<'static> {
        let display_text = match self.version {
            Some(version) => format!("{} v{}", self.title, version),
            None => self.title,
        };

        Paragraph::new(display_text)
            .block(Block::default().borders(Borders::ALL))
            .style(
                Style::default()
                    .fg(UI_COLOR_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            )
    }
}

pub fn create_loading_gauge(percentage: u16) -> Gauge<'static> {
    Gauge::default()
        .block(Block::default().title("Loading").borders(Borders::ALL))
        .gauge_style(Style::default().fg(UI_COLOR_HIGHLIGHT))
        .percent(percentage)
}

pub fn create_help_text() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled(
                "q",
                Style::default()
                    .fg(UI_COLOR_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Quit"),
        ]),
        Line::from(vec![
            Span::styled(
                "r",
                Style::default()
                    .fg(UI_COLOR_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Refresh devices"),
        ]),
        Line::from(vec![
            Span::styled(
                "Tab",
                Style::default()
                    .fg(UI_COLOR_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Switch panel"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(UI_COLOR_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Start/Stop device"),
        ]),
        Line::from(vec![
            Span::styled(
                "↑/↓",
                Style::default()
                    .fg(UI_COLOR_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Navigate"),
        ]),
    ]
}

/// Returns an animated moon emoji based on the current time.
/// The moon cycles through different phases to create a loading animation.
pub fn get_animated_moon() -> &'static str {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let moon_phases = ["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"];
    let index =
        ((elapsed / ANIMATION_TIMING_DURATION_MS as u128) % moon_phases.len() as u128) as usize;
    moon_phases[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test DeviceListWidget creation and configuration
    #[test]
    fn test_device_list_widget() {
        let widget = DeviceListWidget::new("Test Devices".to_string())
            .items(vec!["Device 1".to_string(), "Device 2".to_string()])
            .selected(Some(1))
            .active(true);

        assert_eq!(widget.title, "Test Devices");
        assert_eq!(widget.items.len(), 2);
        assert_eq!(widget.selected, Some(1));
        assert!(widget.is_active);
    }

    /// Test StatusBar widget configuration
    #[test]
    fn test_status_bar() {
        let status_bar = StatusBar::new("Ready".to_string()).loading(true);

        assert_eq!(status_bar.text, "Ready");
        assert!(status_bar.is_loading);
    }

    /// Test EnhancedDeviceListWidget creation
    #[test]
    fn test_enhanced_device_list_widget() {
        let widget = EnhancedDeviceListWidget::new("Enhanced".to_string())
            .selected(1, 2)
            .active(true);

        assert_eq!(widget.title, "Enhanced");
        assert_eq!(widget.selected_android, 1);
        assert_eq!(widget.selected_ios, 2);
        assert!(widget.is_active);
    }

    /// Test ProgressWidget with determinate progress
    #[test]
    fn test_progress_widget_determinate() {
        let widget =
            ProgressWidget::new("Progress".to_string(), "Loading".to_string()).with_progress(0.75);

        assert_eq!(widget.title, "Progress");
        assert_eq!(widget.message, "Loading");
        assert_eq!(widget.progress, 0.75);
        assert!(!widget.is_indeterminate);
    }

    /// Test ProgressWidget with indeterminate progress
    #[test]
    fn test_progress_widget_indeterminate() {
        let widget = ProgressWidget::new("Progress".to_string(), "Working".to_string());

        assert_eq!(widget.progress, 0.0);
        assert!(widget.is_indeterminate);
    }

    /// Test Header widget with and without version
    #[test]
    fn test_header_widget() {
        let header_no_version = Header::new("Emu".to_string());
        assert_eq!(header_no_version.title, "Emu");
        assert!(header_no_version.version.is_none());

        let header_with_version = Header::new("Emu".to_string()).version("1.0.0".to_string());
        assert_eq!(header_with_version.version, Some("1.0.0".to_string()));
    }

    /// Test create_loading_gauge function
    #[test]
    fn test_create_loading_gauge() {
        let _gauge = create_loading_gauge(50);
        // Just test that it doesn't panic and creates a valid gauge
        // Actual rendering can't be easily tested without a terminal backend
    }

    /// Test create_help_text function
    #[test]
    fn test_create_help_text() {
        let help_text = create_help_text();

        // Should have multiple help lines
        assert!(!help_text.is_empty());

        // First line should be about quitting
        if let Some(first_line) = help_text.first() {
            let line_str = format!("{first_line:?}");
            assert!(line_str.contains("q"));
            assert!(line_str.contains("Quit"));
        }
    }

    /// Test widget rendering doesn't panic
    #[test]
    fn test_widget_rendering() {
        // These tests just verify widgets can be created and rendered without panicking
        let _device_list = DeviceListWidget::new("Devices".to_string())
            .items(vec!["Device 1".to_string()])
            .render();

        let _status_bar = StatusBar::new("Status".to_string()).render();

        let _progress = ProgressWidget::new("Progress".to_string(), "Loading".to_string())
            .with_progress(0.5)
            .render();

        let _header = Header::new("App".to_string()).render();

        // If we get here without panicking, the test passes
    }

    /// Test progress clamping
    #[test]
    fn test_progress_clamping() {
        let widget_over =
            ProgressWidget::new("Test".to_string(), "Over".to_string()).with_progress(1.5);
        assert_eq!(widget_over.progress, PROGRESS_MAX_BOUND);

        let widget_under =
            ProgressWidget::new("Test".to_string(), "Under".to_string()).with_progress(-0.5);
        assert_eq!(widget_under.progress, PROGRESS_MIN_BOUND);
    }

    /// Test DeviceListWidget default state
    #[test]
    fn test_device_list_widget_defaults() {
        let widget = DeviceListWidget::new("Test".to_string());

        assert!(widget.items.is_empty());
        assert_eq!(widget.selected, None);
        assert!(!widget.is_active);
    }

    /// Test device status color mapping
    #[test]
    fn test_device_status_formatting() {
        // Test that we can create widgets with different device status combinations
        let android_device = AndroidDevice {
            name: "Test Device".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 33,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        };

        let ios_device = IosDevice {
            name: "iPhone 15".to_string(),
            udid: "ABC123".to_string(),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        };

        let widget = EnhancedDeviceListWidget::new("Test".to_string())
            .android_devices(vec![android_device])
            .ios_devices(vec![ios_device]);

        assert_eq!(widget.android_devices.len(), 1);
        assert_eq!(widget.ios_devices.len(), 1);
    }
}
