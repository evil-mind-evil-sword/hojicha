//! Scrollable list component with selection support
//!
//! A list provides navigation through a collection of items with keyboard and mouse support.

use crate::event::{Key, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Widget};
use std::cmp::min;

/// Options for customizing list behavior
#[derive(Debug, Clone)]
pub struct ListOptions {
    /// Style for normal items
    pub item_style: Style,
    /// Style for the selected item
    pub selected_style: Style,
    /// Whether to highlight the selected item
    pub highlight_selection: bool,
    /// Whether the list wraps around at the ends
    pub wrap_around: bool,
    /// Number of items to skip when page up/down
    pub page_size: usize,
}

impl Default for ListOptions {
    fn default() -> Self {
        Self {
            item_style: Style::default(),
            selected_style: Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            highlight_selection: true,
            wrap_around: false,
            page_size: 10,
        }
    }
}

/// A scrollable list component
#[derive(Clone)]
pub struct List<T> {
    /// The items in the list
    items: Vec<T>,
    /// Currently selected index
    selected: usize,
    /// Viewport offset (first visible item)
    offset: usize,
    /// Whether the list has focus
    focused: bool,
    /// Visible height (set during render)
    height: usize,
    /// List options
    options: ListOptions,
    /// Optional block for borders/title
    block: Option<Block<'static>>,
}

impl<T> List<T> {
    /// Create a new list with the given items
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            selected: 0,
            offset: 0,
            focused: false,
            height: 10,
            options: ListOptions::default(),
            block: None,
        }
    }

    /// Set the list options
    pub fn with_options(mut self, options: ListOptions) -> Self {
        self.options = options;
        self
    }

    /// Set the block (borders/title)
    pub fn with_block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Get the currently selected index
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Get a reference to the selected item
    pub fn selected_item(&self) -> Option<&T> {
        self.items.get(self.selected)
    }

    /// Get a mutable reference to the selected item
    pub fn selected_item_mut(&mut self) -> Option<&mut T> {
        self.items.get_mut(self.selected)
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Set whether the list has focus
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Select a specific index
    pub fn select(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
            self.ensure_visible();
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        if self.selected > 0 {
            self.selected -= 1;
        } else if self.options.wrap_around {
            self.selected = self.items.len() - 1;
        }
        self.ensure_visible();
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        if self.selected + 1 < self.items.len() {
            self.selected += 1;
        } else if self.options.wrap_around {
            self.selected = 0;
        }
        self.ensure_visible();
    }

    /// Move selection up by page
    pub fn page_up(&mut self) {
        if self.selected > self.options.page_size {
            self.selected -= self.options.page_size;
        } else {
            self.selected = 0;
        }
        self.ensure_visible();
    }

    /// Move selection down by page
    pub fn page_down(&mut self) {
        self.selected = min(
            self.selected + self.options.page_size,
            self.items.len().saturating_sub(1),
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
        if !self.items.is_empty() {
            self.selected = self.items.len() - 1;
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
            Key::Home => {
                self.select_first();
                true
            }
            Key::End => {
                self.select_last();
                true
            }
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
                if clicked_index < self.items.len() {
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

    /// Get the items as a slice
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Get the items as a mutable slice
    pub fn items_mut(&mut self) -> &mut [T] {
        &mut self.items
    }

    /// Add an item to the list
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Remove the selected item
    pub fn remove_selected(&mut self) -> Option<T> {
        if self.items.is_empty() {
            return None;
        }

        let removed = self.items.remove(self.selected);

        // Adjust selection
        if self.selected >= self.items.len() && !self.items.is_empty() {
            self.selected = self.items.len() - 1;
        }

        self.ensure_visible();
        Some(removed)
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.selected = 0;
        self.offset = 0;
    }
}

impl<T: ToString> List<T> {
    /// Render the list to a buffer
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
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
        let end = min(self.offset + self.height, self.items.len());

        // Render visible items
        for (i, item_index) in (self.offset..end).enumerate() {
            if let Some(item) = self.items.get(item_index) {
                let y = inner.y + i as u16;

                // Determine style
                let style = if item_index == self.selected && self.options.highlight_selection {
                    self.options.selected_style
                } else {
                    self.options.item_style
                };

                // Render the item
                let text = item.to_string();
                let line = Line::from(Span::styled(text, style));
                buf.set_line(inner.x, y, &line, inner.width);
            }
        }

        // Draw scrollbar if needed
        if self.items.len() > self.height {
            let scrollbar_x = inner.x + inner.width - 1;
            let scrollbar_height = inner.height;

            // Calculate thumb size and position
            let thumb_height = max(
                1,
                (self.height * scrollbar_height as usize) / self.items.len(),
            ) as u16;
            let thumb_pos = ((self.offset * scrollbar_height as usize) / self.items.len()) as u16;

            // Draw scrollbar track
            for y in 0..scrollbar_height {
                buf[(scrollbar_x, inner.y + y)]
                    .set_char('│')
                    .set_style(Style::default().fg(Color::DarkGray));
            }

            // Draw scrollbar thumb
            for y in thumb_pos..min(thumb_pos + thumb_height, scrollbar_height) {
                buf[(scrollbar_x, inner.y + y)]
                    .set_char('█')
                    .set_style(Style::default().fg(Color::Gray));
            }
        }
    }
}

use std::cmp::max;

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::{Block, Borders};

    #[test]
    fn test_list_creation() {
        let items = vec!["Item 1", "Item 2", "Item 3"];
        let list = List::new(items);

        assert_eq!(list.len(), 3);
        assert_eq!(list.selected(), 0);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_list_navigation() {
        let items = vec!["A", "B", "C", "D"];
        let mut list = List::new(items);

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
    fn test_list_wrap_around() {
        let items = vec![1, 2, 3];
        let mut list = List::new(items).with_options(ListOptions {
            wrap_around: true,
            ..Default::default()
        });

        // Test wrap from beginning
        list.select_previous();
        assert_eq!(list.selected(), 2);

        // Test wrap from end
        list.select_next();
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn test_list_modification() {
        let mut list = List::new(vec!["A", "B", "C"]);

        // Test push
        list.push("D");
        assert_eq!(list.len(), 4);

        // Test remove
        list.select(1); // Select "B"
        let removed = list.remove_selected();
        assert_eq!(removed, Some("B"));
        assert_eq!(list.len(), 3);
        assert_eq!(list.selected(), 1); // Should now point to "C"

        // Test clear
        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn test_list_builder_pattern() {
        let items = vec![1, 2, 3];
        let block = Block::default().borders(Borders::ALL).title("Test List");
        let options = ListOptions {
            wrap_around: true,
            page_size: 5,
            ..Default::default()
        };

        let list = List::new(items.clone())
            .with_block(block.clone())
            .with_options(options.clone());

        assert_eq!(list.len(), 3);
        assert!(list.block.is_some());
        assert!(list.options.wrap_around);
        assert_eq!(list.options.page_size, 5);
    }

    #[test]
    fn test_list_select_bounds() {
        let items = vec!["A", "B", "C"];
        let mut list = List::new(items);

        // Test valid selection
        list.select(1);
        assert_eq!(list.selected(), 1);

        // Test out of bounds selection
        list.select(10);
        assert_eq!(list.selected(), 1); // Should stay at previous

        // Test select_first
        list.select_first();
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn test_list_page_navigation() {
        let items: Vec<i32> = (1..=20).collect();
        let mut list = List::new(items).with_options(ListOptions {
            page_size: 5,
            ..Default::default()
        });

        // Start at 0
        assert_eq!(list.selected(), 0);

        // Page down
        list.page_down();
        assert_eq!(list.selected(), 5);

        // Page down again
        list.page_down();
        assert_eq!(list.selected(), 10);

        // Page up
        list.page_up();
        assert_eq!(list.selected(), 5);

        // Page up to top
        list.page_up();
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn test_list_page_navigation_at_boundaries() {
        let items: Vec<i32> = (1..=10).collect();
        let mut list = List::new(items).with_options(ListOptions {
            page_size: 3,
            ..Default::default()
        });

        // Navigate to end
        list.select_last();
        assert_eq!(list.selected(), 9);

        // Page down at end should stay at end
        list.page_down();
        assert_eq!(list.selected(), 9);

        // Page up from end
        list.page_up();
        assert_eq!(list.selected(), 6);
    }

    #[test]
    fn test_list_empty_operations() {
        let mut list: List<String> = List::new(vec![]);

        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.selected(), 0);
        assert_eq!(list.selected_item(), None);

        // Navigation on empty list should be safe
        list.select_next();
        assert_eq!(list.selected(), 0);

        list.select_previous();
        assert_eq!(list.selected(), 0);

        list.page_down();
        assert_eq!(list.selected(), 0);

        list.page_up();
        assert_eq!(list.selected(), 0);

        // Remove from empty list
        assert_eq!(list.remove_selected(), None);
    }

    #[test]
    fn test_list_selected_item_mut() {
        let mut list = List::new(vec!["A", "B", "C"]);

        // Modify selected item
        if let Some(item) = list.selected_item_mut() {
            *item = "Modified";
        }

        assert_eq!(list.selected_item(), Some(&"Modified"));
    }

    #[test]
    fn test_list_offset_management() {
        let items: Vec<i32> = (1..=30).collect();
        let mut list = List::new(items);
        list.height = 10; // Simulate viewport height

        // Initially offset should be 0
        assert_eq!(list.offset, 0);

        // Navigate down past viewport
        for _ in 0..15 {
            list.select_next();
        }

        // Offset should have adjusted
        assert!(list.offset > 0);

        // Selected should be visible
        assert!(list.selected >= list.offset);
        assert!(list.selected < list.offset + list.height);
    }

    #[test]
    fn test_list_ensure_visible() {
        let items: Vec<i32> = (1..=20).collect();
        let mut list = List::new(items);
        list.height = 5;

        // Select item beyond viewport
        list.selected = 10;
        list.ensure_visible();

        // Item should now be visible
        assert!(list.selected >= list.offset);
        assert!(list.selected < list.offset + list.height);

        // Select item before viewport
        list.offset = 10;
        list.selected = 2;
        list.ensure_visible();

        // Offset should adjust to show selected item
        assert_eq!(list.offset, 2);
    }

    #[test]
    fn test_list_remove_last_item() {
        let mut list = List::new(vec!["Only"]);

        let removed = list.remove_selected();
        assert_eq!(removed, Some("Only"));
        assert!(list.is_empty());
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn test_list_remove_adjusts_selection() {
        let mut list = List::new(vec!["A", "B", "C", "D"]);

        // Select last item
        list.select_last();
        assert_eq!(list.selected(), 3);

        // Remove last item
        let removed = list.remove_selected();
        assert_eq!(removed, Some("D"));

        // Selection should adjust to new last item
        assert_eq!(list.selected(), 2);
        assert_eq!(list.selected_item(), Some(&"C"));
    }

    #[test]
    fn test_list_wrap_with_single_navigation() {
        let items: Vec<i32> = (1..=10).collect();
        let mut list = List::new(items).with_options(ListOptions {
            wrap_around: true,
            page_size: 3,
            ..Default::default()
        });

        // Test single-item navigation wrapping
        // From start, previous should wrap to end
        list.select_previous();
        assert_eq!(list.selected(), 9); // Should wrap to last item

        // From end, next should wrap to start
        list.select_next();
        assert_eq!(list.selected(), 0); // Should wrap to first item
    }

    #[test]
    fn test_list_rendering() {
        use ratatui::buffer::Buffer;

        let items = vec!["Item 1", "Item 2", "Item 3"];
        let mut list = List::new(items);

        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 10));
        list.render(Rect::new(0, 0, 20, 10), &mut buf);

        // Basic smoke test - buffer should have content
        assert!(!buf.content().is_empty());
    }

    #[test]
    fn test_list_default_options() {
        let options = ListOptions::default();
        assert!(!options.wrap_around);
        assert_eq!(options.page_size, 10);
        assert!(options.highlight_selection);
    }

    #[test]
    fn test_list_mouse_support() {
        use crate::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        let items = vec!["A", "B", "C", "D", "E"];
        let mut list = List::new(items);
        list.height = 5;
        list.offset = 0;
        list.focused = true; // Need to be focused to handle mouse

        // Simulate click on third item (index 2)
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 2,
            modifiers: KeyModifiers::empty(),
        };

        let handled = list.handle_mouse(&mouse_event, Rect::new(0, 0, 20, 5));
        assert!(handled);
        assert_eq!(list.selected(), 2);

        // Test scroll down
        let scroll_event = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 5,
            row: 2,
            modifiers: KeyModifiers::empty(),
        };

        list.handle_mouse(&scroll_event, Rect::new(0, 0, 20, 5));
        assert_eq!(list.selected(), 3);

        // Test scroll up
        let scroll_event = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 5,
            row: 2,
            modifiers: KeyModifiers::empty(),
        };

        list.handle_mouse(&scroll_event, Rect::new(0, 0, 20, 5));
        assert_eq!(list.selected(), 2);
    }
}
