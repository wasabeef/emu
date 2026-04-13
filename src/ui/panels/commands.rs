use crate::{
    app::{AppState, Mode, Panel},
    constants::{
        colors::*,
        ui_text::shortcuts::{ANDROID_NORMAL_MODE_SHORTCUTS, IOS_NORMAL_MODE_SHORTCUTS},
    },
    ui::{widgets::get_animated_moon, Theme},
};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    widgets::Paragraph,
    Frame,
};

pub(crate) fn render_device_commands(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    _theme: &Theme,
) {
    let device_text = match state.mode {
        Mode::Normal => normal_mode_shortcuts(state.active_panel),
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

fn normal_mode_shortcuts(panel: Panel) -> &'static str {
    match panel {
        Panel::Android => ANDROID_NORMAL_MODE_SHORTCUTS,
        Panel::Ios => IOS_NORMAL_MODE_SHORTCUTS,
    }
}

pub(crate) fn render_log_commands(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let (log_text, style) = match state.mode {
        Mode::Normal => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_mode_shortcuts_include_install_for_android_only() {
        assert!(normal_mode_shortcuts(Panel::Android).contains("[i]nstall"));
        assert!(!normal_mode_shortcuts(Panel::Ios).contains("[i]nstall"));
    }
}
