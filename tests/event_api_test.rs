//! Tests for the improved Event API helper methods

use hojicha_core::event::{Event, Key, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

#[test]
fn test_event_key_helpers() {
    let key_event = KeyEvent::new(Key::Enter, KeyModifiers::empty());
    let event = Event::<String>::Key(key_event);
    
    assert!(event.is_key());
    assert!(event.is_key_press(Key::Enter));
    assert!(!event.is_key_press(Key::Esc));
    assert!(event.as_key().is_some());
    assert!(!event.is_mouse());
}

#[test]
fn test_event_key_with_modifiers() {
    let key_event = KeyEvent::new(Key::Char('s'), KeyModifiers::CONTROL);
    let event = Event::<String>::Key(key_event);
    
    assert!(event.is_key_with_modifiers(Key::Char('s'), KeyModifiers::CONTROL));
    assert!(!event.is_key_with_modifiers(Key::Char('s'), KeyModifiers::ALT));
    assert!(!event.is_key_with_modifiers(Key::Char('a'), KeyModifiers::CONTROL));
}

#[test]
fn test_event_mouse_helpers() {
    let mouse_event = MouseEvent::new(
        MouseEventKind::Down(MouseButton::Left),
        10,
        20,
        KeyModifiers::empty()
    );
    let event = Event::<String>::Mouse(mouse_event);
    
    assert!(event.is_mouse());
    assert!(event.is_click());
    assert_eq!(event.as_click(), Some((10, 20)));
    assert!(event.as_mouse().is_some());
    assert!(!event.is_key());
}

#[test]
fn test_event_resize_helpers() {
    let event = Event::<String>::Resize { width: 100, height: 50 };
    
    assert!(event.is_resize());
    assert_eq!(event.as_resize(), Some((100, 50)));
    assert!(!event.is_key());
    assert!(!event.is_mouse());
}

#[test]
fn test_event_user_helpers() {
    let event = Event::User("test message".to_string());
    
    assert!(event.is_user());
    assert_eq!(event.as_user(), Some(&"test message".to_string()));
    
    let event_clone = event.clone();
    assert_eq!(event_clone.into_user(), Some("test message".to_string()));
}

#[test]
fn test_event_simple_variants() {
    assert!(Event::<String>::Quit.is_quit());
    assert!(Event::<String>::Tick.is_tick());
    assert!(Event::<String>::Focus.is_focus());
    assert!(Event::<String>::Blur.is_blur());
    assert!(Event::<String>::Suspend.is_suspend());
    assert!(Event::<String>::Resume.is_resume());
    
    let paste_event = Event::<String>::Paste("hello world".to_string());
    assert!(paste_event.is_paste());
    assert_eq!(paste_event.as_paste(), Some("hello world"));
}

#[test]
fn test_key_event_helpers() {
    let key = KeyEvent::new(Key::Char('a'), KeyModifiers::empty());
    
    assert!(key.is(Key::Char('a')));
    assert!(!key.is(Key::Char('b')));
    assert!(key.no_modifiers());
    assert!(!key.has_modifiers());
    assert!(key.is_char());
    assert_eq!(key.char(), Some('a'));
}

#[test]
fn test_key_event_modifiers() {
    let key = KeyEvent::new(
        Key::Char('s'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT
    );
    
    assert!(key.is_ctrl());
    assert!(key.is_shift());
    assert!(!key.is_alt());
    assert!(!key.is_super());
    assert!(key.has_modifiers());
    assert!(!key.no_modifiers());
    assert!(key.is_with_modifiers(
        Key::Char('s'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT
    ));
}

#[test]
fn test_key_event_categories() {
    // Navigation keys
    assert!(KeyEvent::new(Key::Up, KeyModifiers::empty()).is_navigation());
    assert!(KeyEvent::new(Key::Down, KeyModifiers::empty()).is_navigation());
    assert!(KeyEvent::new(Key::Left, KeyModifiers::empty()).is_navigation());
    assert!(KeyEvent::new(Key::Right, KeyModifiers::empty()).is_navigation());
    assert!(KeyEvent::new(Key::Home, KeyModifiers::empty()).is_navigation());
    assert!(KeyEvent::new(Key::End, KeyModifiers::empty()).is_navigation());
    assert!(KeyEvent::new(Key::PageUp, KeyModifiers::empty()).is_navigation());
    assert!(KeyEvent::new(Key::PageDown, KeyModifiers::empty()).is_navigation());
    assert!(!KeyEvent::new(Key::Char('a'), KeyModifiers::empty()).is_navigation());
    
    // Function keys
    assert!(KeyEvent::new(Key::F(1), KeyModifiers::empty()).is_function_key());
    assert!(KeyEvent::new(Key::F(12), KeyModifiers::empty()).is_function_key());
    assert!(!KeyEvent::new(Key::Char('a'), KeyModifiers::empty()).is_function_key());
    
    // Media keys
    assert!(KeyEvent::new(Key::MediaPlay, KeyModifiers::empty()).is_media_key());
    assert!(KeyEvent::new(Key::MediaPause, KeyModifiers::empty()).is_media_key());
    assert!(KeyEvent::new(Key::MediaVolumeUp, KeyModifiers::empty()).is_media_key());
    assert!(!KeyEvent::new(Key::Char('a'), KeyModifiers::empty()).is_media_key());
}

#[test]
fn test_mouse_event_clicks() {
    let left_click = MouseEvent::new(
        MouseEventKind::Down(MouseButton::Left),
        10, 20,
        KeyModifiers::empty()
    );
    assert!(left_click.is_left_click());
    assert!(left_click.is_click());
    assert!(!left_click.is_right_click());
    assert!(!left_click.is_middle_click());
    
    let right_click = MouseEvent::new(
        MouseEventKind::Down(MouseButton::Right),
        10, 20,
        KeyModifiers::empty()
    );
    assert!(right_click.is_right_click());
    assert!(right_click.is_click());
    assert!(!right_click.is_left_click());
    
    let middle_click = MouseEvent::new(
        MouseEventKind::Down(MouseButton::Middle),
        10, 20,
        KeyModifiers::empty()
    );
    assert!(middle_click.is_middle_click());
    assert!(middle_click.is_click());
}

#[test]
fn test_mouse_event_drag() {
    let left_drag = MouseEvent::new(
        MouseEventKind::Drag(MouseButton::Left),
        15, 25,
        KeyModifiers::empty()
    );
    assert!(left_drag.is_drag());
    assert!(left_drag.is_left_drag());
    assert!(!left_drag.is_right_drag());
    assert!(!left_drag.is_middle_drag());
    assert_eq!(left_drag.button(), Some(MouseButton::Left));
}

#[test]
fn test_mouse_event_scroll() {
    let scroll_up = MouseEvent::new(
        MouseEventKind::ScrollUp,
        10, 20,
        KeyModifiers::empty()
    );
    assert!(scroll_up.is_scroll_up());
    assert!(scroll_up.is_scroll());
    assert!(!scroll_up.is_scroll_down());
    
    let scroll_down = MouseEvent::new(
        MouseEventKind::ScrollDown,
        10, 20,
        KeyModifiers::empty()
    );
    assert!(scroll_down.is_scroll_down());
    assert!(scroll_down.is_scroll());
}

#[test]
fn test_mouse_event_position() {
    let mouse = MouseEvent::new(
        MouseEventKind::Down(MouseButton::Left),
        42, 24,
        KeyModifiers::empty()
    );
    
    assert_eq!(mouse.position(), (42, 24));
    assert!(mouse.is_at(42, 24));
    assert!(!mouse.is_at(41, 24));
    
    // Test is_within
    assert!(mouse.is_within(40, 20, 10, 10)); // Within bounds
    assert!(!mouse.is_within(50, 20, 10, 10)); // Outside bounds
    assert!(mouse.is_within(0, 0, 100, 100)); // Within large area
}

#[test]
fn test_mouse_event_modifiers() {
    let mouse_with_ctrl = MouseEvent::new(
        MouseEventKind::Down(MouseButton::Left),
        10, 20,
        KeyModifiers::CONTROL
    );
    
    assert!(mouse_with_ctrl.has_modifier(KeyModifiers::CONTROL));
    assert!(mouse_with_ctrl.is_ctrl());
    assert!(!mouse_with_ctrl.is_alt());
    assert!(!mouse_with_ctrl.is_shift());
    assert!(mouse_with_ctrl.has_modifiers());
    assert!(!mouse_with_ctrl.no_modifiers());
    
    let mouse_no_mods = MouseEvent::new(
        MouseEventKind::Down(MouseButton::Left),
        10, 20,
        KeyModifiers::empty()
    );
    
    assert!(!mouse_no_mods.has_modifiers());
    assert!(mouse_no_mods.no_modifiers());
}

#[test]
fn test_mouse_event_release_and_move() {
    let release = MouseEvent::new(
        MouseEventKind::Up(MouseButton::Left),
        10, 20,
        KeyModifiers::empty()
    );
    assert!(release.is_release());
    assert!(!release.is_click());
    assert_eq!(release.button(), Some(MouseButton::Left));
    
    let mouse_move = MouseEvent::new(
        MouseEventKind::Moved,
        30, 40,
        KeyModifiers::empty()
    );
    assert!(mouse_move.is_move());
    assert!(!mouse_move.is_drag());
    assert_eq!(mouse_move.button(), None);
}