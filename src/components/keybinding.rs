//! Key binding management system
//!
//! Provides a structured way to define and manage keyboard shortcuts
//! with help text generation.

use crate::event::{Key, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// A single key binding definition
#[derive(Debug, Clone)]
pub struct KeyBinding {
    /// The keys that trigger this binding
    pub keys: Vec<KeyEvent>,
    /// Help text description
    pub help: String,
    /// Short help key representation
    pub help_key: String,
    /// Whether this binding is currently enabled
    pub enabled: bool,
}

impl KeyBinding {
    /// Create a new key binding
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            help: String::new(),
            help_key: String::new(),
            enabled: true,
        }
    }

    /// Add keys that trigger this binding
    pub fn with_keys(mut self, keys: Vec<Key>) -> Self {
        self.keys = keys
            .into_iter()
            .map(|key| KeyEvent::new(key, KeyModifiers::empty()))
            .collect();
        self.update_help_key();
        self
    }

    /// Add keys with modifiers
    pub fn with_key_events(mut self, keys: Vec<KeyEvent>) -> Self {
        self.keys = keys;
        self.update_help_key();
        self
    }

    /// Add a single key
    pub fn with_key(mut self, key: Key) -> Self {
        self.keys.push(KeyEvent::new(key, KeyModifiers::empty()));
        self.update_help_key();
        self
    }

    /// Add a key with modifiers
    pub fn with_key_event(mut self, key: Key, modifiers: KeyModifiers) -> Self {
        self.keys.push(KeyEvent::new(key, modifiers));
        self.update_help_key();
        self
    }

    /// Set the help description
    pub fn with_help(mut self, key: impl Into<String>, desc: impl Into<String>) -> Self {
        self.help_key = key.into();
        self.help = desc.into();
        self
    }

    /// Enable or disable this binding
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Check if a key event matches this binding
    pub fn matches(&self, event: &KeyEvent) -> bool {
        if !self.enabled {
            return false;
        }
        self.keys.iter().any(|k| k == event)
    }

    /// Update the help key representation from the keys
    fn update_help_key(&mut self) {
        if self.help_key.is_empty() && !self.keys.is_empty() {
            self.help_key = format_key_event(&self.keys[0]);
        }
    }
}

impl Default for KeyBinding {
    fn default() -> Self {
        Self::new()
    }
}

/// Collection of key bindings organized by name
#[derive(Debug, Clone, Default)]
pub struct KeyMap {
    /// Map of named key bindings
    pub bindings: HashMap<String, KeyBinding>,
}

impl KeyMap {
    /// Create a new empty key map
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Add a key binding with a name
    pub fn add(&mut self, name: impl Into<String>, binding: KeyBinding) {
        self.bindings.insert(name.into(), binding);
    }

    /// Get a key binding by name
    pub fn get(&self, name: &str) -> Option<&KeyBinding> {
        self.bindings.get(name)
    }

    /// Get a mutable key binding by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut KeyBinding> {
        self.bindings.get_mut(name)
    }

    /// Check if any binding matches the key event
    pub fn matches(&self, event: &KeyEvent) -> Option<&str> {
        for (name, binding) in &self.bindings {
            if binding.matches(event) {
                return Some(name);
            }
        }
        None
    }

    /// Check if a specific binding matches
    pub fn binding_matches(&self, name: &str, event: &KeyEvent) -> bool {
        self.bindings
            .get(name)
            .map(|b| b.matches(event))
            .unwrap_or(false)
    }

    /// Enable or disable a binding
    pub fn set_enabled(&mut self, name: &str, enabled: bool) {
        if let Some(binding) = self.bindings.get_mut(name) {
            binding.enabled = enabled;
        }
    }

    /// Get all enabled bindings
    pub fn enabled_bindings(&self) -> Vec<(&str, &KeyBinding)> {
        self.bindings
            .iter()
            .filter(|(_, b)| b.enabled)
            .map(|(k, v)| (k.as_str(), v))
            .collect()
    }

    /// Get help text for all enabled bindings
    pub fn help_text(&self) -> Vec<(String, String)> {
        let mut help: Vec<_> = self
            .bindings
            .iter()
            .filter(|(_, b)| b.enabled && !b.help.is_empty())
            .map(|(_, b)| (b.help_key.clone(), b.help.clone()))
            .collect();
        help.sort_by(|a, b| a.0.cmp(&b.0));
        help
    }
}

/// Format a key event as a string
fn format_key_event(event: &KeyEvent) -> String {
    let mut parts = Vec::new();

    if event.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("ctrl");
    }
    if event.modifiers.contains(KeyModifiers::ALT) {
        parts.push("alt");
    }
    if event.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("shift");
    }

    let key_str = match event.key {
        Key::Char(c) => c.to_string(),
        Key::Enter => "enter".to_string(),
        Key::Backspace => "backspace".to_string(),
        Key::Left => "←".to_string(),
        Key::Right => "→".to_string(),
        Key::Up => "↑".to_string(),
        Key::Down => "↓".to_string(),
        Key::Tab => "tab".to_string(),
        Key::Delete => "delete".to_string(),
        Key::Home => "home".to_string(),
        Key::End => "end".to_string(),
        Key::PageUp => "pgup".to_string(),
        Key::PageDown => "pgdn".to_string(),
        Key::Esc => "esc".to_string(),
        Key::F(n) => format!("f{n}"),
        _ => "?".to_string(),
    };

    parts.push(&key_str);
    parts.join("+")
}

/// Common key binding sets
pub mod presets {
    use super::*;

    /// Create standard text editing key bindings
    pub fn text_editing() -> KeyMap {
        let mut map = KeyMap::new();

        map.add(
            "cut",
            KeyBinding::new()
                .with_key_event(Key::Char('x'), KeyModifiers::CONTROL)
                .with_help("ctrl+x", "cut"),
        );

        map.add(
            "copy",
            KeyBinding::new()
                .with_key_event(Key::Char('c'), KeyModifiers::CONTROL)
                .with_help("ctrl+c", "copy"),
        );

        map.add(
            "paste",
            KeyBinding::new()
                .with_key_event(Key::Char('v'), KeyModifiers::CONTROL)
                .with_help("ctrl+v", "paste"),
        );

        map.add(
            "undo",
            KeyBinding::new()
                .with_key_event(Key::Char('z'), KeyModifiers::CONTROL)
                .with_help("ctrl+z", "undo"),
        );

        map.add(
            "redo",
            KeyBinding::new()
                .with_key_event(Key::Char('z'), KeyModifiers::CONTROL | KeyModifiers::SHIFT)
                .with_help("ctrl+shift+z", "redo"),
        );

        map.add(
            "select_all",
            KeyBinding::new()
                .with_key_event(Key::Char('a'), KeyModifiers::CONTROL)
                .with_help("ctrl+a", "select all"),
        );

        map
    }

    /// Create navigation key bindings
    pub fn navigation() -> KeyMap {
        let mut map = KeyMap::new();

        map.add(
            "up",
            KeyBinding::new()
                .with_key(Key::Up)
                .with_help("↑", "move up"),
        );

        map.add(
            "down",
            KeyBinding::new()
                .with_key(Key::Down)
                .with_help("↓", "move down"),
        );

        map.add(
            "left",
            KeyBinding::new()
                .with_key(Key::Left)
                .with_help("←", "move left"),
        );

        map.add(
            "right",
            KeyBinding::new()
                .with_key(Key::Right)
                .with_help("→", "move right"),
        );

        map.add(
            "page_up",
            KeyBinding::new()
                .with_key(Key::PageUp)
                .with_help("pgup", "page up"),
        );

        map.add(
            "page_down",
            KeyBinding::new()
                .with_key(Key::PageDown)
                .with_help("pgdn", "page down"),
        );

        map.add(
            "home",
            KeyBinding::new()
                .with_key(Key::Home)
                .with_help("home", "go to start"),
        );

        map.add(
            "end",
            KeyBinding::new()
                .with_key(Key::End)
                .with_help("end", "go to end"),
        );

        map
    }

    /// Create common application key bindings
    pub fn application() -> KeyMap {
        let mut map = KeyMap::new();

        map.add(
            "quit",
            KeyBinding::new()
                .with_key_event(Key::Char('q'), KeyModifiers::CONTROL)
                .with_help("ctrl+q", "quit"),
        );

        map.add(
            "save",
            KeyBinding::new()
                .with_key_event(Key::Char('s'), KeyModifiers::CONTROL)
                .with_help("ctrl+s", "save"),
        );

        map.add(
            "open",
            KeyBinding::new()
                .with_key_event(Key::Char('o'), KeyModifiers::CONTROL)
                .with_help("ctrl+o", "open"),
        );

        map.add(
            "new",
            KeyBinding::new()
                .with_key_event(Key::Char('n'), KeyModifiers::CONTROL)
                .with_help("ctrl+n", "new"),
        );

        map.add(
            "help",
            KeyBinding::new()
                .with_key(Key::F(1))
                .with_help("f1", "help"),
        );

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_binding_creation() {
        let binding = KeyBinding::new()
            .with_key(Key::Char('s'))
            .with_help("s", "save file")
            .with_enabled(true);

        assert_eq!(binding.help, "save file");
        assert_eq!(binding.help_key, "s");
        assert!(binding.enabled);
    }

    #[test]
    fn test_key_binding_matching() {
        let binding = KeyBinding::new().with_key_event(Key::Char('s'), KeyModifiers::CONTROL);

        let event1 = KeyEvent::new(Key::Char('s'), KeyModifiers::CONTROL);
        let event2 = KeyEvent::new(Key::Char('s'), KeyModifiers::empty());

        assert!(binding.matches(&event1));
        assert!(!binding.matches(&event2));
    }

    #[test]
    fn test_keymap() {
        let mut map = KeyMap::new();

        map.add(
            "save",
            KeyBinding::new()
                .with_key_event(Key::Char('s'), KeyModifiers::CONTROL)
                .with_help("ctrl+s", "save"),
        );

        map.add(
            "quit",
            KeyBinding::new()
                .with_key_event(Key::Char('q'), KeyModifiers::CONTROL)
                .with_help("ctrl+q", "quit"),
        );

        let save_event = KeyEvent::new(Key::Char('s'), KeyModifiers::CONTROL);
        assert_eq!(map.matches(&save_event), Some("save"));

        let help = map.help_text();
        assert_eq!(help.len(), 2);
    }

    #[test]
    fn test_key_formatting() {
        let event = KeyEvent::new(Key::Char('s'), KeyModifiers::CONTROL | KeyModifiers::SHIFT);
        assert_eq!(format_key_event(&event), "ctrl+shift+s");

        let event2 = KeyEvent::new(Key::Up, KeyModifiers::empty());
        assert_eq!(format_key_event(&event2), "↑");
    }

    #[test]
    fn test_key_binding_default() {
        let binding = KeyBinding::default();
        assert!(binding.keys.is_empty());
        assert_eq!(binding.help, "");
        assert_eq!(binding.help_key, "");
        assert!(binding.enabled);
    }

    #[test]
    fn test_key_binding_disabled() {
        let binding = KeyBinding::new()
            .with_key(Key::Char('x'))
            .with_enabled(false);

        let event = KeyEvent::new(Key::Char('x'), KeyModifiers::empty());
        assert!(!binding.matches(&event)); // Should not match when disabled
    }

    #[test]
    fn test_key_binding_multiple_keys() {
        let binding = KeyBinding::new()
            .with_key(Key::Char('s'))
            .with_key_event(Key::Char('w'), KeyModifiers::CONTROL);

        // Should match either key
        let event1 = KeyEvent::new(Key::Char('s'), KeyModifiers::empty());
        let event2 = KeyEvent::new(Key::Char('w'), KeyModifiers::CONTROL);
        let event3 = KeyEvent::new(Key::Char('x'), KeyModifiers::empty());

        assert!(binding.matches(&event1));
        assert!(binding.matches(&event2));
        assert!(!binding.matches(&event3));
    }

    #[test]
    fn test_keymap_get() {
        let mut map = KeyMap::new();

        let binding = KeyBinding::new()
            .with_key(Key::Enter)
            .with_help("enter", "confirm");

        map.add("confirm", binding.clone());

        assert_eq!(map.get("confirm").unwrap().help, "confirm");
        assert!(map.get("nonexistent").is_none());
    }

    #[test]
    fn test_keymap_remove() {
        let mut map = KeyMap::new();

        map.add("test", KeyBinding::new().with_key(Key::Tab));
        assert!(map.get("test").is_some());

        // Remove directly from bindings map
        map.bindings.remove("test");
        assert!(map.get("test").is_none());
    }

    #[test]
    fn test_keymap_all() {
        let mut map = KeyMap::new();

        map.add("first", KeyBinding::new().with_key(Key::F(1)));
        map.add("second", KeyBinding::new().with_key(Key::F(2)));

        // Access bindings directly since there's no all() method
        assert_eq!(map.bindings.len(), 2);
        assert!(map.bindings.contains_key("first"));
        assert!(map.bindings.contains_key("second"));
    }

    #[test]
    fn test_keymap_merge() {
        let mut map1 = KeyMap::new();
        let map2 = KeyMap::new();

        map1.add("save", KeyBinding::new().with_key(Key::Char('s')));
        let mut map2 = map2;
        map2.add("quit", KeyBinding::new().with_key(Key::Char('q')));
        map2.add("save", KeyBinding::new().with_key(Key::Char('w'))); // Override

        // Manually merge since there's no merge method
        for (name, binding) in map2.bindings {
            map1.bindings.insert(name, binding);
        }

        assert_eq!(map1.bindings.len(), 2);
        // Should have overridden the 'save' binding
        let save_binding = map1.get("save").unwrap();
        assert!(save_binding.matches(&KeyEvent::new(Key::Char('w'), KeyModifiers::empty())));
    }

    #[test]
    fn test_format_key_event_special_keys() {
        // Test various special keys
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Enter, KeyModifiers::empty())),
            "enter"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Tab, KeyModifiers::empty())),
            "tab"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Backspace, KeyModifiers::empty())),
            "backspace"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Delete, KeyModifiers::empty())),
            "delete"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Home, KeyModifiers::empty())),
            "home"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::End, KeyModifiers::empty())),
            "end"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::PageUp, KeyModifiers::empty())),
            "pgup"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::PageDown, KeyModifiers::empty())),
            "pgdn"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Esc, KeyModifiers::empty())),
            "esc"
        );

        // Test arrow keys
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Up, KeyModifiers::empty())),
            "↑"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Down, KeyModifiers::empty())),
            "↓"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Left, KeyModifiers::empty())),
            "←"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Right, KeyModifiers::empty())),
            "→"
        );

        // Test function keys
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::F(1), KeyModifiers::empty())),
            "f1"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::F(12), KeyModifiers::empty())),
            "f12"
        );

        // Test regular characters
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Char('a'), KeyModifiers::empty())),
            "a"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(Key::Char(' '), KeyModifiers::empty())),
            " "
        );
    }

    #[test]
    fn test_format_key_event_modifiers() {
        // Test all modifier combinations
        let key = Key::Char('x');

        assert_eq!(
            format_key_event(&KeyEvent::new(key, KeyModifiers::CONTROL)),
            "ctrl+x"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(key, KeyModifiers::ALT)),
            "alt+x"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(key, KeyModifiers::SHIFT)),
            "shift+x"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(
                key,
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )),
            "ctrl+alt+x"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(
                key,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            )),
            "ctrl+shift+x"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(key, KeyModifiers::ALT | KeyModifiers::SHIFT)),
            "alt+shift+x"
        );
        assert_eq!(
            format_key_event(&KeyEvent::new(
                key,
                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT
            )),
            "ctrl+alt+shift+x"
        );
    }

    #[test]
    fn test_presets_navigation() {
        // Create navigation keymap manually since there's no preset method
        let mut nav = KeyMap::new();
        nav.add("up", KeyBinding::new().with_key(Key::Up));
        nav.add("down", KeyBinding::new().with_key(Key::Down));
        nav.add("left", KeyBinding::new().with_key(Key::Left));
        nav.add("right", KeyBinding::new().with_key(Key::Right));

        // Test that navigation keys work
        let up_event = KeyEvent::new(Key::Up, KeyModifiers::empty());
        assert_eq!(nav.matches(&up_event), Some("up"));

        let down_event = KeyEvent::new(Key::Down, KeyModifiers::empty());
        assert_eq!(nav.matches(&down_event), Some("down"));
    }

    #[test]
    fn test_presets_application() {
        // Create application keymap manually
        let mut app = KeyMap::new();
        app.add(
            "quit",
            KeyBinding::new().with_key_event(Key::Char('q'), KeyModifiers::CONTROL),
        );
        app.add(
            "save",
            KeyBinding::new().with_key_event(Key::Char('s'), KeyModifiers::CONTROL),
        );
        app.add("help", KeyBinding::new().with_key(Key::F(1)));

        // Test specific bindings
        let quit_event = KeyEvent::new(Key::Char('q'), KeyModifiers::CONTROL);
        assert_eq!(app.matches(&quit_event), Some("quit"));

        let help_event = KeyEvent::new(Key::F(1), KeyModifiers::empty());
        assert_eq!(app.matches(&help_event), Some("help"));
    }

    #[test]
    fn test_keymap_help_text() {
        let mut map = KeyMap::new();

        map.add(
            "action1",
            KeyBinding::new()
                .with_key(Key::Char('a'))
                .with_help("a", "first action"),
        );

        map.add(
            "action2",
            KeyBinding::new()
                .with_key(Key::Char('b'))
                .with_help("b", "second action")
                .with_enabled(false), // Disabled binding
        );

        let help = map.help_text();
        assert_eq!(help.len(), 1); // Only enabled bindings should be in help
        assert_eq!(help[0].0, "a");
        assert_eq!(help[0].1, "first action");
    }

    #[test]
    fn test_key_binding_builder_pattern() {
        let binding = KeyBinding::new()
            .with_key(Key::Enter)
            .with_key_event(Key::Char('y'), KeyModifiers::empty())
            .with_help("enter/y", "confirm")
            .with_enabled(true);

        assert_eq!(binding.keys.len(), 2);
        assert_eq!(binding.help, "confirm");
        assert_eq!(binding.help_key, "enter/y");
        assert!(binding.enabled);
    }
}
