//! High-performance UI rendering optimized for 125 FPS operation.
//!
//! This module provides optimized rendering functions designed for smooth, responsive
//! terminal user interface operation. All rendering functions are optimized for:
//! - **Consistent 125 FPS** operation with 8ms frame budgets
//! - **Minimal allocations** through efficient string handling and pre-calculated styles
//! - **Responsive layout** that adapts to terminal size changes
//! - **Smooth animations** with moon phase transitions at 125 FPS
//!
//! # Performance Features
//!
//! - **Style caching**: Pre-calculated styles to avoid repeated computations
//! - **Efficient text rendering**: Single-allocation string building where possible
//! - **Smart scrolling**: Dynamic scroll indicators with minimal overhead
//! - **Memory bounded**: Log rotation and efficient item rendering
//!
//! # Layout Architecture
//!
//! The UI uses a three-panel layout optimized for device management:
//! - **Android panel** (30%): AVD list with status indicators
//! - **iOS panel** (30%): Simulator list with availability status
//! - **Details panel** (40%): Real-time device information and specifications

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
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

// UI Icons
const EMOJI_EMU: &str = "🦤";
const EMOJI_ANDROID: &str = "🤖";
const EMOJI_IOS: &str = "🍎";
const EMOJI_PACKAGE: &str = "📦";
const EMOJI_DELETE: &str = "🗑️";
const EMOJI_FOLDER: &str = "📂";
const EMOJI_TAG: &str = "🏷️";
const EMOJI_BRAIN: &str = "🧠";
const EMOJI_DISK: &str = "💾";
const EMOJI_PHONE: &str = "📱";
const EMOJI_TARGET: &str = "🎯";
const EMOJI_KEY: &str = "🔑";
const EMOJI_MAGNIFIER: &str = "🔍";
const EMOJI_GEAR: &str = "⚙️";
const EMOJI_DOOR: &str = "🚪";
const EMOJI_MEMO: &str = "📝";

// Status Icons
const ICON_SUCCESS: &str = "✓";
const ICON_ERROR: &str = "✗";
const ICON_WARNING: &str = "⚠";
const ICON_INFO: &str = "ℹ";
const ICON_CHECK: &str = "✅";
const ICON_CROSS: &str = "❌";
const ICON_RUNNING: char = '●';
const ICON_STOPPED: char = '○';

// Layout Constants
const HEADER_HEIGHT: u16 = 3;
const MIN_CONTENT_HEIGHT: u16 = 10;
const STATUS_BAR_HEIGHT: u16 = 1;
const ANDROID_PANEL_WIDTH: u16 = 30;
const IOS_PANEL_WIDTH: u16 = 30;
const DETAILS_PANEL_WIDTH: u16 = 40;
const DEVICE_PANELS_HEIGHT: u16 = 40;
const NOTIFICATION_WIDTH: u16 = 60;
const NOTIFICATION_HEIGHT: u16 = 4;

// Animation Constants
const MOON_ANIMATION_INTERVAL_MS: u128 = 200;
const MOON_PHASES: usize = 8;
const MOON_PHASE_0: &str = "🌑";
const MOON_PHASE_1: &str = "🌒";
const MOON_PHASE_2: &str = "🌓";
const MOON_PHASE_3: &str = "🌔";
const MOON_PHASE_4: &str = "🌕";
const MOON_PHASE_5: &str = "🌖";
const MOON_PHASE_6: &str = "🌗";
const MOON_PHASE_7: &str = "🌘";

// Device Commands
const CMD_ANDROID: &str = "🔄 [r]efresh  🔀 [Tab]switch panels  🔁 [h/l/←/→]switch  🚀 [Enter]start/stop  🔃 [k/j/↑/↓]move  ➕ [c]reate  📦 [i]nstall API  ❌ [d]elete  🧹 [w]ipe";
const CMD_IOS: &str = "🔄 [r]efresh  🔀 [Tab]switch panels  🔁 [h/l/←/→]switch  🚀 [Enter]start/stop  🔃 [k/j/↑/↓]move  ➕ [c]reate  ❌ [d]elete  🧹 [w]ipe";

// Log Commands
const CMD_LOG_DEFAULT: &str = "🗑️ [Shift+L]clear logs  🔍 [f]ilter  🖥️ [Shift+F]ullscreen";

/// Helper function to get animated moon phase based on elapsed time
fn get_animated_moon(elapsed_ms: u128) -> &'static str {
    let moon_index = (elapsed_ms / MOON_ANIMATION_INTERVAL_MS) % MOON_PHASES as u128;
    match moon_index {
        0 => MOON_PHASE_0,
        1 => MOON_PHASE_1,
        2 => MOON_PHASE_2,
        3 => MOON_PHASE_3,
        4 => MOON_PHASE_4,
        5 => MOON_PHASE_5,
        6 => MOON_PHASE_6,
        _ => MOON_PHASE_7,
    }
}

/// Helper function to clear dialog area
fn clear_dialog_area(frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);
    let background_block = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(background_block, area);
}

/// Helper function to calculate centered dialog area
fn calculate_centered_dialog_area(frame: &Frame, width: u16, height: u16) -> Rect {
    let size = frame.area();
    let dialog_width = width.min(size.width - 4);
    let dialog_height = height.min(size.height - 4);

    let x = (size.width.saturating_sub(dialog_width)) / 2;
    let y = (size.height.saturating_sub(dialog_height)) / 2;

    Rect::new(x, y, dialog_width, dialog_height)
}

/// Helper function to get log level style
fn get_log_level_style(level: &str) -> Style {
    match level {
        "ERROR" => Style::default().fg(Color::Red),
        "WARN" => Style::default().fg(Color::Yellow),
        "INFO" => Style::default().fg(Color::Green),
        "DEBUG" => Style::default().fg(Color::Gray),
        _ => Style::default(),
    }
}

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
            Constraint::Length(HEADER_HEIGHT),     // Header
            Constraint::Min(MIN_CONTENT_HEIGHT),   // Main content
            Constraint::Length(STATUS_BAR_HEIGHT), // Status bar
        ])
        .split(size);

    // Header with icon
    let header_text = if state.fullscreen_logs {
        format!(" {} Emu - Device Manager [FULLSCREEN LOGS]", EMOJI_EMU)
    } else {
        format!(" {} Emu - Device Manager", EMOJI_EMU)
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
                Constraint::Min(10),                   // Log panel takes all space
                Constraint::Length(STATUS_BAR_HEIGHT), // Log commands
            ])
            .split(chunks[1])
    } else {
        // Normal mode
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(DEVICE_PANELS_HEIGHT), // Device panels with device commands
                Constraint::Min(10),                          // Log panel
                Constraint::Length(STATUS_BAR_HEIGHT),        // Log commands
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
                Constraint::Percentage(ANDROID_PANEL_WIDTH), // Android
                Constraint::Percentage(IOS_PANEL_WIDTH),     // iOS
                Constraint::Percentage(DETAILS_PANEL_WIDTH), // Device Details
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
        crate::app::Mode::Normal => format!("{} {}", EMOJI_DOOR, status_text),
        crate::app::Mode::CreateDevice => format!("{} {}", EMOJI_MEMO, status_text),
        crate::app::Mode::ConfirmDelete => status_text.to_string(),
        crate::app::Mode::ConfirmWipe => status_text.to_string(),
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
        crate::app::Mode::ApiLevelInstall => {
            render_api_level_install_dialog(frame, state, theme);
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

    // Pre-calculate styles once
    let selected_style = Style::default().bg(theme.primary).fg(Color::Black);
    let running_style = Style::default().fg(Color::Green);
    let normal_style = Style::default().fg(theme.text);

    // Build items more efficiently
    let items: Vec<ListItem> = state
        .android_devices
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(available_height)
        .map(|(i, device)| {
            let selected = i == state.selected_android && is_active;
            let status_char = if device.is_running {
                ICON_RUNNING
            } else {
                ICON_STOPPED
            };

            // Build text with single allocation
            let text = format!("{} {} (API {})", status_char, device.name, device.api_level);

            let style = if selected {
                selected_style
            } else if device.is_running {
                running_style
            } else {
                normal_style
            };

            ListItem::new(text).style(style)
        })
        .collect();

    // Add scroll indicators to title if needed
    let title = if is_active && !state.android_devices.is_empty() {
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
        // Use Cow to avoid allocation when no scroll indicator
        if scroll_indicator.is_empty() {
            format!(
                " 🤖 Android ({}/{}) ",
                state.selected_android + 1,
                total_devices
            )
        } else {
            format!(
                " 🤖 Android ({}/{}){} ",
                state.selected_android + 1,
                total_devices,
                scroll_indicator
            )
        }
    } else {
        format!(" 🤖 Android ({}) ", total_devices)
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

    // Use the centralized scroll calculation method
    let scroll_offset = state.get_ios_scroll_offset(available_height);

    // Update the state's scroll offset
    state.ios_scroll_offset = scroll_offset;

    // Pre-calculate styles once
    let selected_style = Style::default().bg(theme.primary).fg(Color::Black);
    let running_style = Style::default().fg(Color::Green);
    let unavailable_style = Style::default().fg(Color::DarkGray);
    let normal_style = Style::default().fg(theme.text);

    // Build items more efficiently
    let items: Vec<ListItem> = state
        .ios_devices
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(available_height)
        .map(|(i, device)| {
            let selected = i == state.selected_ios && is_active;

            // Build text more efficiently with single allocation
            let status_char = if device.is_running {
                ICON_RUNNING
            } else {
                ICON_STOPPED
            };
            let text = if device.is_available {
                format!("{} {}", status_char, device.name)
            } else {
                format!("{} {} (unavailable)", status_char, device.name)
            };

            let style = if selected {
                selected_style
            } else if device.is_running {
                running_style
            } else if !device.is_available {
                unavailable_style
            } else {
                normal_style
            };

            ListItem::new(text).style(style)
        })
        .collect();

    // Add scroll indicators to title if needed
    let title = if cfg!(target_os = "macos") {
        if is_active && !state.ios_devices.is_empty() {
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
            // Use Cow to avoid allocation when no scroll indicator
            if scroll_indicator.is_empty() {
                format!(
                    " {} iOS ({}/{}) ",
                    EMOJI_IOS,
                    state.selected_ios + 1,
                    total_devices
                )
            } else {
                format!(
                    " {} iOS ({}/{}){} ",
                    EMOJI_IOS,
                    state.selected_ios + 1,
                    total_devices,
                    scroll_indicator
                )
            }
        } else {
            format!(" {} iOS ({}) ", EMOJI_IOS, total_devices)
        }
    } else {
        format!("{} iOS (macOS only)", EMOJI_IOS)
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
    let mut title_spans = vec![
        Span::raw(" 📋 Logs - "),
        Span::raw(&selected_device),
        Span::raw(" "),
    ];

    if let Some(ref filter) = state.log_filter_level {
        title_spans.push(Span::raw(" [Filter: "));

        let filter_style = get_log_level_style(filter).add_modifier(Modifier::BOLD);

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
            let level_style = get_log_level_style(&entry.level);

            // Truncate message if it's too long (safely handle UTF-8 boundaries)
            let message = if entry.message.chars().count() > message_width && message_width > 3 {
                let truncate_len = message_width.saturating_sub(3);
                // Avoid collecting into intermediate string
                let mut truncated = String::with_capacity(truncate_len + 3);
                truncated.extend(entry.message.chars().take(truncate_len));
                truncated.push_str("...");
                truncated
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
            // Use static empty string instead of allocating new one
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
    // Calculate dialog dimensions
    let dialog_area = calculate_centered_dialog_area(frame, 80, 16);

    // Clear the area behind the dialog
    clear_dialog_area(frame, dialog_area);

    // Add a background block to ensure full coverage
    let background_block = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(background_block, dialog_area);

    // Dialog title with icon based on active panel
    let title = match state.active_panel {
        Panel::Android => format!("{} Create Android Device", EMOJI_ANDROID),
        Panel::Ios => format!("{} Create iOS Device", EMOJI_IOS),
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
        let elapsed = state.app_start_time.elapsed().as_millis();
        let moon = get_animated_moon(elapsed);
        let progress_msg = if let Some(ref status) = form.creation_status {
            format!("{} {}", moon, status)
        } else {
            format!("{} Creating device... Please wait...", moon)
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
        let elapsed = state.app_start_time.elapsed().as_millis();
        let moon = get_animated_moon(elapsed);
        let loading_msg = Paragraph::new(format!("{} Loading device information...", moon))
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
        // Calculate dialog dimensions
        let dialog_area = calculate_centered_dialog_area(frame, 50, 8);

        // Clear the area behind the dialog
        clear_dialog_area(frame, dialog_area);

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
            Panel::Android => EMOJI_ANDROID,
            Panel::Ios => EMOJI_IOS,
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
        let shortcuts = Paragraph::new(format!(
            "{} [Y]es   {} [N]o / [Esc] Cancel",
            ICON_CHECK, ICON_CROSS
        ))
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
        // Calculate dialog dimensions
        let dialog_area = calculate_centered_dialog_area(frame, 50, 8);

        // Clear the area behind the dialog
        clear_dialog_area(frame, dialog_area);

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
            Panel::Android => EMOJI_ANDROID,
            Panel::Ios => EMOJI_IOS,
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
        let shortcuts = Paragraph::new(format!(
            "{} [Y]es   {} [N]o / [Esc] Cancel",
            ICON_CHECK, ICON_CROSS
        ))
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
            crate::app::Panel::Android => EMOJI_ANDROID,
            crate::app::Panel::Ios => EMOJI_IOS,
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
                format!("{} {}", ICON_RUNNING, details.status),
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

        // Platform-specific hardware info
        match details.platform {
            crate::app::Panel::Android => {
                if let Some(ref ram) = details.ram_size {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{} RAM: ", EMOJI_BRAIN), Style::default()),
                        Span::styled(ram.clone(), Style::default().fg(Color::Cyan)),
                    ]));
                }

                if let Some(ref storage) = details.storage_size {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{} Storage: ", EMOJI_DISK), Style::default()),
                        Span::styled(storage.clone(), Style::default().fg(Color::Cyan)),
                    ]));
                }

                if let Some(ref resolution) = details.resolution {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{} Resolution: ", EMOJI_PHONE), Style::default()),
                        Span::raw(resolution.clone()),
                    ]));
                }

                if let Some(ref dpi) = details.dpi {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{} DPI: ", EMOJI_TARGET), Style::default()),
                        Span::raw(dpi.clone()),
                    ]));
                }
            }
            crate::app::Panel::Ios => {
                // iOS-specific information
                if let Some(ref udid) = details.udid {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{} UDID: ", EMOJI_KEY), Style::default()),
                        Span::styled(udid.clone(), Style::default().fg(Color::Cyan)),
                    ]));
                }

                if let Some(ref resolution) = details.resolution {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{} Resolution: ", EMOJI_PHONE), Style::default()),
                        Span::raw(resolution.clone()),
                    ]));
                }

                if let Some(ref scale) = details.dpi {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{} Scale: ", EMOJI_MAGNIFIER), Style::default()),
                        Span::raw(format!("{}x", scale)),
                    ]));
                }

                if let Some(ref runtime) = details.runtime {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{} Runtime: ", EMOJI_GEAR), Style::default()),
                        Span::styled(runtime.clone(), Style::default().fg(Color::Yellow)),
                    ]));
                }
            }
        }

        // Path info
        if let Some(ref path) = details.device_path {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::raw(format!(
                "{} Path:",
                EMOJI_FOLDER
            ))]));
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
                lines.push(Line::from(vec![Span::raw(format!(
                    "{} System Image:",
                    EMOJI_TAG
                ))]));
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
                    .title(" Device Details ")
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
                    .title(" Device Details ")
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
    let notification_width = NOTIFICATION_WIDTH.min(size.width - 4);
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
            NotificationType::Success => (Color::Green, Color::White, ICON_SUCCESS),
            NotificationType::Error => (Color::Red, Color::White, ICON_ERROR),
            NotificationType::Warning => (Color::Yellow, Color::Black, ICON_WARNING),
            NotificationType::Info => (Color::Blue, Color::White, ICON_INFO),
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
            // Show different commands based on active panel
            match state.active_panel {
                crate::app::Panel::Android => CMD_ANDROID,
                crate::app::Panel::Ios => CMD_IOS,
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

fn render_log_commands(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let log_text = match state.mode {
        crate::app::Mode::Normal => {
            if state.is_loading {
                let elapsed = state.app_start_time.elapsed().as_millis();
                let moon = get_animated_moon(elapsed);

                format!("{} Loading devices...", moon)
            } else if let Some(ref operation) = state.device_operation_status {
                let elapsed = state.app_start_time.elapsed().as_millis();
                let moon = get_animated_moon(elapsed);

                format!("{} {}", moon, operation)
            } else if let Some((api_level, ref status)) = &state.background_install_status {
                // Show background installation status
                match status {
                    crate::models::SdkInstallStatus::Installing { progress, message } => {
                        let elapsed = state.app_start_time.elapsed().as_millis();
                        let moon = get_animated_moon(elapsed);
                        format!(
                            "{} Installing API {}: {} ({}%) - Press [Esc] to hide",
                            moon, api_level, message, progress
                        )
                    }
                    crate::models::SdkInstallStatus::Completed => {
                        format!(
                            "{} API {} installed successfully - Press [Esc] to hide",
                            ICON_CHECK, api_level
                        )
                    }
                    crate::models::SdkInstallStatus::Failed { error } => {
                        format!(
                            "{} API {} installation failed: {} - Press [Esc] to hide",
                            ICON_CROSS, api_level, error
                        )
                    }
                    _ => CMD_LOG_DEFAULT.to_string(),
                }
            } else {
                CMD_LOG_DEFAULT.to_string()
            }
        }
        _ => String::new(),
    };

    // Status messages with slightly more prominent styling
    let style = if state.is_loading
        || state.device_operation_status.is_some()
        || state.background_install_status.is_some()
    {
        // Make status messages more visible
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::ITALIC)
    } else {
        // Normal log commands remain dimmed
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM)
    };

    let log_commands = Paragraph::new(log_text)
        .style(style)
        .alignment(Alignment::Center);
    frame.render_widget(log_commands, area);
}

/// Renders the API Level installation dialog.
fn render_api_level_install_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
    let Some(dialog) = &state.api_level_install else {
        return;
    };

    let size = frame.area();

    // Calculate dialog size (60% width, 70% height)
    let dialog_width = (size.width as f32 * 0.6) as u16;
    let dialog_height = (size.height as f32 * 0.7) as u16;
    let dialog_area = calculate_centered_dialog_area(frame, dialog_width, dialog_height);

    // Clear the area behind the dialog
    clear_dialog_area(frame, dialog_area);

    // Main dialog block
    let title = if dialog.uninstall_mode {
        format!("{} Uninstall Android API Level", EMOJI_DELETE)
    } else {
        format!("{} Install Android API Level", EMOJI_PACKAGE)
    };

    let dialog_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.primary));

    let inner_area = dialog_block.inner(dialog_area);
    frame.render_widget(dialog_block, dialog_area);

    // Handle error state
    if let Some(error) = &dialog.error_message {
        let error_text = vec![
            Line::from(vec![Span::styled(
                format!("{} Error loading API levels:", ICON_CROSS),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(error, Style::default().fg(Color::Red))]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press [Esc] to close",
                Style::default().fg(Color::Gray),
            )]),
        ];

        let error_paragraph = Paragraph::new(error_text)
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(error_paragraph, inner_area);
        return;
    }

    // Handle loading state
    if dialog.is_loading {
        let loading_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                {
                    let elapsed = state.app_start_time.elapsed().as_millis();
                    let moon = get_animated_moon(elapsed);

                    format!("{} Loading available API levels...", moon)
                },
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Please wait...",
                Style::default().fg(Color::Gray),
            )]),
        ];

        let loading_paragraph = Paragraph::new(loading_text).alignment(Alignment::Center);

        frame.render_widget(loading_paragraph, inner_area);
        return;
    }

    // Split content area
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Instructions (increased for additional line)
            Constraint::Min(8),    // API level list
            Constraint::Length(4), // Installation status
            Constraint::Length(2), // Controls
        ])
        .split(inner_area);

    // Instructions
    let instructions = vec![
        Line::from(vec![Span::styled(
            "Select an API level to install:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "Use ↑/↓ or j/k to navigate, Enter to install, Esc to cancel",
            Style::default().fg(Color::Gray),
        )]),
        Line::from(vec![Span::styled(
            format!("{} Green items are already installed", ICON_CHECK),
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let instructions_paragraph = Paragraph::new(instructions).alignment(Alignment::Left);

    frame.render_widget(instructions_paragraph, content_chunks[0]);

    // API Level list
    if dialog.available_api_levels.is_empty() {
        let empty_text = vec![Line::from(vec![Span::styled(
            "No API levels available for installation",
            Style::default().fg(Color::Yellow),
        )])];

        let empty_paragraph = Paragraph::new(empty_text).alignment(Alignment::Center);

        frame.render_widget(empty_paragraph, content_chunks[1]);
    } else {
        // Create a scrollable viewport for the list if needed
        let list_area = content_chunks[1];
        let list_height = list_area.height as usize;
        let total_items = dialog.available_api_levels.len();

        // Calculate scroll offset to keep selected item visible
        let scroll_offset = if total_items > list_height {
            let selected = dialog.selected_index;
            if selected < list_height / 2 {
                0
            } else if selected > total_items - list_height / 2 {
                total_items.saturating_sub(list_height)
            } else {
                selected.saturating_sub(list_height / 2)
            }
        } else {
            0
        };

        // Pre-calculate styles to avoid repeated computations
        let installed_style = Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
        let normal_style = Style::default().fg(Color::White);
        let disabled_style = Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM);
        // Static strings to avoid repeated allocations
        const INSTALLED_SUFFIX: &str = " ✓ INSTALLED";

        // Only render visible items for performance
        let visible_end = (scroll_offset + list_height).min(total_items);
        let api_level_items: Vec<ListItem> = dialog
            .available_api_levels
            .iter()
            .skip(scroll_offset)
            .take(visible_end - scroll_offset)
            .map(|api_level| {
                let (status_icon, base_style, text_style) = if api_level.installed {
                    (ICON_CHECK, installed_style, disabled_style)
                } else {
                    (EMOJI_PACKAGE, normal_style, normal_style)
                };

                let mut spans = Vec::with_capacity(4);
                spans.push(Span::styled(status_icon, base_style));
                spans.push(Span::raw(" "));
                spans.push(Span::styled(api_level.display_name(), text_style));
                if api_level.installed {
                    spans.push(Span::styled(INSTALLED_SUFFIX, installed_style));
                }
                let line = Line::from(spans);

                ListItem::new(line)
            })
            .collect();

        let api_level_list = List::new(api_level_items)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Available API Levels ({}/{})",
                dialog.selected_index + 1,
                total_items
            )))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        let mut list_state = ListState::default();
        list_state.select(Some(dialog.selected_index.saturating_sub(scroll_offset)));

        frame.render_stateful_widget(api_level_list, list_area, &mut list_state);
    }

    // Installation/Uninstallation status
    let status_text = if dialog.uninstall_mode {
        // Show uninstall status if available, otherwise show ready state
        match dialog.uninstall_status.as_ref() {
            Some(status) => match status {
                crate::models::SdkInstallStatus::Installing {
                    progress: _,
                    message,
                } => {
                    let elapsed = state.app_start_time.elapsed().as_millis();
                    let moon = get_animated_moon(elapsed);

                    vec![
                        Line::from(vec![Span::styled(
                            format!("{} Uninstalling...", moon),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )]),
                        Line::from(vec![Span::styled(
                            message.clone(),
                            Style::default().fg(Color::Gray),
                        )]),
                    ]
                }
                crate::models::SdkInstallStatus::Completed => {
                    vec![Line::from(vec![Span::styled(
                        format!("{} Uninstallation completed successfully!", ICON_CHECK),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )])]
                }
                crate::models::SdkInstallStatus::Failed { error } => {
                    vec![
                        Line::from(vec![Span::styled(
                            format!("{} Uninstallation failed", ICON_CROSS),
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        )]),
                        Line::from(vec![Span::styled(
                            error.clone(),
                            Style::default().fg(Color::Red),
                        )]),
                    ]
                }
                _ => vec![Line::from(vec![Span::styled(
                    "Ready to uninstall",
                    Style::default().fg(Color::White),
                )])],
            },
            None => vec![Line::from(vec![Span::styled(
                "Ready to uninstall (installed APIs only)",
                Style::default().fg(Color::White),
            )])],
        }
    } else {
        match &dialog.install_status {
            crate::models::SdkInstallStatus::Pending => {
                vec![Line::from(vec![Span::styled(
                    "Ready to install",
                    Style::default().fg(Color::White),
                )])]
            }
            crate::models::SdkInstallStatus::Installing {
                progress: _,
                message,
            } => {
                let elapsed = state.app_start_time.elapsed().as_millis();
                let moon = get_animated_moon(elapsed);

                vec![
                    Line::from(vec![Span::styled(
                        format!("{} Installing...", moon),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![Span::styled(
                        message.clone(),
                        Style::default().fg(Color::Gray),
                    )]),
                ]
            }
            crate::models::SdkInstallStatus::Completed => {
                vec![Line::from(vec![Span::styled(
                    format!("{} Installation completed successfully!", ICON_CHECK),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )])]
            }
            crate::models::SdkInstallStatus::Failed { error } => {
                vec![
                    Line::from(vec![Span::styled(
                        format!("{} Installation failed:", ICON_CROSS),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![Span::styled(error, Style::default().fg(Color::Red))]),
                ]
            }
        }
    };

    let status_paragraph = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(status_paragraph, content_chunks[2]);

    // Controls
    let controls_text = if dialog.uninstall_mode {
        if dialog
            .uninstall_status
            .as_ref()
            .is_some_and(|s| s.is_in_progress())
        {
            "Uninstalling... Please wait"
        } else {
            let selected_installed = dialog
                .selected_api_level()
                .map(|api| api.installed)
                .unwrap_or(false);

            if selected_installed {
                "[Enter] Uninstall  [d] Switch to Install  [Esc] Cancel"
            } else {
                "[Enter] Not Installed  [d] Switch to Install  [Esc] Cancel"
            }
        }
    } else if dialog.install_status.is_in_progress() {
        "Installing... Please wait"
    } else {
        // Check if selected API level is already installed
        let selected_installed = dialog
            .selected_api_level()
            .map(|api| api.installed)
            .unwrap_or(false);

        if selected_installed {
            "[Enter] Already Installed  [d] Switch to Uninstall  [Esc] Cancel"
        } else {
            "[Enter] Install Selected  [d] Switch to Uninstall  [Esc] Cancel"
        }
    };

    let controls = Paragraph::new(controls_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(controls, content_chunks[3]);
}
