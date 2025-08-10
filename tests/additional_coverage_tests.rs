//! Additional tests to improve code coverage

use crossterm::event::{MouseButton, MouseEventKind};
use hojicha::error::{Error, ErrorContext};
use hojicha::event::ModifierKey;
use hojicha::prelude::*;
use hojicha::program::{MouseMode, ProgramOptions};
use std::io;

#[derive(Debug, Clone)]
enum TestMsg {
    Inc,
    Dec,
    Set(i32),
}

struct TestModel {
    value: i32,
}

impl Model for TestModel {
    type Message = TestMsg;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        Some(Cmd::new(|| Some(TestMsg::Set(0))))
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(msg) => match msg {
                TestMsg::Inc => {
                    self.value += 1;
                    Cmd::none()
                }
                TestMsg::Dec => {
                    self.value -= 1;
                    Cmd::none()
                }
                TestMsg::Set(v) => {
                    self.value = v;
                    Cmd::none()
                }
            },
            Event::Key(key) if key.key == Key::Char('q') => None, // Quit
            Event::Quit => None,
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {
        // Simple view
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let result: std::result::Result<(), io::Error> = Err(io_error);
        let error = result.context("Failed to open config").unwrap_err();

        let error_string = format!("{}", error);
        // The error should contain information about the failure
        assert!(!error_string.is_empty());

        // Test debug format
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("Error"));
    }

    #[test]
    fn test_error_from_string() {
        let error = Error::Custom(Box::new(std::io::Error::other(
            "Something went wrong",
        )));
        let error_string = format!("{}", error);
        assert!(error_string.contains("Something went wrong"));
    }

    #[test]
    fn test_error_chain() {
        let base_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let result: std::result::Result<(), io::Error> = Err(base_error);
        let error = result
            .context("Failed to write file")
            .and_then(|_| -> Result<()> { Err(Error::Terminal("Another error".to_string())) })
            .context("Failed to save state")
            .unwrap_err();

        let error_string = format!("{}", error);
        assert!(!error_string.is_empty());
    }

    #[test]
    fn test_program_options_all_fields() {
        let options = ProgramOptions::default()
            .with_alt_screen(false)
            .with_mouse_mode(MouseMode::AllMotion)
            .with_bracketed_paste(true)
            .with_focus_reporting(true)
            .with_fps(120)
            .headless()
            .without_signal_handler()
            .without_renderer();

        assert!(!options.alt_screen);
        assert_eq!(options.mouse_mode, MouseMode::AllMotion);
        assert!(options.bracketed_paste);
        assert!(options.focus_reporting);
        assert_eq!(options.fps, 120);
        assert!(options.headless);
        assert!(!options.install_signal_handler);
        assert!(options.without_renderer);
    }

    #[test]
    fn test_event_is_methods() {
        let key_event = Event::<TestMsg>::Key(KeyEvent::new(Key::Char('a'), KeyModifiers::empty()));
        assert!(matches!(key_event, Event::Key(_)));
        assert!(!matches!(key_event, Event::Quit));

        let quit_event = Event::<TestMsg>::Quit;
        assert!(matches!(quit_event, Event::Quit));
        assert!(!matches!(quit_event, Event::Key(_)));

        let mouse_event = Event::<TestMsg>::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::empty(),
        });
        assert!(!matches!(mouse_event, Event::Key(_)));
        assert!(!matches!(mouse_event, Event::Quit));
    }

    #[test]
    fn test_window_size() {
        let size = WindowSize {
            width: 100,
            height: 50,
        };
        assert_eq!(size.width, 100);
        assert_eq!(size.height, 50);

        // Test equality
        let size2 = WindowSize {
            width: 100,
            height: 50,
        };
        assert_eq!(size, size2);

        let size3 = WindowSize {
            width: 80,
            height: 24,
        };
        assert_ne!(size, size3);
    }

    #[test]
    fn test_mouse_event_all_fields() {
        let event = MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 45,
            row: 23,
            modifiers: KeyModifiers::ALT | KeyModifiers::SHIFT,
        };

        assert_eq!(event.column, 45);
        assert_eq!(event.row, 23);
        assert!(matches!(
            event.kind,
            MouseEventKind::Drag(MouseButton::Left)
        ));
        assert!(event.modifiers.contains(KeyModifiers::ALT));
        assert!(event.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_cmd_variants() {
        // Test Cmd::new
        let cmd = Cmd::new(|| Some(TestMsg::Inc));
        assert!(cmd.test_execute().is_ok());

        // Test Cmd::fallible with success
        let cmd = Cmd::fallible(|| Ok(Some(TestMsg::Dec)));
        assert!(cmd.test_execute().is_ok());

        // Test Cmd::fallible with error
        let cmd: Cmd<TestMsg> = Cmd::fallible(|| Err(Error::Command("test error".to_string())));
        assert!(cmd.test_execute().is_err());

        // Test Cmd::none - it returns Some(Cmd) not None
        let cmd: Option<Cmd<TestMsg>> = Cmd::none();
        assert!(cmd.is_some());
    }

    #[test]
    fn test_all_event_variants() {
        // Create all event variants
        let events = vec![
            Event::Key(KeyEvent::new(Key::Tab, KeyModifiers::empty())),
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Right),
                column: 10,
                row: 10,
                modifiers: KeyModifiers::empty(),
            }),
            Event::Resize {
                width: 120,
                height: 40,
            },
            Event::Tick,
            Event::User(TestMsg::Inc),
            Event::Quit,
            Event::Focus,
            Event::Blur,
            Event::Suspend,
            Event::Resume,
            Event::Paste("clipboard content".to_string()),
        ];

        // Ensure all can be created and matched
        for event in events {
            match event {
                Event::Key(_) => {}
                Event::Mouse(_) => {}
                Event::Resize { .. } => {}
                Event::Tick => {}
                Event::User(_) => {}
                Event::Quit => {}
                Event::Focus => {}
                Event::Blur => {}
                Event::Suspend => {}
                Event::Resume => {}
                Event::Paste(_) => {}
                Event::ExecProcess => {}
            }
        }
    }

    #[test]
    fn test_key_all_variants() {
        let keys = vec![
            Key::Char('z'),
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
            Key::CapsLock,
            Key::ScrollLock,
            Key::NumLock,
            Key::PrintScreen,
            Key::Pause,
            Key::Menu,
            Key::F(1),
            Key::F(12),
            Key::MediaPlay,
            Key::MediaPause,
            Key::MediaStop,
            Key::MediaNext,
            Key::MediaPrevious,
            Key::MediaRewind,
            Key::MediaFastForward,
            Key::MediaVolumeUp,
            Key::MediaVolumeDown,
            Key::MediaMute,
            Key::Null,
        ];

        for key in keys {
            let event = KeyEvent::new(key, KeyModifiers::empty());
            assert_eq!(event.key, key);
        }
    }

    #[test]
    fn test_modifier_key_enum() {
        let modifiers = vec![
            ModifierKey::Shift,
            ModifierKey::Control,
            ModifierKey::Alt,
            ModifierKey::Super,
            ModifierKey::Meta,
            ModifierKey::Hyper,
        ];

        for modifier in modifiers {
            // Test that each variant can be created and matched
            match modifier {
                ModifierKey::Shift => {}
                ModifierKey::Control => {}
                ModifierKey::Alt => {}
                ModifierKey::Super => {}
                ModifierKey::Meta => {}
                ModifierKey::Hyper => {}
            }
        }
    }
}
