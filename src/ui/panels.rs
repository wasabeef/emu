//! Panel rendering helpers.

use crate::{
    app::{AppState, Panel},
    constants::{
        colors::*,
        ui_layout::{
            LOADING_INDICATOR_MARGIN, LOG_LEVEL_WIDTH, LOG_TIMESTAMP_WIDTH,
            MESSAGE_TRUNCATE_SUFFIX_LENGTH, SEPARATOR_LENGTH,
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
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub(super) fn render_android_panel(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    theme: &Theme,
) {
    let is_active = state.active_panel == Panel::Android;
    let is_focused = state.focused_panel == crate::app::FocusedPanel::DeviceList;
    let border_style = if is_active && is_focused {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let available_height = area.height.saturating_sub(2) as usize;
    let total_devices = state.android_devices.len();
    let scroll_offset = state.get_android_scroll_offset(available_height);
    state.android_scroll_offset = scroll_offset;

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

    let title = if is_active && !state.android_devices.is_empty() {
        let position_info = format!("{}/{total_devices}", state.selected_android + 1);
        let scroll_indicator = if total_devices > available_height {
            if scroll_offset > 0 && scroll_offset + available_height < total_devices {
                SCROLL_BOTH
            } else if scroll_offset > 0 {
                SCROLL_UP
            } else if scroll_offset + available_height < total_devices {
                SCROLL_DOWN
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

pub(super) fn render_ios_panel(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme) {
    let is_active = state.active_panel == Panel::Ios;
    let is_focused = state.focused_panel == crate::app::FocusedPanel::DeviceList;
    let border_style = if is_active && is_focused {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let available_height = area.height.saturating_sub(2) as usize;
    let total_devices = state.ios_devices.len();
    let scroll_offset = state.get_ios_scroll_offset(available_height);
    state.ios_scroll_offset = scroll_offset;

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

    let title = if cfg!(target_os = "macos") {
        if is_active && !state.ios_devices.is_empty() {
            let position_info = format!("{}/{total_devices}", state.selected_ios + 1);
            let scroll_indicator = if total_devices > available_height {
                if scroll_offset > 0 && scroll_offset + available_height < total_devices {
                    SCROLL_BOTH
                } else if scroll_offset > 0 {
                    SCROLL_UP
                } else if scroll_offset + available_height < total_devices {
                    SCROLL_DOWN
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

pub(super) fn render_log_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
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
    let available_height = area.height.saturating_sub(2) as usize;
    let available_width = area.width.saturating_sub(2) as usize;
    let timestamp_width = LOG_TIMESTAMP_WIDTH;
    let level_width = LOG_LEVEL_WIDTH;
    let message_width = available_width.saturating_sub(timestamp_width + level_width);

    let filtered_logs = state.get_filtered_logs();
    let visible_logs: Vec<&_> = if filtered_logs.len() > available_height {
        let start_idx = filtered_logs.len().saturating_sub(available_height);
        filtered_logs[start_idx..].to_vec()
    } else {
        filtered_logs
    };

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

            let message = if entry.message.chars().count() > message_width
                && message_width > MESSAGE_TRUNCATE_SUFFIX_LENGTH
            {
                let truncate_len = message_width.saturating_sub(MESSAGE_TRUNCATE_SUFFIX_LENGTH);
                let truncated: String = entry.message.chars().take(truncate_len).collect();
                format!("{truncated}...")
            } else {
                entry.message.clone()
            };

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
                Span::raw(padding),
            ])
        })
        .collect();

    let mut all_lines = log_lines;
    while all_lines.len() < available_height {
        all_lines.push(Line::from(vec![Span::raw(" ".repeat(available_width))]));
    }

    let logs = Paragraph::new(all_lines)
        .block(
            Block::default()
                .title(title_line)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.text)),
        )
        .style(Style::default().fg(theme.text))
        .wrap(Wrap { trim: false });

    frame.render_widget(logs, area);
}

pub(super) fn render_device_details_panel(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
) {
    let border_style = Style::default().fg(theme.text);

    if let Some(details) = state.get_selected_device_details() {
        let mut lines = Vec::new();

        let platform_icon = match details.platform {
            Platform::Android => "🤖",
            Platform::Ios => "🍎",
        };
        lines.push(Line::from(vec![
            Span::styled(platform_icon, Style::default().fg(theme.primary)),
            Span::raw(" "),
            Span::styled(
                details.name.replace('_', " "),
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(vec![Span::styled(
            "━".repeat(SEPARATOR_LENGTH as usize),
            Style::default().fg(UI_COLOR_TEXT_DIM),
        )]));

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

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("🆔 ID: "),
            Span::styled(
                details.identifier.clone(),
                Style::default().fg(STATUS_COLOR_INFO),
            ),
        ]));

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
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);

        let is_loading = details.platform == Platform::Android && details.device_path.is_none();

        if is_loading {
            let moon_icon = get_animated_moon();
            let loading_text = format!("{moon_icon} {LOADING}");
            let loading_width = "🌙 Loading".len() as u16;
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

pub(super) fn render_device_commands(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    _theme: &Theme,
) {
    let device_text = match state.mode {
        crate::app::Mode::Normal => NORMAL_MODE_SHORTCUTS,
        _ => "",
    };

    let device_commands = Paragraph::new(device_text)
        .style(
            Style::default()
                .fg(UI_COLOR_TEXT_DIM)
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Center);
    frame.render_widget(device_commands, area);
}

pub(super) fn render_log_commands(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
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
