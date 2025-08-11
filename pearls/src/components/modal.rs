//! Modal/Dialog component with theme support
//!
//! Overlay dialogs for confirmations, forms, and information display.

use hojicha_core::event::{Event, Key, KeyEvent};
use crate::style::{BorderStyle, Color, ColorProfile, Style, Theme};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Modal size preset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalSize {
    /// Small modal (40% width, 30% height)
    Small,
    /// Medium modal (60% width, 50% height)
    Medium,
    /// Large modal (80% width, 70% height)
    Large,
    /// Full screen modal (95% width, 95% height)
    FullScreen,
    /// Custom modal size with width and height percentages
    Custom(u16, u16),
}

/// Modal component
#[derive(Clone)]
pub struct Modal {
    /// Modal title
    title: Option<String>,
    /// Modal content (body text)
    content: String,
    /// Modal size
    size: ModalSize,
    /// Whether the modal is open
    open: bool,
    /// Whether to show close button hint
    show_close_hint: bool,
    /// Whether the modal can be closed with Escape
    closeable: bool,
    /// Custom overlay style
    overlay_style: Style,
    /// Custom modal style
    modal_style: Style,
    /// Custom title style
    title_style: Style,
    /// Custom content style
    content_style: Style,
    /// Footer content (optional)
    footer: Option<String>,
    /// Footer style
    footer_style: Style,
}

impl Modal {
    /// Create a new modal with content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            title: None,
            content: content.into(),
            size: ModalSize::Medium,
            open: false,
            show_close_hint: true,
            closeable: true,
            overlay_style: Style::new().bg(Color::Fixed(ratatui::style::Color::Black)),
            modal_style: Style::new().border(BorderStyle::Rounded),
            title_style: Style::new().bold(),
            content_style: Style::new(),
            footer: None,
            footer_style: Style::new().italic(),
        }
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the footer
    pub fn with_footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Set the size
    pub fn with_size(mut self, size: ModalSize) -> Self {
        self.size = size;
        self
    }

    /// Set whether the modal can be closed with Escape
    pub fn with_closeable(mut self, closeable: bool) -> Self {
        self.closeable = closeable;
        self.show_close_hint = closeable;
        self
    }

    /// Set whether to show close hint
    pub fn with_close_hint(mut self, show: bool) -> Self {
        self.show_close_hint = show && self.closeable;
        self
    }

    /// Set custom overlay style
    pub fn with_overlay_style(mut self, style: Style) -> Self {
        self.overlay_style = style;
        self
    }

    /// Set custom modal style
    pub fn with_modal_style(mut self, style: Style) -> Self {
        self.modal_style = style;
        self
    }

    /// Set custom title style
    pub fn with_title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Set custom content style
    pub fn with_content_style(mut self, style: Style) -> Self {
        self.content_style = style;
        self
    }

    /// Set custom footer style
    pub fn with_footer_style(mut self, style: Style) -> Self {
        self.footer_style = style;
        self
    }

    /// Open the modal
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Close the modal
    pub fn close(&mut self) {
        if self.closeable {
            self.open = false;
        }
    }

    /// Check if the modal is open
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Toggle the modal open/closed state
    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Handle keyboard events
    /// Returns true if the modal was closed
    pub fn handle_event(&mut self, event: Event<()>) -> bool {
        if !self.open {
            return false;
        }

        match event {
            Event::Key(KeyEvent { key: Key::Esc, .. }) if self.closeable => {
                self.close();
                true
            }
            _ => false,
        }
    }

    /// Calculate modal area based on size
    fn calculate_modal_area(&self, area: Rect) -> Rect {
        let (width_percent, height_percent) = match self.size {
            ModalSize::Small => (40, 30),
            ModalSize::Medium => (60, 50),
            ModalSize::Large => (80, 70),
            ModalSize::FullScreen => (95, 95),
            ModalSize::Custom(w, h) => (w.min(100), h.min(100)),
        };

        let width = (area.width * width_percent) / 100;
        let height = (area.height * height_percent) / 100;
        let x = area.x + (area.width - width) / 2;
        let y = area.y + (area.height - height) / 2;

        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Apply a theme to this modal
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.modal_style = Style::new()
            .bg(theme.colors.surface.clone())
            .border(BorderStyle::Rounded)
            .border_color(theme.colors.border.clone());

        self.title_style = Style::new().fg(theme.colors.text.clone()).bold();

        self.content_style = Style::new().fg(theme.colors.text.clone());

        self.footer_style = Style::new()
            .fg(theme.colors.text_secondary.clone())
            .italic();
    }

    /// Render the modal
    pub fn render(&self, frame: &mut Frame, area: Rect, _theme: &Theme, profile: &ColorProfile) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        if !self.open {
            return;
        }

        // Draw overlay (semi-transparent background)
        let overlay_block = Block::default().style(self.overlay_style.to_ratatui(profile));
        frame.render_widget(Clear, area);
        frame.render_widget(overlay_block, area);

        // Calculate modal area
        let modal_area = self.calculate_modal_area(area);

        // Create modal block with border
        let mut modal_block = Block::default().style(self.modal_style.to_ratatui(profile));

        if self.modal_style.get_border() != &BorderStyle::None {
            modal_block = modal_block
                .borders(Borders::ALL)
                .border_type(self.modal_style.get_border().to_ratatui());

            if let Some(border_color) = self.modal_style.get_border_color() {
                modal_block = modal_block.border_style(
                    ratatui::style::Style::default().fg(border_color.to_ratatui(profile)),
                );
            }
        }

        // Add title if present
        if let Some(ref title) = self.title {
            let title_text = if self.show_close_hint && self.closeable {
                format!("{} (ESC to close)", title)
            } else {
                title.clone()
            };
            modal_block = modal_block
                .title(title_text)
                .title_style(self.title_style.to_ratatui(profile))
                .title_alignment(Alignment::Center);
        } else if self.show_close_hint && self.closeable {
            modal_block = modal_block
                .title("(ESC to close)")
                .title_style(self.footer_style.to_ratatui(profile))
                .title_alignment(Alignment::Right);
        }

        // Clear and render modal background
        frame.render_widget(Clear, modal_area);
        frame.render_widget(modal_block.clone(), modal_area);

        // Calculate inner area for content
        let inner_area = modal_block.inner(modal_area);

        // Split inner area if we have a footer
        let areas = if self.footer.is_some() {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(2)])
                .split(inner_area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)])
                .split(inner_area)
        };

        // Render content
        let content_paragraph = Paragraph::new(self.content.as_str())
            .style(self.content_style.to_ratatui(profile))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);
        frame.render_widget(content_paragraph, areas[0]);

        // Render footer if present
        if let Some(ref footer) = self.footer {
            let footer_paragraph = Paragraph::new(footer.as_str())
                .style(self.footer_style.to_ratatui(profile))
                .alignment(Alignment::Center);
            frame.render_widget(footer_paragraph, areas[1]);
        }
    }
}

/// Confirmation dialog preset
impl Modal {
    /// Create a confirmation dialog
    pub fn confirm(message: impl Into<String>) -> Self {
        Self::new(message)
            .with_title("Confirm")
            .with_footer("Press Enter to confirm, ESC to cancel")
            .with_size(ModalSize::Small)
    }

    /// Create an error dialog
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message)
            .with_title("Error")
            .with_size(ModalSize::Small)
    }

    /// Create an info dialog
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message)
            .with_title("Information")
            .with_size(ModalSize::Medium)
    }

    /// Create a warning dialog
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message)
            .with_title("Warning")
            .with_size(ModalSize::Small)
    }
}
