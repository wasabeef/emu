use crate::{
    app::{
        state::{CreateDeviceField, NotificationType},
        AppState, Panel,
    },
    constants::{
        colors::*,
        messages::{
            notifications::INSTALL_PROGRESS_COMPLETE,
            ui::{DIALOG_SHORTCUT_CANCEL, DIALOG_SHORTCUT_NO, DIALOG_SHORTCUT_YES},
        },
        ui_layout::{
            API_LEVEL_LIST_MIN_HEIGHT, DIALOG_HEIGHT_LARGE, DIALOG_HEIGHT_MEDIUM,
            DIALOG_HEIGHT_SMALL, DIALOG_MARGIN, DIALOG_WIDTH_LARGE, DIALOG_WIDTH_MEDIUM,
            DIALOG_WIDTH_SMALL, FORM_FOOTER_HEIGHT, FORM_LABEL_WIDTH,
        },
        ui_text::{api_management::*, notification_icons::*, progress::*},
    },
    ui::{widgets::get_animated_moon, Theme},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
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
        .constraints([Constraint::Length(FORM_LABEL_WIDTH), Constraint::Min(1)])
        .split(area);

    let label_widget = Paragraph::new(label).style(Style::default().fg(theme.text));
    frame.render_widget(label_widget, chunks[0]);

    let input_style = if is_active {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let display_value = if is_active {
        format!("{value}_")
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
        .constraints([Constraint::Length(FORM_LABEL_WIDTH), Constraint::Min(1)])
        .split(area);

    let label_widget = Paragraph::new(label).style(Style::default().fg(theme.text));
    frame.render_widget(label_widget, chunks[0]);

    let style = if is_active {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let display_value = if is_active && options.len() > 1 {
        format!("< {value} >")
    } else {
        value.to_string()
    };

    let select_widget = Paragraph::new(display_value)
        .style(style)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(select_widget, chunks[1]);
}

pub(super) fn render_create_device_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
    let size = frame.area();
    let dialog_width = DIALOG_WIDTH_MEDIUM.min(size.width - 4);
    let dialog_height = DIALOG_HEIGHT_MEDIUM.min(size.height - 4);
    let x = (size.width.saturating_sub(dialog_width)) / 2;
    let y = (size.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);
    let background_block = Block::default().style(Style::default().bg(UI_COLOR_BACKGROUND));
    frame.render_widget(background_block, dialog_area);

    let title = match state.active_panel {
        Panel::Android => "🤖 Create Android Device",
        Panel::Ios => "🍎 Create iOS Device",
    };

    let dialog_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));

    let inner_area = dialog_block.inner(dialog_area);
    frame.render_widget(dialog_block, dialog_area);

    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(inner_area);

    let form = &state.create_device_form;

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

    let msg_chunk = if matches!(state.active_panel, Panel::Android) {
        form_chunks[7]
    } else {
        form_chunks[4]
    };

    if form.is_creating {
        let progress_msg = if let Some(ref status) = form.creation_status {
            format!("{} {status}", get_animated_moon())
        } else {
            format!("{} Creating device... Please wait...", get_animated_moon())
        };

        let creating_msg = Paragraph::new(progress_msg)
            .style(
                Style::default()
                    .fg(STATUS_COLOR_WARNING)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(creating_msg, msg_chunk);
    } else if form.is_loading_cache && form.available_device_types.is_empty() {
        let loading_msg = Paragraph::new(format!(
            "{} Loading device information...",
            get_animated_moon()
        ))
        .style(Style::default().fg(theme.primary))
        .alignment(Alignment::Center);
        frame.render_widget(loading_msg, msg_chunk);
    } else if let Some(error) = &form.error_message {
        let error_msg = Paragraph::new(error.as_str())
            .style(Style::default().fg(STATUS_COLOR_ERROR))
            .alignment(Alignment::Center);
        frame.render_widget(error_msg, msg_chunk);
    }
}

pub(super) fn render_confirm_delete_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
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

pub(super) fn render_confirm_wipe_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
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

pub(super) fn render_notifications(frame: &mut Frame, state: &AppState, _theme: &Theme) {
    if state.notifications.is_empty() {
        return;
    }

    let size = frame.area();
    let notification_width = DIALOG_WIDTH_SMALL.min(size.width - DIALOG_MARGIN);
    let notification_height = crate::constants::ui_layout::NOTIFICATION_HEIGHT;

    for (i, notification) in state.notifications.iter().enumerate() {
        let y_offset = i as u16 * (notification_height + 1);

        if y_offset + notification_height > size.height {
            break;
        }

        let x = size.width.saturating_sub(notification_width + 2);
        let y = 1 + y_offset;
        let notification_area = Rect::new(x, y, notification_width, notification_height);

        frame.render_widget(Clear, notification_area);

        let (border_color, text_color, icon) = match notification.notification_type {
            NotificationType::Success => (STATUS_COLOR_SUCCESS, UI_COLOR_TEXT_BRIGHT, SUCCESS),
            NotificationType::Error => (STATUS_COLOR_ERROR, UI_COLOR_TEXT_BRIGHT, ERROR),
            NotificationType::Warning => (STATUS_COLOR_WARNING, UI_COLOR_BACKGROUND, WARNING),
            NotificationType::Info => (STATUS_COLOR_INFO, UI_COLOR_TEXT_BRIGHT, INFO),
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

pub(super) fn render_api_level_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
    let size = frame.area();

    let api_mgmt = match &state.api_level_management {
        Some(mgmt) => mgmt,
        None => return,
    };

    let dialog_width = DIALOG_WIDTH_LARGE.min(size.width - 2);
    let dialog_height = DIALOG_HEIGHT_LARGE.min(size.height - 2);

    let dialog_area = Rect {
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
