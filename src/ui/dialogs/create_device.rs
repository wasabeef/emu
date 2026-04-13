use crate::{
    app::{state::CreateDeviceField, AppState, Panel},
    constants::{
        colors::*,
        ui_layout::{DIALOG_HEIGHT_MEDIUM, DIALOG_WIDTH_MEDIUM, FORM_LABEL_WIDTH},
    },
    ui::{widgets::get_animated_moon, Theme},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

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

pub(crate) fn render_create_device_dialog(frame: &mut Frame, state: &AppState, theme: &Theme) {
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
