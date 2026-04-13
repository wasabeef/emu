use crate::{
    app::{AppState, Panel},
    constants::{
        colors::*,
        ui_layout::{LOG_LEVEL_WIDTH, LOG_TIMESTAMP_WIDTH, MESSAGE_TRUNCATE_SUFFIX_LENGTH},
    },
    ui::Theme,
};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub(crate) fn render_log_panel(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let log_device_name = if let Some((panel, device_name)) = &state.current_log_device {
        format!("{panel:?} - {device_name}")
    } else {
        match state.active_panel {
            Panel::Android => state
                .android_devices
                .get(state.selected_android)
                .map(|device| {
                    if device.is_running {
                        format!("{} (not streaming)", device.name)
                    } else {
                        format!("{} (stopped)", device.name)
                    }
                })
                .unwrap_or_else(|| "No device selected".to_string()),
            Panel::Ios => state
                .ios_devices
                .get(state.selected_ios)
                .map(|device| {
                    if device.is_running {
                        format!("{} (not streaming)", device.name)
                    } else {
                        format!("{} (stopped)", device.name)
                    }
                })
                .unwrap_or_else(|| "No device selected".to_string()),
        }
    };

    let mut title_spans = vec![Span::raw("📋 Logs - "), Span::raw(&log_device_name)];

    if let Some(ref filter) = state.log_filter_level {
        title_spans.push(Span::raw(" [Filter: "));
        title_spans.push(Span::styled(filter, filter_style(filter, theme)));
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
                Span::styled(
                    format!("[{}]", &entry.level),
                    level_style(&entry.level, theme),
                ),
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

fn filter_style(filter: &str, theme: &Theme) -> Style {
    match filter {
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
    }
}

fn level_style(level: &str, theme: &Theme) -> Style {
    match level {
        "ERROR" => Style::default().fg(LOG_COLOR_ERROR),
        "WARN" => Style::default().fg(LOG_COLOR_WARN),
        "INFO" => Style::default().fg(LOG_COLOR_INFO),
        "DEBUG" => Style::default().fg(LOG_COLOR_DEBUG),
        _ => Style::default().fg(theme.text),
    }
}
