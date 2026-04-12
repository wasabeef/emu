//! UI rendering

use super::dialogs::{
    render_api_level_dialog, render_confirm_delete_dialog, render_confirm_wipe_dialog,
    render_create_device_dialog, render_notifications,
};
use super::panels::{
    render_android_panel, render_device_commands, render_device_details_panel, render_ios_panel,
    render_log_commands, render_log_panel,
};
use crate::{
    app::AppState,
    constants::{
        colors::*,
        messages::ui::TERMINAL_TOO_SMALL_ERROR,
        ui_layout::{
            ANDROID_PANEL_PERCENTAGE, DEVICE_DETAILS_PANEL_PERCENTAGE, DEVICE_PANELS_PERCENTAGE,
            HEADER_HEIGHT, IOS_PANEL_PERCENTAGE, MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH,
            STATUS_BAR_HEIGHT,
        },
    },
    ui::Theme,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_app(frame: &mut Frame, state: &mut AppState, theme: &Theme) {
    let size = frame.area();

    // Ensure we have enough space
    if size.height < MIN_TERMINAL_HEIGHT || size.width < MIN_TERMINAL_WIDTH {
        let msg =
            Paragraph::new(TERMINAL_TOO_SMALL_ERROR).style(Style::default().fg(STATUS_COLOR_ERROR));
        frame.render_widget(msg, size);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),     // Header
            Constraint::Min(10),                   // Main content
            Constraint::Length(STATUS_BAR_HEIGHT), // Status bar
        ])
        .split(size);

    // Header with icon and version
    let version = env!("CARGO_PKG_VERSION");
    let header_text = if state.fullscreen_logs {
        format!(" 🦤 Emu v{version} - Device Manager [FULLSCREEN LOGS]")
    } else {
        format!(" 🦤 Emu v{version} - Device Manager")
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
                Constraint::Percentage(DEVICE_PANELS_PERCENTAGE), // Device panels with device commands
                Constraint::Min(10),                              // Log panel
                Constraint::Length(1),                            // Log commands
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
                Constraint::Percentage(ANDROID_PANEL_PERCENTAGE), // Android
                Constraint::Percentage(IOS_PANEL_PERCENTAGE),     // iOS
                Constraint::Percentage(DEVICE_DETAILS_PANEL_PERCENTAGE), // Device Details
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
        crate::app::Mode::Normal => format!("🚪 {status_text}"),
        crate::app::Mode::CreateDevice => format!("📝 {status_text}"),
        crate::app::Mode::ConfirmDelete => status_text.to_string(),
        crate::app::Mode::ConfirmWipe => status_text.to_string(),
        _ => status_text.to_string(),
    };
    let status = Paragraph::new(status_with_icon)
        .style(
            Style::default()
                .fg(UI_COLOR_TEXT_DIM)
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
        crate::app::Mode::ManageApiLevels => {
            render_api_level_dialog(frame, state, theme);
        }
        _ => {}
    }

    // Render notifications on top of everything
    render_notifications(frame, state, theme);
}
