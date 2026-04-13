use crate::{
    app::AppState,
    constants::{
        colors::*,
        ui_layout::{LOADING_INDICATOR_MARGIN, SEPARATOR_LENGTH},
        ui_text::{architectures::*, progress::*},
    },
    models::Platform,
    ui::{widgets::get_animated_moon, Theme},
};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub(crate) fn render_device_details_panel(
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
                .map(|dpi| format!(" ({dpi})"))
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
            render_loading_indicator(frame, area);
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

fn render_loading_indicator(frame: &mut Frame, area: Rect) {
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
