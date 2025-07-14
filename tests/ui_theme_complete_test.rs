//! Comprehensive tests for ui::theme module
//!
//! These tests ensure complete coverage of Theme struct and all its methods,
//! including edge cases, serialization, and style applications.

use emu::ui::theme::Theme;
use ratatui::style::{Color, Modifier, Style};

#[test]
fn test_theme_struct_completeness() {
    let dark_theme = Theme::dark();
    let light_theme = Theme::light();

    // Test all fields are properly initialized
    assert_ne!(dark_theme.primary, Color::Reset);
    assert_ne!(dark_theme.background, Color::Reset);
    assert_ne!(dark_theme.text, Color::Reset);
    assert_ne!(dark_theme.selected, Color::Reset);
    assert_ne!(dark_theme.running, Color::Reset);
    assert_ne!(dark_theme.stopped, Color::Reset);
    assert_ne!(dark_theme.error, Color::Reset);
    assert_ne!(dark_theme.border, Color::Reset);
    assert_ne!(dark_theme.focused_bg, Color::Reset);
    assert_ne!(dark_theme.unfocused_bg, Color::Reset);

    // Test light theme fields
    assert_ne!(light_theme.primary, Color::Reset);
    assert_ne!(light_theme.background, Color::Reset);
    assert_ne!(light_theme.text, Color::Reset);
    assert_ne!(light_theme.selected, Color::Reset);
    assert_ne!(light_theme.running, Color::Reset);
    assert_ne!(light_theme.stopped, Color::Reset);
    assert_ne!(light_theme.error, Color::Reset);
    assert_ne!(light_theme.border, Color::Reset);
    assert_ne!(light_theme.focused_bg, Color::Reset);
    assert_ne!(light_theme.unfocused_bg, Color::Reset);
}

#[test]
fn test_dark_theme_creation() {
    let theme = Theme::dark();

    // Test specific dark theme colors
    assert_eq!(theme.primary, Color::Yellow);
    assert_eq!(theme.background, Color::Black);
    assert_eq!(theme.text, Color::White);
    assert_eq!(theme.selected, Color::Yellow);
    assert_eq!(theme.running, Color::Green);
    assert_eq!(theme.stopped, Color::Gray);
    assert_eq!(theme.error, Color::Red);
    assert_eq!(theme.border, Color::Gray);
    assert_eq!(theme.focused_bg, Color::Rgb(25, 25, 35));
    assert_eq!(theme.unfocused_bg, Color::Rgb(20, 20, 25));
}

#[test]
fn test_light_theme_creation() {
    let theme = Theme::light();

    // Test specific light theme colors
    assert_eq!(theme.primary, Color::Blue);
    assert_eq!(theme.background, Color::White);
    assert_eq!(theme.text, Color::Black);
    assert_eq!(theme.selected, Color::Blue);
    assert_eq!(theme.running, Color::Green);
    assert_eq!(theme.stopped, Color::Gray);
    assert_eq!(theme.error, Color::Red);
    assert_eq!(theme.border, Color::Black);
    assert_eq!(theme.focused_bg, Color::Rgb(240, 245, 250));
    assert_eq!(theme.unfocused_bg, Color::Rgb(250, 250, 255));
}

#[test]
fn test_theme_style_properties() {
    let dark_theme = Theme::dark();
    let light_theme = Theme::light();

    // Test header styles
    assert_eq!(dark_theme.header.fg, Some(Color::Yellow));
    assert!(dark_theme.header.add_modifier.contains(Modifier::BOLD));

    assert_eq!(light_theme.header.fg, Some(Color::Blue));
    assert!(light_theme.header.add_modifier.contains(Modifier::BOLD));

    // Test status styles
    assert_eq!(dark_theme.status.fg, Some(Color::Cyan));
    assert_eq!(light_theme.status.fg, Some(Color::DarkGray));
}

#[test]
fn test_device_status_color_all_combinations() {
    let theme = Theme::dark();

    // Test all possible combinations of is_running and is_available

    // Case 1: Running and available
    let color1 = theme.device_status_color(true, true);
    assert_eq!(color1, theme.running);
    assert_eq!(color1, Color::Green);

    // Case 2: Not running but available
    let color2 = theme.device_status_color(false, true);
    assert_eq!(color2, theme.stopped);
    assert_eq!(color2, Color::Gray);

    // Case 3: Running but not available (corrupted device)
    let color3 = theme.device_status_color(true, false);
    assert_eq!(color3, theme.error);
    assert_eq!(color3, Color::Red);

    // Case 4: Not running and not available
    let color4 = theme.device_status_color(false, false);
    assert_eq!(color4, theme.error);
    assert_eq!(color4, Color::Red);
}

#[test]
fn test_device_status_color_light_theme() {
    let theme = Theme::light();

    // Test device status colors with light theme
    assert_eq!(theme.device_status_color(true, true), Color::Green);
    assert_eq!(theme.device_status_color(false, true), Color::Gray);
    assert_eq!(theme.device_status_color(true, false), Color::Red);
    assert_eq!(theme.device_status_color(false, false), Color::Red);
}

#[test]
fn test_theme_cloning() {
    let original = Theme::dark();
    let cloned = original.clone();

    // Test that cloning preserves all properties
    assert_eq!(cloned.primary, original.primary);
    assert_eq!(cloned.background, original.background);
    assert_eq!(cloned.text, original.text);
    assert_eq!(cloned.selected, original.selected);
    assert_eq!(cloned.running, original.running);
    assert_eq!(cloned.stopped, original.stopped);
    assert_eq!(cloned.error, original.error);
    assert_eq!(cloned.border, original.border);
    assert_eq!(cloned.focused_bg, original.focused_bg);
    assert_eq!(cloned.unfocused_bg, original.unfocused_bg);

    // Test that styles are also cloned correctly
    assert_eq!(cloned.header.fg, original.header.fg);
    assert_eq!(cloned.header.add_modifier, original.header.add_modifier);
    assert_eq!(cloned.status.fg, original.status.fg);
}

#[test]
fn test_theme_debug_formatting() {
    let theme = Theme::dark();
    let debug_output = format!("{theme:?}");

    // Test that debug output contains expected information
    assert!(debug_output.contains("Theme"));
    assert!(debug_output.contains("primary"));
    assert!(debug_output.contains("background"));
    assert!(debug_output.contains("text"));
    assert!(debug_output.contains("selected"));
    assert!(debug_output.contains("running"));
    assert!(debug_output.contains("stopped"));
    assert!(debug_output.contains("error"));
    assert!(debug_output.contains("border"));
    assert!(debug_output.contains("focused_bg"));
    assert!(debug_output.contains("unfocused_bg"));
    assert!(debug_output.contains("header"));
    assert!(debug_output.contains("status"));
}

#[test]
fn test_theme_consistency_between_themes() {
    let dark_theme = Theme::dark();
    let light_theme = Theme::light();

    // Test that some colors are consistent between themes
    assert_eq!(dark_theme.running, light_theme.running); // Green
    assert_eq!(dark_theme.stopped, light_theme.stopped); // Gray
    assert_eq!(dark_theme.error, light_theme.error); // Red

    // Test that other colors are different between themes
    assert_ne!(dark_theme.primary, light_theme.primary);
    assert_ne!(dark_theme.background, light_theme.background);
    assert_ne!(dark_theme.text, light_theme.text);
    assert_ne!(dark_theme.border, light_theme.border);
    assert_ne!(dark_theme.focused_bg, light_theme.focused_bg);
    assert_ne!(dark_theme.unfocused_bg, light_theme.unfocused_bg);
}

#[test]
fn test_theme_color_accessibility() {
    let dark_theme = Theme::dark();
    let light_theme = Theme::light();

    // Test that dark theme has appropriate contrast
    assert_eq!(dark_theme.background, Color::Black);
    assert_eq!(dark_theme.text, Color::White);

    // Test that light theme has appropriate contrast
    assert_eq!(light_theme.background, Color::White);
    assert_eq!(light_theme.text, Color::Black);

    // Test that error colors are consistent and highly visible
    assert_eq!(dark_theme.error, Color::Red);
    assert_eq!(light_theme.error, Color::Red);

    // Test that status colors are meaningful
    assert_eq!(dark_theme.running, Color::Green); // Green for go/active
    assert_eq!(dark_theme.stopped, Color::Gray); // Gray for inactive
}

#[test]
fn test_theme_background_colors_specificity() {
    let dark_theme = Theme::dark();
    let light_theme = Theme::light();

    // Test that RGB values are specific and intentional
    match dark_theme.focused_bg {
        Color::Rgb(r, g, b) => {
            assert_eq!(r, 25);
            assert_eq!(g, 25);
            assert_eq!(b, 35);
        }
        _ => panic!("Expected RGB color for dark theme focused background"),
    }

    match dark_theme.unfocused_bg {
        Color::Rgb(r, g, b) => {
            assert_eq!(r, 20);
            assert_eq!(g, 20);
            assert_eq!(b, 25);
        }
        _ => panic!("Expected RGB color for dark theme unfocused background"),
    }

    match light_theme.focused_bg {
        Color::Rgb(r, g, b) => {
            assert_eq!(r, 240);
            assert_eq!(g, 245);
            assert_eq!(b, 250);
        }
        _ => panic!("Expected RGB color for light theme focused background"),
    }

    match light_theme.unfocused_bg {
        Color::Rgb(r, g, b) => {
            assert_eq!(r, 250);
            assert_eq!(g, 250);
            assert_eq!(b, 255);
        }
        _ => panic!("Expected RGB color for light theme unfocused background"),
    }
}

#[test]
fn test_theme_style_modifiers() {
    let dark_theme = Theme::dark();
    let light_theme = Theme::light();

    // Test header style modifiers
    assert!(dark_theme.header.add_modifier.contains(Modifier::BOLD));
    assert!(!dark_theme.header.add_modifier.contains(Modifier::ITALIC));
    assert!(!dark_theme
        .header
        .add_modifier
        .contains(Modifier::UNDERLINED));

    assert!(light_theme.header.add_modifier.contains(Modifier::BOLD));
    assert!(!light_theme.header.add_modifier.contains(Modifier::ITALIC));
    assert!(!light_theme
        .header
        .add_modifier
        .contains(Modifier::UNDERLINED));

    // Test status style (should have no modifiers)
    assert!(!dark_theme.status.add_modifier.contains(Modifier::BOLD));
    assert!(!dark_theme.status.add_modifier.contains(Modifier::ITALIC));
    assert!(!dark_theme
        .status
        .add_modifier
        .contains(Modifier::UNDERLINED));

    assert!(!light_theme.status.add_modifier.contains(Modifier::BOLD));
    assert!(!light_theme.status.add_modifier.contains(Modifier::ITALIC));
    assert!(!light_theme
        .status
        .add_modifier
        .contains(Modifier::UNDERLINED));
}

#[test]
fn test_theme_edge_cases() {
    let theme = Theme::dark();

    // Test device status with edge case values
    // These should all work without panicking
    let _color1 = theme.device_status_color(true, true);
    let _color2 = theme.device_status_color(false, false);
    let _color3 = theme.device_status_color(true, false);
    let _color4 = theme.device_status_color(false, true);

    // Test that theme can be created multiple times
    let theme1 = Theme::dark();
    let theme2 = Theme::dark();
    assert_eq!(theme1.primary, theme2.primary);

    let theme3 = Theme::light();
    let theme4 = Theme::light();
    assert_eq!(theme3.primary, theme4.primary);
}

#[test]
fn test_theme_usage_patterns() {
    let theme = Theme::dark();

    // Test typical usage patterns

    // Pattern 1: Style creation for different elements
    let title_style = Style::default()
        .fg(theme.primary)
        .add_modifier(Modifier::BOLD);
    assert_eq!(title_style.fg, Some(theme.primary));
    assert!(title_style.add_modifier.contains(Modifier::BOLD));

    // Pattern 2: Conditional styling based on state
    let device_style = |is_running: bool, is_available: bool| {
        Style::default().fg(theme.device_status_color(is_running, is_available))
    };

    assert_eq!(device_style(true, true).fg, Some(Color::Green));
    assert_eq!(device_style(false, true).fg, Some(Color::Gray));
    assert_eq!(device_style(false, false).fg, Some(Color::Red));

    // Pattern 3: Panel background styling
    let panel_style = |is_focused: bool| {
        Style::default().bg(if is_focused {
            theme.focused_bg
        } else {
            theme.unfocused_bg
        })
    };

    assert_eq!(panel_style(true).bg, Some(theme.focused_bg));
    assert_eq!(panel_style(false).bg, Some(theme.unfocused_bg));
}
