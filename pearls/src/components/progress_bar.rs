//! Progress bar component with theme support
//!
//! Visual progress indicators with various styles.

use crate::style::{Color, ColorProfile, Style, Theme};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Gauge, LineGauge, Sparkline},
    Frame,
};

/// Progress bar style variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStyle {
    /// Standard filled bar
    Bar,
    /// Line-based progress
    Line,
    /// Sparkline graph
    Spark,
    /// Custom characters for filled/empty
    Custom {
        /// Character to use for filled portion
        filled: char,
        /// Character to use for empty portion
        empty: char,
    },
}

/// Progress bar component
#[derive(Clone)]
pub struct ProgressBar {
    /// Current progress value (0.0 to 1.0)
    value: f64,
    /// Progress history for sparkline
    history: Vec<u64>,
    /// Maximum history length
    max_history: usize,
    /// Progress bar style
    style_variant: ProgressStyle,
    /// Label to display
    label: Option<String>,
    /// Show percentage
    show_percentage: bool,
    /// Show fraction (e.g., "3/10")
    show_fraction: bool,
    /// Total items (for fraction display)
    total: Option<usize>,
    /// Current item (for fraction display)
    current: Option<usize>,
    /// Bar style
    bar_style: Style,
    /// Background style
    background_style: Style,
    /// Label style
    label_style: Style,
    /// Container style
    container_style: Style,
    /// Use gradient colors
    use_gradient: bool,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new() -> Self {
        Self {
            value: 0.0,
            history: Vec::new(),
            max_history: 100,
            style_variant: ProgressStyle::Bar,
            label: None,
            show_percentage: true,
            show_fraction: false,
            total: None,
            current: None,
            bar_style: Style::new().fg(Color::green()),
            background_style: Style::new().fg(Color::gray()),
            label_style: Style::new().bold(),
            container_style: Style::new(),
            use_gradient: false,
        }
    }

    /// Set the progress value (0.0 to 1.0)
    pub fn set_progress(&mut self, value: f64) {
        self.value = value.clamp(0.0, 1.0);

        // Add to history for sparkline
        let history_value = (self.value * 100.0) as u64;
        self.history.push(history_value);

        // Trim history if needed
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Set progress as fraction
    pub fn set_fraction(&mut self, current: usize, total: usize) {
        self.current = Some(current);
        self.total = Some(total);
        if total > 0 {
            self.value = current as f64 / total as f64;
        } else {
            self.value = 0.0;
        }

        // Update history
        let history_value = (self.value * 100.0) as u64;
        self.history.push(history_value);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Increment progress
    pub fn increment(&mut self, amount: f64) {
        self.set_progress(self.value + amount);
    }

    /// Set the style variant
    pub fn with_style_variant(mut self, variant: ProgressStyle) -> Self {
        self.style_variant = variant;
        self
    }

    /// Set the label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set whether to show percentage
    pub fn with_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set whether to show fraction
    pub fn with_fraction(mut self, show: bool) -> Self {
        self.show_fraction = show;
        self
    }

    /// Set bar style
    pub fn with_bar_style(mut self, style: Style) -> Self {
        self.bar_style = style;
        self
    }

    /// Set background style
    pub fn with_background_style(mut self, style: Style) -> Self {
        self.background_style = style;
        self
    }

    /// Set container style
    pub fn with_container_style(mut self, style: Style) -> Self {
        self.container_style = style;
        self
    }

    /// Enable gradient colors
    pub fn with_gradient(mut self, enable: bool) -> Self {
        self.use_gradient = enable;
        self
    }

    /// Set maximum history length for sparkline
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Get the current progress value
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Check if progress is complete
    pub fn is_complete(&self) -> bool {
        self.value >= 1.0
    }

    /// Reset the progress bar
    pub fn reset(&mut self) {
        self.value = 0.0;
        self.current = None;
        self.history.clear();
    }

    /// Get dynamic color based on progress
    fn get_progress_color(&self, theme: &Theme) -> Color {
        if !self.use_gradient {
            return self
                .bar_style
                .get_foreground()
                .cloned()
                .unwrap_or_else(|| theme.colors.primary.clone());
        }

        // Gradient from red -> yellow -> green based on progress
        if self.value < 0.33 {
            theme.colors.error.clone()
        } else if self.value < 0.66 {
            theme.colors.warning.clone()
        } else {
            theme.colors.success.clone()
        }
    }

    /// Build the label text
    fn build_label(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref label) = self.label {
            parts.push(label.clone());
        }

        if self.show_fraction && self.current.is_some() && self.total.is_some() {
            parts.push(format!("{}/{}", self.current.unwrap(), self.total.unwrap()));
        }

        if self.show_percentage {
            parts.push(format!("{:.0}%", self.value * 100.0));
        }

        parts.join(" ")
    }

    /// Apply a theme to this progress bar
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.bar_style = Style::new().fg(theme.colors.primary.clone());

        self.background_style = Style::new().fg(theme.colors.surface.clone());

        self.label_style = Style::new().fg(theme.colors.text.clone()).bold();

        if self.use_gradient {
            // Gradient will be applied dynamically based on value
        }
    }

    /// Render the progress bar
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, profile: &ColorProfile) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        let label_text = self.build_label();
        let progress_color = self.get_progress_color(theme);

        match self.style_variant {
            ProgressStyle::Bar => {
                let gauge = Gauge::default()
                    .label(label_text)
                    .percent((self.value * 100.0) as u16)
                    .gauge_style(Style::new().fg(progress_color).to_ratatui(profile))
                    .style(self.background_style.to_ratatui(profile));

                frame.render_widget(gauge, area);
            }

            ProgressStyle::Line => {
                let line_gauge = LineGauge::default()
                    .label(label_text)
                    .ratio(self.value)
                    .line_set(ratatui::symbols::line::THICK)
                    .filled_style(Style::new().fg(progress_color).to_ratatui(profile))
                    .style(self.background_style.to_ratatui(profile));

                frame.render_widget(line_gauge, area);
            }

            ProgressStyle::Spark => {
                // Use sparkline for history visualization
                // Ensure we have some data to display
                let data = if self.history.is_empty() {
                    vec![0]
                } else {
                    self.history.clone()
                };

                let sparkline = Sparkline::default()
                    .data(&data)
                    .style(Style::new().fg(progress_color).to_ratatui(profile));

                // If we have a label, split the area
                if !label_text.is_empty() {
                    use ratatui::layout::{Constraint, Direction, Layout};

                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(1), Constraint::Min(1)])
                        .split(area);

                    // Render label
                    let label_line = Line::from(Span::styled(
                        label_text,
                        self.label_style.to_ratatui(profile),
                    ));
                    frame.render_widget(ratatui::widgets::Paragraph::new(label_line), chunks[0]);

                    // Render sparkline
                    frame.render_widget(sparkline, chunks[1]);
                } else {
                    frame.render_widget(sparkline, area);
                }
            }

            ProgressStyle::Custom { filled, empty } => {
                // Custom character-based progress bar
                let bar_width = area.width.saturating_sub(2) as usize; // Account for borders/padding
                if bar_width == 0 {
                    return;
                }
                let filled_count = (bar_width as f64 * self.value).min(bar_width as f64) as usize;
                let empty_count = bar_width.saturating_sub(filled_count);

                let filled_part = filled.to_string().repeat(filled_count);
                let empty_part = empty.to_string().repeat(empty_count);

                let content = if !label_text.is_empty() {
                    vec![
                        Line::from(Span::styled(
                            label_text,
                            self.label_style.to_ratatui(profile),
                        )),
                        Line::from(vec![
                            Span::styled(
                                filled_part.clone(),
                                Style::new().fg(progress_color.clone()).to_ratatui(profile),
                            ),
                            Span::styled(
                                empty_part.clone(),
                                self.background_style.to_ratatui(profile),
                            ),
                        ]),
                    ]
                } else {
                    vec![Line::from(vec![
                        Span::styled(
                            filled_part,
                            Style::new().fg(progress_color.clone()).to_ratatui(profile),
                        ),
                        Span::styled(empty_part, self.background_style.to_ratatui(profile)),
                    ])]
                };

                let paragraph = ratatui::widgets::Paragraph::new(content);
                frame.render_widget(paragraph, area);
            }
        }
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}
