//! UI rendering

use crate::{
    app::{
        state::{CreateDeviceField, NotificationType},
        AppState, Panel,
    },
    constants::{
        colors::*,
        messages::ui::{
            DIALOG_SHORTCUT_CANCEL, DIALOG_SHORTCUT_NO, DIALOG_SHORTCUT_YES,
            TERMINAL_TOO_SMALL_ERROR,
        },
        ui_layout::{
            ANDROID_PANEL_PERCENTAGE, API_LEVEL_LIST_MIN_HEIGHT, DEVICE_DETAILS_PANEL_PERCENTAGE,
            DEVICE_PANELS_PERCENTAGE, DIALOG_HEIGHT_LARGE, DIALOG_HEIGHT_MEDIUM,
            DIALOG_HEIGHT_SMALL, DIALOG_MARGIN, DIALOG_WIDTH_LARGE, DIALOG_WIDTH_MEDIUM,
            DIALOG_WIDTH_SMALL, FORM_FOOTER_HEIGHT, FORM_LABEL_WIDTH, HEADER_HEIGHT,
            IOS_PANEL_PERCENTAGE, LOADING_INDICATOR_MARGIN, LOG_LEVEL_WIDTH, LOG_TIMESTAMP_WIDTH,
            MESSAGE_TRUNCATE_SUFFIX_LENGTH, MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH,
            NOTIFICATION_HEIGHT, SEPARATOR_LENGTH, STATUS_BAR_HEIGHT,
        },
    },
    ui::{widgets::get_animated_moon, Theme},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Renders a generic confirmation dialog with customizable title, message, and style
fn render_confirmation_dialog(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    message: &str,
    icon: &str,
    border_color: Color,
    theme: &Theme,
) {
    let dialog_width = DIALOG_WIDTH_SMALL.min(area.width - DIALOG_MARGIN);
    let dialog_height = DIALOG_HEIGHT_SMALL.min(area.height - DIALOG_MARGIN);
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear background
    frame.render_widget(Clear, dialog_area);

    // Dialog background
    let background_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(format!("{icon} {title}"))
        .style(Style::default().bg(UI_COLOR_BACKGROUND));
    frame.render_widget(background_block, dialog_area);

    // Inner area for content
    let inner_area = Rect::new(
        dialog_area.x + 1,
        dialog_area.y + 1,
        dialog_area.width.saturating_sub(2),
        dialog_area.height.saturating_sub(2),
    );

    // Split inner area
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(2),    // Message
            Constraint::Length(2), // Shortcuts
        ])
        .split(inner_area);

    // Message
    let message_text = Paragraph::new(message)
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(message_text, inner_chunks[0]);

    // Shortcuts
    let shortcuts = vec![
        Span::styled(
            "y",
            Style::default()
                .fg(STATUS_COLOR_SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(DIALOG_SHORTCUT_YES),
        Span::styled(
            "n",
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(DIALOG_SHORTCUT_NO),
        Span::styled(
            "Esc",
            Style::default()
                .fg(UI_COLOR_TEXT_DIM)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(DIALOG_SHORTCUT_CANCEL),
    ];
    let shortcuts_paragraph = Paragraph::new(Line::from(shortcuts))
        .style(Style::default().fg(UI_COLOR_TEXT_DIM))
        .alignment(Alignment::Center);
    frame.render_widget(shortcuts_paragraph, inner_chunks[1]);
}

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
            let status_indicator = if device.is_running { "●" } else { "○" };
            let text = format!("{status_indicator} {}", device.name.replace('_', " "));

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
                " [↕]" // Can scroll both ways
            } else if scroll_offset > 0 {
                " [↑]" // Can scroll up
            } else if scroll_offset + available_height < total_devices {
                " [↓]" // Can scroll down
            } else {
                ""
            }
        } else {
            ""
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
            let status_indicator = if device.is_running { "●" } else { "○" };
            let availability = if device.is_available {
                ""
            } else {
                " (unavailable)"
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
                    " [↕]" // Can scroll both ways
                } else if scroll_offset > 0 {
                    " [↑]" // Can scroll up
                } else if scroll_offset + available_height < total_devices {
                    " [↓]" // Can scroll down
                } else {
                    ""
                }
            } else {
                ""
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

fn render_create_device_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
    let size = frame.area();

    // Calculate dialog dimensions
    let dialog_width = DIALOG_WIDTH_MEDIUM.min(size.width - 4);
    let dialog_height = DIALOG_HEIGHT_MEDIUM.min(size.height - 4);

    // Center the dialog
    let x = (size.width.saturating_sub(dialog_width)) / 2;
    let y = (size.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Add a background block to ensure full coverage
    let background_block = Block::default().style(Style::default().bg(UI_COLOR_BACKGROUND));
    frame.render_widget(background_block, dialog_area);

    // Dialog title with icon based on active panel
    let title = match state.active_panel {
        Panel::Android => "🤖 Create Android Device",
        Panel::Ios => "🍎 Create iOS Device",
    };

    let dialog_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));

    let inner_area = dialog_block.inner(dialog_area);
    frame.render_widget(dialog_block, dialog_area);

    // Layout for form fields with margin at top
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top margin
            Constraint::Length(2), // API Level
            Constraint::Length(2), // Category
            Constraint::Length(2), // Device Type
            Constraint::Length(2), // RAM Size
            Constraint::Length(2), // Storage Size
            Constraint::Length(2), // Name
            Constraint::Min(1),    // Error message
        ])
        .split(inner_area);

    let form = &state.create_device_form;

    // Render each field (in the new order)
    // Skip form_chunks[0] which is the top margin

    // 1. API Level
    if form.available_versions.is_empty() {
        render_input_field(
            frame,
            form_chunks[1],
            "API Level:",
            &form.version,
            form.active_field == CreateDeviceField::ApiLevel,
            theme,
        );
    } else {
        render_select_field(
            frame,
            form_chunks[1],
            "API Level:",
            &form.version_display,
            &form
                .available_versions
                .iter()
                .map(|(_, display)| display.clone())
                .collect::<Vec<String>>(),
            form.active_field == CreateDeviceField::ApiLevel,
            theme,
        );
    }

    // 2. Category (Android only)
    if matches!(state.active_panel, Panel::Android) {
        let default_category = "all".to_string();
        let current_category = form
            .available_categories
            .get(form.selected_category_index)
            .unwrap_or(&default_category);

        render_select_field(
            frame,
            form_chunks[2],
            "Category:",
            current_category,
            &form
                .available_categories
                .iter()
                .map(|cat| match cat.as_str() {
                    "all" => "All Devices".to_string(),
                    "phone" => "Phone".to_string(),
                    "tablet" => "Tablet".to_string(),
                    "wear" => "Wear".to_string(),
                    "tv" => "TV".to_string(),
                    "automotive" => "Automotive".to_string(),
                    "desktop" => "Desktop".to_string(),
                    _ => cat.clone(),
                })
                .collect::<Vec<String>>(),
            form.active_field == CreateDeviceField::Category,
            theme,
        );
    }

    // 3. Device Type
    let device_type_chunk = if matches!(state.active_panel, Panel::Android) {
        form_chunks[3]
    } else {
        form_chunks[2]
    };
    render_select_field(
        frame,
        device_type_chunk,
        "Device Type:",
        &form.device_type,
        &form
            .available_device_types
            .iter()
            .map(|(_, display)| display.clone())
            .collect::<Vec<String>>(),
        form.active_field == CreateDeviceField::DeviceType,
        theme,
    );

    // 4. RAM/Storage (Only for Android)
    if matches!(state.active_panel, Panel::Android) {
        render_input_field(
            frame,
            form_chunks[4],
            "RAM Size (MB):",
            &form.ram_size,
            form.active_field == CreateDeviceField::RamSize,
            theme,
        );

        render_input_field(
            frame,
            form_chunks[5],
            "Storage Size (MB):",
            &form.storage_size,
            form.active_field == CreateDeviceField::StorageSize,
            theme,
        );
    }

    // 5. Name (last)
    let name_chunk = if matches!(state.active_panel, Panel::Android) {
        form_chunks[6]
    } else {
        form_chunks[3]
    };
    render_input_field(
        frame,
        name_chunk,
        "Name:",
        &form.name,
        form.active_field == CreateDeviceField::Name,
        theme,
    );

    // Error message, loading state, or creation progress
    let msg_chunk = if matches!(state.active_panel, Panel::Android) {
        form_chunks[7]
    } else {
        form_chunks[4]
    };

    if form.is_creating {
        // Show creation progress with moon animation
        let progress_msg = if let Some(ref status) = form.creation_status {
            format!("{} {status}", get_animated_moon())
        } else {
            format!("{} Creating device... Please wait...", get_animated_moon())
        };

        let creating_msg = Paragraph::new(progress_msg)
            .style(
                Style::default()
                    .fg(STATUS_COLOR_WARNING)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(creating_msg, msg_chunk);
    } else if form.is_loading_cache && form.available_device_types.is_empty() {
        let loading_msg = Paragraph::new(format!(
            "{} Loading device information...",
            get_animated_moon()
        ))
        .style(Style::default().fg(theme.primary))
        .alignment(Alignment::Center);
        frame.render_widget(loading_msg, msg_chunk);
    } else if let Some(error) = &form.error_message {
        let error_msg = Paragraph::new(error.as_str())
            .style(Style::default().fg(STATUS_COLOR_ERROR))
            .alignment(Alignment::Center);
        frame.render_widget(error_msg, msg_chunk);
    }
}

fn render_input_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    is_active: bool,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(FORM_LABEL_WIDTH), Constraint::Min(1)])
        .split(area);

    // Label
    let label_widget = Paragraph::new(label).style(Style::default().fg(theme.text));
    frame.render_widget(label_widget, chunks[0]);

    // Input field
    let input_style = if is_active {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let display_value = if is_active {
        format!("{value}_")
    } else {
        value.to_string()
    };

    let input_widget = Paragraph::new(display_value)
        .style(input_style)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(input_widget, chunks[1]);
}

fn render_select_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    options: &[String],
    is_active: bool,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(FORM_LABEL_WIDTH), Constraint::Min(1)])
        .split(area);

    // Label
    let label_widget = Paragraph::new(label).style(Style::default().fg(theme.text));
    frame.render_widget(label_widget, chunks[0]);

    // Select field
    let style = if is_active {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    // Only show < > brackets if there are multiple options and field is active
    let display_value = if is_active && options.len() > 1 {
        format!("< {value} >")
    } else {
        value.to_string()
    };

    let select_widget = Paragraph::new(display_value)
        .style(style)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(select_widget, chunks[1]);
}

fn render_confirm_delete_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
    if let Some(ref dialog) = state.confirm_delete_dialog {
        let platform_name = match dialog.platform {
            Panel::Android => "Android device",
            Panel::Ios => "iOS simulator",
        };

        let device_icon = match dialog.platform {
            Panel::Android => "🤖",
            Panel::Ios => "🍎",
        };

        let message = format!(
            "Are you sure you want to delete this {}?\n\n{} {}\n\nThis action cannot be undone.",
            platform_name, device_icon, dialog.device_name
        );

        render_confirmation_dialog(
            frame,
            frame.area(),
            "Confirm Delete",
            &message,
            "🗑",
            STATUS_COLOR_ERROR,
            theme,
        );
    }
}

fn render_confirm_wipe_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
    if let Some(ref dialog) = state.confirm_wipe_dialog {
        let platform_name = match dialog.platform {
            Panel::Android => "Android device",
            Panel::Ios => "iOS simulator",
        };

        let device_icon = match dialog.platform {
            Panel::Android => "🤖",
            Panel::Ios => "🍎",
        };

        let message = format!(
            "Are you sure you want to wipe this {}?\n\n{} {}\n\nThis will erase all data and reset to factory state.",
            platform_name, device_icon, dialog.device_name
        );

        render_confirmation_dialog(
            frame,
            frame.area(),
            "Confirm Wipe",
            &message,
            "🧹",
            STATUS_COLOR_WARNING,
            theme,
        );
    }
}

fn render_device_details_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let border_style = Style::default().fg(theme.text);

    if let Some(details) = state.get_selected_device_details() {
        let mut lines = Vec::new();

        // === HEADER: Device Name & Platform ===
        let platform_icon = match details.platform {
            crate::app::Panel::Android => "🤖",
            crate::app::Panel::Ios => "🍎",
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
        if details.platform == crate::app::Panel::Android {
            if let Some(ref sys_img) = details.system_image {
                let architecture = if sys_img.contains("arm64") {
                    "arm64-v8a"
                } else if sys_img.contains("x86_64") {
                    "x86_64"
                } else if sys_img.contains("x86") {
                    "x86"
                } else {
                    "unknown"
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
        let is_loading =
            details.platform == crate::app::Panel::Android && details.device_path.is_none();

        if is_loading {
            let moon_icon = get_animated_moon();
            let loading_text = format!("{moon_icon} Loading");
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

fn render_notifications(frame: &mut Frame, state: &AppState, _theme: &Theme) {
    if state.notifications.is_empty() {
        return;
    }

    let size = frame.area();

    // Position notifications in the top-right corner
    let notification_width = DIALOG_WIDTH_SMALL.min(size.width - DIALOG_MARGIN);
    let notification_height = NOTIFICATION_HEIGHT; // Height per notification

    for (i, notification) in state.notifications.iter().enumerate() {
        let y_offset = i as u16 * (notification_height + 1); // Add spacing between notifications

        if y_offset + notification_height > size.height {
            break; // Don't render notifications that would go off-screen
        }

        let x = size.width.saturating_sub(notification_width + 2);
        let y = 1 + y_offset; // Start from top with margin

        let notification_area = Rect::new(x, y, notification_width, notification_height);

        // Clear the area behind the notification
        frame.render_widget(Clear, notification_area);

        // Determine colors based on notification type
        let (border_color, text_color, icon) = match notification.notification_type {
            NotificationType::Success => (STATUS_COLOR_SUCCESS, UI_COLOR_TEXT_BRIGHT, "✓"),
            NotificationType::Error => (STATUS_COLOR_ERROR, UI_COLOR_TEXT_BRIGHT, "✗"),
            NotificationType::Warning => (STATUS_COLOR_WARNING, UI_COLOR_BACKGROUND, "⚠"),
            NotificationType::Info => (STATUS_COLOR_INFO, UI_COLOR_TEXT_BRIGHT, "ℹ"),
        };

        let notification_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner_area = notification_block.inner(notification_area);
        frame.render_widget(notification_block, notification_area);

        // Create notification content
        let lines = vec![
            Line::from(vec![
                Span::styled(
                    icon,
                    Style::default()
                        .fg(border_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(&notification.message, Style::default().fg(text_color)),
            ]),
            Line::from(vec![Span::styled(
                format!("  {}", notification.timestamp.format("%H:%M:%S")),
                Style::default().fg(UI_COLOR_TEXT_NORMAL),
            )]),
        ];

        let notification_content =
            Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(notification_content, inner_area);
    }
}

fn render_device_commands(frame: &mut Frame, area: Rect, state: &AppState, _theme: &Theme) {
    let device_text = match state.mode {
        crate::app::Mode::Normal => {
            "🔄 [r]efresh  🔀 [Tab]switch panels  🔁 [h/l/←/→]switch  🚀 [Enter]start/stop  🔃 [k/j/↑/↓]move  ➕ [c]reate  ❌ [d]elete  🧹 [w]ipe  📦 [i]nstall"
        }
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

fn render_api_level_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
    let size = frame.area();

    // Get API level management state
    let api_mgmt = match &state.api_level_management {
        Some(mgmt) => mgmt,
        None => return,
    };

    // Calculate dialog dimensions (larger to prevent content being cut off)
    let dialog_width = DIALOG_WIDTH_LARGE.min(size.width - 2);
    let dialog_height = DIALOG_HEIGHT_LARGE.min(size.height - 2);

    let dialog_area = Rect {
        x: (size.width - dialog_width) / 2,
        y: (size.height - dialog_height) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear background
    frame.render_widget(Clear, dialog_area);

    // Dialog border with statistics
    let installed_count = api_mgmt
        .api_levels
        .iter()
        .filter(|api| api.is_installed)
        .count();
    let total_count = api_mgmt.api_levels.len();
    let title = format!("📦 Android System Images ({installed_count}/{total_count} installed)");

    let dialog_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));

    let inner_area = dialog_block.inner(dialog_area);
    frame.render_widget(dialog_block, dialog_area);

    // Layout for content with proper spacing (similar to create device dialog)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                      // Top margin
            Constraint::Length(2),                      // Instructions
            Constraint::Min(API_LEVEL_LIST_MIN_HEIGHT), // API level list (bigger area)
            Constraint::Length(FORM_FOOTER_HEIGHT),     // Progress/error/status area
            Constraint::Length(1),                      // Shortcuts
        ])
        .split(inner_area);

    // Skip chunks[0] for top margin

    // Instructions with better formatting
    let instruction_text = "✅ Green = Installed  📦 Gray = Available  Select and press Enter/d";
    let instructions = Paragraph::new(instruction_text)
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Center);
    frame.render_widget(instructions, chunks[1]);

    // API level list - always show list area, even if empty or loading
    if api_mgmt.api_levels.is_empty() {
        let empty_msg = if api_mgmt.is_loading {
            // Don't show anything in the list area while loading, status shown below
            ""
        } else {
            "No API levels found. Please check your Android SDK installation."
        };

        let empty_widget = Paragraph::new(empty_msg)
            .style(Style::default().fg(UI_COLOR_TEXT_DIM))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.text)),
            );
        frame.render_widget(empty_widget, chunks[2]);
    } else {
        // Calculate visible range for scrolling
        let available_height = chunks[2].height.saturating_sub(2) as usize; // Subtract borders
        let total_items = api_mgmt.api_levels.len();

        // Calculate scroll offset to keep selected item visible
        let scroll_offset = api_mgmt.get_scroll_offset(available_height);

        // Get visible items
        let visible_items: Vec<_> = api_mgmt
            .api_levels
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(available_height)
            .collect();

        // Create list items
        let items: Vec<ListItem> = visible_items
            .into_iter()
            .map(|(i, api)| {
                let selected = i == api_mgmt.selected_index;

                // Status icon only (no text needed)
                let status_icon = if api.is_installed { "✅" } else { "📦" };

                // Show recommended variant info
                let variant_info = if let Some(variant) = api.get_recommended_variant() {
                    format!(" - {}", variant.display_name)
                } else {
                    String::new()
                };

                let text = format!("{status_icon} {}{variant_info}", api.display_name);

                let style = if selected {
                    // Selected item: use theme primary background with contrasting text
                    if api.is_installed {
                        Style::default()
                            .bg(theme.primary)
                            .fg(UI_COLOR_TEXT_BRIGHT)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .bg(theme.primary)
                            .fg(UI_COLOR_BACKGROUND)
                            .add_modifier(Modifier::BOLD)
                    }
                } else {
                    // Non-selected items: use color coding
                    if api.is_installed {
                        Style::default()
                            .fg(STATUS_COLOR_SUCCESS)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(UI_COLOR_TEXT_DIM)
                    }
                };

                ListItem::new(text).style(style)
            })
            .collect();

        // Add scroll indicators to title if needed
        let list_title = if total_items > available_height {
            let position_info = format!("{}/{total_items}", api_mgmt.selected_index + 1);
            let scroll_indicator =
                if scroll_offset > 0 && scroll_offset + available_height < total_items {
                    " [↕]" // Can scroll both ways
                } else if scroll_offset > 0 {
                    " [↑]" // Can scroll up
                } else if scroll_offset + available_height < total_items {
                    " [↓]" // Can scroll down
                } else {
                    ""
                };
            format!("API Levels ({position_info}){scroll_indicator}")
        } else {
            format!("API Levels ({total_items})")
        };

        let list = List::new(items).block(
            Block::default()
                .title(list_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.text)),
        );
        frame.render_widget(list, chunks[2]);
    }

    // Status display (similar to create device dialog)
    if api_mgmt.is_loading {
        let loading_msg = Paragraph::new(format!("{} Loading API levels...", get_animated_moon()))
            .style(
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(loading_msg, chunks[3]);
    } else if let Some(ref progress) = api_mgmt.install_progress {
        let (progress_text, color) = if progress.percentage >= 100 {
            (
                "✅ Installation completed successfully!".to_string(),
                STATUS_COLOR_SUCCESS,
            )
        } else {
            (
                format!(
                    "{} {} - {}%",
                    get_animated_moon(),
                    progress.operation,
                    progress.percentage
                ),
                STATUS_COLOR_WARNING,
            )
        };

        let progress_widget = Paragraph::new(progress_text)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(progress_widget, chunks[3]);
    } else if let Some(ref package) = api_mgmt.installing_package {
        let installing_msg =
            Paragraph::new(format!("{} Processing: {package}", get_animated_moon()))
                .style(
                    Style::default()
                        .fg(STATUS_COLOR_WARNING)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
        frame.render_widget(installing_msg, chunks[3]);
    } else if let Some(ref error) = api_mgmt.error_message {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(STATUS_COLOR_ERROR))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(error_widget, chunks[3]);
    }

    // Shortcuts at the bottom - dynamic based on selected item
    let shortcuts = if api_mgmt.install_progress.is_some() || api_mgmt.installing_package.is_some()
    {
        "⏳ Processing... Please wait..."
    } else if let Some(selected_api) = api_mgmt.get_selected_api_level() {
        if selected_api.is_installed {
            "[↑/↓/j/k] Navigate  [d] Uninstall Selected  [Esc] Cancel"
        } else {
            "[↑/↓/j/k] Navigate  [Enter] Install Selected  [Esc] Cancel"
        }
    } else {
        "[↑/↓/j/k] Navigate  [Enter] Install  [d] Uninstall  [Esc] Cancel"
    };
    let shortcuts_widget = Paragraph::new(shortcuts)
        .style(
            Style::default()
                .fg(UI_COLOR_TEXT_DIM)
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Center);
    frame.render_widget(shortcuts_widget, chunks[4]);
}
