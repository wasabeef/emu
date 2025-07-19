//! Mock Terminal Backend for UI testing
//!
//! This module provides a mock implementation of ratatui's Backend trait
//! for testing UI rendering without requiring an actual terminal.

use ratatui::backend::Backend;
use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::{Position, Rect, Size};
use ratatui::style::Style;
use std::io::Result;

/// Mock Terminal Backend for testing UI components
#[derive(Debug, Clone)]
pub struct MockBackend {
    /// Terminal width in characters
    pub width: u16,
    /// Terminal height in characters  
    pub height: u16,
    /// Buffer history for test verification
    pub buffers: Vec<Buffer>,
    /// Cursor position
    pub cursor_position: Option<(u16, u16)>,
    /// Whether cursor is shown
    pub cursor_visible: bool,
}

impl MockBackend {
    /// Create a new MockBackend with specified dimensions
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            buffers: Vec::new(),
            cursor_position: None,
            cursor_visible: false,
        }
    }

    /// Get the most recent buffer
    pub fn get_last_buffer(&self) -> Option<&Buffer> {
        self.buffers.last()
    }

    /// Get all buffer history
    pub fn get_buffer_history(&self) -> &[Buffer] {
        &self.buffers
    }

    /// Check if the last buffer contains specified text
    pub fn assert_contains_text(&self, text: &str) -> bool {
        if let Some(buffer) = self.get_last_buffer() {
            return self.buffer_contains_text(buffer, text);
        }
        false
    }

    /// Check if a specific buffer contains text
    pub fn buffer_contains_text(&self, buffer: &Buffer, text: &str) -> bool {
        // Convert buffer to string and search for text
        let full_text = self.buffer_to_string(buffer);
        full_text.contains(text)
    }

    /// Get text content at specific coordinates
    pub fn get_text_at(&self, x: u16, y: u16) -> Option<String> {
        if let Some(buffer) = self.get_last_buffer() {
            if let Some(cell) = buffer.cell((x, y)) {
                return Some(cell.symbol().to_string());
            }
        }
        None
    }

    /// Get the full text content of the last buffer
    pub fn get_buffer_text(&self) -> String {
        if let Some(buffer) = self.get_last_buffer() {
            return self.buffer_to_string(buffer);
        }
        String::new()
    }

    /// Convert buffer to string representation
    pub fn buffer_to_string(&self, buffer: &Buffer) -> String {
        let mut result = String::new();
        let area = buffer.area();

        for y in area.top()..area.bottom() {
            let mut line = String::new();
            for x in area.left()..area.right() {
                if let Some(cell) = buffer.cell((x, y)) {
                    line.push_str(cell.symbol());
                } else {
                    line.push(' ');
                }
            }
            // Remove trailing whitespace
            line = line.trim_end().to_string();
            result.push_str(&line);
            if y < area.bottom() - 1 {
                result.push('\n');
            }
        }

        result
    }

    /// Check if text appears at specific coordinates
    pub fn assert_text_at(&self, x: u16, y: u16, expected: &str) -> bool {
        if let Some(actual) = self.get_text_at(x, y) {
            return actual.contains(expected);
        }
        false
    }

    /// Check if text appears in a specific rectangular area
    pub fn assert_text_in_area(&self, area: Rect, expected: &str) -> bool {
        if let Some(buffer) = self.get_last_buffer() {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buffer.cell((x, y)) {
                        if cell.symbol().contains(expected) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Get style at specific coordinates
    pub fn get_style_at(&self, x: u16, y: u16) -> Option<Style> {
        if let Some(buffer) = self.get_last_buffer() {
            if let Some(cell) = buffer.cell((x, y)) {
                return Some(cell.style());
            }
        }
        None
    }

    /// Check if specific style is applied at coordinates
    pub fn assert_style_at(&self, x: u16, y: u16, expected_style: Style) -> bool {
        if let Some(style) = self.get_style_at(x, y) {
            return style == expected_style;
        }
        false
    }

    /// Clear buffer history
    pub fn clear_history(&mut self) {
        self.buffers.clear();
    }

    /// Reset backend to initial state
    pub fn reset(&mut self) {
        self.buffers.clear();
        self.cursor_position = None;
        self.cursor_visible = false;
    }
}

impl Backend for MockBackend {
    fn draw<'a, I>(&mut self, content: I) -> Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        let mut buffer = Buffer::empty(Rect::new(0, 0, self.width, self.height));

        for (x, y, cell) in content {
            if x < self.width && y < self.height {
                buffer[(x, y)] = cell.clone();
            }
        }

        self.buffers.push(buffer);
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        self.cursor_visible = false;
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        self.cursor_visible = true;
        Ok(())
    }

    fn get_cursor_position(&mut self) -> Result<Position> {
        let (x, y) = self.cursor_position.unwrap_or((0, 0));
        Ok(Position { x, y })
    }

    fn set_cursor_position<P>(&mut self, position: P) -> Result<()>
    where
        P: Into<Position>,
    {
        let pos = position.into();
        self.cursor_position = Some((pos.x, pos.y));
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        let empty_buffer = Buffer::empty(Rect::new(0, 0, self.width, self.height));
        self.buffers.push(empty_buffer);
        Ok(())
    }

    fn size(&self) -> Result<Size> {
        Ok(Size::new(self.width, self.height))
    }

    fn window_size(&mut self) -> Result<ratatui::backend::WindowSize> {
        Ok(ratatui::backend::WindowSize {
            columns_rows: Size::new(self.width, self.height),
            pixels: Size::new(self.width * 8, self.height * 16),
        })
    }

    fn flush(&mut self) -> Result<()> {
        // No-op for mock backend
        Ok(())
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new(80, 24) // Standard terminal size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::Terminal;

    #[test]
    fn test_mock_backend_creation() {
        let backend = MockBackend::new(120, 40);
        assert_eq!(backend.width, 120);
        assert_eq!(backend.height, 40);
        assert!(backend.buffers.is_empty());
    }

    #[test]
    fn test_mock_backend_default() {
        let backend = MockBackend::default();
        assert_eq!(backend.width, 80);
        assert_eq!(backend.height, 24);
    }

    #[test]
    fn test_buffer_text_extraction() {
        let backend = MockBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let paragraph =
                    Paragraph::new("Hello, World!").block(Block::default().borders(Borders::ALL));
                frame.render_widget(paragraph, frame.area());
            })
            .unwrap();

        let backend = terminal.backend();
        assert!(backend.assert_contains_text("Hello, World!"));
    }

    #[test]
    fn test_cursor_operations() {
        let mut backend = MockBackend::new(80, 24);

        // Test cursor visibility
        assert!(!backend.cursor_visible);
        backend.show_cursor().unwrap();
        assert!(backend.cursor_visible);
        backend.hide_cursor().unwrap();
        assert!(!backend.cursor_visible);

        // Test cursor position
        backend.set_cursor_position(Position::new(10, 5)).unwrap();
        let pos = backend.get_cursor_position().unwrap();
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn test_buffer_history() {
        let mut backend = MockBackend::new(80, 24);

        // Initial state
        assert_eq!(backend.get_buffer_history().len(), 0);

        // Draw something
        backend.clear().unwrap();
        assert_eq!(backend.get_buffer_history().len(), 1);

        // Clear history
        backend.clear_history();
        assert_eq!(backend.get_buffer_history().len(), 0);
    }

    #[test]
    fn test_text_assertions() {
        let backend = MockBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let paragraph = Paragraph::new("Test Content");
                frame.render_widget(paragraph, frame.area());
            })
            .unwrap();

        let backend = terminal.backend();
        assert!(backend.assert_contains_text("Test Content"));
        assert!(!backend.assert_contains_text("Not Present"));
    }
}
