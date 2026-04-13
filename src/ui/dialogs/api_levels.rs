use crate::{
    app::AppState,
    constants::{
        colors::*,
        messages::notifications::INSTALL_PROGRESS_COMPLETE,
        ui_layout::{
            API_LEVEL_LIST_MIN_HEIGHT, DIALOG_HEIGHT_LARGE, DIALOG_WIDTH_LARGE, FORM_FOOTER_HEIGHT,
        },
        ui_text::{api_management::*, progress::*},
    },
    ui::{widgets::get_animated_moon, Theme},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub(crate) fn render_api_level_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
    let size = frame.area();

    let api_mgmt = match &state.api_level_management {
        Some(mgmt) => mgmt,
        None => return,
    };

    let dialog_width = DIALOG_WIDTH_LARGE.min(size.width - 2);
    let dialog_height = DIALOG_HEIGHT_LARGE.min(size.height - 2);

    let dialog_area = ratatui::layout::Rect {
        x: (size.width - dialog_width) / 2,
        y: (size.height - dialog_height) / 2,
        width: dialog_width,
        height: dialog_height,
    };

    frame.render_widget(Clear, dialog_area);

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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(API_LEVEL_LIST_MIN_HEIGHT),
            Constraint::Length(FORM_FOOTER_HEIGHT),
            Constraint::Length(1),
        ])
        .split(inner_area);

    let instruction_text = "✅ Green = Installed  📦 Gray = Available  Select and press Enter/d";
    let instructions = Paragraph::new(instruction_text)
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Center);
    frame.render_widget(instructions, chunks[1]);

    if api_mgmt.api_levels.is_empty() {
        let empty_msg = if api_mgmt.is_loading {
            ""
        } else {
            "No API levels found. Please check your Android SDK installation."
        };

        let empty_widget = Paragraph::new(empty_msg)
            .style(Style::default().fg(UI_COLOR_TEXT_DIM))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.text)),
            );
        frame.render_widget(empty_widget, chunks[2]);
    } else {
        let available_height = chunks[2].height.saturating_sub(2) as usize;
        let total_items = api_mgmt.api_levels.len();
        let scroll_offset = api_mgmt.get_scroll_offset(available_height);

        let visible_items: Vec<_> = api_mgmt
            .api_levels
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(available_height)
            .collect();

        let items: Vec<ListItem> = visible_items
            .into_iter()
            .map(|(i, api)| {
                let selected = i == api_mgmt.selected_index;
                let status_icon = if api.is_installed { "✅" } else { "📦" };

                let variant_info = if let Some(variant) = api.get_recommended_variant() {
                    format!(" - {}", variant.display_name)
                } else {
                    String::new()
                };

                let text = format!("{status_icon} {}{variant_info}", api.display_name);

                let style = if selected {
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
                } else if api.is_installed {
                    Style::default()
                        .fg(STATUS_COLOR_SUCCESS)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(UI_COLOR_TEXT_DIM)
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let list_title = if total_items > available_height {
            let position_info = format!("{}/{total_items}", api_mgmt.selected_index + 1);
            let scroll_indicator =
                if scroll_offset > 0 && scroll_offset + available_height < total_items {
                    " [↕]"
                } else if scroll_offset > 0 {
                    " [↑]"
                } else if scroll_offset + available_height < total_items {
                    " [↓]"
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
            (INSTALL_PROGRESS_COMPLETE.to_string(), STATUS_COLOR_SUCCESS)
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
            .wrap(Wrap { trim: true });
        frame.render_widget(error_widget, chunks[3]);
    }

    let shortcuts = if api_mgmt.is_busy() {
        PROCESSING_WAIT
    } else if let Some(selected_api) = api_mgmt.get_selected_api_level() {
        if selected_api.is_installed {
            NAV_UNINSTALL
        } else {
            NAV_INSTALL
        }
    } else {
        NAV_GENERAL
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
