use crate::{
    app::{AppState, FocusedPanel, Panel},
    constants::{
        colors::*,
        ui_text::{
            device_states::IOS_UNAVAILABLE, navigation::*, status_indicators::*, text_formatting::*,
        },
    },
    ui::Theme,
};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub(crate) fn render_android_panel(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    theme: &Theme,
) {
    let is_active = state.active_panel == Panel::Android;
    let is_focused = state.focused_panel == FocusedPanel::DeviceList;
    let border_style = if is_active && is_focused {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let available_height = area.height.saturating_sub(2) as usize;
    let total_devices = state.android_devices.len();
    let scroll_offset = state.get_android_scroll_offset(available_height);
    state.android_scroll_offset = scroll_offset;

    let visible_devices: Vec<_> = state
        .android_devices
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(available_height)
        .collect();

    let items: Vec<ListItem> = visible_devices
        .into_iter()
        .map(|(i, device)| {
            let selected = i == state.selected_android && is_active;
            let status_indicator = if device.is_running {
                ACTIVE_INDICATOR
            } else {
                INACTIVE_INDICATOR
            };
            let text = format!(
                "{status_indicator} {}",
                device.name.replace(UNDERSCORE_STR, SPACE_STR_SINGLE)
            );

            let style = if selected {
                Style::default().bg(theme.primary).fg(UI_COLOR_BACKGROUND)
            } else if device.is_running {
                Style::default().fg(STATUS_COLOR_ACTIVE)
            } else {
                Style::default().fg(theme.text)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let title = build_panel_title(
        "🤖 Android",
        is_active,
        total_devices,
        available_height,
        scroll_offset,
        state.selected_android,
    );

    let block_style = if is_active {
        Style::default().bg(theme.focused_bg)
    } else {
        Style::default().bg(theme.unfocused_bg)
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(block_style),
    );

    frame.render_widget(list, area);
}

pub(crate) fn render_ios_panel(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme) {
    let is_active = state.active_panel == Panel::Ios;
    let is_focused = state.focused_panel == FocusedPanel::DeviceList;
    let border_style = if is_active && is_focused {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let available_height = area.height.saturating_sub(2) as usize;
    let total_devices = state.ios_devices.len();
    let scroll_offset = state.get_ios_scroll_offset(available_height);
    state.ios_scroll_offset = scroll_offset;

    let visible_devices: Vec<_> = state
        .ios_devices
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(available_height)
        .collect();

    let items: Vec<ListItem> = visible_devices
        .into_iter()
        .map(|(i, device)| {
            let selected = i == state.selected_ios && is_active;
            let status_indicator = if device.is_running {
                ACTIVE_INDICATOR
            } else {
                INACTIVE_INDICATOR
            };
            let availability = if device.is_available {
                ""
            } else {
                IOS_UNAVAILABLE
            };
            let text = format!("{status_indicator} {}{availability}", device.name);

            let style = if selected {
                Style::default().bg(theme.primary).fg(UI_COLOR_BACKGROUND)
            } else if device.is_running {
                Style::default().fg(STATUS_COLOR_ACTIVE)
            } else if !device.is_available {
                Style::default().fg(UI_COLOR_TEXT_DIM)
            } else {
                Style::default().fg(theme.text)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let title = if cfg!(target_os = "macos") {
        build_panel_title(
            "🍎 iOS",
            is_active,
            total_devices,
            available_height,
            scroll_offset,
            state.selected_ios,
        )
    } else {
        "🍎 iOS (macOS only)".to_string()
    };

    let block_style = if is_active {
        Style::default().bg(theme.focused_bg)
    } else {
        Style::default().bg(theme.unfocused_bg)
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(block_style),
    );

    frame.render_widget(list, area);
}

fn build_panel_title(
    title_prefix: &str,
    is_active: bool,
    total_devices: usize,
    available_height: usize,
    scroll_offset: usize,
    selected_index: usize,
) -> String {
    if is_active && total_devices > 0 {
        let position_info = format!("{}/{}", selected_index + 1, total_devices);
        let scroll_indicator = if total_devices > available_height {
            if scroll_offset > 0 && scroll_offset + available_height < total_devices {
                SCROLL_BOTH
            } else if scroll_offset > 0 {
                SCROLL_UP
            } else if scroll_offset + available_height < total_devices {
                SCROLL_DOWN
            } else {
                SCROLL_NONE
            }
        } else {
            SCROLL_NONE
        };
        format!("{title_prefix} ({position_info}){scroll_indicator}")
    } else {
        format!("{title_prefix} ({total_devices})")
    }
}
