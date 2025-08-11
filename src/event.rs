//! Event handling for keyboard, mouse, and other terminal events
//! 
//! This module defines all the event types that your Hojicha application can receive.
//! Events are the primary way your application interacts with the terminal and user input.
//! 
//! ## Event Categories
//! 
//! ### User Events
//! Custom messages specific to your application:
//! ```
//! # use hojicha_core::event::Event;
//! # #[derive(Debug, Clone, PartialEq)]
//! enum MyMessage {
//!     ButtonClicked,
//!     DataLoaded(String),
//! }
//! 
//! let event = Event::User(MyMessage::ButtonClicked);
//! ```
//! 
//! ### Input Events
//! Keyboard and mouse input from the user:
//! ```
//! # use hojicha_core::event::{Event, Key, KeyEvent, KeyModifiers};
//! # type MyMessage = ();
//! # let event: Event<MyMessage> = Event::Tick;
//! match event {
//!     Event::Key(key) if key.key == Key::Char('q') => {
//!         // Handle quit
//!     }
//!     Event::Mouse(mouse) => {
//!         // Handle mouse click/movement
//!     }
//!     _ => {}
//! }
//! ```
//! 
//! ### System Events
//! Terminal and application lifecycle events:
//! - `Event::Resize` - Terminal was resized
//! - `Event::Focus` / `Event::Blur` - Terminal gained/lost focus  
//! - `Event::Suspend` / `Event::Resume` - App was suspended/resumed
//! - `Event::Tick` - Periodic timer tick
//! - `Event::Quit` - Application should exit

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

impl<M> Event<M> {
    /// Check if this is a key event
    pub fn is_key(&self) -> bool {
        matches!(self, Event::Key(_))
    }
    
    /// Check if this is a specific key press
    ///
    /// # Example
    /// ```ignore
    /// if event.is_key_press(Key::Enter) {
    ///     // Handle enter key
    /// }
    /// ```
    pub fn is_key_press(&self, key: Key) -> bool {
        matches!(self, Event::Key(k) if k.key == key)
    }
    
    /// Check if this is a specific key with modifiers
    ///
    /// # Example
    /// ```ignore
    /// if event.is_key_with_modifiers(Key::Char('c'), KeyModifiers::CONTROL) {
    ///     // Handle Ctrl+C
    /// }
    /// ```
    pub fn is_key_with_modifiers(&self, key: Key, modifiers: KeyModifiers) -> bool {
        matches!(self, Event::Key(k) if k.key == key && k.modifiers == modifiers)
    }
    
    /// Get the key event if this is a key event
    pub fn as_key(&self) -> Option<&KeyEvent> {
        match self {
            Event::Key(k) => Some(k),
            _ => None,
        }
    }
    
    /// Check if this is a mouse event
    pub fn is_mouse(&self) -> bool {
        matches!(self, Event::Mouse(_))
    }
    
    /// Get the mouse event if this is a mouse event
    pub fn as_mouse(&self) -> Option<&MouseEvent> {
        match self {
            Event::Mouse(m) => Some(m),
            _ => None,
        }
    }
    
    /// Check if this is a mouse click at any position
    pub fn is_click(&self) -> bool {
        matches!(self, Event::Mouse(m) if m.is_click())
    }
    
    /// Get click position if this is a click event
    ///
    /// # Example
    /// ```ignore
    /// if let Some((x, y)) = event.as_click() {
    ///     // Handle click at position (x, y)
    /// }
    /// ```
    pub fn as_click(&self) -> Option<(u16, u16)> {
        match self {
            Event::Mouse(m) if m.is_click() => Some(m.position()),
            _ => None,
        }
    }
    
    /// Check if this is a resize event
    pub fn is_resize(&self) -> bool {
        matches!(self, Event::Resize { .. })
    }
    
    /// Get resize dimensions if this is a resize event
    ///
    /// # Example
    /// ```ignore
    /// if let Some((width, height)) = event.as_resize() {
    ///     // Handle resize to width x height
    /// }
    /// ```
    pub fn as_resize(&self) -> Option<(u16, u16)> {
        match self {
            Event::Resize { width, height } => Some((*width, *height)),
            _ => None,
        }
    }
    
    /// Check if this is a user message
    pub fn is_user(&self) -> bool {
        matches!(self, Event::User(_))
    }
    
    /// Get the user message if this is a user event
    pub fn as_user(&self) -> Option<&M> {
        match self {
            Event::User(msg) => Some(msg),
            _ => None,
        }
    }
    
    /// Take the user message if this is a user event
    pub fn into_user(self) -> Option<M> {
        match self {
            Event::User(msg) => Some(msg),
            _ => None,
        }
    }
    
    /// Check if this is a quit event
    pub fn is_quit(&self) -> bool {
        matches!(self, Event::Quit)
    }
    
    /// Check if this is a tick event
    pub fn is_tick(&self) -> bool {
        matches!(self, Event::Tick)
    }
    
    /// Check if this is a paste event
    pub fn is_paste(&self) -> bool {
        matches!(self, Event::Paste(_))
    }
    
    /// Get pasted text if this is a paste event
    pub fn as_paste(&self) -> Option<&str> {
        match self {
            Event::Paste(text) => Some(text.as_str()),
            _ => None,
        }
    }
    
    /// Check if this is a focus event
    pub fn is_focus(&self) -> bool {
        matches!(self, Event::Focus)
    }
    
    /// Check if this is a blur event
    pub fn is_blur(&self) -> bool {
        matches!(self, Event::Blur)
    }
    
    /// Check if this is a suspend event
    pub fn is_suspend(&self) -> bool {
        matches!(self, Event::Suspend)
    }
    
    /// Check if this is a resume event
    pub fn is_resume(&self) -> bool {
        matches!(self, Event::Resume)
    }
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
    
    /// Check if this key event matches a specific key
    ///
    /// # Example
    /// ```ignore
    /// if key_event.is(Key::Enter) {
    ///     // Handle enter key
    /// }
    /// ```
    pub fn is(&self, key: Key) -> bool {
        self.key == key
    }
    
    /// Check if this key event matches a specific key with modifiers
    ///
    /// # Example
    /// ```ignore
    /// if key_event.is_with_modifiers(Key::Char('s'), KeyModifiers::CONTROL) {
    ///     // Handle Ctrl+S
    /// }
    /// ```
    pub fn is_with_modifiers(&self, key: Key, modifiers: KeyModifiers) -> bool {
        self.key == key && self.modifiers == modifiers
    }
    
    /// Check if control key is held
    pub fn is_ctrl(&self) -> bool {
        self.modifiers.contains(KeyModifiers::CONTROL)
    }
    
    /// Check if alt key is held
    pub fn is_alt(&self) -> bool {
        self.modifiers.contains(KeyModifiers::ALT)
    }
    
    /// Check if shift key is held
    pub fn is_shift(&self) -> bool {
        self.modifiers.contains(KeyModifiers::SHIFT)
    }
    
    /// Check if super/meta key is held
    pub fn is_super(&self) -> bool {
        self.modifiers.contains(KeyModifiers::SUPER)
    }
    
    /// Check if this is a navigation key (arrows, home, end, page up/down)
    pub fn is_navigation(&self) -> bool {
        matches!(
            self.key,
            Key::Up | Key::Down | Key::Left | Key::Right | 
            Key::Home | Key::End | Key::PageUp | Key::PageDown
        )
    }
    
    /// Check if this is a function key (F1-F24)
    pub fn is_function_key(&self) -> bool {
        matches!(self.key, Key::F(_))
    }
    
    /// Check if this is a media control key
    pub fn is_media_key(&self) -> bool {
        matches!(
            self.key,
            Key::MediaPlay | Key::MediaPause | Key::MediaPlayPause |
            Key::MediaStop | Key::MediaNext | Key::MediaPrevious |
            Key::MediaFastForward | Key::MediaRewind |
            Key::MediaVolumeUp | Key::MediaVolumeDown | Key::MediaMute
        )
    }
    
    /// Check if this key event has any modifiers
    pub fn has_modifiers(&self) -> bool {
        !self.modifiers.is_empty()
    }
    
    /// Check if this key event has no modifiers
    pub fn no_modifiers(&self) -> bool {
        self.modifiers.is_empty()
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
///
/// # Example
/// ```ignore
/// match event {
///     Event::Mouse(mouse) => {
///         if mouse.is_left_click() {
///             println!("Left clicked at ({}, {})", mouse.column, mouse.row);
///         } else if mouse.is_scroll_up() {
///             println!("Scrolled up");
///         }
///     }
///     _ => {}
/// }
/// ```
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

impl MouseEvent {
    /// Create a new mouse event
    pub fn new(kind: MouseEventKind, column: u16, row: u16, modifiers: KeyModifiers) -> Self {
        Self { kind, column, row, modifiers }
    }
    
    /// Check if this is a left button click (button down event)
    pub fn is_left_click(&self) -> bool {
        matches!(self.kind, MouseEventKind::Down(MouseButton::Left))
    }
    
    /// Check if this is a right button click (button down event)
    pub fn is_right_click(&self) -> bool {
        matches!(self.kind, MouseEventKind::Down(MouseButton::Right))
    }
    
    /// Check if this is a middle button click (button down event)
    pub fn is_middle_click(&self) -> bool {
        matches!(self.kind, MouseEventKind::Down(MouseButton::Middle))
    }
    
    /// Check if this is any button click (button down event)
    pub fn is_click(&self) -> bool {
        matches!(self.kind, MouseEventKind::Down(_))
    }
    
    /// Check if this is a button release event
    pub fn is_release(&self) -> bool {
        matches!(self.kind, MouseEventKind::Up(_))
    }
    
    /// Check if this is a drag event (mouse moved while button pressed)
    pub fn is_drag(&self) -> bool {
        matches!(self.kind, MouseEventKind::Drag(_))
    }
    
    /// Check if this is a left button drag
    pub fn is_left_drag(&self) -> bool {
        matches!(self.kind, MouseEventKind::Drag(MouseButton::Left))
    }
    
    /// Check if this is a right button drag
    pub fn is_right_drag(&self) -> bool {
        matches!(self.kind, MouseEventKind::Drag(MouseButton::Right))
    }
    
    /// Check if this is a middle button drag
    pub fn is_middle_drag(&self) -> bool {
        matches!(self.kind, MouseEventKind::Drag(MouseButton::Middle))
    }
    
    /// Check if this is a scroll up event
    pub fn is_scroll_up(&self) -> bool {
        matches!(self.kind, MouseEventKind::ScrollUp)
    }
    
    /// Check if this is a scroll down event
    pub fn is_scroll_down(&self) -> bool {
        matches!(self.kind, MouseEventKind::ScrollDown)
    }
    
    /// Check if this is a scroll left event
    pub fn is_scroll_left(&self) -> bool {
        matches!(self.kind, MouseEventKind::ScrollLeft)
    }
    
    /// Check if this is a scroll right event
    pub fn is_scroll_right(&self) -> bool {
        matches!(self.kind, MouseEventKind::ScrollRight)
    }
    
    /// Check if this is any scroll event
    pub fn is_scroll(&self) -> bool {
        matches!(
            self.kind,
            MouseEventKind::ScrollUp
                | MouseEventKind::ScrollDown
                | MouseEventKind::ScrollLeft
                | MouseEventKind::ScrollRight
        )
    }
    
    /// Check if this is a mouse move event (without button pressed)
    pub fn is_move(&self) -> bool {
        matches!(self.kind, MouseEventKind::Moved)
    }
    
    /// Get the button involved in this event, if any
    pub fn button(&self) -> Option<MouseButton> {
        match self.kind {
            MouseEventKind::Down(btn) | MouseEventKind::Up(btn) | MouseEventKind::Drag(btn) => Some(btn),
            _ => None,
        }
    }
    
    /// Get the position as a tuple (column, row)
    pub fn position(&self) -> (u16, u16) {
        (self.column, self.row)
    }
    
    /// Check if the mouse event occurred within a rectangular area
    ///
    /// # Example
    /// ```ignore
    /// let rect = Rect::new(10, 10, 20, 10);
    /// if mouse_event.is_within_rect(rect) {
    ///     // Mouse event is within the rectangle
    /// }
    /// ```
    pub fn is_within(&self, x: u16, y: u16, width: u16, height: u16) -> bool {
        self.column >= x && self.column < x + width &&
        self.row >= y && self.row < y + height
    }
    
    /// Check if the mouse event occurred at a specific position
    pub fn is_at(&self, column: u16, row: u16) -> bool {
        self.column == column && self.row == row
    }
    
    /// Check if a modifier key was held during the event
    pub fn has_modifier(&self, modifier: KeyModifiers) -> bool {
        self.modifiers.contains(modifier)
    }
    
    /// Check if control key is held
    pub fn is_ctrl(&self) -> bool {
        self.modifiers.contains(KeyModifiers::CONTROL)
    }
    
    /// Check if alt key is held
    pub fn is_alt(&self) -> bool {
        self.modifiers.contains(KeyModifiers::ALT)
    }
    
    /// Check if shift key is held
    pub fn is_shift(&self) -> bool {
        self.modifiers.contains(KeyModifiers::SHIFT)
    }
    
    /// Check if this mouse event has any modifiers
    pub fn has_modifiers(&self) -> bool {
        !self.modifiers.is_empty()
    }
    
    /// Check if this mouse event has no modifiers
    pub fn no_modifiers(&self) -> bool {
        self.modifiers.is_empty()
    }
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

        // Test using new helper methods
        assert!(key_event.is_key());
        assert!(key_event.as_key().is_some());
        
        assert!(mouse_event.is_mouse());
        assert!(mouse_event.as_mouse().is_some());
        
        assert!(resize_event.is_resize());
        assert_eq!(resize_event.as_resize(), Some((80, 24)));
        
        assert!(tick_event.is_tick());
        
        assert!(user_event.is_user());
        assert_eq!(user_event.as_user(), Some(&"test".to_string()));
        
        assert!(quit_event.is_quit());
        assert!(focus_event.is_focus());
        assert!(blur_event.is_blur());
        assert!(suspend_event.is_suspend());
        assert!(resume_event.is_resume());
        
        assert!(paste_event.is_paste());
        assert_eq!(paste_event.as_paste(), Some("pasted text"));
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

            prop_assert!(event.is_user());
            prop_assert_eq!(event.as_user(), Some(&message));
        }

        #[test]
        fn test_event_resize_properties(
            width in 1u16..1000u16,
            height in 1u16..1000u16
        ) {
            let event = Event::<String>::Resize { width, height };

            prop_assert!(event.is_resize());
            prop_assert_eq!(event.as_resize(), Some((width, height)));
        }

        #[test]
        fn test_event_paste_properties(
            paste_text in ".*"
        ) {
            let event = Event::<String>::Paste(paste_text.clone());

            prop_assert!(event.is_paste());
            prop_assert_eq!(event.as_paste(), Some(paste_text.as_str()));
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
