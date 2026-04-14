use crate::{
    app::AppState,
    constants::{
        colors::*,
        ui_layout::{DIALOG_MARGIN, DIALOG_WIDTH_SMALL, NOTIFICATION_HEIGHT},
        ui_text::notification_icons::*,
    },
    ui::Theme,
};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub(crate) fn render_notifications(frame: &mut Frame, state: &AppState, _theme: &Theme) {
    if state.notifications.is_empty() {
        return;
    }

    let size = frame.area();
    let notification_width = DIALOG_WIDTH_SMALL.min(size.width - DIALOG_MARGIN);
    let notification_height = NOTIFICATION_HEIGHT;

    for (i, notification) in state.notifications.iter().enumerate() {
        let y_offset = i as u16 * (notification_height + 1);

        if y_offset + notification_height > size.height {
            break;
        }

        let x = size.width.saturating_sub(notification_width + 2);
        let y = 1 + y_offset;
        let notification_area =
            ratatui::layout::Rect::new(x, y, notification_width, notification_height);

        frame.render_widget(Clear, notification_area);

        let (border_color, text_color, icon) = match notification.notification_type {
            crate::app::state::NotificationType::Success => {
                (STATUS_COLOR_SUCCESS, UI_COLOR_TEXT_BRIGHT, SUCCESS)
            }
            crate::app::state::NotificationType::Error => {
                (STATUS_COLOR_ERROR, UI_COLOR_TEXT_BRIGHT, ERROR)
            }
            crate::app::state::NotificationType::Warning => {
                (STATUS_COLOR_WARNING, UI_COLOR_TEXT_BRIGHT, WARNING)
            }
            crate::app::state::NotificationType::Info => {
                (STATUS_COLOR_INFO, UI_COLOR_TEXT_BRIGHT, INFO)
            }
        };

        let notification_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner_area = notification_block.inner(notification_area);
        frame.render_widget(notification_block, notification_area);

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

        let notification_content = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(notification_content, inner_area);
    }
}
