//! Basic tests for Program functionality

use hojicha::commands;
use hojicha::event::{Event, Key, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use hojicha::prelude::*;
use hojicha::program::{MouseMode, ProgramOptions};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Simple test model
#[derive(Clone)]
struct SimpleModel {
    counter: Arc<AtomicUsize>,
}

impl SimpleModel {
    fn new() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[derive(Debug, Clone)]
enum SimpleMessage {
    Increment,
    Decrement,
}

impl Model for SimpleModel {
    type Message = SimpleMessage;

    fn init(&mut self) -> Cmd<Self::Message> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(SimpleMessage::Increment) => {
                self.counter.fetch_add(1, Ordering::SeqCst);
                Cmd::none()
            }
            Event::User(SimpleMessage::Decrement) => {
                self.counter.fetch_sub(1, Ordering::SeqCst);
                Cmd::none()
            }
            Event::Quit => commands::quit(),
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

#[test]
fn test_model_init() {
    let mut model = SimpleModel::new();
    let cmd = model.init();
    assert!(!cmd.is_quit());
    assert_eq!(model.counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_model_update() {
    let mut model = SimpleModel::new();

    // Test increment
    model.update(Event::User(SimpleMessage::Increment));
    assert_eq!(model.counter.load(Ordering::SeqCst), 1);

    // Test decrement
    model.update(Event::User(SimpleMessage::Decrement));
    assert_eq!(model.counter.load(Ordering::SeqCst), 0);
}

#[test]
fn test_program_options_all_methods() {
    let opts = ProgramOptions::default()
        .with_alt_screen(false)
        .with_mouse_mode(MouseMode::CellMotion)
        .with_bracketed_paste(true)
        .with_focus_reporting(true)
        .with_fps(30);

    assert!(!opts.alt_screen);
    assert_eq!(opts.mouse_mode, MouseMode::CellMotion);
    assert!(opts.bracketed_paste);
    assert!(opts.focus_reporting);
    assert_eq!(opts.fps, 30);
}

#[test]
fn test_mouse_modes() {
    let opts1 = ProgramOptions::default();
    assert_eq!(opts1.mouse_mode, MouseMode::None);

    let opts2 = ProgramOptions::default().with_mouse_mode(MouseMode::CellMotion);
    assert_eq!(opts2.mouse_mode, MouseMode::CellMotion);

    let opts3 = ProgramOptions::default().with_mouse_mode(MouseMode::AllMotion);
    assert_eq!(opts3.mouse_mode, MouseMode::AllMotion);
}

#[test]
fn test_event_handling() {
    let mut model = SimpleModel::new();

    // Test various events
    let events = vec![
        Event::Key(KeyEvent::new(Key::Char('a'), KeyModifiers::empty())),
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::empty(),
        }),
        Event::Resize {
            width: 80,
            height: 24,
        },
        Event::Tick,
        Event::Focus,
        Event::Blur,
        Event::Suspend,
        Event::Resume,
        Event::Paste("test".to_string()),
    ];

    for event in events {
        let result = model.update(event);
        // Cmd::none() returns None, not Some
        assert!(!result.is_quit());
    }
}

#[test]
fn test_quit_event() {
    let mut model = SimpleModel::new();
    let result = model.update(Event::Quit);
    assert!(result.is_quit()); // Quit should return commands::quit()
}
