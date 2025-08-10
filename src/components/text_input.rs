//! Styled text input component
//!
//! A text input field with validation, placeholder text, and theming support.

use crate::style::{Style, Theme, ColorProfile};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style as RatatuiStyle},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::cmp;

/// Text input validation function
pub type Validator = Box<dyn Fn(&str) -> ValidationResult>;

/// Result of input validation
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Input is valid
    Valid,
    /// Input is invalid with an error message
    Invalid(String),
}

/// A styled text input component
pub struct TextInput {
    /// The current text value
    value: String,
    /// Placeholder text when empty
    placeholder: String,
    /// Cursor position
    cursor_position: usize,
    /// Whether the input is focused
    focused: bool,
    /// Validation function
    validator: Option<Validator>,
    /// Last validation result
    validation_result: ValidationResult,
    /// Style for the input
    style: Style,
    /// Style when focused
    focused_style: Style,
    /// Style for placeholder text
    placeholder_style: Style,
    /// Style for error state
    error_style: Style,
    /// Maximum length allowed
    max_length: Option<usize>,
}

impl TextInput {
    /// Create a new text input
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: String::new(),
            cursor_position: 0,
            focused: false,
            validator: None,
            validation_result: ValidationResult::Valid,
            style: Style::new()
                .border(crate::style::BorderStyle::Normal)
                .padding_symmetric(0, 1),
            focused_style: Style::new()
                .border(crate::style::BorderStyle::Normal)
                .border_color(crate::style::Color::cyan())
                .padding_symmetric(0, 1),
            placeholder_style: Style::new()
                .fg(crate::style::Color::gray())
                .dim(),
            error_style: Style::new()
                .border(crate::style::BorderStyle::Normal)
                .border_color(crate::style::Color::red()),
            max_length: None,
        }
    }

    /// Set the current value
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor_position = self.value.len();
        self.validate();
        self
    }

    /// Set the placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the validation function
    pub fn with_validation<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> ValidationResult + 'static,
    {
        self.validator = Some(Box::new(validator));
        self.validate();
        self
    }

    /// Set a simple required validation
    pub fn required(self) -> Self {
        self.with_validation(|value| {
            if value.is_empty() {
                ValidationResult::Invalid("This field is required".to_string())
            } else {
                ValidationResult::Valid
            }
        })
    }

    /// Set maximum length
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Set focused state
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// Remove focus
    pub fn blur(&mut self) {
        self.focused = false;
    }

    /// Check if focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Get the current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if input is valid
    pub fn is_valid(&self) -> bool {
        matches!(self.validation_result, ValidationResult::Valid)
    }

    /// Get validation error message if any
    pub fn error_message(&self) -> Option<&str> {
        match &self.validation_result {
            ValidationResult::Invalid(msg) => Some(msg.as_str()),
            ValidationResult::Valid => None,
        }
    }

    /// Handle character input
    pub fn insert_char(&mut self, c: char) {
        if let Some(max) = self.max_length {
            if self.value.len() >= max {
                return;
            }
        }

        self.value.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.validate();
    }

    /// Handle backspace
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.value.remove(self.cursor_position);
            self.validate();
        }
    }

    /// Handle delete key
    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.value.len() {
            self.value.remove(self.cursor_position);
            self.validate();
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        self.cursor_position = cmp::min(self.cursor_position + 1, self.value.len());
    }

    /// Move cursor to start
    pub fn move_cursor_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.value.len();
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
        self.validate();
    }

    /// Validate the current value
    fn validate(&mut self) {
        self.validation_result = if let Some(ref validator) = self.validator {
            validator(&self.value)
        } else {
            ValidationResult::Valid
        };
    }

    /// Set the base style
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the focused style
    pub fn with_focused_style(mut self, style: Style) -> Self {
        self.focused_style = style;
        self
    }

    /// Set the placeholder style
    pub fn with_placeholder_style(mut self, style: Style) -> Self {
        self.placeholder_style = style;
        self
    }

    /// Set the error style
    pub fn with_error_style(mut self, style: Style) -> Self {
        self.error_style = style;
        self
    }

    /// Render the text input
    pub fn render(&self, frame: &mut ratatui::Frame, area: Rect, profile: &ColorProfile) {
        // Determine which style to use
        let style = if !self.is_valid() {
            &self.error_style
        } else if self.focused {
            &self.focused_style
        } else {
            &self.style
        };

        // Create the block with border
        let mut block = Block::default();
        
        if style.get_border() != &crate::style::BorderStyle::None {
            block = block
                .borders(Borders::ALL)
                .border_type(style.get_border().to_ratatui());
            
            if let Some(border_color) = style.get_border_color() {
                block = block.border_style(
                    RatatuiStyle::default()
                        .fg(border_color.to_ratatui(profile))
                );
            }
        }

        // Render the block
        frame.render_widget(&block, area);

        // Calculate inner area for text
        let inner = block.inner(area);
        let padding = style.get_padding();
        let text_area = Rect {
            x: inner.x + padding.left,
            y: inner.y + padding.top,
            width: inner.width.saturating_sub(padding.left + padding.right),
            height: inner.height.saturating_sub(padding.top + padding.bottom),
        };

        // Prepare the text to display
        let display_text = if self.value.is_empty() && !self.focused {
            // Show placeholder
            vec![Span::styled(
                &self.placeholder,
                self.placeholder_style.to_ratatui(profile),
            )]
        } else {
            // Show value with cursor
            let mut spans = Vec::new();
            
            if self.focused {
                // Add text before cursor
                if self.cursor_position > 0 {
                    spans.push(Span::styled(
                        &self.value[..self.cursor_position],
                        style.to_ratatui(profile),
                    ));
                }
                
                // Add cursor
                if self.cursor_position < self.value.len() {
                    spans.push(Span::styled(
                        &self.value[self.cursor_position..=self.cursor_position],
                        style.to_ratatui(profile).add_modifier(Modifier::REVERSED),
                    ));
                    
                    // Add text after cursor
                    if self.cursor_position + 1 < self.value.len() {
                        spans.push(Span::styled(
                            &self.value[self.cursor_position + 1..],
                            style.to_ratatui(profile),
                        ));
                    }
                } else {
                    // Cursor at end
                    spans.push(Span::styled(
                        " ",
                        style.to_ratatui(profile).add_modifier(Modifier::REVERSED),
                    ));
                }
            } else {
                // Not focused, just show the value
                spans.push(Span::styled(&self.value, style.to_ratatui(profile)));
            }
            
            spans
        };

        let paragraph = Paragraph::new(Line::from(display_text));
        frame.render_widget(paragraph, text_area);

        // Render error message if any
        if let Some(error_msg) = self.error_message() {
            let error_y = area.y + area.height;
            if error_y < frame.area().height {
                let error_area = Rect {
                    x: area.x,
                    y: error_y,
                    width: area.width,
                    height: 1,
                };
                
                let error_text = Paragraph::new(Line::from(vec![
                    Span::styled(error_msg, self.error_style.to_ratatui(profile)),
                ]));
                
                frame.render_widget(error_text, error_area);
            }
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    /// Apply a theme to this text input
    pub fn apply_theme(&mut self, theme: &Theme) {
        if let Some(style) = theme.get_style("input") {
            self.style = style.clone();
        }
        
        if let Some(style) = theme.get_style("input.focused") {
            self.focused_style = style.clone();
        }
        
        if let Some(style) = theme.get_style("input.placeholder") {
            self.placeholder_style = style.clone();
        }
        
        if let Some(style) = theme.get_style("input.error") {
            self.error_style = style.clone();
        }
    }
}