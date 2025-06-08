//! UI themes

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub background: Color,
    pub text: Color,
    pub selected: Color,
    pub running: Color,
    pub stopped: Color,
    pub error: Color,
    pub border: Color,
    pub focused_bg: Color,   // フォーカス時の背景色
    pub unfocused_bg: Color, // 非フォーカス時の背景色
    pub header: Style,
    pub status: Style,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            primary: Color::Yellow,
            background: Color::Black,
            text: Color::White,
            selected: Color::Yellow,
            running: Color::Green,
            stopped: Color::Gray,
            error: Color::Red,
            border: Color::White,
            focused_bg: Color::Rgb(25, 25, 35), // 薄いグレー系背景
            unfocused_bg: Color::Rgb(20, 20, 25), // より薄い背景
            header: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            status: Style::default().fg(Color::Cyan),
        }
    }

    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            background: Color::White,
            text: Color::Black,
            selected: Color::Blue,
            running: Color::Green,
            stopped: Color::Gray,
            error: Color::Red,
            border: Color::Black,
            focused_bg: Color::Rgb(240, 245, 250), // 薄いブルー系背景
            unfocused_bg: Color::Rgb(250, 250, 255), // より薄い背景
            header: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            status: Style::default().fg(Color::DarkGray),
        }
    }

    pub fn device_status_color(&self, is_running: bool, is_available: bool) -> Color {
        if !is_available {
            self.error
        } else if is_running {
            self.running
        } else {
            self.stopped
        }
    }
}
