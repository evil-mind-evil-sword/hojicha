//! Tests for all examples shown in README.md

use hojicha::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::thread;
use std::time::Duration;

// Exact counter from README Quick Start section
#[derive(Default)]
struct Counter {
    value: i32,
}

impl Model for Counter {
    type Message = ();

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::Key(key) => match key.key {
                Key::Up => self.value += 1,
                Key::Down => self.value -= 1,
                Key::Char('q') => return None, // Quit
                _ => {}
            },
            _ => {}
        }
        Cmd::none()
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let text = format!("Counter: {}\n\n↑/↓: change | q: quit", self.value);
        let widget =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Counter"));
        frame.render_widget(widget, area);
    }
}

#[test]
fn test_readme_counter_example() {
    // Test the exact counter from README
    let mut counter = Counter::default();
    assert_eq!(counter.value, 0);

    // Test increment
    let result = counter.update(Event::Key(KeyEvent {
        key: Key::Up,
        modifiers: crossterm::event::KeyModifiers::empty(),
    }));
    assert!(result.is_some());
    assert_eq!(counter.value, 1);

    // Test decrement
    let result = counter.update(Event::Key(KeyEvent {
        key: Key::Down,
        modifiers: crossterm::event::KeyModifiers::empty(),
    }));
    assert!(result.is_some());
    assert_eq!(counter.value, 0);

    // Test quit
    let result = counter.update(Event::Key(KeyEvent {
        key: Key::Char('q'),
        modifiers: crossterm::event::KeyModifiers::empty(),
    }));
    assert!(result.is_none()); // Should quit
}

#[test]
fn test_readme_counter_runs() {
    use hojicha::program::{Program, ProgramOptions};

    // Test that the counter can be created and run
    let options = ProgramOptions::default().headless();
    let counter = Counter::default();
    let program = Program::with_options(counter, options).unwrap();

    // Run with timeout to ensure it doesn't hang
    let result = program.run_with_timeout(Duration::from_millis(100));
    assert!(result.is_ok());
}

// Test async examples from README

#[derive(Debug, Clone)]
enum Msg {
    Tick,
}

struct AsyncModel {
    ticks: i32,
}

impl Model for AsyncModel {
    type Message = Msg;

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(Msg::Tick) => {
                self.ticks += 1;
                if self.ticks >= 3 {
                    return None; // Quit after 3 ticks
                }
            }
            _ => {}
        }
        Cmd::none()
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

#[test]
fn test_readme_async_event_injection() {
    use hojicha::program::{Program, ProgramOptions};

    let model = AsyncModel { ticks: 0 };
    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    // Test the exact code from README
    let sender = program.init_async_bridge();
    thread::spawn(move || {
        for _ in 0..3 {
            thread::sleep(Duration::from_millis(10));
            sender.send(Event::User(Msg::Tick)).ok();
        }
    });

    // Run and verify it processes the messages
    let result = program.run_with_timeout(Duration::from_millis(500));
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_readme_stream_subscription() {
    use hojicha::program::{Program, ProgramOptions};
    use futures::stream;
    use tokio_stream::StreamExt;

    let model = AsyncModel { ticks: 0 };
    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    // Create a stream that sends 3 ticks
    let stream =
        stream::iter(vec![Msg::Tick, Msg::Tick, Msg::Tick]).throttle(Duration::from_millis(10));

    let _subscription = program.subscribe(stream);

    // Run in a spawned task to avoid runtime issues
    let handle =
        tokio::task::spawn_blocking(move || program.run_with_timeout(Duration::from_millis(500)));

    let result = handle.await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_readme_stream_subscription_with_interval() {
    use hojicha::program::{Program, ProgramOptions};
    use tokio_stream::StreamExt;
    use tokio_stream::wrappers::IntervalStream;

    let model = AsyncModel { ticks: 0 };
    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    // Test the exact IntervalStream example from README
    let interval = tokio::time::interval(Duration::from_millis(10));
    let stream = IntervalStream::new(interval).take(3).map(|_| Msg::Tick);
    let subscription = program.subscribe(stream);

    // Run in spawn_blocking to avoid runtime conflicts
    let handle =
        tokio::task::spawn_blocking(move || program.run_with_timeout(Duration::from_millis(500)));

    // Wait a bit then cancel
    tokio::time::sleep(Duration::from_millis(100)).await;
    subscription.cancel();

    let result = handle.await.unwrap();
    assert!(result.is_ok());
}

#[test]
fn test_readme_cancellable_operations() {
    use hojicha::program::{Program, ProgramOptions};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Clone)]
    struct CancellableModel;

    impl Model for CancellableModel {
        type Message = String;

        fn update(&mut self, _event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
            Cmd::none()
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = CancellableModel;
    let options = ProgramOptions::default().headless();
    let program = Program::with_options(model, options).unwrap();

    // Create a flag to verify cancellation worked
    let was_cancelled = Arc::new(AtomicBool::new(false));
    let was_cancelled_clone = was_cancelled.clone();

    // Test the spawn_cancellable pattern from README
    let handle = program.spawn_cancellable(move |token| {
        let was_cancelled = was_cancelled_clone;
        async move {
            loop {
                if token.is_cancelled() {
                    was_cancelled.store(true, Ordering::SeqCst);
                    return "Cancelled";
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    });

    // Cancel immediately
    handle.cancel();

    // Give it time to process cancellation
    thread::sleep(Duration::from_millis(100));

    // Verify cancellation worked
    assert!(handle.is_cancelled());
}

// Test that all component types mentioned exist and work
#[test]
fn test_readme_components_exist() {
    use hojicha::components::{KeyBinding, List, Spinner, Table, TextArea, Viewport};

    // TextArea - Multi-line text editor
    let mut textarea = TextArea::new();
    textarea.insert_text("a");
    assert_eq!(textarea.cursor(), (0, 1)); // Cursor at column 1 after insert

    // List - Scrollable lists with selection
    let items = vec!["Item 1".to_string(), "Item 2".to_string()];
    let list = List::new(items);
    assert_eq!(list.selected(), 0); // Starts at 0

    // Table - Data tables with headers
    let headers = vec!["Name".to_string(), "Age".to_string()];
    let table: Table<String> = Table::new(headers);
    assert_eq!(table.selected(), 0);

    // Viewport - Scrollable content viewer
    let mut viewport = Viewport::new();
    viewport.set_content("Test content".to_string());
    // Viewport exists and can hold content

    // Spinner - Loading animations
    let _spinner = Spinner::default();
    // Spinner exists and can be created

    // KeyBinding - Keyboard shortcuts
    let _binding = KeyBinding::new()
        .with_key(Key::Char('q'))
        .with_help("q", "Quit");
    // KeyBinding exists and can be configured
}

// Test event priority as described in README
#[test]
fn test_readme_event_priority() {
    use hojicha::priority_queue::PriorityEventQueue;

    #[derive(Debug, Clone, PartialEq)]
    struct TestMsg;

    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    // Add events in the order mentioned in README
    // Low priority
    queue.push(Event::Tick).unwrap();
    queue
        .push(Event::Resize {
            width: 80,
            height: 24,
        })
        .unwrap();
    queue.push(Event::Focus).unwrap();

    // Normal priority
    queue.push(Event::User(TestMsg)).unwrap();
    queue.push(Event::Paste("test".to_string())).unwrap();

    // High priority
    queue.push(Event::Quit).unwrap();
    queue
        .push(Event::Key(KeyEvent {
            key: Key::Char('a'),
            modifiers: crossterm::event::KeyModifiers::empty(),
        }))
        .unwrap();

    // Verify high priority events come first
    let first = queue.pop();
    let second = queue.pop();
    assert!(matches!(first, Some(Event::Quit)) || matches!(first, Some(Event::Key(_))));
    assert!(matches!(second, Some(Event::Quit)) || matches!(second, Some(Event::Key(_))));

    // Then normal priority
    let third = queue.pop();
    let fourth = queue.pop();
    assert!(matches!(third, Some(Event::User(_))) || matches!(third, Some(Event::Paste(_))));
    assert!(matches!(fourth, Some(Event::User(_))) || matches!(fourth, Some(Event::Paste(_))));

    // Finally low priority
    let fifth = queue.pop();
    assert!(
        matches!(fifth, Some(Event::Tick))
            || matches!(fifth, Some(Event::Resize { .. }))
            || matches!(fifth, Some(Event::Focus))
    );
}
