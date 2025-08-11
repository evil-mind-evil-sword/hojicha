//! Help component for displaying keyboard shortcuts
//!
//! A customizable horizontal or vertical help view that can be auto-generated
//! from keybindings or manually configured.

use crate::style::{Color, ColorProfile, Style, Theme};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::BTreeMap;

/// A key binding entry for the help component
#[derive(Clone, Debug)]
pub struct HelpEntry {
    /// The key combination (e.g., "Ctrl+C", "q", "↑/↓")
    pub key: String,
    /// Description of what the key does
    pub description: String,
    /// Whether this binding is currently available/active
    pub available: bool,
}

impl HelpEntry {
    /// Create a new help entry
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            available: true,
        }
    }

    /// Set availability of this binding
    pub fn with_availability(mut self, available: bool) -> Self {
        self.available = available;
        self
    }
}

/// Display mode for the help component
#[derive(Clone, Debug, PartialEq)]
pub enum HelpMode {
    /// Display shortcuts horizontally in a single line
    Horizontal,
    /// Display shortcuts vertically in a list
    Vertical,
    /// Compact mode with minimal spacing
    Compact,
}

/// A help component for displaying keyboard shortcuts and commands
#[derive(Clone)]
pub struct Help {
    /// The help entries to display
    entries: Vec<HelpEntry>,
    /// Display mode
    mode: HelpMode,
    /// Style for keys
    key_style: Style,
    /// Style for descriptions
    description_style: Style,
    /// Style for separator between entries
    separator_style: Style,
    /// Style for unavailable entries
    disabled_style: Style,
    /// Container style
    container_style: Style,
    /// Separator string between key and description
    key_separator: String,
    /// Separator between entries (horizontal mode)
    entry_separator: String,
    /// Show only available bindings
    hide_unavailable: bool,
    /// Maximum width for keys (for alignment)
    key_width: Option<usize>,
    /// Title for the help section
    title: Option<String>,
}

impl Help {
    /// Create a new help component
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            mode: HelpMode::Horizontal,
            key_style: Style::new().bold().fg(Color::cyan()),
            description_style: Style::new().fg(Color::gray()),
            separator_style: Style::new().fg(Color::gray()),
            disabled_style: Style::new().fg(Color::gray()).italic(),
            container_style: Style::new(),
            key_separator: ": ".to_string(),
            entry_separator: " • ".to_string(),
            hide_unavailable: false,
            key_width: None,
            title: None,
        }
    }

    /// Create help from a list of entries
    pub fn from_entries(entries: Vec<HelpEntry>) -> Self {
        let mut help = Self::new();
        help.entries = entries;
        help.calculate_key_width();
        help
    }

    /// Create help from a BTreeMap (maintains key order)
    pub fn from_map(bindings: BTreeMap<String, String>) -> Self {
        let entries: Vec<HelpEntry> = bindings
            .into_iter()
            .map(|(key, desc)| HelpEntry::new(key, desc))
            .collect();
        Self::from_entries(entries)
    }

    /// Add a help entry
    pub fn add(&mut self, key: impl Into<String>, description: impl Into<String>) -> &mut Self {
        self.entries.push(HelpEntry::new(key, description));
        self.calculate_key_width();
        self
    }

    /// Add a help entry with availability
    pub fn add_with_availability(
        &mut self,
        key: impl Into<String>,
        description: impl Into<String>,
        available: bool,
    ) -> &mut Self {
        self.entries
            .push(HelpEntry::new(key, description).with_availability(available));
        self.calculate_key_width();
        self
    }

    /// Set all entries at once
    pub fn with_entries(mut self, entries: Vec<HelpEntry>) -> Self {
        self.entries = entries;
        self.calculate_key_width();
        self
    }

    /// Set display mode
    pub fn with_mode(&mut self, mode: HelpMode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Set title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set key style
    pub fn with_key_style(mut self, style: Style) -> Self {
        self.key_style = style;
        self
    }

    /// Set description style
    pub fn with_description_style(mut self, style: Style) -> Self {
        self.description_style = style;
        self
    }

    /// Set separator style
    pub fn with_separator_style(mut self, style: Style) -> Self {
        self.separator_style = style;
        self
    }

    /// Set disabled style
    pub fn with_disabled_style(mut self, style: Style) -> Self {
        self.disabled_style = style;
        self
    }

    /// Set container style
    pub fn with_container_style(mut self, style: Style) -> Self {
        self.container_style = style;
        self
    }

    /// Set the separator between key and description
    pub fn with_key_separator(mut self, separator: impl Into<String>) -> Self {
        self.key_separator = separator.into();
        self
    }

    /// Set the separator between entries (horizontal mode)
    pub fn with_entry_separator(mut self, separator: impl Into<String>) -> Self {
        self.entry_separator = separator.into();
        self
    }

    /// Set whether to hide unavailable bindings
    pub fn hide_unavailable(mut self, hide: bool) -> Self {
        self.hide_unavailable = hide;
        self
    }

    /// Update availability of a specific key binding
    pub fn set_availability(&mut self, key: &str, available: bool) {
        for entry in &mut self.entries {
            if entry.key == key {
                entry.available = available;
                break;
            }
        }
    }

    /// Calculate the maximum key width for alignment
    fn calculate_key_width(&mut self) {
        self.key_width = self.entries.iter().map(|e| e.key.len()).max();
    }

    /// Apply a theme to this help component
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.key_style = Style::new().bold().fg(theme.colors.primary.clone());

        self.description_style = Style::new().fg(theme.colors.text_secondary.clone());

        self.separator_style = Style::new().fg(theme.colors.border.clone());

        self.disabled_style = Style::new()
            .fg(theme.colors.text_secondary.clone())
            .italic();

        if let Some(style) = theme.get_style("help.container") {
            self.container_style = style.clone();
        }
    }

    /// Render the help component
    pub fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        let visible_entries: Vec<&HelpEntry> = self
            .entries
            .iter()
            .filter(|e| !self.hide_unavailable || e.available)
            .collect();

        if visible_entries.is_empty() {
            return;
        }

        match self.mode {
            HelpMode::Horizontal => self.render_horizontal(frame, area, profile, &visible_entries),
            HelpMode::Vertical => self.render_vertical(frame, area, profile, &visible_entries),
            HelpMode::Compact => self.render_compact(frame, area, profile, &visible_entries),
        }
    }

    /// Render in horizontal mode
    fn render_horizontal(
        &self,
        frame: &mut Frame,
        area: Rect,
        profile: &ColorProfile,
        entries: &[&HelpEntry],
    ) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        let mut spans = Vec::new();

        for (i, entry) in entries.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(
                    &self.entry_separator,
                    self.separator_style.to_ratatui(profile),
                ));
            }

            let (key_style, desc_style) = if entry.available {
                (
                    self.key_style.to_ratatui(profile),
                    self.description_style.to_ratatui(profile),
                )
            } else {
                (
                    self.disabled_style.to_ratatui(profile),
                    self.disabled_style.to_ratatui(profile),
                )
            };

            spans.push(Span::styled(&entry.key, key_style));
            spans.push(Span::styled(
                &self.key_separator,
                self.separator_style.to_ratatui(profile),
            ));
            spans.push(Span::styled(&entry.description, desc_style));
        }

        let mut block = Block::default();
        if let Some(ref title) = self.title {
            block = block.title(title.as_str());
        }

        if self.container_style.get_border() != &crate::style::BorderStyle::None {
            block = block
                .borders(Borders::ALL)
                .border_style(self.container_style.to_ratatui(profile));
        }

        let paragraph = Paragraph::new(Line::from(spans))
            .block(block)
            .style(self.container_style.to_ratatui(profile));

        frame.render_widget(paragraph, area);
    }

    /// Render in vertical mode
    fn render_vertical(
        &self,
        frame: &mut Frame,
        area: Rect,
        profile: &ColorProfile,
        entries: &[&HelpEntry],
    ) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        let mut lines = Vec::new();

        for entry in entries {
            let (key_style, desc_style) = if entry.available {
                (
                    self.key_style.to_ratatui(profile),
                    self.description_style.to_ratatui(profile),
                )
            } else {
                (
                    self.disabled_style.to_ratatui(profile),
                    self.disabled_style.to_ratatui(profile),
                )
            };

            let key = if let Some(width) = self.key_width {
                format!("{:width$}", entry.key, width = width)
            } else {
                entry.key.clone()
            };

            lines.push(Line::from(vec![
                Span::styled(key, key_style),
                Span::styled(
                    &self.key_separator,
                    self.separator_style.to_ratatui(profile),
                ),
                Span::styled(&entry.description, desc_style),
            ]));
        }

        let mut block = Block::default();
        if let Some(ref title) = self.title {
            block = block.title(title.as_str());
        }

        if self.container_style.get_border() != &crate::style::BorderStyle::None {
            block = block
                .borders(Borders::ALL)
                .border_style(self.container_style.to_ratatui(profile));
        }

        let paragraph = Paragraph::new(lines)
            .block(block)
            .style(self.container_style.to_ratatui(profile));

        frame.render_widget(paragraph, area);
    }

    /// Render in compact mode (minimal spacing)
    fn render_compact(
        &self,
        frame: &mut Frame,
        area: Rect,
        profile: &ColorProfile,
        entries: &[&HelpEntry],
    ) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        let mut spans = Vec::new();

        for (i, entry) in entries.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" ")); // Minimal spacing
            }

            let style = if entry.available {
                self.key_style.to_ratatui(profile)
            } else {
                self.disabled_style.to_ratatui(profile)
            };

            spans.push(Span::styled(
                format!("{}:{}", entry.key, entry.description),
                style,
            ));
        }

        let paragraph =
            Paragraph::new(Line::from(spans)).style(self.container_style.to_ratatui(profile));

        frame.render_widget(paragraph, area);
    }
}

impl Default for Help {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating help components with common shortcuts
pub struct HelpBuilder {
    help: Help,
}

impl HelpBuilder {
    /// Create a new help builder
    pub fn new() -> Self {
        Self { help: Help::new() }
    }

    /// Add navigation shortcuts
    pub fn with_navigation(mut self) -> Self {
        self.help
            .add("↑/↓", "Navigate")
            .add("←/→", "Switch tabs")
            .add("Enter", "Select");
        self
    }

    /// Add common shortcuts
    pub fn with_common(mut self) -> Self {
        self.help
            .add("q", "Quit")
            .add("?", "Help")
            .add("Esc", "Cancel");
        self
    }

    /// Add editing shortcuts
    pub fn with_editing(mut self) -> Self {
        self.help
            .add("i", "Insert mode")
            .add("Esc", "Normal mode")
            .add("Ctrl+S", "Save");
        self
    }

    /// Build the help component
    pub fn build(self) -> Help {
        self.help
    }
}

impl Default for HelpBuilder {
    fn default() -> Self {
        Self::new()
    }
}
