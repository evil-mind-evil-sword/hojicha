//! Paginator component for navigating through pages of content
//!
//! Supports both dot-style and numeric pagination displays.

use crate::style::{Color, ColorProfile, Style, Theme};
use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Style of pagination display
#[derive(Clone, Debug, PartialEq)]
pub enum PaginatorStyle {
    /// Show dots for each page (• • ● • •)
    Dots,
    /// Show page numbers (1 2 [3] 4 5)
    Numeric,
    /// Show as "Page X of Y"
    Text,
    /// Show as a progress bar
    ProgressBar,
}

/// A paginator component for navigating through pages
#[derive(Clone)]
pub struct Paginator {
    /// Current page (0-indexed)
    current_page: usize,
    /// Total number of pages
    total_pages: usize,
    /// Display style
    style: PaginatorStyle,
    /// Maximum visible page indicators (for Dots and Numeric styles)
    max_visible: usize,
    /// Style for active page
    active_style: Style,
    /// Style for inactive pages
    inactive_style: Style,
    /// Style for navigation hints
    nav_style: Style,
    /// Container style
    container_style: Style,
    /// Show navigation arrows
    show_arrows: bool,
    /// Custom active indicator for dots style
    active_dot: String,
    /// Custom inactive indicator for dots style
    inactive_dot: String,
    /// Show first/last page shortcuts
    show_shortcuts: bool,
}

impl Paginator {
    /// Create a new paginator
    pub fn new(total_pages: usize) -> Self {
        Self {
            current_page: 0,
            total_pages,
            style: PaginatorStyle::Dots,
            max_visible: 7,
            active_style: Style::new().bold().fg(Color::cyan()),
            inactive_style: Style::new().fg(Color::gray()),
            nav_style: Style::new().fg(Color::gray()),
            container_style: Style::new(),
            show_arrows: true,
            active_dot: "●".to_string(),
            inactive_dot: "○".to_string(),
            show_shortcuts: false,
        }
    }

    /// Set the current page (0-indexed)
    pub fn set_page(&mut self, page: usize) {
        self.current_page = page.min(self.total_pages.saturating_sub(1));
    }

    /// Get the current page (0-indexed)
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// Get total pages
    pub fn total_pages(&self) -> usize {
        self.total_pages
    }

    /// Set total pages
    pub fn set_total_pages(&mut self, total: usize) {
        self.total_pages = total;
        if self.current_page >= total {
            self.current_page = total.saturating_sub(1);
        }
    }

    /// Go to the next page
    pub fn next_page(&mut self) {
        if self.current_page < self.total_pages.saturating_sub(1) {
            self.current_page += 1;
        }
    }

    /// Go to the previous page
    pub fn previous_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }

    /// Go to the first page
    pub fn first_page(&mut self) {
        self.current_page = 0;
    }

    /// Go to the last page
    pub fn last_page(&mut self) {
        self.current_page = self.total_pages.saturating_sub(1);
    }

    /// Check if on the first page
    pub fn is_first_page(&self) -> bool {
        self.current_page == 0
    }

    /// Check if on the last page
    pub fn is_last_page(&self) -> bool {
        self.current_page >= self.total_pages.saturating_sub(1)
    }

    /// Set the display style
    pub fn with_style(mut self, style: PaginatorStyle) -> Self {
        self.style = style;
        self
    }

    /// Set maximum visible indicators
    pub fn with_max_visible(mut self, max: usize) -> Self {
        self.max_visible = max.max(3); // Minimum of 3 for proper display
        self
    }

    /// Set active page style
    pub fn with_active_style(mut self, style: Style) -> Self {
        self.active_style = style;
        self
    }

    /// Set inactive page style
    pub fn with_inactive_style(mut self, style: Style) -> Self {
        self.inactive_style = style;
        self
    }

    /// Set navigation style
    pub fn with_nav_style(mut self, style: Style) -> Self {
        self.nav_style = style;
        self
    }

    /// Set container style
    pub fn with_container_style(mut self, style: Style) -> Self {
        self.container_style = style;
        self
    }

    /// Set whether to show navigation arrows
    pub fn with_arrows(mut self, show: bool) -> Self {
        self.show_arrows = show;
        self
    }

    /// Set custom dot characters
    pub fn with_dots(mut self, active: impl Into<String>, inactive: impl Into<String>) -> Self {
        self.active_dot = active.into();
        self.inactive_dot = inactive.into();
        self
    }

    /// Set whether to show first/last shortcuts
    pub fn with_shortcuts(mut self, show: bool) -> Self {
        self.show_shortcuts = show;
        self
    }

    /// Apply a theme
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.active_style = Style::new()
            .bold()
            .fg(theme.colors.primary.clone());
        
        self.inactive_style = Style::new()
            .fg(theme.colors.text_secondary.clone());
        
        self.nav_style = Style::new()
            .fg(theme.colors.border.clone());

        if let Some(style) = theme.get_style("paginator.container") {
            self.container_style = style.clone();
        }
    }

    /// Render the paginator
    pub fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        if self.total_pages <= 1 {
            return; // No pagination needed
        }

        let content = match self.style {
            PaginatorStyle::Dots => self.render_dots(profile),
            PaginatorStyle::Numeric => self.render_numeric(profile),
            PaginatorStyle::Text => self.render_text(profile),
            PaginatorStyle::ProgressBar => self.render_progress(profile),
        };

        let mut block = Block::default();
        if self.container_style.get_border() != &crate::style::BorderStyle::None {
            block = block
                .borders(Borders::ALL)
                .border_style(self.container_style.to_ratatui(profile));
        }

        let paragraph = Paragraph::new(content)
            .block(block)
            .alignment(Alignment::Center)
            .style(self.container_style.to_ratatui(profile));

        frame.render_widget(paragraph, area);
    }

    /// Render dots style
    fn render_dots(&self, profile: &ColorProfile) -> Line {
        let mut spans = Vec::new();

        // Add left arrow if enabled
        if self.show_arrows {
            let arrow = if self.is_first_page() { " " } else { "◀ " };
            spans.push(Span::styled(arrow, self.nav_style.to_ratatui(profile)));
        }

        // Calculate visible range
        let (start, end) = self.calculate_visible_range();

        // Add first page shortcut if needed
        if self.show_shortcuts && start > 0 {
            spans.push(Span::styled(
                &self.inactive_dot,
                self.inactive_style.to_ratatui(profile),
            ));
            spans.push(Span::raw(" "));
            if start > 1 {
                spans.push(Span::styled("… ", self.nav_style.to_ratatui(profile)));
            }
        }

        // Add page dots
        for i in start..=end {
            if i > start {
                spans.push(Span::raw(" "));
            }

            let (dot, style) = if i == self.current_page {
                (&self.active_dot, self.active_style.to_ratatui(profile))
            } else {
                (&self.inactive_dot, self.inactive_style.to_ratatui(profile))
            };

            spans.push(Span::styled(dot, style));
        }

        // Add last page shortcut if needed
        if self.show_shortcuts && end < self.total_pages - 1 {
            if end < self.total_pages - 2 {
                spans.push(Span::styled(" …", self.nav_style.to_ratatui(profile)));
            }
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                &self.inactive_dot,
                self.inactive_style.to_ratatui(profile),
            ));
        }

        // Add right arrow if enabled
        if self.show_arrows {
            let arrow = if self.is_last_page() { " " } else { " ▶" };
            spans.push(Span::styled(arrow, self.nav_style.to_ratatui(profile)));
        }

        Line::from(spans)
    }

    /// Render numeric style
    fn render_numeric(&self, profile: &ColorProfile) -> Line {
        let mut spans = Vec::new();

        // Add left arrow if enabled
        if self.show_arrows {
            let arrow = if self.is_first_page() { "  " } else { "◀ " };
            spans.push(Span::styled(arrow, self.nav_style.to_ratatui(profile)));
        }

        // Calculate visible range
        let (start, end) = self.calculate_visible_range();

        // Add first page shortcut if needed
        if self.show_shortcuts && start > 0 {
            spans.push(Span::styled("1", self.inactive_style.to_ratatui(profile)));
            spans.push(Span::raw(" "));
            if start > 1 {
                spans.push(Span::styled("… ", self.nav_style.to_ratatui(profile)));
            }
        }

        // Add page numbers
        for i in start..=end {
            if i > start || (self.show_shortcuts && start > 0) {
                spans.push(Span::raw(" "));
            }

            if i == self.current_page {
                // Add brackets around current page
                spans.push(Span::styled("[", self.active_style.to_ratatui(profile)));
                spans.push(Span::styled(format!("{}", i + 1), self.active_style.to_ratatui(profile)));
                spans.push(Span::styled("]", self.active_style.to_ratatui(profile)));
            } else {
                spans.push(Span::styled(
                    format!("{}", i + 1),
                    self.inactive_style.to_ratatui(profile),
                ));
            }
        }

        // Add last page shortcut if needed
        if self.show_shortcuts && end < self.total_pages - 1 {
            if end < self.total_pages - 2 {
                spans.push(Span::styled(" …", self.nav_style.to_ratatui(profile)));
            }
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("{}", self.total_pages),
                self.inactive_style.to_ratatui(profile),
            ));
        }

        // Add right arrow if enabled
        if self.show_arrows {
            let arrow = if self.is_last_page() { "  " } else { " ▶" };
            spans.push(Span::styled(arrow, self.nav_style.to_ratatui(profile)));
        }

        Line::from(spans)
    }

    /// Render text style
    fn render_text(&self, profile: &ColorProfile) -> Line {
        let mut spans = Vec::new();

        if self.show_arrows && !self.is_first_page() {
            spans.push(Span::styled("◀ ", self.nav_style.to_ratatui(profile)));
        }

        spans.push(Span::styled("Page ", self.inactive_style.to_ratatui(profile)));
        spans.push(Span::styled(
            format!("{}", self.current_page + 1),
            self.active_style.to_ratatui(profile),
        ));
        spans.push(Span::styled(" of ", self.inactive_style.to_ratatui(profile)));
        spans.push(Span::styled(
            format!("{}", self.total_pages),
            self.inactive_style.to_ratatui(profile),
        ));

        if self.show_arrows && !self.is_last_page() {
            spans.push(Span::styled(" ▶", self.nav_style.to_ratatui(profile)));
        }

        Line::from(spans)
    }

    /// Render progress bar style
    fn render_progress(&self, profile: &ColorProfile) -> Line {
        let width = 20; // Fixed width for progress bar
        let filled = ((self.current_page as f32 + 1.0) / self.total_pages as f32 * width as f32) as usize;
        
        let mut spans = Vec::new();

        spans.push(Span::styled("[", self.nav_style.to_ratatui(profile)));
        
        // Filled portion
        if filled > 0 {
            spans.push(Span::styled(
                "█".repeat(filled),
                self.active_style.to_ratatui(profile),
            ));
        }
        
        // Empty portion
        if filled < width {
            spans.push(Span::styled(
                "░".repeat(width - filled),
                self.inactive_style.to_ratatui(profile),
            ));
        }
        
        spans.push(Span::styled("]", self.nav_style.to_ratatui(profile)));
        
        // Add percentage
        let percentage = ((self.current_page as f32 + 1.0) / self.total_pages as f32 * 100.0) as u8;
        spans.push(Span::styled(
            format!(" {}%", percentage),
            self.inactive_style.to_ratatui(profile),
        ));

        Line::from(spans)
    }

    /// Calculate the visible page range
    fn calculate_visible_range(&self) -> (usize, usize) {
        if self.total_pages <= self.max_visible {
            return (0, self.total_pages - 1);
        }

        let half = self.max_visible / 2;
        
        if self.current_page <= half {
            (0, self.max_visible - 1)
        } else if self.current_page >= self.total_pages - half - 1 {
            (self.total_pages - self.max_visible, self.total_pages - 1)
        } else {
            (self.current_page - half, self.current_page + half)
        }
    }
}

impl Default for Paginator {
    fn default() -> Self {
        Self::new(1)
    }
}