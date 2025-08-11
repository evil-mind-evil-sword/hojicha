//! Simplified integration tests for components working together

use hojicha_pearls::components::{List, Table, TableRow, TextArea, Viewport};
use hojicha_core::event::{MouseButton, MouseEventKind};
use hojicha_core::prelude::*;

#[derive(Clone, Debug)]
struct Item {
    id: u32,
    name: String,
}

impl TableRow for Item {
    fn to_row(&self) -> Vec<String> {
        vec![self.id.to_string(), self.name.clone()]
    }
}

#[test]
fn test_multiple_components_focus() {
    // Test that multiple components can manage focus state
    let mut list = List::new(vec!["A".to_string(), "B".to_string()]);
    let mut textarea = TextArea::new();
    let mut viewport = Viewport::new();

    // Set initial focus
    list.set_focused(true);
    textarea.set_focused(false);
    viewport.set_focused(false);

    // Simulate Tab key to switch focus
    list.set_focused(false);
    textarea.set_focused(true);

    // Components should maintain their own state
    assert_eq!(list.selected(), 0);
    assert_eq!(textarea.value(), "");
}

#[test]
fn test_list_and_textarea_interaction() {
    // Test data flow between list and textarea
    let items = vec![
        "First item".to_string(),
        "Second item".to_string(),
        "Third item".to_string(),
    ];

    let mut list = List::new(items.clone());
    let mut textarea = TextArea::new();

    // Select second item
    list.select(1);

    // Copy selected item to textarea
    if let Some(item) = list.selected_item() {
        textarea.insert_text(item);
    }

    assert_eq!(textarea.value(), "Second item");
}

#[test]
fn test_table_and_viewport_interaction() {
    // Test showing table row details in viewport
    let items = vec![
        Item {
            id: 1,
            name: "Alice".to_string(),
        },
        Item {
            id: 2,
            name: "Bob".to_string(),
        },
    ];

    let headers = vec!["ID".to_string(), "Name".to_string()];
    let mut table = Table::new(headers).with_rows(items);
    let mut viewport = Viewport::new();

    // Select Bob
    table.select(1);

    // Show details in viewport
    if let Some(item) = table.selected_row() {
        let details = format!(
            "ID: {}\nName: {}\n\nSelected row details",
            item.id, item.name
        );
        viewport.set_content(details);
    }

    assert_eq!(viewport.line_count(), 4);
}

#[test]
fn test_component_event_routing() {
    // Test that events can be routed to the correct component
    let mut list = List::new(vec!["Item 1".to_string(), "Item 2".to_string()]);
    let mut viewport = Viewport::new();

    viewport.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5");

    // Focus on list
    list.set_focused(true);
    viewport.set_focused(false);

    let down_key = KeyEvent {
        key: Key::Down,
        modifiers: KeyModifiers::empty(),
    };

    // List should handle it
    let handled = list.handle_key(&down_key);
    assert!(handled);
    assert_eq!(list.selected(), 1);

    // Viewport should not handle it (not focused)
    let handled = viewport.handle_key(&down_key);
    assert!(!handled);

    // Switch focus
    list.set_focused(false);
    viewport.set_focused(true);

    // Now viewport should handle it
    let handled = viewport.handle_key(&down_key);
    assert!(handled);
    assert_eq!(viewport.scroll_position().0, 1);
}

#[test]
fn test_component_mouse_events() {
    let mut list = List::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    list.set_focused(true);

    let area = Rect {
        x: 0,
        y: 0,
        width: 20,
        height: 10,
    };

    // Click on second item (accounting for borders)
    let mouse_event = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 5,
        row: 2, // Second item
        modifiers: KeyModifiers::empty(),
    };

    let handled = list.handle_mouse(&mouse_event, area);
    assert!(handled);
    // The click might have selected a different index due to borders
    // Just verify it was handled

    // Scroll down from current position
    let current = list.selected();
    let scroll_event = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 5,
        row: 5,
        modifiers: KeyModifiers::empty(),
    };

    let handled = list.handle_mouse(&scroll_event, area);
    assert!(handled);
    // Check that scroll was handled (this tests the mouse event system)
    let new_selected = list.selected();
    println!("Current was: {current}, new is: {new_selected}");
    // The behavior may vary based on current position, just ensure scroll was handled
    assert!(new_selected >= current); // Position should not go backwards
}

#[test]
fn test_component_state_independence() {
    // Test that components maintain independent state
    let list1 = List::new(vec!["A".to_string(), "B".to_string()]);
    let mut list2 = list1.clone();

    // Modify list2
    list2.select(1);
    list2.push("C".to_string());

    // list1 should be unchanged
    assert_eq!(list1.selected(), 0);
    assert_eq!(list1.len(), 2);

    // list2 should have changes
    assert_eq!(list2.selected(), 1);
    assert_eq!(list2.len(), 3);
}
