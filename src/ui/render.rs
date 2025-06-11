//! UI rendering

use crate::{
    app::{
        state::{CreateDeviceField, NotificationType},
        AppState, Panel,
    },
    ui::Theme,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn draw_app(frame: &mut Frame, state: &mut AppState, theme: &Theme) {
    let size = frame.area();

    // Ensure we have enough space
    if size.height < 10 || size.width < 40 {
        let msg = Paragraph::new("Terminal too small").style(Style::default().fg(Color::Red));
        frame.render_widget(msg, size);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(1), // Status bar (reduced from 3 to 1)
        ])
        .split(size);

    // Header with icon
    let header_text = if state.fullscreen_logs {
        " 🦤 Emu - Device Manager [FULLSCREEN LOGS]"
    } else {
        " 🦤 Emu - Device Manager"
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
                Constraint::Percentage(40), // Device panels with device commands
                Constraint::Min(10),        // Log panel
                Constraint::Length(1),      // Log commands
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
                Constraint::Percentage(30), // Android
                Constraint::Percentage(30), // iOS
                Constraint::Percentage(40), // Device Details
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
        crate::app::Mode::Normal => "[Ctrl+q]uit",
        crate::app::Mode::CreateDevice => {
            "[Tab]next field [Shift+Tab]prev field [Enter]submit [Esc]cancel"
        }
        crate::app::Mode::ConfirmDelete => "[Ctrl+q]uit",
        crate::app::Mode::ConfirmWipe => "[Ctrl+q]uit",
        _ => "[Ctrl+q]uit",
    };

    // Status bar without borders, smaller text
    let status_with_icon = match state.mode {
        crate::app::Mode::Normal => format!("🚪 {}", status_text),
        crate::app::Mode::CreateDevice => format!("📝 {}", status_text),
        crate::app::Mode::ConfirmDelete => format!("{}", status_text),
        crate::app::Mode::ConfirmWipe => format!("{}", status_text),
        _ => status_text.to_string(),
    };
    let status = Paragraph::new(status_with_icon)
        .style(
            Style::default()
                .fg(Color::DarkGray)
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
            let text = format!(
                "{} {} (API {})",
                status_indicator, device.name, device.api_level
            );

            let style = if selected {
                Style::default().bg(theme.primary).fg(Color::Black)
            } else if device.is_running {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(theme.text)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    // Add scroll indicators to title if needed
    let title = if is_active && !state.android_devices.is_empty() {
        let position_info = format!("{}/{}", state.selected_android + 1, total_devices);
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
        format!("🤖 Android ({}){}", position_info, scroll_indicator)
    } else {
        format!("🤖 Android ({})", total_devices)
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
            let text = format!("{} {}{}", status_indicator, device.name, availability);

            let style = if selected {
                Style::default().bg(theme.primary).fg(Color::Black)
            } else if device.is_running {
                Style::default().fg(Color::Green)
            } else if !device.is_available {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(theme.text)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    // Add scroll indicators to title if needed
    let title = if cfg!(target_os = "macos") {
        if is_active && !state.ios_devices.is_empty() {
            let position_info = format!("{}/{}", state.selected_ios + 1, total_devices);
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
            format!("🍎 iOS ({}){}", position_info, scroll_indicator)
        } else {
            format!("🍎 iOS ({})", total_devices)
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
    // Get current selected device name
    let selected_device = match state.active_panel {
        Panel::Android => state
            .android_devices
            .get(state.selected_android)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "No device selected".to_string()),
        Panel::Ios => state
            .ios_devices
            .get(state.selected_ios)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "No device selected".to_string()),
    };

    // Build title with colored filter level
    let mut title_spans = vec![Span::raw("📋 Logs - "), Span::raw(&selected_device)];

    if let Some(ref filter) = state.log_filter_level {
        title_spans.push(Span::raw(" [Filter: "));

        let filter_style = match filter.as_str() {
            "ERROR" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            "WARN" => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            "INFO" => Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            "DEBUG" => Style::default()
                .fg(Color::Gray)
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
    let timestamp_width = 9; // "HH:MM:SS "
    let level_width = 9; // "[ERROR] " max width
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
                "ERROR" => Style::default().fg(Color::Red),
                "WARN" => Style::default().fg(Color::Yellow),
                "INFO" => Style::default().fg(Color::Green),
                "DEBUG" => Style::default().fg(Color::Gray),
                _ => Style::default().fg(theme.text),
            };

            // Truncate message if it's too long (safely handle UTF-8 boundaries)
            let message = if entry.message.chars().count() > message_width && message_width > 3 {
                let truncate_len = message_width.saturating_sub(3);
                let truncated: String = entry.message.chars().take(truncate_len).collect();
                format!("{}...", truncated)
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
                    Style::default().fg(Color::DarkGray),
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
    let dialog_width = 80.min(size.width - 4);
    let dialog_height = 16.min(size.height - 4);

    // Center the dialog
    let x = (size.width.saturating_sub(dialog_width)) / 2;
    let y = (size.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Add a background block to ensure full coverage
    let background_block = Block::default().style(Style::default().bg(Color::Black));
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
        // Show creation progress
        let progress_msg = if let Some(ref status) = form.creation_status {
            status.clone()
        } else {
            "Creating device... Please wait...".to_string()
        };

        let creating_msg = Paragraph::new(progress_msg)
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(creating_msg, msg_chunk);
    } else if form.is_loading_cache && form.available_device_types.is_empty() {
        let loading_msg = Paragraph::new("Loading device information...")
            .style(Style::default().fg(theme.primary))
            .alignment(Alignment::Center);
        frame.render_widget(loading_msg, msg_chunk);
    } else if let Some(error) = &form.error_message {
        let error_msg = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
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
        .constraints([Constraint::Length(20), Constraint::Min(1)])
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
        format!("{}_", value)
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
        .constraints([Constraint::Length(20), Constraint::Min(1)])
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
        format!("< {} >", value)
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
        let size = frame.area();

        // Calculate dialog dimensions
        let dialog_width = 50.min(size.width - 4);
        let dialog_height = 8.min(size.height - 4);

        // Center the dialog
        let x = (size.width.saturating_sub(dialog_width)) / 2;
        let y = (size.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        // Clear the area behind the dialog
        frame.render_widget(Clear, dialog_area);

        // Add a background block to ensure full coverage
        let background_block = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(background_block, dialog_area);

        let dialog_block = Block::default()
            .title("Confirm Delete")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));

        let inner_area = dialog_block.inner(dialog_area);
        frame.render_widget(dialog_block, dialog_area);

        // Layout for content and shortcuts
        let layout_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Message content
                Constraint::Length(1), // Shortcuts
            ])
            .split(inner_area);

        // Message text
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

        let message_widget = Paragraph::new(message)
            .style(Style::default().fg(theme.text))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(message_widget, layout_chunks[0]);

        // Shortcuts at the bottom
        let shortcuts = Paragraph::new("✅ [Y]es   ❌ [N]o / [Esc] Cancel")
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);

        frame.render_widget(shortcuts, layout_chunks[1]);
    }
}

fn render_confirm_wipe_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
    if let Some(ref dialog) = state.confirm_wipe_dialog {
        let size = frame.area();

        // Calculate dialog dimensions
        let dialog_width = 50.min(size.width - 4);
        let dialog_height = 8.min(size.height - 4);

        // Center the dialog
        let x = (size.width.saturating_sub(dialog_width)) / 2;
        let y = (size.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        // Clear the area behind the dialog
        frame.render_widget(Clear, dialog_area);

        // Add a background block to ensure full coverage
        let background_block = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(background_block, dialog_area);

        let dialog_block = Block::default()
            .title("Confirm Wipe")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner_area = dialog_block.inner(dialog_area);
        frame.render_widget(dialog_block, dialog_area);

        // Layout for content and shortcuts
        let layout_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Message content
                Constraint::Length(1), // Shortcuts
            ])
            .split(inner_area);

        // Message text
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

        let message_widget = Paragraph::new(message)
            .style(Style::default().fg(theme.text))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(message_widget, layout_chunks[0]);

        // Shortcuts at the bottom
        let shortcuts = Paragraph::new("✅ [Y]es   ❌ [N]o / [Esc] Cancel")
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);

        frame.render_widget(shortcuts, layout_chunks[1]);
    }
}

fn render_device_details_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let border_style = Style::default().fg(theme.text);

    if let Some(details) = state.get_selected_device_details() {
        // Create content lines
        let mut lines = Vec::new();

        // Device name with platform icon
        let platform_icon = match details.platform {
            crate::app::Panel::Android => "🤖",
            crate::app::Panel::Ios => "🍎",
        };
        lines.push(Line::from(vec![
            Span::styled(platform_icon, Style::default()),
            Span::raw(" "),
            Span::styled(
                details.name.clone(),
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(""));

        // Status
        let status_color = if details.status == "Running" || details.status == "Booted" {
            Color::Green
        } else {
            Color::Gray
        };
        lines.push(Line::from(vec![
            Span::raw("Status: "),
            Span::styled(
                format!("● {}", details.status),
                Style::default().fg(status_color),
            ),
        ]));

        // API Level / iOS Version
        lines.push(Line::from(vec![
            Span::raw("Version: "),
            Span::styled(
                details.api_level_or_version,
                Style::default().fg(theme.primary),
            ),
        ]));

        // Device Type
        lines.push(Line::from(vec![
            Span::raw("Type: "),
            Span::raw(details.device_type),
        ]));

        lines.push(Line::from(""));

        // Hardware info (Android only for now)
        if details.platform == crate::app::Panel::Android {
            if let Some(ref ram) = details.ram_size {
                lines.push(Line::from(vec![
                    Span::raw("🧠 RAM: "),
                    Span::styled(ram.clone(), Style::default().fg(Color::Cyan)),
                ]));
            }

            if let Some(ref storage) = details.storage_size {
                lines.push(Line::from(vec![
                    Span::raw("💾 Storage: "),
                    Span::styled(storage.clone(), Style::default().fg(Color::Cyan)),
                ]));
            }

            if let Some(ref resolution) = details.resolution {
                lines.push(Line::from(vec![
                    Span::raw("📱 Resolution: "),
                    Span::raw(resolution.clone()),
                ]));
            }

            if let Some(ref dpi) = details.dpi {
                lines.push(Line::from(vec![
                    Span::raw("🎯 DPI: "),
                    Span::raw(dpi.clone()),
                ]));
            }
        }

        // Path info
        if let Some(ref path) = details.device_path {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::raw("📂 Path:")]));
            // Show full path with word wrapping
            lines.push(Line::from(vec![Span::styled(
                path.clone(),
                Style::default().fg(Color::DarkGray),
            )]));
        }

        // System image (Android only)
        if details.platform == crate::app::Panel::Android {
            if let Some(ref system_image) = details.system_image {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::raw("🏷️  System Image:")]));
                // Show full system image path with word wrapping
                lines.push(Line::from(vec![Span::styled(
                    system_image.clone(),
                    Style::default().fg(Color::DarkGray),
                )]));
            }
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
    } else {
        // No device selected
        let no_device_text = Paragraph::new("No device selected")
            .block(
                Block::default()
                    .title("Device Details")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .style(Style::default().fg(Color::DarkGray))
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
    let notification_width = 60.min(size.width - 4);
    let notification_height = 4; // Height per notification

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
            NotificationType::Success => (Color::Green, Color::White, "✓"),
            NotificationType::Error => (Color::Red, Color::White, "✗"),
            NotificationType::Warning => (Color::Yellow, Color::Black, "⚠"),
            NotificationType::Info => (Color::Blue, Color::White, "ℹ"),
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
                Style::default().fg(Color::Gray),
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
            if state.is_loading {
                "⏳ Loading devices..."
            } else if let Some(ref operation) = state.device_operation_status {
                &format!("⏳ {}", operation)
            } else {
                "🔄 [r]efresh  🔀 [Tab]switch panels  🔁 [h/l/←/→]switch  🚀 [Enter]start/stop  🔃 [k/j/↑/↓]move  ➕ [c]reate  ❌ [d]elete  🧹 [w]ipe"
            }
        }
        _ => "",
    };

    // Device commands without borders, smaller and dimmer text, centered
    let device_commands = Paragraph::new(device_text)
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Center);
    frame.render_widget(device_commands, area);
}

fn render_log_commands(frame: &mut Frame, area: Rect, state: &AppState, _theme: &Theme) {
    let log_text = match state.mode {
        crate::app::Mode::Normal => "🗑️ [Shift+L]clear logs  🔍 [f]ilter  🖥️ [Shift+F]ullscreen",
        _ => "",
    };

    // Log commands without borders, smaller and dimmer text, centered
    let log_commands = Paragraph::new(log_text)
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Center);
    frame.render_widget(log_commands, area);
}
