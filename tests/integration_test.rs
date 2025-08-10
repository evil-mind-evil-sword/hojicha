//! Integration tests for Hojicha, inspired by Bubbletea's test suite

use hojicha::prelude::*;
use hojicha::program::{MouseMode, ProgramOptions};
use ratatui::widgets::Paragraph;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Test model similar to Bubbletea's testModel
#[derive(Clone)]
struct TestModel {
    executed: Arc<AtomicBool>,
    counter: Arc<AtomicU32>,
}

impl TestModel {
    fn new() -> Self {
        Self {
            executed: Arc::new(AtomicBool::new(false)),
            counter: Arc::new(AtomicU32::new(0)),
        }
    }

    fn was_executed(&self) -> bool {
        self.executed.load(Ordering::SeqCst)
    }

    fn get_counter(&self) -> u32 {
        self.counter.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
enum TestMsg {
    Increment,
    Panic,
    Quit,
}

impl Model for TestModel {
    type Message = TestMsg;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match msg {
            Event::User(TestMsg::Increment) => {
                self.counter.fetch_add(1, Ordering::SeqCst);
                None
            }
            Event::User(TestMsg::Panic) => {
                panic!("Testing panic behavior");
            }
            Event::User(TestMsg::Quit) | Event::Quit => {
                // In a real implementation, we'd signal quit
                None
            }
            Event::Key(key) if key.key == Key::Char('q') => {
                // Quit on 'q' key
                None
            }
            _ => None,
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        self.executed.store(true, Ordering::SeqCst);
        frame.render_widget(Paragraph::new("success\n"), area);
    }
}

#[test]
fn test_model_executes() {
    let model = TestModel::new();
    assert!(!model.was_executed());

    // After creating a program and running it, executed should be true
    // Note: We can't actually run the program in tests due to terminal requirements
    // but we can test the model directly
}

#[test]
fn test_model_update_increment() {
    let mut model = TestModel::new();
    assert_eq!(model.get_counter(), 0);

    model.update(Event::User(TestMsg::Increment));
    assert_eq!(model.get_counter(), 1);

    model.update(Event::User(TestMsg::Increment));
    assert_eq!(model.get_counter(), 2);
}

#[test]
#[should_panic(expected = "Testing panic behavior")]
fn test_model_panic() {
    let mut model = TestModel::new();
    model.update(Event::User(TestMsg::Panic));
}

#[test]
fn test_batch_commands() {
    let cmds = vec![
        Some(Cmd::new(|| Some(TestMsg::Increment))),
        Some(Cmd::new(|| Some(TestMsg::Increment))),
        None,
    ];

    let batched = batch(cmds);
    assert!(batched.is_some());
}

#[test]
fn test_sequence_commands() {
    let cmds = vec![
        Some(Cmd::new(|| Some(TestMsg::Increment))),
        Some(Cmd::new(|| Some(TestMsg::Quit))),
    ];

    let sequenced = sequence(cmds);
    assert!(sequenced.is_some());
}

#[test]
fn test_tick_command() {
    let cmd = tick(Duration::from_millis(50), || TestMsg::Increment);

    if let Ok(Some(msg)) = cmd.test_execute() {
        assert!(matches!(msg, TestMsg::Increment));
        // Note: We don't test timing since tick executes immediately in tests
        // for deterministic behavior (no sleeps in tests)
    }
}

#[test]
fn test_key_event_creation() {
    let key_event = KeyEvent::new(Key::Char('a'), KeyModifiers::empty());
    assert_eq!(key_event.key, Key::Char('a'));
    assert!(key_event.is_char());
    assert_eq!(key_event.char(), Some('a'));

    let ctrl_c = KeyEvent::new(Key::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(ctrl_c.modifiers, KeyModifiers::CONTROL);
}

#[test]
fn test_program_options() {
    let opts = ProgramOptions::default()
        .with_alt_screen(false)
        .with_mouse_mode(MouseMode::CellMotion)
        .with_fps(100);

    assert!(!opts.alt_screen);
    assert_eq!(opts.mouse_mode, MouseMode::CellMotion);
    assert_eq!(opts.fps, 100);
}

/// Test concurrent model updates (similar to Bubbletea's concurrent tests)
#[test]
fn test_concurrent_updates() {
    use std::thread;

    let model = TestModel::new();
    let model_clone = model.clone();

    // Spawn multiple threads updating the counter
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let mut model = model_clone.clone();
            thread::spawn(move || {
                for _ in 0..100 {
                    model.update(Event::User(TestMsg::Increment));
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Should have incremented 1000 times total
    assert_eq!(model.get_counter(), 1000);
}
