//! Test for improved MouseEvent API with helper methods

use hojicha_core::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

#[test]
fn test_mouse_click_helpers() {
    let left_click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 20,
        modifiers: KeyModifiers::empty(),
    };
    
    assert!(left_click.is_left_click());
    assert!(left_click.is_click());
    assert!(!left_click.is_right_click());
    assert!(!left_click.is_release());
    assert!(!left_click.is_drag());
    
    let right_click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Right),
        column: 10,
        row: 20,
        modifiers: KeyModifiers::empty(),
    };
    
    assert!(right_click.is_right_click());
    assert!(right_click.is_click());
    assert!(!right_click.is_left_click());
}

#[test]
fn test_mouse_scroll_helpers() {
    let scroll_up = MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 5,
        row: 10,
        modifiers: KeyModifiers::empty(),
    };
    
    assert!(scroll_up.is_scroll_up());
    assert!(scroll_up.is_scroll());
    assert!(!scroll_up.is_scroll_down());
    assert!(!scroll_up.is_click());
    
    let scroll_down = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 5,
        row: 10,
        modifiers: KeyModifiers::empty(),
    };
    
    assert!(scroll_down.is_scroll_down());
    assert!(scroll_down.is_scroll());
}

#[test]
fn test_mouse_position_helper() {
    let event = MouseEvent {
        kind: MouseEventKind::Moved,
        column: 42,
        row: 13,
        modifiers: KeyModifiers::empty(),
    };
    
    assert_eq!(event.position(), (42, 13));
    assert!(event.is_move());
}

#[test]
fn test_mouse_modifier_helper() {
    let event_with_ctrl = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 0,
        row: 0,
        modifiers: KeyModifiers::CONTROL,
    };
    
    assert!(event_with_ctrl.has_modifier(KeyModifiers::CONTROL));
    assert!(!event_with_ctrl.has_modifier(KeyModifiers::SHIFT));
    
    let event_with_multiple = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 0,
        row: 0,
        modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    };
    
    assert!(event_with_multiple.has_modifier(KeyModifiers::CONTROL));
    assert!(event_with_multiple.has_modifier(KeyModifiers::SHIFT));
}

#[test]
fn test_mouse_drag_and_release() {
    let drag = MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: 15,
        row: 25,
        modifiers: KeyModifiers::empty(),
    };
    
    assert!(drag.is_drag());
    assert!(!drag.is_click());
    assert!(!drag.is_release());
    
    let release = MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: 15,
        row: 25,
        modifiers: KeyModifiers::empty(),
    };
    
    assert!(release.is_release());
    assert!(!release.is_click());
    assert!(!release.is_drag());
}