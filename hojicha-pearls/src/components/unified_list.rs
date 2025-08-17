//! Unified scrollable list component with selection support and styling
//!
//! This component combines the functionality of both List and StyledList
//! into a single, configurable implementation.

use crate::style::{ColorProfile, Style as HojichaStyle, Theme};
use hojicha_core::event::{Event, Key, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style as RatatuiStyle},
    text::{Line, Span},
    widgets::{Block, Borders, List as RatatuiList, ListItem as RatatuiListItem, ListState, Widget},
    Frame,
};
use std::cmp::{max, min};

/// Trait for items that can be displayed in a list
pub trait ListItem: Clone {
    /// Get the display text for this item
    fn display(&self) -> String;

    /// Get the search text for filtering (defaults to display text)
    fn search_text(&self) -> String {
        self.display()
    }
}

impl ListItem for String {
    fn display(&self) -> String {
        self.clone()
    }
}

impl ListItem for &str {
    fn display(&self) -> String {
        self.to_string()
    }
}

impl<T: ToString + Clone> ListItem for T {
    fn display(&self) -> String {
        self.to_string()
    }
}

/// Configuration for list behavior and appearance
#[derive(Debug, Clone)]
pub struct ListConfig {
    /// Style for normal items
    pub item_style: Option<HojichaStyle>,
    /// Style for the selected item  
    pub selected_style: Option<HojichaStyle>,
    /// Style for the container
    pub container_style: Option<HojichaStyle>,
    /// Style for the title
    pub title_style: Option<HojichaStyle>,
    /// Style for the filter indicator
    pub filter_style: Option<HojichaStyle>,
    /// Whether to highlight the selected item
    pub highlight_selection: bool,
    /// Whether the list wraps around at the ends
    pub wrap_around: bool,
    /// Number of items to skip when page up/down
    pub page_size: usize,
    /// Show selection indicator
    pub show_selection_indicator: bool,
    /// Selection indicator character
    pub selection_indicator: String,
    /// Maximum height (in rows)
    pub max_height: Option<u16>,
    /// Whether filtering is enabled
    pub filtering_enabled: bool,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            item_style: None, // Use defaults
            selected_style: None,
            container_style: None,
            title_style: None,
            filter_style: None,
            highlight_selection: true,
            wrap_around: false,
            page_size: 10,
            show_selection_indicator: true,
            selection_indicator: "> ".to_string(),
            max_height: None,
            filtering_enabled: false,
        }
    }
}

/// A unified scrollable list component with styling and filtering support
pub struct UnifiedList<T: ListItem> {
    /// The items in the list
    items: Vec<T>,
    /// Filtered items (subset of items based on filter)
    filtered_items: Vec<T>,
    /// Current filter string
    filter: String,
    /// Configuration
    config: ListConfig,
    /// Currently selected index (in filtered items)
    selected: usize,
    /// Viewport offset (first visible item)
    offset: usize,
    /// Whether the list has focus
    focused: bool,
    /// Visible height (set during render)
    height: usize,
    /// Optional block for borders/title
    block: Option<Block<'static>>,
    /// Title of the list
    title: Option<String>,
}

impl<T: ListItem> UnifiedList<T> {
    /// Create a new unified list
    pub fn new(items: Vec<T>) -> Self {
        let mut list = Self {
            items: items.clone(),
            filtered_items: items,
            filter: String::new(),
            config: ListConfig::default(),
            selected: 0,
            offset: 0,
            focused: false,
            height: 10,
            block: None,
            title: None,
        };
        list.apply_filter();
        list
    }

    /// Configure the list with custom config
    pub fn with_config(mut self, config: ListConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the block (borders/title)
    pub fn with_block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Enable or disable filtering
    pub fn enable_filtering(mut self, enabled: bool) -> Self {
        self.config.filtering_enabled = enabled;
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
            self.selected = 0;
        } else if self.selected >= self.filtered_items.len() {
            self.selected = self.filtered_items.len() - 1;
        }
        self.ensure_visible();
    }

    /// Update the items in the list
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.apply_filter();
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&T> {
        self.filtered_items.get(self.selected)
    }

    /// Get a mutable reference to the selected item
    pub fn selected_item_mut(&mut self) -> Option<&mut T> {
        self.filtered_items.get_mut(self.selected)
    }

    /// Get the currently selected index
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Get the number of items (after filtering)
    pub fn len(&self) -> usize {
        self.filtered_items.len()
    }

    /// Check if the list is empty (after filtering)
    pub fn is_empty(&self) -> bool {
        self.filtered_items.is_empty()
    }

    /// Set whether the list has focus
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Focus the list
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// Remove focus from the list
    pub fn blur(&mut self) {
        self.focused = false;
    }

    /// Check if the list is focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Select a specific index
    pub fn select(&mut self, index: usize) {
        if index < self.filtered_items.len() {
            self.selected = index;
            self.ensure_visible();
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }

        if self.selected > 0 {
            self.selected -= 1;
        } else if self.config.wrap_around {
            self.selected = self.filtered_items.len() - 1;
        }
        self.ensure_visible();
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }

        if self.selected + 1 < self.filtered_items.len() {
            self.selected += 1;
        } else if self.config.wrap_around {
            self.selected = 0;
        }
        self.ensure_visible();
    }

    /// Move selection up by page
    pub fn page_up(&mut self) {
        if self.selected > self.config.page_size {
            self.selected -= self.config.page_size;
        } else {
            self.selected = 0;
        }
        self.ensure_visible();
    }

    /// Move selection down by page
    pub fn page_down(&mut self) {
        self.selected = min(
            self.selected + self.config.page_size,
            self.filtered_items.len().saturating_sub(1),
        );
        self.ensure_visible();
    }

    /// Select the first item
    pub fn select_first(&mut self) {
        self.selected = 0;
        self.ensure_visible();
    }

    /// Select the last item
    pub fn select_last(&mut self) {
        if !self.filtered_items.is_empty() {
            self.selected = self.filtered_items.len() - 1;
            self.ensure_visible();
        }
    }

    /// Ensure the selected item is visible
    fn ensure_visible(&mut self) {
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + self.height {
            self.offset = self.selected.saturating_sub(self.height - 1);
        }
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: &KeyEvent) -> bool {
        if !self.focused {
            return false;
        }

        match key.key {
            Key::Up | Key::Char('k') => {
                self.select_previous();
                true
            }
            Key::Down | Key::Char('j') => {
                self.select_next();
                true
            }
            Key::PageUp => {
                self.page_up();
                true
            }
            Key::PageDown => {
                self.page_down();
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
        }
    }

    /// Handle generic events (for compatibility)
    pub fn handle_event(&mut self, event: Event<()>) -> bool {
        if !self.focused {
            return false;
        }

        match event {
            Event::Key(key_event) => self.handle_key(&key_event),
            _ => false,
        }
    }

    /// Handle mouse events
    pub fn handle_mouse(&mut self, mouse: &MouseEvent, area: Rect) -> bool {
        if !self.focused {
            return false;
        }

        // Calculate inner area (accounting for borders)
        let inner = if self.block.is_some() {
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(2),
            }
        } else {
            area
        };

        // Check if click is within the list area
        if mouse.column < inner.x
            || mouse.column >= inner.x + inner.width
            || mouse.row < inner.y
            || mouse.row >= inner.y + inner.height
        {
            return false;
        }

        match mouse.kind {
            MouseEventKind::Down(_) => {
                // Calculate which item was clicked
                let clicked_index = (mouse.row - inner.y) as usize + self.offset;
                if clicked_index < self.filtered_items.len() {
                    self.selected = clicked_index;
                    return true;
                }
            }
            MouseEventKind::ScrollUp => {
                self.select_previous();
                return true;
            }
            MouseEventKind::ScrollDown => {
                self.select_next();
                return true;
            }
            _ => {}
        }

        false
    }

    /// Apply a theme to this list
    pub fn apply_theme(&mut self, theme: &Theme) {
        if let Some(style) = theme.get_style("list.item") {
            self.config.item_style = Some(style.clone());
        }

        if let Some(style) = theme.get_style("list.selected") {
            self.config.selected_style = Some(style.clone());
        }

        if let Some(style) = theme.get_style("list.container") {
            self.config.container_style = Some(style.clone());
        }

        if let Some(style) = theme.get_style("list.title") {
            self.config.title_style = Some(style.clone());
        }
    }

    /// Add an item to the list
    pub fn push(&mut self, item: T) {
        self.items.push(item);
        self.apply_filter();
    }

    /// Remove the selected item
    pub fn remove_selected(&mut self) -> Option<T> {
        if self.filtered_items.is_empty() {
            return None;
        }

        // Find the item in the original items list
        let selected_item = self.filtered_items[self.selected].clone();
        if let Some(pos) = self.items.iter().position(|item| {
            // Simple comparison - in a real implementation you might want a better equality check
            item.display() == selected_item.display()
        }) {
            let removed = self.items.remove(pos);
            self.apply_filter();
            
            // Adjust selection
            if self.selected >= self.filtered_items.len() && !self.filtered_items.is_empty() {
                self.selected = self.filtered_items.len() - 1;
            }
            self.ensure_visible();
            
            Some(removed)
        } else {
            None
        }
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.filtered_items.clear();
        self.selected = 0;
        self.offset = 0;
    }

    /// Get the items as a slice
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Get the filtered items as a slice
    pub fn filtered_items(&self) -> &[T] {
        &self.filtered_items
    }

    /// Get the items as a mutable slice
    pub fn items_mut(&mut self) -> &mut [T] {
        &mut self.items
    }
}

// Rendering implementations
impl<T: ListItem> UnifiedList<T> {
    /// Render using simple buffer approach (for compatibility with old List)
    pub fn render_to_buffer(&mut self, area: Rect, buf: &mut Buffer) {
        // Draw block if present
        let inner = if let Some(ref block) = self.block {
            let widget = block.clone();
            widget.render(area, buf);
            block.inner(area)
        } else {
            area
        };

        // Update height for scrolling calculations
        self.height = inner.height as usize;

        // Calculate visible range
        let end = min(self.offset + self.height, self.filtered_items.len());

        // Render visible items
        for (i, item_index) in (self.offset..end).enumerate() {
            if let Some(item) = self.filtered_items.get(item_index) {
                let y = inner.y + i as u16;

                // Determine style - use simple ratatui styles for buffer rendering
                let style = if item_index == self.selected && self.config.highlight_selection {
                    RatatuiStyle::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else {
                    RatatuiStyle::default()
                };

                // Render the item
                let mut text = item.display();
                if item_index == self.selected && self.config.show_selection_indicator {
                    text = format!("{}{}", self.config.selection_indicator, text);
                }

                let line = Line::from(Span::styled(text, style));
                buf.set_line(inner.x, y, &line, inner.width);
            }
        }

        // Draw scrollbar if needed
        if self.filtered_items.len() > self.height {
            let scrollbar_x = inner.x + inner.width - 1;
            let scrollbar_height = inner.height;

            // Calculate thumb size and position
            let thumb_height = max(
                1,
                (self.height * scrollbar_height as usize) / self.filtered_items.len(),
            ) as u16;
            let thumb_pos = ((self.offset * scrollbar_height as usize) / self.filtered_items.len()) as u16;

            // Draw scrollbar track
            for y in 0..scrollbar_height {
                buf[(scrollbar_x, inner.y + y)]
                    .set_char('│')
                    .set_style(RatatuiStyle::default().fg(Color::DarkGray));
            }

            // Draw scrollbar thumb
            for y in thumb_pos..min(thumb_pos + thumb_height, scrollbar_height) {
                buf[(scrollbar_x, inner.y + y)]
                    .set_char('█')
                    .set_style(RatatuiStyle::default().fg(Color::Gray));
            }
        }
    }

    /// Render using styled frame approach (for compatibility with StyledList)
    pub fn render_styled(&mut self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        // Create list items with styling
        let items: Vec<RatatuiListItem> = self
            .filtered_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = self.selected == i;
                let mut text = item.display();

                if is_selected && self.config.show_selection_indicator {
                    text = format!("{}{}", self.config.selection_indicator, text);
                }

                let style = if is_selected {
                    self.config.selected_style
                        .as_ref()
                        .map(|s| s.to_ratatui(profile))
                        .unwrap_or_else(|| RatatuiStyle::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
                } else {
                    self.config.item_style
                        .as_ref()
                        .map(|s| s.to_ratatui(profile))
                        .unwrap_or_default()
                };

                RatatuiListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        // Create the block with title and filter indicator
        let mut block = Block::default();

        if let Some(ref container_style) = self.config.container_style {
            if container_style.get_border() != &crate::style::BorderStyle::None {
                block = block
                    .borders(Borders::ALL)
                    .border_type(container_style.get_border().to_ratatui());

                if let Some(border_color) = container_style.get_border_color() {
                    let mut border_style = RatatuiStyle::default().fg(border_color.to_ratatui(profile));

                    if self.focused {
                        border_style = border_style.add_modifier(Modifier::BOLD);
                    }

                    block = block.border_style(border_style);
                }
            }
        } else if self.block.is_some() {
            // Use the provided block
            block = self.block.clone().unwrap();
        }

        // Add title with filter indicator
        if let Some(ref title) = self.title {
            let title_text = if self.config.filtering_enabled && !self.filter.is_empty() {
                format!("{} [filter: {}]", title, self.filter)
            } else {
                title.clone()
            };

            let title_style = self.config.title_style
                .as_ref()
                .map(|s| s.to_ratatui(profile))
                .unwrap_or_else(|| RatatuiStyle::default().add_modifier(Modifier::BOLD));

            block = block
                .title(title_text)
                .title_style(title_style);
        } else if self.config.filtering_enabled && !self.filter.is_empty() {
            let filter_style = self.config.filter_style
                .as_ref()
                .map(|s| s.to_ratatui(profile))
                .unwrap_or_else(|| RatatuiStyle::default().fg(Color::Yellow));

            block = block
                .title(format!("[filter: {}]", self.filter))
                .title_style(filter_style);
        }

        // Create the list widget
        let container_style = self.config.container_style
            .as_ref()
            .map(|s| s.to_ratatui(profile))
            .unwrap_or_default();

        let list = RatatuiList::new(items)
            .block(block)
            .style(container_style);

        // Apply max height if set
        let render_area = if let Some(max_h) = self.config.max_height {
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: area.height.min(max_h),
            }
        } else {
            area
        };

        // Use ListState for stateful rendering
        let mut state = ListState::default();
        if !self.filtered_items.is_empty() {
            state.select(Some(self.selected));
        }

        // Render the list
        frame.render_stateful_widget(list, render_area, &mut state);
    }
}

// Type aliases for backward compatibility
pub type List<T> = UnifiedList<T>;
pub type StyledList<T> = UnifiedList<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_list_creation() {
        let items = vec!["Item 1", "Item 2", "Item 3"];
        let list = UnifiedList::new(items);

        assert_eq!(list.len(), 3);
        assert_eq!(list.selected(), 0);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_list_navigation() {
        let items = vec!["A", "B", "C", "D"];
        let mut list = UnifiedList::new(items);

        // Test next
        list.select_next();
        assert_eq!(list.selected(), 1);
        assert_eq!(list.selected_item(), Some(&"B"));

        // Test previous
        list.select_previous();
        assert_eq!(list.selected(), 0);

        // Test bounds without wrap
        list.select_previous();
        assert_eq!(list.selected(), 0); // Should stay at 0

        // Test last
        list.select_last();
        assert_eq!(list.selected(), 3);

        // Test bounds at end
        list.select_next();
        assert_eq!(list.selected(), 3); // Should stay at 3
    }

    #[test]
    fn test_list_filtering() {
        let items = vec!["Apple", "Banana", "Cherry", "Date"];
        let mut list = UnifiedList::new(items).enable_filtering(true);

        // Test filter
        list.set_filter("a".to_string());
        assert_eq!(list.len(), 3); // Apple, Banana, Date contain 'a'

        list.set_filter("an".to_string());
        assert_eq!(list.len(), 1); // Only Banana contains 'an'

        // Clear filter
        list.clear_filter();
        assert_eq!(list.len(), 4); // All items visible again
    }

    #[test]
    fn test_list_wrap_around() {
        let items = vec![1, 2, 3];
        let config = ListConfig {
            wrap_around: true,
            ..Default::default()
        };
        let mut list = UnifiedList::new(items).with_config(config);

        // Test wrap from beginning
        list.select_previous();
        assert_eq!(list.selected(), 2);

        // Test wrap from end
        list.select_next();
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn test_list_modification() {
        let mut list = UnifiedList::new(vec!["A", "B", "C"]);

        // Test push
        list.push("D");
        assert_eq!(list.len(), 4);

        // Test remove
        list.select(1); // Select "B"
        let removed = list.remove_selected();
        assert_eq!(removed, Some("B"));
        assert_eq!(list.len(), 3);

        // Test clear
        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.selected(), 0);
    }
}