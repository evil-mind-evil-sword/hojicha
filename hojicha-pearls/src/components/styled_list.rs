//! Styled list component with theme support
//!
//! **DEPRECATED**: This component has been superseded by `unified_list::UnifiedList`.
//! Use `UnifiedList` for new code as it provides all the functionality of both
//! `List` and `StyledList` in a single, configurable component.
//!
//! A scrollable list with selection, filtering, and rich styling options.

use crate::style::{ColorProfile, Style, Theme};
use hojicha_core::event::{Event, Key, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style as RatatuiStyle},
    text::{Line, Span},
    widgets::{Block, Borders, List as RatatuiList, ListItem as RatatuiListItem, ListState},
    Frame,
};

/// List item that can be displayed
pub trait ListItemTrait: Clone {
    /// Get the display text for this item
    fn display(&self) -> String;

    /// Get the search text for filtering (defaults to display text)
    fn search_text(&self) -> String {
        self.display()
    }
}

impl ListItemTrait for String {
    fn display(&self) -> String {
        self.clone()
    }
}

impl ListItemTrait for &str {
    fn display(&self) -> String {
        self.to_string()
    }
}

impl ListItemTrait for i32 {
    fn display(&self) -> String {
        self.to_string()
    }
}

impl ListItemTrait for usize {
    fn display(&self) -> String {
        self.to_string()
    }
}

/// A styled, scrollable list component
pub struct StyledList<T: ListItemTrait> {
    /// The items in the list
    items: Vec<T>,
    /// Filtered items (subset of items based on filter)
    filtered_items: Vec<T>,
    /// Current filter string
    filter: String,
    /// Whether filtering is enabled
    filter_enabled: bool,
    /// The list state (selection, scroll position)
    state: ListState,
    /// Title of the list
    title: Option<String>,
    /// Style for normal items
    item_style: Style,
    /// Style for the selected item
    selected_style: Style,
    /// Style for the list container
    container_style: Style,
    /// Style for the title
    title_style: Style,
    /// Style for the filter indicator
    filter_style: Style,
    /// Whether the list is focused
    focused: bool,
    /// Show selection indicator
    show_selection: bool,
    /// Selection indicator character
    selection_indicator: String,
    /// Maximum height (in rows)
    max_height: Option<u16>,
}

impl<T: ListItemTrait> StyledList<T> {
    /// Create a new styled list
    pub fn new(items: Vec<T>) -> Self {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }

        let filtered_items = items.clone();

        Self {
            items: items.clone(),
            filtered_items,
            filter: String::new(),
            filter_enabled: false,
            state,
            title: None,
            item_style: Style::new(),
            selected_style: Style::new().bg(crate::style::Color::blue()).bold(),
            container_style: Style::new().border(crate::style::BorderStyle::Normal),
            title_style: Style::new().bold(),
            filter_style: Style::new().fg(crate::style::Color::yellow()).italic(),
            focused: false,
            show_selection: true,
            selection_indicator: "> ".to_string(),
            max_height: None,
        }
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Enable or disable filtering
    pub fn with_filter(mut self, enabled: bool) -> Self {
        self.filter_enabled = enabled;
        self
    }

    /// Set the filter string
    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter.to_lowercase();
        self.apply_filter();
    }

    /// Clear the filter
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.apply_filter();
    }

    /// Apply the current filter to items
    fn apply_filter(&mut self) {
        if self.filter.is_empty() {
            self.filtered_items = self.items.clone();
        } else {
            self.filtered_items = self
                .items
                .iter()
                .filter(|item| item.search_text().to_lowercase().contains(&self.filter))
                .cloned()
                .collect();
        }

        // Reset selection if needed
        if self.filtered_items.is_empty() {
            self.state.select(None);
        } else if self
            .state
            .selected()
            .map_or(true, |i| i >= self.filtered_items.len())
        {
            self.state.select(Some(0));
        }
    }

    /// Update the items in the list
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.apply_filter();
    }

    /// Get the currently selected item
    pub fn selected(&self) -> Option<&T> {
        self.state
            .selected()
            .and_then(|i| self.filtered_items.get(i))
    }

    /// Get the index of the selected item
    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }

    /// Select a specific item by index
    pub fn select(&mut self, index: Option<usize>) {
        if let Some(i) = index {
            if i < self.filtered_items.len() {
                self.state.select(Some(i));
            }
        } else {
            self.state.select(None);
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        let current = self.state.selected().unwrap_or(0);
        let new_index = if current == 0 {
            self.filtered_items.len().saturating_sub(1)
        } else {
            current.saturating_sub(1)
        };

        if !self.filtered_items.is_empty() {
            self.state.select(Some(new_index));
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        let current = self.state.selected().unwrap_or(0);
        let new_index = if current >= self.filtered_items.len().saturating_sub(1) {
            0
        } else {
            current + 1
        };

        if !self.filtered_items.is_empty() {
            self.state.select(Some(new_index));
        }
    }

    /// Move to the first item
    pub fn select_first(&mut self) {
        if !self.filtered_items.is_empty() {
            self.state.select(Some(0));
        }
    }

    /// Move to the last item
    pub fn select_last(&mut self) {
        if !self.filtered_items.is_empty() {
            self.state.select(Some(self.filtered_items.len() - 1));
        }
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

    /// Set the item style
    pub fn with_item_style(mut self, style: Style) -> Self {
        self.item_style = style;
        self
    }

    /// Set the selected item style
    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set the container style
    pub fn with_container_style(mut self, style: Style) -> Self {
        self.container_style = style;
        self
    }

    /// Set the title style
    pub fn with_title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Set whether to show selection indicator
    pub fn with_selection_indicator(mut self, show: bool) -> Self {
        self.show_selection = show;
        self
    }

    /// Set custom selection indicator
    pub fn with_custom_indicator(mut self, indicator: impl Into<String>) -> Self {
        self.selection_indicator = indicator.into();
        self
    }

    /// Set maximum height
    pub fn with_max_height(mut self, height: u16) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Handle keyboard input
    pub fn handle_event(&mut self, event: Event<()>) -> bool {
        if !self.focused {
            return false;
        }

        match event {
            Event::Key(KeyEvent { key, .. }) => match key {
                Key::Up | Key::Char('k') => {
                    self.select_previous();
                    true
                }
                Key::Down | Key::Char('j') => {
                    self.select_next();
                    true
                }
                Key::Home | Key::Char('g') => {
                    self.select_first();
                    true
                }
                Key::End | Key::Char('G') => {
                    self.select_last();
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Apply a theme to this list
    pub fn apply_theme(&mut self, theme: &Theme) {
        if let Some(style) = theme.get_style("list.item") {
            self.item_style = style.clone();
        }

        if let Some(style) = theme.get_style("list.selected") {
            self.selected_style = style.clone();
        }

        if let Some(style) = theme.get_style("list.container") {
            self.container_style = style.clone();
        }

        if let Some(style) = theme.get_style("list.title") {
            self.title_style = style.clone();
        }
    }

    /// Render the list
    pub fn render(&mut self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        // Create list items with styling
        let items: Vec<RatatuiListItem> = self
            .filtered_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = self.state.selected() == Some(i);
                let mut text = item.display();

                if is_selected && self.show_selection {
                    text = format!("{}{}", self.selection_indicator, text);
                }

                let style = if is_selected {
                    self.selected_style.to_ratatui(profile)
                } else {
                    self.item_style.to_ratatui(profile)
                };

                RatatuiListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        // Create the block with title and filter indicator
        let mut block = Block::default();

        if self.container_style.get_border() != &crate::style::BorderStyle::None {
            block = block
                .borders(Borders::ALL)
                .border_type(self.container_style.get_border().to_ratatui());

            if let Some(border_color) = self.container_style.get_border_color() {
                let mut border_style = RatatuiStyle::default().fg(border_color.to_ratatui(profile));

                if self.focused {
                    border_style = border_style.add_modifier(Modifier::BOLD);
                }

                block = block.border_style(border_style);
            }
        }

        // Add title with filter indicator
        if let Some(ref title) = self.title {
            let title_text = if self.filter_enabled && !self.filter.is_empty() {
                format!("{} [filter: {}]", title, self.filter)
            } else {
                title.clone()
            };

            block = block
                .title(title_text)
                .title_style(self.title_style.to_ratatui(profile));
        } else if self.filter_enabled && !self.filter.is_empty() {
            block = block
                .title(format!("[filter: {}]", self.filter))
                .title_style(self.filter_style.to_ratatui(profile));
        }

        // Create the list widget
        let list = RatatuiList::new(items)
            .block(block)
            .style(self.container_style.to_ratatui(profile));

        // Apply max height if set
        let render_area = if let Some(max_h) = self.max_height {
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: area.height.min(max_h),
            }
        } else {
            area
        };

        // Render the list
        frame.render_stateful_widget(list, render_area, &mut self.state);
    }

    /// Get the number of items (after filtering)
    pub fn len(&self) -> usize {
        self.filtered_items.len()
    }

    /// Check if the list is empty (after filtering)
    pub fn is_empty(&self) -> bool {
        self.filtered_items.is_empty()
    }
}
