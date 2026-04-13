use crate::{
    app::{AppState, Panel},
    constants::{
        colors::*,
        messages::ui::{DIALOG_SHORTCUT_CANCEL, DIALOG_SHORTCUT_NO, DIALOG_SHORTCUT_YES},
        ui_layout::{DIALOG_HEIGHT_SMALL, DIALOG_MARGIN, DIALOG_WIDTH_SMALL},
    },
    ui::Theme,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

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

    frame.render_widget(Clear, dialog_area);

    let background_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(format!("{icon} {title}"))
        .style(Style::default().bg(UI_COLOR_BACKGROUND));
    frame.render_widget(background_block, dialog_area);

    let inner_area = Rect::new(
        dialog_area.x + 1,
        dialog_area.y + 1,
        dialog_area.width.saturating_sub(2),
        dialog_area.height.saturating_sub(2),
    );

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(2), Constraint::Length(2)])
        .split(inner_area);

    let message_text = Paragraph::new(message)
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(message_text, inner_chunks[0]);

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

pub(crate) fn render_confirm_delete_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
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

pub(crate) fn render_confirm_wipe_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
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
