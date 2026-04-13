use crate::{
    app::{AppState, Mode, Panel},
    constants::{
        colors::*,
        ui_layout::{
            COMMAND_SHORTCUT_MAX_HEIGHT, COMMAND_SHORTCUT_WRAP_PADDING,
            DEVICE_COMMAND_SHORTCUT_DEFAULT_HEIGHT, LOG_COMMAND_SHORTCUT_DEFAULT_HEIGHT,
        },
        ui_text::{
            log_shortcuts::LOG_MODE_SHORTCUTS,
            shortcuts::{
                CREATE, DELETE, HORIZONTAL_NAV, INSTALL, REFRESH, START_STOP, SWITCH_PANELS,
                VERTICAL_NAV, WIPE,
            },
        },
    },
    ui::{widgets::get_animated_moon, Theme},
};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};

pub(crate) fn render_device_commands(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    _theme: &Theme,
) {
    let device_commands = Paragraph::new(format_device_commands_text(state, area.width))
        .style(
            Style::default()
                .fg(UI_COLOR_TEXT_DIM)
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Center);
    frame.render_widget(device_commands, area);
}

pub(crate) fn device_commands_height(state: &AppState, width: u16) -> u16 {
    let lines = device_command_lines(state);
    command_lines_height(&lines, width, DEVICE_COMMAND_SHORTCUT_DEFAULT_HEIGHT)
}

fn format_device_commands_text(state: &AppState, width: u16) -> String {
    format_command_lines(&device_command_lines(state), width)
}

fn device_command_lines(state: &AppState) -> Vec<String> {
    match state.mode {
        Mode::Normal => {
            let navigation_line = [
                REFRESH,
                SWITCH_PANELS,
                HORIZONTAL_NAV,
                START_STOP,
                VERTICAL_NAV,
            ]
            .join("  ");

            let mut actions = vec![CREATE, DELETE, WIPE];
            if matches!(state.active_panel, Panel::Android) {
                actions.push(INSTALL);
            }
            let action_line = actions.join("  ");

            vec![navigation_line, action_line]
        }
        _ => vec![String::new()],
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
                    format_command_lines(&[LOG_MODE_SHORTCUTS.to_string()], area.width),
                    Style::default()
                        .fg(UI_COLOR_TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                )
            }
        }
        _ => (String::new(), Style::default()),
    };

    let log_commands = Paragraph::new(log_text)
        .style(style)
        .alignment(Alignment::Center);
    frame.render_widget(log_commands, area);
}

pub(crate) fn log_commands_height(state: &AppState, width: u16) -> u16 {
    command_lines_height(
        &[log_commands_text(state)],
        width,
        LOG_COMMAND_SHORTCUT_DEFAULT_HEIGHT,
    )
}

fn log_commands_text(state: &AppState) -> String {
    match state.mode {
        Mode::Normal => {
            if state.is_loading {
                format!("{} Loading devices...", get_animated_moon())
            } else if let Some(ref operation) = state.device_operation_status {
                format!("{} {operation}...", get_animated_moon())
            } else {
                LOG_MODE_SHORTCUTS.to_string()
            }
        }
        _ => String::new(),
    }
}

fn format_command_lines(lines: &[String], width: u16) -> String {
    let lines = wrap_command_lines(lines, width);
    if lines.len() <= 1 {
        lines.into_iter().next().unwrap_or_default()
    } else {
        lines.join("\n")
    }
}

fn command_lines_height(lines: &[String], width: u16, default_height: u16) -> u16 {
    if lines.iter().all(|line| line.trim().is_empty()) {
        return 1;
    }

    (wrap_command_lines(lines, width).len() as u16)
        .max(default_height)
        .min(COMMAND_SHORTCUT_MAX_HEIGHT)
}

fn wrap_command_lines(lines: &[String], width: u16) -> Vec<String> {
    let mut wrapped = Vec::new();

    for line in lines {
        for wrapped_line in wrap_single_command_line(line, width) {
            if wrapped.len() == usize::from(COMMAND_SHORTCUT_MAX_HEIGHT) {
                return wrapped;
            }
            wrapped.push(wrapped_line);
        }
    }

    if wrapped.is_empty() {
        return vec![String::new()];
    }

    wrapped
}

fn wrap_single_command_line(text: &str, width: u16) -> Vec<String> {
    if text.trim().is_empty() {
        return vec![String::new()];
    }

    let available_width = usize::from(width.saturating_sub(COMMAND_SHORTCUT_WRAP_PADDING).max(1));
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        push_wrapped_word(
            word,
            available_width,
            usize::from(COMMAND_SHORTCUT_MAX_HEIGHT),
            &mut lines,
            &mut current,
        );
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    }
}

fn push_wrapped_word(
    word: &str,
    width: usize,
    max_lines: usize,
    lines: &mut Vec<String>,
    current: &mut String,
) {
    if lines.len() == max_lines {
        return;
    }

    let word_width = display_width(word);
    if current.is_empty() {
        if word_width <= width {
            current.push_str(word);
        } else {
            push_long_word_chunks(word, width, max_lines, lines, current);
        }
        return;
    }

    if display_width(current) + 1 + word_width <= width {
        current.push(' ');
        current.push_str(word);
        return;
    }

    if lines.len() < max_lines {
        lines.push(std::mem::take(current));
    }

    if lines.len() == max_lines {
        return;
    }

    if word_width <= width {
        current.push_str(word);
    } else {
        push_long_word_chunks(word, width, max_lines, lines, current);
    }
}

fn push_long_word_chunks(
    word: &str,
    width: usize,
    max_lines: usize,
    lines: &mut Vec<String>,
    current: &mut String,
) {
    let mut chunk = String::new();

    for ch in word.chars() {
        if display_width(&chunk) >= width {
            if lines.len() < max_lines {
                lines.push(std::mem::take(&mut chunk));
            }
            if lines.len() == max_lines {
                current.clear();
                return;
            }
        }
        chunk.push(ch);
    }

    *current = chunk;
}

fn display_width(text: &str) -> usize {
    Line::from(text).width()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::ui_text::shortcuts::{
        ANDROID_NORMAL_MODE_SHORTCUTS, IOS_NORMAL_MODE_SHORTCUTS,
    };

    #[test]
    fn test_normal_mode_shortcuts_include_install_for_android_only() {
        assert!(ANDROID_NORMAL_MODE_SHORTCUTS.contains("[i]nstall"));
        assert!(!IOS_NORMAL_MODE_SHORTCUTS.contains("[i]nstall"));
    }

    #[test]
    fn test_device_commands_height_defaults_to_two_lines() {
        let mut state = AppState::new();
        state.mode = Mode::Normal;
        state.active_panel = Panel::Android;

        assert_eq!(
            device_commands_height(&state, 240),
            DEVICE_COMMAND_SHORTCUT_DEFAULT_HEIGHT
        );
    }

    #[test]
    fn test_device_commands_height_is_capped_at_three_lines() {
        let mut state = AppState::new();
        state.mode = Mode::Normal;
        state.active_panel = Panel::Android;

        assert_eq!(
            device_commands_height(&state, 20),
            COMMAND_SHORTCUT_MAX_HEIGHT
        );
    }

    #[test]
    fn test_device_commands_are_split_into_navigation_and_actions_lines() {
        let mut state = AppState::new();
        state.mode = Mode::Normal;
        state.active_panel = Panel::Android;
        let formatted = format_device_commands_text(&state, 240);
        let lines: Vec<_> = formatted.lines().collect();

        assert!(formatted.contains('\n'));
        assert!(lines[0].contains("[Enter]start/stop"));
        assert!(lines[0].contains("[k/j/↑/↓]move"));
        assert!(lines[1].contains("[c]reate"));
        assert!(lines[1].contains("[i]nstall"));
    }

    #[test]
    fn test_emoji_width_is_counted_for_wrapping() {
        let lines = vec!["🔄 [r]efresh  🚀 [Enter]start/stop".to_string()];
        let single_line = format_command_lines(&lines, 40);
        let wrapped = format_command_lines(&lines, 20);

        assert!(!single_line.contains('\n'));
        assert!(wrapped.contains('\n'));
    }

    #[test]
    fn test_wrap_uses_safety_padding_for_earlier_line_breaks() {
        let lines = vec![format!("{REFRESH}  {SWITCH_PANELS}  {HORIZONTAL_NAV}")];
        let wrapped = format_command_lines(&lines, 50);

        assert!(wrapped.contains('\n'));
    }

    #[test]
    fn test_non_empty_shortcuts_keep_two_lines_minimum() {
        let mut state = AppState::new();
        state.mode = Mode::Normal;
        state.active_panel = Panel::Ios;

        assert_eq!(
            device_commands_height(&state, 240),
            DEVICE_COMMAND_SHORTCUT_DEFAULT_HEIGHT
        );
    }
}
