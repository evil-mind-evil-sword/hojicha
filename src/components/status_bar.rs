//! Status bar component for persistent information display
//!
//! A customizable status bar that can display multiple segments with different styles.

use crate::style::{Color, ColorProfile, Style, TextAlign, Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Position of the status bar
#[derive(Clone, Debug, PartialEq)]
pub enum StatusBarPosition {
    /// Display at the top of the screen
    Top,
    /// Display at the bottom of the screen
    Bottom,
}

/// A segment in the status bar
#[derive(Clone)]
pub struct StatusSegment {
    /// Content of the segment
    content: String,
    /// Style for this segment
    style: Style,
    /// Width constraint for this segment
    constraint: Constraint,
    /// Text alignment within the segment
    alignment: TextAlign,
}

impl StatusSegment {
    /// Create a new status segment
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: Style::new(),
            constraint: Constraint::Min(0),
            alignment: TextAlign::Left,
        }
    }

    /// Set the content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Set the style
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the width constraint
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraint = constraint;
        self
    }

    /// Set the text alignment
    pub fn with_alignment(mut self, alignment: TextAlign) -> Self {
        self.alignment = alignment;
        self
    }

    /// Update the content
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    /// Get the content
    pub fn content(&self) -> &str {
        &self.content
    }
}

/// A status bar component with multiple segments
#[derive(Clone)]
pub struct StatusBar {
    /// Segments in the status bar
    segments: Vec<StatusSegment>,
    /// Position of the status bar
    position: StatusBarPosition,
    /// Container style
    container_style: Style,
    /// Default segment style
    default_style: Style,
    /// Separator between segments
    separator: Option<String>,
    /// Separator style
    separator_style: Style,
    /// Height of the status bar
    height: u16,
    /// Whether to show borders
    show_borders: bool,
}

impl StatusBar {
    /// Create a new status bar
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            position: StatusBarPosition::Bottom,
            container_style: Style::new().bg(Color::rgb(40, 40, 40)),
            default_style: Style::new().fg(Color::white()),
            separator: Some(" │ ".to_string()),
            separator_style: Style::new().fg(Color::gray()),
            height: 1,
            show_borders: false,
        }
    }

    /// Add a segment to the status bar
    pub fn add_segment(&mut self, segment: StatusSegment) -> &mut Self {
        self.segments.push(segment);
        self
    }

    /// Add a simple text segment with default styling
    pub fn add_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.segments.push(StatusSegment::new(text));
        self
    }

    /// Add a segment with specific constraint
    pub fn add_with_constraint(
        &mut self,
        text: impl Into<String>,
        constraint: Constraint,
    ) -> &mut Self {
        self.segments.push(
            StatusSegment::new(text)
                .with_constraint(constraint)
                .with_style(self.default_style.clone()),
        );
        self
    }

    /// Set the segments
    pub fn with_segments(mut self, segments: Vec<StatusSegment>) -> Self {
        self.segments = segments;
        self
    }

    /// Clear all segments
    pub fn clear(&mut self) {
        self.segments.clear();
    }

    /// Update a segment by index
    pub fn update_segment(&mut self, index: usize, content: impl Into<String>) {
        if let Some(segment) = self.segments.get_mut(index) {
            segment.set_content(content);
        }
    }

    /// Set the position
    pub fn with_position(mut self, position: StatusBarPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the container style
    pub fn with_container_style(mut self, style: Style) -> Self {
        self.container_style = style;
        self
    }

    /// Set the default segment style
    pub fn with_default_style(mut self, style: Style) -> Self {
        self.default_style = style;
        self
    }

    /// Set the separator
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = Some(separator.into());
        self
    }

    /// Remove the separator
    pub fn without_separator(mut self) -> Self {
        self.separator = None;
        self
    }

    /// Set the separator style
    pub fn with_separator_style(mut self, style: Style) -> Self {
        self.separator_style = style;
        self
    }

    /// Set the height
    pub fn with_height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    /// Set whether to show borders
    pub fn with_borders(mut self, show: bool) -> Self {
        self.show_borders = show;
        self
    }

    /// Get the position
    pub fn position(&self) -> &StatusBarPosition {
        &self.position
    }

    /// Get the height
    pub fn height(&self) -> u16 {
        if self.show_borders {
            self.height + 2
        } else {
            self.height
        }
    }

    /// Apply a theme
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.container_style = Style::new()
            .bg(theme.colors.surface.clone())
            .fg(theme.colors.text.clone());

        self.default_style = Style::new().fg(theme.colors.text_secondary.clone());

        self.separator_style = Style::new().fg(theme.colors.border.clone());

        if let Some(style) = theme.get_style("statusbar.container") {
            self.container_style = style.clone();
        }
    }

    /// Render the status bar
    pub fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        if self.segments.is_empty() {
            return;
        }

        // Create the layout for segments
        let constraints: Vec<Constraint> = self.segments.iter().map(|s| s.constraint).collect();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // Render background
        let background = Block::default().style(self.container_style.to_ratatui(profile));

        if self.show_borders {
            let bordered = background.borders(Borders::ALL);
            frame.render_widget(bordered, area);
        } else {
            frame.render_widget(background, area);
        }

        // Render each segment
        for (i, (segment, chunk)) in self.segments.iter().zip(chunks.iter()).enumerate() {
            // Render separator before segment (except first)
            if i > 0 && self.separator.is_some() {
                let sep_text = self.separator.as_ref().unwrap();
                let sep_width = sep_text.len() as u16;

                if chunk.width > sep_width {
                    let sep_area = Rect {
                        x: chunk.x,
                        y: chunk.y,
                        width: sep_width,
                        height: chunk.height,
                    };

                    let separator = Paragraph::new(sep_text.as_str())
                        .style(self.separator_style.to_ratatui(profile));
                    frame.render_widget(separator, sep_area);

                    // Adjust chunk for content
                    let content_area = Rect {
                        x: chunk.x + sep_width,
                        y: chunk.y,
                        width: chunk.width - sep_width,
                        height: chunk.height,
                    };

                    self.render_segment(frame, segment, content_area, profile);
                } else {
                    self.render_segment(frame, segment, *chunk, profile);
                }
            } else {
                self.render_segment(frame, segment, *chunk, profile);
            }
        }
    }

    /// Render a single segment
    fn render_segment(
        &self,
        frame: &mut Frame,
        segment: &StatusSegment,
        area: Rect,
        profile: &ColorProfile,
    ) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        let alignment = segment.alignment.to_ratatui();

        let paragraph = Paragraph::new(segment.content.as_str())
            .style(segment.style.to_ratatui(profile))
            .alignment(alignment);

        frame.render_widget(paragraph, area);
    }

    /// Create a layout that includes the status bar
    pub fn layout(&self, area: Rect) -> (Rect, Rect) {
        let height = self.height();

        match self.position {
            StatusBarPosition::Top => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(height), Constraint::Min(0)])
                    .split(area);
                (chunks[0], chunks[1]) // (status_bar, main_content)
            }
            StatusBarPosition::Bottom => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(height)])
                    .split(area);
                (chunks[1], chunks[0]) // (status_bar, main_content)
            }
        }
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating status bars with common patterns
pub struct StatusBarBuilder {
    status_bar: StatusBar,
}

impl StatusBarBuilder {
    /// Create a new status bar builder
    pub fn new() -> Self {
        Self {
            status_bar: StatusBar::new(),
        }
    }

    /// Add a left-aligned segment
    pub fn left(mut self, content: impl Into<String>) -> Self {
        self.status_bar.add_segment(
            StatusSegment::new(content)
                .with_alignment(TextAlign::Left)
                .with_constraint(Constraint::Min(0)),
        );
        self
    }

    /// Add a center-aligned segment
    pub fn center(mut self, content: impl Into<String>) -> Self {
        self.status_bar.add_segment(
            StatusSegment::new(content)
                .with_alignment(TextAlign::Center)
                .with_constraint(Constraint::Min(0)),
        );
        self
    }

    /// Add a right-aligned segment
    pub fn right(mut self, content: impl Into<String>) -> Self {
        self.status_bar.add_segment(
            StatusSegment::new(content)
                .with_alignment(TextAlign::Right)
                .with_constraint(Constraint::Min(0)),
        );
        self
    }

    /// Build the status bar
    pub fn build(self) -> StatusBar {
        self.status_bar
    }
}

impl Default for StatusBarBuilder {
    fn default() -> Self {
        Self::new()
    }
}
