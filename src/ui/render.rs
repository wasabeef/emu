//! UI rendering

use super::dialogs::{
    render_api_level_dialog, render_confirm_delete_dialog, render_confirm_wipe_dialog,
    render_create_device_dialog, render_notifications,
};
use crate::{
    app::{AppState, Panel},
    constants::{
        colors::*,
        messages::ui::TERMINAL_TOO_SMALL_ERROR,
        ui_layout::{
            ANDROID_PANEL_PERCENTAGE, DEVICE_DETAILS_PANEL_PERCENTAGE, DEVICE_PANELS_PERCENTAGE,
            HEADER_HEIGHT, IOS_PANEL_PERCENTAGE, LOADING_INDICATOR_MARGIN, LOG_LEVEL_WIDTH,
            LOG_TIMESTAMP_WIDTH, MESSAGE_TRUNCATE_SUFFIX_LENGTH, MIN_TERMINAL_HEIGHT,
            MIN_TERMINAL_WIDTH, SEPARATOR_LENGTH, STATUS_BAR_HEIGHT,
        },
        ui_text::{
            architectures::*, device_states::*, navigation::*, progress::*, shortcuts::*,
            status_indicators::*, text_formatting::*,
        },
    },
    models::Platform,
    ui::{widgets::get_animated_moon, Theme},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw_app(frame: &mut Frame, state: &mut AppState, theme: &Theme) {
    let size = frame.area();

    // Ensure we have enough space
    if size.height < MIN_TERMINAL_HEIGHT || size.width < MIN_TERMINAL_WIDTH {
        let msg =
            Paragraph::new(TERMINAL_TOO_SMALL_ERROR).style(Style::default().fg(STATUS_COLOR_ERROR));
        frame.render_widget(msg, size);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),     // Header
            Constraint::Min(10),                   // Main content
            Constraint::Length(STATUS_BAR_HEIGHT), // Status bar
        ])
        .split(size);

    // Header with icon and version
    let version = env!("CARGO_PKG_VERSION");
    let header_text = if state.fullscreen_logs {
        format!(" 🦤 Emu v{version} - Device Manager [FULLSCREEN LOGS]")
    } else {
        format!(" 🦤 Emu v{version} - Device Manager")
    };
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(theme.primary));
    frame.render_widget(header, chunks[0]);

    // Split main content based on fullscreen mode
    let main_chunks = if state.fullscreen_logs {
        // In fullscreen mode, give all space to logs
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Log panel takes all space
                Constraint::Length(1), // Log commands
            ])
            .split(chunks[1])
    } else {
        // Normal mode
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(DEVICE_PANELS_PERCENTAGE), // Device panels with device commands
                Constraint::Min(10),                              // Log panel
                Constraint::Length(1),                            // Log commands
            ])
            .split(chunks[1])
    };

    // Only render device panels if not in fullscreen mode
    if !state.fullscreen_logs {
        // Split device area into panels and device commands
        let device_area_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),    // Device panels
                Constraint::Length(1), // Device commands (reduced from 3 to 1)
            ])
            .split(main_chunks[0]);

        // Device panels (Android | iOS | Details - 3 columns)
        let device_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(ANDROID_PANEL_PERCENTAGE), // Android
                Constraint::Percentage(IOS_PANEL_PERCENTAGE),     // iOS
                Constraint::Percentage(DEVICE_DETAILS_PANEL_PERCENTAGE), // Device Details
            ])
            .split(device_area_chunks[0]);

        // Android panel
        render_android_panel(frame, device_chunks[0], state, theme);

        // iOS panel
        render_ios_panel(frame, device_chunks[1], state, theme);

        // Device details panel
        render_device_details_panel(frame, device_chunks[2], state, theme);

        // Device commands (no border, smaller, centered)
        render_device_commands(frame, device_area_chunks[1], state, theme);

        // Log panel
        let log_index = 1;
        render_log_panel(frame, main_chunks[log_index], state, theme);

        // Log commands
        render_log_commands(frame, main_chunks[log_index + 1], state, theme);
    } else {
        // Fullscreen logs mode
        render_log_panel(frame, main_chunks[0], state, theme);
        render_log_commands(frame, main_chunks[1], state, theme);
    }

    // Global status bar (now only shows app-level commands)
    let status_text = match state.mode {
        crate::app::Mode::Normal => "[q/Ctrl+q]:Quit",
        crate::app::Mode::CreateDevice => {
            "[Tab]next field [Shift+Tab]prev field [Enter]submit [Esc]cancel"
        }
        crate::app::Mode::ConfirmDelete => "[q/Ctrl+q]:Quit",
        crate::app::Mode::ConfirmWipe => "[q/Ctrl+q]:Quit",
        _ => "[q/Ctrl+q]:Quit",
    };

    // Status bar without borders, smaller text
    let status_with_icon = match state.mode {
        crate::app::Mode::Normal => format!("🚪 {status_text}"),
        crate::app::Mode::CreateDevice => format!("📝 {status_text}"),
        crate::app::Mode::ConfirmDelete => status_text.to_string(),
        crate::app::Mode::ConfirmWipe => status_text.to_string(),
        _ => status_text.to_string(),
    };
    let status = Paragraph::new(status_with_icon)
        .style(
            Style::default()
                .fg(UI_COLOR_TEXT_DIM)
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Right);
    frame.render_widget(status, chunks[2]);

    // Render modal dialogs on top
    match state.mode {
        crate::app::Mode::CreateDevice => {
            render_create_device_dialog(frame, state, theme);
        }
        crate::app::Mode::ConfirmDelete => {
            render_confirm_delete_dialog(frame, state, theme);
        }
        crate::app::Mode::ConfirmWipe => {
            render_confirm_wipe_dialog(frame, state, theme);
        }
        crate::app::Mode::ManageApiLevels => {
            render_api_level_dialog(frame, state, theme);
        }
        _ => {}
    }

    // Render notifications on top of everything
    render_notifications(frame, state, theme);
}

fn render_android_panel(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme) {
    let is_active = state.active_panel == Panel::Android;
    let is_focused = state.focused_panel == crate::app::FocusedPanel::DeviceList;
    let border_style = if is_active && is_focused {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    // Calculate visible range
    let available_height = area.height.saturating_sub(2) as usize; // Subtract borders
    let total_devices = state.android_devices.len();

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = state.get_android_scroll_offset(available_height);

    // Update the state's scroll offset
    state.android_scroll_offset = scroll_offset;

    // Get visible devices
    let visible_devices: Vec<_> = state
        .android_devices
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(available_height)
        .collect();

    let items: Vec<ListItem> = visible_devices
        .into_iter()
        .map(|(i, device)| {
            let selected = i == state.selected_android && is_active;
            let status_indicator = if device.is_running {
                ACTIVE_INDICATOR
            } else {
                INACTIVE_INDICATOR
            };
            let text = format!(
                "{status_indicator} {}",
                device.name.replace(UNDERSCORE_STR, SPACE_STR_SINGLE)
            );

            let style = if selected {
                Style::default().bg(theme.primary).fg(UI_COLOR_BACKGROUND)
            } else if device.is_running {
                Style::default().fg(STATUS_COLOR_ACTIVE)
            } else {
                Style::default().fg(theme.text)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    // Add scroll indicators to title if needed
    let title = if is_active && !state.android_devices.is_empty() {
        let position_info = format!("{}/{total_devices}", state.selected_android + 1);
        let scroll_indicator = if total_devices > available_height {
            if scroll_offset > 0 && scroll_offset + available_height < total_devices {
                SCROLL_BOTH // Can scroll both ways
            } else if scroll_offset > 0 {
                SCROLL_UP // Can scroll up
            } else if scroll_offset + available_height < total_devices {
                SCROLL_DOWN // Can scroll down
            } else {
                SCROLL_NONE
            }
        } else {
            SCROLL_NONE
        };
        format!("🤖 Android ({position_info}){scroll_indicator}")
    } else {
        format!("🤖 Android ({total_devices})")
    };

    // Use focused background color for active panel
    let block_style = if is_active {
        Style::default().bg(theme.focused_bg)
    } else {
        Style::default().bg(theme.unfocused_bg)
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(block_style),
    );

    frame.render_widget(list, area);
}

fn render_ios_panel(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme) {
    let is_active = state.active_panel == Panel::Ios;
    let is_focused = state.focused_panel == crate::app::FocusedPanel::DeviceList;
    let border_style = if is_active && is_focused {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    // Calculate visible range
    let available_height = area.height.saturating_sub(2) as usize; // Subtract borders
    let total_devices = state.ios_devices.len();

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = state.get_ios_scroll_offset(available_height);

    // Update the state's scroll offset
    state.ios_scroll_offset = scroll_offset;

    // Get visible devices
    let visible_devices: Vec<_> = state
        .ios_devices
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(available_height)
        .collect();

    let items: Vec<ListItem> = visible_devices
        .into_iter()
        .map(|(i, device)| {
            let selected = i == state.selected_ios && is_active;
            let status_indicator = if device.is_running {
                ACTIVE_INDICATOR
            } else {
                INACTIVE_INDICATOR
            };
            let availability = if device.is_available {
                ""
            } else {
                IOS_UNAVAILABLE
            };
            let text = format!("{status_indicator} {}{availability}", device.name);

            let style = if selected {
                Style::default().bg(theme.primary).fg(UI_COLOR_BACKGROUND)
            } else if device.is_running {
                Style::default().fg(STATUS_COLOR_ACTIVE)
            } else if !device.is_available {
                Style::default().fg(UI_COLOR_TEXT_DIM)
            } else {
                Style::default().fg(theme.text)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    // Add scroll indicators to title if needed
    let title = if cfg!(target_os = "macos") {
        if is_active && !state.ios_devices.is_empty() {
            let position_info = format!("{}/{total_devices}", state.selected_ios + 1);
            let scroll_indicator = if total_devices > available_height {
                if scroll_offset > 0 && scroll_offset + available_height < total_devices {
                    SCROLL_BOTH // Can scroll both ways
                } else if scroll_offset > 0 {
                    SCROLL_UP // Can scroll up
                } else if scroll_offset + available_height < total_devices {
                    SCROLL_DOWN // Can scroll down
                } else {
                    SCROLL_NONE
                }
            } else {
                SCROLL_NONE
            };
            format!("🍎 iOS ({position_info}){scroll_indicator}")
        } else {
            format!("🍎 iOS ({total_devices})")
        }
    } else {
        "🍎 iOS (macOS only)".to_string()
    };

    // Use focused background color for active panel
    let block_style = if is_active {
        Style::default().bg(theme.focused_bg)
    } else {
        Style::default().bg(theme.unfocused_bg)
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(block_style),
    );

    frame.render_widget(list, area);
}

fn render_log_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    // Get device name that is actually streaming logs
    let log_device_name = if let Some((panel, device_name)) = &state.current_log_device {
        format!("{panel:?} - {device_name}")
    } else {
        match state.active_panel {
            Panel::Android => state
                .android_devices
                .get(state.selected_android)
                .map(|d| {
                    if d.is_running {
                        format!("{} (not streaming)", d.name)
                    } else {
                        format!("{} (stopped)", d.name)
                    }
                })
                .unwrap_or_else(|| "No device selected".to_string()),
            Panel::Ios => state
                .ios_devices
                .get(state.selected_ios)
                .map(|d| {
                    if d.is_running {
                        format!("{} (not streaming)", d.name)
                    } else {
                        format!("{} (stopped)", d.name)
                    }
                })
                .unwrap_or_else(|| "No device selected".to_string()),
        }
    };

    // Build title with colored filter level
    let mut title_spans = vec![Span::raw("📋 Logs - "), Span::raw(&log_device_name)];

    if let Some(ref filter) = state.log_filter_level {
        title_spans.push(Span::raw(" [Filter: "));

        let filter_style = match filter.as_str() {
            "ERROR" => Style::default()
                .fg(LOG_COLOR_ERROR)
                .add_modifier(Modifier::BOLD),
            "WARN" => Style::default()
                .fg(LOG_COLOR_WARN)
                .add_modifier(Modifier::BOLD),
            "INFO" => Style::default()
                .fg(LOG_COLOR_INFO)
                .add_modifier(Modifier::BOLD),
            "DEBUG" => Style::default()
                .fg(LOG_COLOR_DEBUG)
                .add_modifier(Modifier::BOLD),
            _ => Style::default().fg(theme.text),
        };

        title_spans.push(Span::styled(filter, filter_style));
        title_spans.push(Span::raw("]"));
    }

    let title_line = Line::from(title_spans);

    // Calculate available space for logs (area height minus borders)
    let available_height = area.height.saturating_sub(2) as usize;
    let available_width = area.width.saturating_sub(2) as usize;

    // Calculate widths for each component
    let timestamp_width = LOG_TIMESTAMP_WIDTH; // "HH:MM:SS "
    let level_width = LOG_LEVEL_WIDTH; // "[ERROR] " max width
    let message_width = available_width.saturating_sub(timestamp_width + level_width);

    // Get filtered logs
    let filtered_logs = state.get_filtered_logs();

    // Always show the most recent logs (no scrolling)
    let visible_logs: Vec<&_> = if filtered_logs.len() > available_height {
        // Show the last N logs that fit in the available space
        let start_idx = filtered_logs.len().saturating_sub(available_height);
        filtered_logs[start_idx..].to_vec()
    } else {
        // If all logs fit on screen, show them all
        filtered_logs
    };

    // Create log lines
    let log_lines: Vec<Line> = visible_logs
        .into_iter()
        .map(|entry| {
            let level_style = match entry.level.as_str() {
                "ERROR" => Style::default().fg(LOG_COLOR_ERROR),
                "WARN" => Style::default().fg(LOG_COLOR_WARN),
                "INFO" => Style::default().fg(LOG_COLOR_INFO),
                "DEBUG" => Style::default().fg(LOG_COLOR_DEBUG),
                _ => Style::default().fg(theme.text),
            };

            // Truncate message if it's too long (safely handle UTF-8 boundaries)
            let message = if entry.message.chars().count() > message_width
                && message_width > MESSAGE_TRUNCATE_SUFFIX_LENGTH
            {
                let truncate_len = message_width.saturating_sub(MESSAGE_TRUNCATE_SUFFIX_LENGTH);
                let truncated: String = entry.message.chars().take(truncate_len).collect();
                format!("{truncated}...")
            } else {
                entry.message.clone()
            };

            // Calculate padding to fill the rest of the line (use char counts for display width)
            // Account for: timestamp + space + [LEVEL] + space + message
            let used_width = entry.timestamp.chars().count()
                + 1
                + entry.level.chars().count()
                + 3
                + message.chars().count();
            let padding = if used_width < available_width {
                " ".repeat(available_width - used_width)
            } else {
                String::new()
            };

            Line::from(vec![
                Span::styled(
                    entry.timestamp.clone(),
                    Style::default().fg(UI_COLOR_TEXT_DIM),
                ),
                Span::raw(" "),
                Span::styled(format!("[{}]", &entry.level), level_style),
                Span::raw(" "),
                Span::raw(message),
                // Add padding spaces to fill the rest of the line
                Span::raw(padding),
            ])
        })
        .collect();

    // Fill remaining space with empty lines if needed
    let mut all_lines = log_lines;
    while all_lines.len() < available_height {
        // Create empty lines that fill the entire width to clear any leftover content
        all_lines.push(Line::from(vec![Span::raw(" ".repeat(available_width))]));
    }

    // Log area never has focus, so use normal border style
    let border_style = Style::default().fg(theme.text);

    let logs = Paragraph::new(all_lines)
        .block(
            Block::default()
                .title(title_line)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(Style::default().fg(theme.text))
        .wrap(ratatui::widgets::Wrap { trim: false }); // Don't wrap, keep lines as is

    frame.render_widget(logs, area);
}

fn render_device_details_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let border_style = Style::default().fg(theme.text);

    if let Some(details) = state.get_selected_device_details() {
        let mut lines = Vec::new();

        // === HEADER: Device Name & Platform ===
        let platform_icon = match details.platform {
            Platform::Android => "🤖",
            Platform::Ios => "🍎",
        };
        lines.push(Line::from(vec![
            Span::styled(platform_icon, Style::default().fg(theme.primary)),
            Span::raw(" "),
            Span::styled(
                details.name.replace('_', " "), // Format name for display
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Separator line
        lines.push(Line::from(vec![Span::styled(
            "━".repeat(SEPARATOR_LENGTH as usize),
            Style::default().fg(UI_COLOR_TEXT_DIM),
        )]));

        // === ESSENTIAL: Status ===
        let (status_icon, status_color) =
            if details.status == "Running" || details.status == "Booted" {
                ("●", STATUS_COLOR_ACTIVE)
            } else {
                ("○", STATUS_COLOR_INACTIVE)
            };
        lines.push(Line::from(vec![
            Span::styled(status_icon, Style::default().fg(status_color)),
            Span::raw(" "),
            Span::styled(
                &details.status,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(""));

        // === DEVELOPMENT INFO ===
        // Display resolution (prioritized for app development)
        if let Some(ref resolution) = details.resolution {
            let dpi_info = details
                .dpi
                .as_ref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default();
            lines.push(Line::from(vec![
                Span::raw("📱 Display: "),
                Span::styled(
                    format!("{resolution}{dpi_info}"),
                    Style::default().fg(STATUS_COLOR_WARNING),
                ),
            ]));
        }

        // Hardware specs
        if let Some(ref ram) = details.ram_size {
            lines.push(Line::from(vec![
                Span::raw("🧠 RAM: "),
                Span::styled(ram.clone(), Style::default().fg(STATUS_COLOR_DEBUG)),
            ]));
        }

        if let Some(ref storage) = details.storage_size {
            lines.push(Line::from(vec![
                Span::raw("💾 Storage: "),
                Span::styled(storage.clone(), Style::default().fg(STATUS_COLOR_DEBUG)),
            ]));
        }

        // Architecture info (only if available from system image)
        if details.platform == Platform::Android {
            if let Some(ref sys_img) = details.system_image {
                let architecture = if sys_img.contains("arm64") {
                    ARM64
                } else if sys_img.contains("x86_64") {
                    X86_64
                } else if sys_img.contains("x86") {
                    X86
                } else {
                    UNKNOWN
                };
                lines.push(Line::from(vec![
                    Span::raw("🔧 Arch: "),
                    Span::styled(architecture, Style::default().fg(LOG_COLOR_VERBOSE)),
                ]));
            }
        }

        lines.push(Line::from(""));

        // === VERSION INFO ===
        lines.push(Line::from(vec![
            Span::raw("📋 Version: "),
            Span::styled(
                details.api_level_or_version,
                Style::default().fg(theme.primary),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::raw("🏷️  Type: "),
            Span::raw(details.device_type),
        ]));

        // === MANAGEMENT INFO ===
        // Device ID (useful for automation/scripting)
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("🆔 ID: "),
            Span::styled(
                details.identifier.clone(),
                Style::default().fg(STATUS_COLOR_INFO),
            ),
        ]));

        // Path info (full path)
        if let Some(ref path) = details.device_path {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("📂 "),
                Span::styled(path.clone(), Style::default().fg(UI_COLOR_TEXT_DIM)),
            ]));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title("Device Details")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(paragraph, area);

        // Show loading indicator in bottom-right corner if key data is still loading
        // Only show for Android devices that should have additional data loaded
        let is_loading = details.platform == Platform::Android && details.device_path.is_none();

        if is_loading {
            let moon_icon = get_animated_moon();
            let loading_text = format!("{moon_icon} {LOADING}");
            let loading_width = "🌙 Loading".len() as u16; // Use fixed width for consistent positioning
            let loading_area = Rect::new(
                area.x
                    + area
                        .width
                        .saturating_sub(loading_width + LOADING_INDICATOR_MARGIN),
                area.y + area.height.saturating_sub(2),
                loading_width + 2,
                1,
            );

            let loading_paragraph = Paragraph::new(loading_text)
                .style(Style::default().fg(UI_COLOR_TEXT_DIM))
                .alignment(Alignment::Right);

            frame.render_widget(loading_paragraph, loading_area);
        }
    } else {
        // No device selected
        let no_device_text = Paragraph::new("No device selected")
            .block(
                Block::default()
                    .title("Device Details")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .style(Style::default().fg(UI_COLOR_TEXT_DIM))
            .alignment(Alignment::Center);

        frame.render_widget(no_device_text, area);
    }
}

fn render_device_commands(frame: &mut Frame, area: Rect, state: &AppState, _theme: &Theme) {
    let device_text = match state.mode {
        crate::app::Mode::Normal => NORMAL_MODE_SHORTCUTS,
        _ => "",
    };

    // Device commands without borders, smaller and dimmer text, centered
    let device_commands = Paragraph::new(device_text)
        .style(
            Style::default()
                .fg(UI_COLOR_TEXT_DIM)
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Center);
    frame.render_widget(device_commands, area);
}

fn render_log_commands(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let (log_text, style) = match state.mode {
        crate::app::Mode::Normal => {
            if state.is_loading {
                (
                    format!("{} Loading devices...", get_animated_moon()),
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                )
            } else if let Some(ref operation) = state.device_operation_status {
                (
                    format!("{} {operation}...", get_animated_moon()),
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    "🗑️ [Shift+L]clear logs  🔍 [f]ilter  🖥️ [Shift+F]ullscreen".to_string(),
                    Style::default()
                        .fg(UI_COLOR_TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                )
            }
        }
        _ => ("".to_string(), Style::default()),
    };

    let log_commands = Paragraph::new(log_text)
        .style(style)
        .alignment(Alignment::Center);
    frame.render_widget(log_commands, area);
}
