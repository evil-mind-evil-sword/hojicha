//! Event handling for keyboard, mouse, and other terminal events

use crossterm::event::KeyCode;
pub use crossterm::event::{KeyModifiers, MouseButton, MouseEventKind};

/// An event that can be received by the program
#[derive(Debug, Clone, PartialEq)]
pub enum Event<M> {
    /// A keyboard event
    Key(KeyEvent),
    /// A mouse event  
    Mouse(MouseEvent),
    /// Terminal was resized
    Resize {
        /// New terminal width in columns
        width: u16,
        /// New terminal height in rows
        height: u16,
    },
    /// A tick event (for animations, etc.)
    Tick,
    /// User-defined message
    User(M),
    /// Request to quit the program
    Quit,
    /// Terminal gained focus
    Focus,
    /// Terminal lost focus
    Blur,
    /// Program suspend request (Ctrl+Z)
    Suspend,
    /// Program resumed from suspend
    Resume,
    /// Bracketed paste event
    Paste(String),
    /// Internal event to trigger external process execution
    #[doc(hidden)]
    ExecProcess,
}

/// Window size information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSize {
    /// Width in columns
    pub width: u16,
    /// Height in rows
    pub height: u16,
}

/// A keyboard event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    /// The key that was pressed
    pub key: Key,
    /// Key modifiers (Ctrl, Alt, Shift)
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    /// Create a new key event
    pub fn new(key: Key, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    /// Check if this is a simple character key press
    pub fn is_char(&self) -> bool {
        matches!(self.key, Key::Char(_))
    }

    /// Get the character if this is a character key
    pub fn char(&self) -> Option<char> {
        match self.key {
            Key::Char(c) => Some(c),
            _ => None,
        }
    }
}

/// Modifier key types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierKey {
    /// Shift key
    Shift,
    /// Control key
    Control,
    /// Alt/Option key
    Alt,
    /// Super/Windows/Command key
    Super,
    /// Meta key
    Meta,
    /// Hyper key
    Hyper,
}

/// Represents a key on the keyboard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A character key
    Char(char),
    /// Backspace key
    Backspace,
    /// Enter/Return key
    Enter,
    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Home key
    Home,
    /// End key
    End,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Tab key
    Tab,
    /// Delete key
    Delete,
    /// Insert key
    Insert,
    /// Escape key
    Esc,
    /// Function keys
    F(u8),
    /// Null key (usually Ctrl+@)
    Null,
    /// Caps Lock key
    CapsLock,
    /// Scroll Lock key
    ScrollLock,
    /// Num Lock key
    NumLock,
    /// Print Screen key
    PrintScreen,
    /// Pause/Break key
    Pause,
    /// Menu/Application key
    Menu,
    /// Keypad Begin (5 on keypad with NumLock off)
    KeypadBegin,
    /// Media Play
    MediaPlay,
    /// Media Pause
    MediaPause,
    /// Media Play/Pause toggle
    MediaPlayPause,
    /// Media Stop
    MediaStop,
    /// Media Next Track
    MediaNext,
    /// Media Previous Track
    MediaPrevious,
    /// Media Fast Forward
    MediaFastForward,
    /// Media Rewind
    MediaRewind,
    /// Media Volume Up
    MediaVolumeUp,
    /// Media Volume Down
    MediaVolumeDown,
    /// Media Mute
    MediaMute,
    /// Modifier key (Shift, Ctrl, Alt, Super/Meta)
    Modifier(ModifierKey),
}

impl From<crossterm::event::KeyEvent> for KeyEvent {
    fn from(event: crossterm::event::KeyEvent) -> Self {
        let key = match event.code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Enter => Key::Enter,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Tab => Key::Tab,
            KeyCode::BackTab => Key::Tab, // BackTab is Shift+Tab
            KeyCode::Delete => Key::Delete,
            KeyCode::Insert => Key::Insert,
            KeyCode::Esc => Key::Esc,
            KeyCode::F(n) => Key::F(n),
            KeyCode::Null => Key::Null,
            KeyCode::CapsLock => Key::CapsLock,
            KeyCode::ScrollLock => Key::ScrollLock,
            KeyCode::NumLock => Key::NumLock,
            KeyCode::PrintScreen => Key::PrintScreen,
            KeyCode::Pause => Key::Pause,
            KeyCode::Menu => Key::Menu,
            KeyCode::KeypadBegin => Key::KeypadBegin,
            // Media keys - crossterm doesn't have all of these, so we handle what we can
            KeyCode::Media(crossterm::event::MediaKeyCode::Play) => Key::MediaPlay,
            KeyCode::Media(crossterm::event::MediaKeyCode::Pause) => Key::MediaPause,
            KeyCode::Media(crossterm::event::MediaKeyCode::PlayPause) => Key::MediaPlayPause,
            KeyCode::Media(crossterm::event::MediaKeyCode::Stop) => Key::MediaStop,
            KeyCode::Media(crossterm::event::MediaKeyCode::FastForward) => Key::MediaFastForward,
            KeyCode::Media(crossterm::event::MediaKeyCode::Rewind) => Key::MediaRewind,
            KeyCode::Media(crossterm::event::MediaKeyCode::TrackNext) => Key::MediaNext,
            KeyCode::Media(crossterm::event::MediaKeyCode::TrackPrevious) => Key::MediaPrevious,
            KeyCode::Media(crossterm::event::MediaKeyCode::LowerVolume) => Key::MediaVolumeDown,
            KeyCode::Media(crossterm::event::MediaKeyCode::RaiseVolume) => Key::MediaVolumeUp,
            KeyCode::Media(crossterm::event::MediaKeyCode::MuteVolume) => Key::MediaMute,
            // Modifier keys - these are usually not sent as separate events but we can handle them
            KeyCode::Modifier(crossterm::event::ModifierKeyCode::LeftShift)
            | KeyCode::Modifier(crossterm::event::ModifierKeyCode::RightShift) => {
                Key::Modifier(ModifierKey::Shift)
            }
            KeyCode::Modifier(crossterm::event::ModifierKeyCode::LeftControl)
            | KeyCode::Modifier(crossterm::event::ModifierKeyCode::RightControl) => {
                Key::Modifier(ModifierKey::Control)
            }
            KeyCode::Modifier(crossterm::event::ModifierKeyCode::LeftAlt)
            | KeyCode::Modifier(crossterm::event::ModifierKeyCode::RightAlt) => {
                Key::Modifier(ModifierKey::Alt)
            }
            KeyCode::Modifier(crossterm::event::ModifierKeyCode::LeftSuper)
            | KeyCode::Modifier(crossterm::event::ModifierKeyCode::RightSuper) => {
                Key::Modifier(ModifierKey::Super)
            }
            KeyCode::Modifier(crossterm::event::ModifierKeyCode::LeftMeta)
            | KeyCode::Modifier(crossterm::event::ModifierKeyCode::RightMeta) => {
                Key::Modifier(ModifierKey::Meta)
            }
            KeyCode::Modifier(crossterm::event::ModifierKeyCode::LeftHyper)
            | KeyCode::Modifier(crossterm::event::ModifierKeyCode::RightHyper) => {
                Key::Modifier(ModifierKey::Hyper)
            }
            _ => Key::Null, // Map unmapped keys to Null
        };

        Self {
            key,
            modifiers: event.modifiers,
        }
    }
}

/// A mouse event
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseEvent {
    /// The kind of mouse event
    pub kind: MouseEventKind,
    /// Column (x coordinate)
    pub column: u16,
    /// Row (y coordinate)
    pub row: u16,
    /// Key modifiers held during the event
    pub modifiers: KeyModifiers,
}

impl From<crossterm::event::MouseEvent> for MouseEvent {
    fn from(event: crossterm::event::MouseEvent) -> Self {
        Self {
            kind: event.kind,
            column: event.column,
            row: event.row,
            modifiers: event.modifiers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_key_event_creation() {
        let event = KeyEvent::new(Key::Char('a'), KeyModifiers::empty());
        assert_eq!(event.key, Key::Char('a'));
        assert!(event.is_char());
        assert_eq!(event.char(), Some('a'));
    }

    #[test]
    fn test_key_event_modifiers() {
        let event = KeyEvent::new(Key::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(event.key, Key::Char('c'));
        assert_eq!(event.modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn test_non_char_keys() {
        let event = KeyEvent::new(Key::Enter, KeyModifiers::empty());
        assert!(!event.is_char());
        assert_eq!(event.char(), None);
    }

    #[test]
    fn test_window_size_creation() {
        let size = WindowSize {
            width: 80,
            height: 24,
        };
        assert_eq!(size.width, 80);
        assert_eq!(size.height, 24);
    }

    #[test]
    fn test_mouse_event_creation() {
        let event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::empty(),
        };
        assert_eq!(event.column, 10);
        assert_eq!(event.row, 5);
    }

    #[test]
    fn test_event_variants() {
        let key_event = Event::<String>::Key(KeyEvent::new(Key::Char('a'), KeyModifiers::empty()));
        let mouse_event = Event::<String>::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::empty(),
        });
        let resize_event = Event::<String>::Resize {
            width: 80,
            height: 24,
        };
        let tick_event = Event::<String>::Tick;
        let user_event = Event::User("test".to_string());
        let quit_event = Event::<String>::Quit;
        let focus_event = Event::<String>::Focus;
        let blur_event = Event::<String>::Blur;
        let suspend_event = Event::<String>::Suspend;
        let resume_event = Event::<String>::Resume;
        let paste_event = Event::<String>::Paste("pasted text".to_string());

        // Test that all variants can be created and pattern matched
        match key_event {
            Event::Key(_) => {}
            _ => panic!("Expected Key event"),
        }
        match mouse_event {
            Event::Mouse(_) => {}
            _ => panic!("Expected Mouse event"),
        }
        match resize_event {
            Event::Resize { .. } => {}
            _ => panic!("Expected Resize event"),
        }
        match tick_event {
            Event::Tick => {}
            _ => panic!("Expected Tick event"),
        }
        match user_event {
            Event::User(_) => {}
            _ => panic!("Expected User event"),
        }
        match quit_event {
            Event::Quit => {}
            _ => panic!("Expected Quit event"),
        }
        match focus_event {
            Event::Focus => {}
            _ => panic!("Expected Focus event"),
        }
        match blur_event {
            Event::Blur => {}
            _ => panic!("Expected Blur event"),
        }
        match suspend_event {
            Event::Suspend => {}
            _ => panic!("Expected Suspend event"),
        }
        match resume_event {
            Event::Resume => {}
            _ => panic!("Expected Resume event"),
        }
        match paste_event {
            Event::Paste(_) => {}
            _ => panic!("Expected Paste event"),
        }
    }

    #[test]
    fn test_key_variants() {
        let keys = vec![
            Key::Char('a'),
            Key::Backspace,
            Key::Enter,
            Key::Left,
            Key::Right,
            Key::Up,
            Key::Down,
            Key::Home,
            Key::End,
            Key::PageUp,
            Key::PageDown,
            Key::Tab,
            Key::Delete,
            Key::Insert,
            Key::Esc,
            Key::F(1),
            Key::Null,
        ];

        for key in keys {
            let event = KeyEvent::new(key, KeyModifiers::empty());
            assert_eq!(event.key, key);

            // Test is_char and char methods
            match key {
                Key::Char(c) => {
                    assert!(event.is_char());
                    assert_eq!(event.char(), Some(c));
                }
                _ => {
                    assert!(!event.is_char());
                    assert_eq!(event.char(), None);
                }
            }
        }
    }

    #[test]
    fn test_crossterm_key_conversion() {
        let crossterm_event = crossterm::event::KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );

        let key_event: KeyEvent = crossterm_event.into();
        assert_eq!(key_event.key, Key::Char('x'));
        assert_eq!(
            key_event.modifiers,
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        );
    }

    #[test]
    fn test_enhanced_keys() {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Test special keys
        let caps_lock = crossterm::event::KeyEvent::new(KeyCode::CapsLock, KeyModifiers::empty());
        let key_event = KeyEvent::from(caps_lock);
        assert_eq!(key_event.key, Key::CapsLock);

        let scroll_lock =
            crossterm::event::KeyEvent::new(KeyCode::ScrollLock, KeyModifiers::empty());
        let key_event = KeyEvent::from(scroll_lock);
        assert_eq!(key_event.key, Key::ScrollLock);

        let print_screen =
            crossterm::event::KeyEvent::new(KeyCode::PrintScreen, KeyModifiers::empty());
        let key_event = KeyEvent::from(print_screen);
        assert_eq!(key_event.key, Key::PrintScreen);

        // Test media keys
        let play = crossterm::event::KeyEvent::new(
            KeyCode::Media(crossterm::event::MediaKeyCode::Play),
            KeyModifiers::empty(),
        );
        let key_event = KeyEvent::from(play);
        assert_eq!(key_event.key, Key::MediaPlay);

        let volume_up = crossterm::event::KeyEvent::new(
            KeyCode::Media(crossterm::event::MediaKeyCode::RaiseVolume),
            KeyModifiers::empty(),
        );
        let key_event = KeyEvent::from(volume_up);
        assert_eq!(key_event.key, Key::MediaVolumeUp);

        // Test modifier keys
        let left_shift = crossterm::event::KeyEvent::new(
            KeyCode::Modifier(crossterm::event::ModifierKeyCode::LeftShift),
            KeyModifiers::empty(),
        );
        let key_event = KeyEvent::from(left_shift);
        assert_eq!(key_event.key, Key::Modifier(ModifierKey::Shift));

        let right_alt = crossterm::event::KeyEvent::new(
            KeyCode::Modifier(crossterm::event::ModifierKeyCode::RightAlt),
            KeyModifiers::empty(),
        );
        let key_event = KeyEvent::from(right_alt);
        assert_eq!(key_event.key, Key::Modifier(ModifierKey::Alt));
    }

    #[test]
    fn test_crossterm_key_conversion_all_keys() {
        // Test all key code conversions
        let test_cases = vec![
            (KeyCode::Backspace, Key::Backspace),
            (KeyCode::Enter, Key::Enter),
            (KeyCode::Left, Key::Left),
            (KeyCode::Right, Key::Right),
            (KeyCode::Up, Key::Up),
            (KeyCode::Down, Key::Down),
            (KeyCode::Home, Key::Home),
            (KeyCode::End, Key::End),
            (KeyCode::PageUp, Key::PageUp),
            (KeyCode::PageDown, Key::PageDown),
            (KeyCode::Tab, Key::Tab),
            (KeyCode::Delete, Key::Delete),
            (KeyCode::Insert, Key::Insert),
            (KeyCode::Esc, Key::Esc),
            (KeyCode::F(1), Key::F(1)),
            (KeyCode::F(12), Key::F(12)),
            (KeyCode::Null, Key::Null),
            (KeyCode::BackTab, Key::Tab), // BackTab is Shift+Tab
        ];

        for (crossterm_code, expected_key) in test_cases {
            let crossterm_event =
                crossterm::event::KeyEvent::new(crossterm_code, KeyModifiers::empty());
            let key_event: KeyEvent = crossterm_event.into();
            assert_eq!(key_event.key, expected_key);
        }
    }

    #[test]
    fn test_crossterm_mouse_conversion() {
        let crossterm_event = crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: 42,
            row: 24,
            modifiers: KeyModifiers::ALT,
        };

        let mouse_event: MouseEvent = crossterm_event.into();
        assert_eq!(mouse_event.kind, MouseEventKind::Down(MouseButton::Right));
        assert_eq!(mouse_event.column, 42);
        assert_eq!(mouse_event.row, 24);
        assert_eq!(mouse_event.modifiers, KeyModifiers::ALT);
    }

    // Property-based tests
    proptest! {
        #[test]
        fn test_key_event_properties(
            c in any::<char>(),
            ctrl in any::<bool>(),
            alt in any::<bool>(),
            shift in any::<bool>()
        ) {
            let mut modifiers = KeyModifiers::empty();
            if ctrl { modifiers |= KeyModifiers::CONTROL; }
            if alt { modifiers |= KeyModifiers::ALT; }
            if shift { modifiers |= KeyModifiers::SHIFT; }

            let event = KeyEvent::new(Key::Char(c), modifiers);

            prop_assert_eq!(event.key, Key::Char(c));
            prop_assert_eq!(event.modifiers, modifiers);
            prop_assert!(event.is_char());
            prop_assert_eq!(event.char(), Some(c));
        }

        #[test]
        fn test_window_size_properties(
            width in 1u16..1000u16,
            height in 1u16..1000u16
        ) {
            let size = WindowSize { width, height };

            prop_assert_eq!(size.width, width);
            prop_assert_eq!(size.height, height);

            // Test equality and cloning
            let size2 = size;
            prop_assert_eq!(size, size2);
        }

        #[test]
        fn test_mouse_event_properties(
            column in 0u16..1000u16,
            row in 0u16..1000u16,
            button_idx in 0..3usize
        ) {
            let button = match button_idx {
                0 => MouseButton::Left,
                1 => MouseButton::Right,
                2 => MouseButton::Middle,
                _ => unreachable!(),
            };

            let event = MouseEvent {
                kind: MouseEventKind::Down(button),
                column,
                row,
                modifiers: KeyModifiers::empty(),
            };

            prop_assert_eq!(event.column, column);
            prop_assert_eq!(event.row, row);
            prop_assert_eq!(event.kind, MouseEventKind::Down(button));
        }

        #[test]
        fn test_event_user_message_properties(
            message in ".*"
        ) {
            let event = Event::User(message.clone());

            match event {
                Event::User(msg) => prop_assert_eq!(msg, message),
                _ => prop_assert!(false, "Expected User event"),
            }
        }

        #[test]
        fn test_event_resize_properties(
            width in 1u16..1000u16,
            height in 1u16..1000u16
        ) {
            let event = Event::<String>::Resize { width, height };

            match event {
                Event::Resize { width: w, height: h } => {
                    prop_assert_eq!(w, width);
                    prop_assert_eq!(h, height);
                }
                _ => prop_assert!(false, "Expected Resize event"),
            }
        }

        #[test]
        fn test_event_paste_properties(
            paste_text in ".*"
        ) {
            let event = Event::<String>::Paste(paste_text.clone());

            match event {
                Event::Paste(text) => prop_assert_eq!(text, paste_text),
                _ => prop_assert!(false, "Expected Paste event"),
            }
        }

        #[test]
        fn test_function_key_properties(
            key_num in 1u8..25u8
        ) {
            let key = Key::F(key_num);
            let event = KeyEvent::new(key, KeyModifiers::empty());

            prop_assert_eq!(event.key, Key::F(key_num));
            prop_assert!(!event.is_char());
            prop_assert_eq!(event.char(), None);
        }

        #[test]
        fn test_key_modifier_combinations(
            ctrl in any::<bool>(),
            alt in any::<bool>(),
            shift in any::<bool>(),
            super_key in any::<bool>()
        ) {
            let mut modifiers = KeyModifiers::empty();
            if ctrl { modifiers |= KeyModifiers::CONTROL; }
            if alt { modifiers |= KeyModifiers::ALT; }
            if shift { modifiers |= KeyModifiers::SHIFT; }
            if super_key { modifiers |= KeyModifiers::SUPER; }

            let event = KeyEvent::new(Key::Char('a'), modifiers);

            prop_assert_eq!(event.modifiers.contains(KeyModifiers::CONTROL), ctrl);
            prop_assert_eq!(event.modifiers.contains(KeyModifiers::ALT), alt);
            prop_assert_eq!(event.modifiers.contains(KeyModifiers::SHIFT), shift);
            prop_assert_eq!(event.modifiers.contains(KeyModifiers::SUPER), super_key);
        }
    }
}
