//! Tabs component for organizing content into tabbed panels
//!
//! A flexible tab bar component with support for icons, badges, and closeable tabs.

use crate::event::{Event, Key, KeyEvent};
use crate::style::{Color, ColorProfile, Style, Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Position of the tab bar
#[derive(Clone, Debug, PartialEq)]
pub enum TabPosition {
    /// Tabs at the top (default)
    Top,
    /// Tabs at the bottom
    Bottom,
    /// Tabs on the left side
    Left,
    /// Tabs on the right side
    Right,
}

/// Style variant for tabs
#[derive(Clone, Debug, PartialEq)]
pub enum TabStyle {
    /// Simple line under active tab
    Line,
    /// Box around active tab
    Box,
    /// Rounded corners
    Rounded,
    /// No decoration
    Plain,
}

/// A single tab
#[derive(Clone)]
pub struct Tab {
    /// Tab title
    title: String,
    /// Optional icon/emoji
    icon: Option<String>,
    /// Optional badge (e.g., notification count)
    badge: Option<String>,
    /// Whether the tab can be closed
    closeable: bool,
    /// Whether the tab is enabled
    enabled: bool,
    /// Custom style for this specific tab
    custom_style: Option<Style>,
}

impl Tab {
    /// Create a new tab
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            badge: None,
            closeable: false,
            enabled: true,
            custom_style: None,
        }
    }

    /// Set the icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the badge
    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Make the tab closeable
    pub fn closeable(mut self) -> Self {
        self.closeable = true;
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set custom style
    pub fn with_style(mut self, style: Style) -> Self {
        self.custom_style = Some(style);
        self
    }
}

/// A tabs component
#[derive(Clone)]
pub struct Tabs {
    /// The tabs
    tabs: Vec<Tab>,
    /// Currently selected tab index
    selected: usize,
    /// Position of the tab bar
    position: TabPosition,
    /// Visual style
    style: TabStyle,
    /// Style for inactive tabs
    inactive_style: Style,
    /// Style for active tab
    active_style: Style,
    /// Style for disabled tabs
    disabled_style: Style,
    /// Container style
    container_style: Style,
    /// Whether the component is focused
    focused: bool,
    /// Tab bar height (for horizontal) or width (for vertical)
    bar_size: u16,
    /// Show dividers between tabs
    show_dividers: bool,
    /// Divider character
    divider: String,
    /// Callback for tab close (stored as index)
    on_close_requested: Option<usize>,
}

impl Tabs {
    /// Create a new tabs component
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            tabs,
            selected: 0,
            position: TabPosition::Top,
            style: TabStyle::Line,
            inactive_style: Style::new().fg(Color::gray()),
            active_style: Style::new().fg(Color::white()).bold().underlined(),
            disabled_style: Style::new().fg(Color::gray()).italic(),
            container_style: Style::new(),
            focused: false,
            bar_size: 3,
            show_dividers: false,
            divider: "│".to_string(),
            on_close_requested: None,
        }
    }

    /// Create tabs from titles
    pub fn from_titles(titles: Vec<String>) -> Self {
        let tabs = titles.into_iter().map(Tab::new).collect();
        Self::new(tabs)
    }

    /// Add a tab
    pub fn add_tab(&mut self, tab: Tab) -> &mut Self {
        self.tabs.push(tab);
        self
    }

    /// Remove a tab by index
    pub fn remove_tab(&mut self, index: usize) -> Option<Tab> {
        if index < self.tabs.len() {
            // Adjust selected index if needed
            if self.selected >= index && self.selected > 0 {
                self.selected -= 1;
            } else if self.selected >= self.tabs.len() - 1 && !self.tabs.is_empty() {
                self.selected = self.tabs.len() - 2;
            }
            Some(self.tabs.remove(index))
        } else {
            None
        }
    }

    /// Get the selected tab index
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Set the selected tab
    pub fn select(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.selected = index;
        }
    }

    /// Select next tab
    pub fn select_next(&mut self) {
        if !self.tabs.is_empty() {
            // Skip disabled tabs
            let mut next = (self.selected + 1) % self.tabs.len();
            let start = next;
            while !self.tabs[next].enabled {
                next = (next + 1) % self.tabs.len();
                if next == start {
                    break; // All tabs disabled
                }
            }
            if self.tabs[next].enabled {
                self.selected = next;
            }
        }
    }

    /// Select previous tab
    pub fn select_previous(&mut self) {
        if !self.tabs.is_empty() {
            // Skip disabled tabs
            let mut prev = if self.selected == 0 {
                self.tabs.len() - 1
            } else {
                self.selected - 1
            };
            let start = prev;
            while !self.tabs[prev].enabled {
                prev = if prev == 0 {
                    self.tabs.len() - 1
                } else {
                    prev - 1
                };
                if prev == start {
                    break; // All tabs disabled
                }
            }
            if self.tabs[prev].enabled {
                self.selected = prev;
            }
        }
    }

    /// Set position
    pub fn with_position(mut self, position: TabPosition) -> Self {
        self.position = position;
        self
    }

    /// Set style
    pub fn with_style(mut self, style: TabStyle) -> Self {
        self.style = style;
        self
    }

    /// Set inactive tab style
    pub fn with_inactive_style(mut self, style: Style) -> Self {
        self.inactive_style = style;
        self
    }

    /// Set active tab style
    pub fn with_active_style(mut self, style: Style) -> Self {
        self.active_style = style;
        self
    }

    /// Set disabled tab style
    pub fn with_disabled_style(mut self, style: Style) -> Self {
        self.disabled_style = style;
        self
    }

    /// Set container style
    pub fn with_container_style(mut self, style: Style) -> Self {
        self.container_style = style;
        self
    }

    /// Set bar size
    pub fn with_bar_size(mut self, size: u16) -> Self {
        self.bar_size = size;
        self
    }

    /// Set whether to show dividers
    pub fn with_dividers(mut self, show: bool) -> Self {
        self.show_dividers = show;
        self
    }

    /// Set divider character
    pub fn with_divider(mut self, divider: impl Into<String>) -> Self {
        self.divider = divider.into();
        self
    }

    /// Focus the component
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// Blur the component
    pub fn blur(&mut self) {
        self.focused = false;
    }

    /// Check if focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Check if a close was requested
    pub fn close_requested(&mut self) -> Option<usize> {
        self.on_close_requested.take()
    }

    /// Handle keyboard input
    pub fn handle_event(&mut self, event: Event<()>) -> bool {
        if !self.focused {
            return false;
        }

        match event {
            Event::Key(KeyEvent { key, .. }) => match key {
                Key::Left | Key::Char('h') => match self.position {
                    TabPosition::Top | TabPosition::Bottom => {
                        self.select_previous();
                        true
                    }
                    _ => false,
                },
                Key::Right | Key::Char('l') => match self.position {
                    TabPosition::Top | TabPosition::Bottom => {
                        self.select_next();
                        true
                    }
                    _ => false,
                },
                Key::Up | Key::Char('k') => match self.position {
                    TabPosition::Left | TabPosition::Right => {
                        self.select_previous();
                        true
                    }
                    _ => false,
                },
                Key::Down | Key::Char('j') => match self.position {
                    TabPosition::Left | TabPosition::Right => {
                        self.select_next();
                        true
                    }
                    _ => false,
                },
                Key::Char('w') => {
                    // Close current tab if closeable (Ctrl+W style)
                    if self.selected < self.tabs.len() && self.tabs[self.selected].closeable {
                        self.on_close_requested = Some(self.selected);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Apply a theme
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.inactive_style = Style::new().fg(theme.colors.text_secondary.clone());

        self.active_style = Style::new().fg(theme.colors.primary.clone()).bold();

        self.disabled_style = Style::new().fg(theme.colors.border.clone()).italic();

        if let Some(style) = theme.get_style("tabs.container") {
            self.container_style = style.clone();
        }
    }

    /// Calculate layout for tabs and content
    pub fn layout(&self, area: Rect) -> (Rect, Rect) {
        match self.position {
            TabPosition::Top => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(self.bar_size), Constraint::Min(0)])
                    .split(area);
                (chunks[0], chunks[1]) // (tabs, content)
            }
            TabPosition::Bottom => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(self.bar_size)])
                    .split(area);
                (chunks[1], chunks[0]) // (tabs, content)
            }
            TabPosition::Left => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(self.bar_size * 5), // Wider for vertical tabs
                        Constraint::Min(0),
                    ])
                    .split(area);
                (chunks[0], chunks[1]) // (tabs, content)
            }
            TabPosition::Right => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(self.bar_size * 5), // Wider for vertical tabs
                    ])
                    .split(area);
                (chunks[1], chunks[0]) // (tabs, content)
            }
        }
    }

    /// Render the tabs
    pub fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        // Check if area is valid before rendering
        if !super::utils::is_valid_area(area) {
            return;
        }

        let (tabs_area, _) = self.layout(area);

        // Check if tabs area is valid
        if !super::utils::is_valid_area(tabs_area) {
            return;
        }

        match self.position {
            TabPosition::Top | TabPosition::Bottom => {
                self.render_horizontal(frame, tabs_area, profile);
            }
            TabPosition::Left | TabPosition::Right => {
                self.render_vertical(frame, tabs_area, profile);
            }
        }
    }

    /// Render tabs horizontally
    fn render_horizontal(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        let mut spans = Vec::new();

        for (i, tab) in self.tabs.iter().enumerate() {
            if i > 0 && self.show_dividers {
                spans.push(Span::styled(
                    format!(" {} ", self.divider),
                    self.inactive_style.to_ratatui(profile),
                ));
            }

            let is_selected = i == self.selected;
            let style = if !tab.enabled {
                self.disabled_style.to_ratatui(profile)
            } else if is_selected {
                self.active_style.to_ratatui(profile)
            } else {
                tab.custom_style
                    .as_ref()
                    .map(|s| s.to_ratatui(profile))
                    .unwrap_or_else(|| self.inactive_style.to_ratatui(profile))
            };

            // Build tab content
            let mut tab_text = String::new();

            if is_selected && self.style == TabStyle::Box {
                tab_text.push_str("[ ");
            } else {
                tab_text.push(' ');
            }

            if let Some(ref icon) = tab.icon {
                tab_text.push_str(icon);
                tab_text.push(' ');
            }

            tab_text.push_str(&tab.title);

            if let Some(ref badge) = tab.badge {
                tab_text.push_str(&format!(" [{}]", badge));
            }

            if tab.closeable {
                tab_text.push_str(" ×");
            }

            if is_selected && self.style == TabStyle::Box {
                tab_text.push_str(" ]");
            } else {
                tab_text.push(' ');
            }

            spans.push(Span::styled(tab_text, style));
        }

        let block = if self.style == TabStyle::Line {
            Block::default().borders(Borders::BOTTOM)
        } else {
            Block::default()
        };

        let paragraph = Paragraph::new(Line::from(spans))
            .block(block)
            .style(self.container_style.to_ratatui(profile));

        frame.render_widget(paragraph, area);
    }

    /// Render tabs vertically
    fn render_vertical(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        let mut lines = Vec::new();

        for (i, tab) in self.tabs.iter().enumerate() {
            let is_selected = i == self.selected;
            let style = if !tab.enabled {
                self.disabled_style.to_ratatui(profile)
            } else if is_selected {
                self.active_style.to_ratatui(profile)
            } else {
                tab.custom_style
                    .as_ref()
                    .map(|s| s.to_ratatui(profile))
                    .unwrap_or_else(|| self.inactive_style.to_ratatui(profile))
            };

            // Build tab content
            let mut tab_text = String::new();

            if is_selected {
                tab_text.push_str("▶ ");
            } else {
                tab_text.push_str("  ");
            }

            if let Some(ref icon) = tab.icon {
                tab_text.push_str(icon);
                tab_text.push(' ');
            }

            tab_text.push_str(&tab.title);

            if let Some(ref badge) = tab.badge {
                tab_text.push_str(&format!(" [{}]", badge));
            }

            if tab.closeable {
                tab_text.push_str(" ×");
            }

            lines.push(Line::from(Span::styled(tab_text, style)));
        }

        let block = if self.position == TabPosition::Left {
            Block::default().borders(Borders::RIGHT)
        } else {
            Block::default().borders(Borders::LEFT)
        };

        let paragraph = Paragraph::new(lines)
            .block(block)
            .style(self.container_style.to_ratatui(profile));

        frame.render_widget(paragraph, area);
    }

    /// Get the number of tabs
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Get a reference to the tabs
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// Get a mutable reference to the tabs
    pub fn tabs_mut(&mut self) -> &mut Vec<Tab> {
        &mut self.tabs
    }
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

/// Builder for creating tabs
pub struct TabsBuilder {
    tabs: Vec<Tab>,
    position: TabPosition,
    style: TabStyle,
}

impl TabsBuilder {
    /// Create a new tabs builder
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            position: TabPosition::Top,
            style: TabStyle::Line,
        }
    }

    /// Add a tab
    pub fn tab(mut self, title: impl Into<String>) -> Self {
        self.tabs.push(Tab::new(title));
        self
    }

    /// Add a tab with icon
    pub fn tab_with_icon(mut self, icon: impl Into<String>, title: impl Into<String>) -> Self {
        self.tabs.push(Tab::new(title).with_icon(icon));
        self
    }

    /// Set position
    pub fn position(mut self, position: TabPosition) -> Self {
        self.position = position;
        self
    }

    /// Set style
    pub fn style(mut self, style: TabStyle) -> Self {
        self.style = style;
        self
    }

    /// Build the tabs
    pub fn build(self) -> Tabs {
        Tabs::new(self.tabs)
            .with_position(self.position)
            .with_style(self.style)
    }
}

impl Default for TabsBuilder {
    fn default() -> Self {
        Self::new()
    }
}
