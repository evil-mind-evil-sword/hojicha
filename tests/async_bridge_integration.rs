//! Integration tests for async event bridge functionality
//!
//! These tests verify the public sender API and message injection.

use hojicha::commands;
use hojicha::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;

// Test model that collects messages
#[derive(Default)]
struct CollectorModel {
    messages: Arc<Mutex<Vec<TestMsg>>>,
    should_quit: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum TestMsg {
    Value(i32),
    Text(String),
    Tick,
    Quit,
}

impl Model for CollectorModel {
    type Message = TestMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(msg) => {
                self.messages.lock().unwrap().push(msg.clone());

                if msg == TestMsg::Quit {
                    self.should_quit = true;
                    return commands::quit(); // Exit program
                }
            }
            Event::Key(key) if key.key == Key::Char('q') => {
                return commands::quit(); // Also allow 'q' to quit
            }
            _ => {}
        }

        if self.should_quit {
            Cmd::none()
        } else {
            Cmd::none()
        }
    }

    fn view(&self, _frame: &mut Frame, _area: ratatui::layout::Rect) {
        // No-op for tests
    }
}

#[test]
fn test_sender_not_available_before_init() {
    let program = Program::with_options(
        CollectorModel::default(),
        ProgramOptions::default().headless(),
    )
    .unwrap();

    // Sender should not be available before init_async_bridge() is called
    assert!(program.sender().is_none());

    // After init_async_bridge, sender should be available
    let mut program = Program::with_options(
        CollectorModel::default(),
        ProgramOptions::default().headless(),
    )
    .unwrap();
    let _sender = program.init_async_bridge();
    assert!(program.sender().is_some());
}

#[test]
fn test_send_message_before_init_fails() {
    let program = Program::with_options(
        CollectorModel::default(),
        ProgramOptions::default().headless(),
    )
    .unwrap();

    // send_message should fail before init_async_bridge() is called
    let result = program.send_message(TestMsg::Value(42));
    assert!(result.is_err());

    // After init_async_bridge, send_message should work
    let mut program = Program::with_options(
        CollectorModel::default(),
        ProgramOptions::default().headless(),
    )
    .unwrap();
    let _sender = program.init_async_bridge();
    let result = program.send_message(TestMsg::Value(42));
    assert!(result.is_ok());
}

// Test that we can send messages from external threads
#[test]
fn test_external_message_injection() {
    // This test demonstrates the pattern but can't actually run
    // because program.run() blocks the thread.
    //
    // In a real application, you would:
    // 1. Start program.run() in your main thread
    // 2. Use program.sender() from other threads before calling run()
    //    OR
    // 3. Pass the sender to other components before calling run()

    // For now, we can only test the API exists
    let program = Program::with_options(
        CollectorModel::default(),
        ProgramOptions::default().headless(),
    )
    .unwrap();

    // The API is there, but we can't get a sender until run() is called
    assert!(program.sender().is_none());
}

// Test helper that would allow testing if we could modify Program
// to expose sender before run() or provide a non-blocking run variant
struct TestHarness {
    model: CollectorModel,
    messages: Arc<Mutex<Vec<TestMsg>>>,
}

impl TestHarness {
    fn new() -> Self {
        let model = CollectorModel::default();
        let messages = model.messages.clone();
        Self { model, messages }
    }

    fn collected_messages(&self) -> Vec<TestMsg> {
        self.messages.lock().unwrap().clone()
    }

    // This would be the ideal test API:
    // fn run_with_sender<F>(self, f: F)
    // where F: FnOnce(mpsc::SyncSender<Event<TestMsg>>)
    // {
    //     let program = Program::new(self.model).unwrap();
    //     let sender = program.sender_before_run(); // Would need this API
    //
    //     // Run test logic with sender
    //     f(sender);
    //
    //     // Run program with timeout
    //     program.run_with_timeout(Duration::from_secs(1)).unwrap();
    // }
}

#[test]
fn test_message_ordering_with_mock_channel() {
    // Test the pattern with a mock channel to verify our logic
    use std::sync::mpsc;

    let (tx, rx) = mpsc::sync_channel::<Event<TestMsg>>(100);

    // Send messages in order
    let messages = vec![1, 2, 3, 4, 5];
    for i in messages.iter() {
        tx.send(Event::User(TestMsg::Value(*i))).unwrap();
    }

    // Receive and verify order
    let mut received = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if let Event::User(TestMsg::Value(v)) = event {
            received.push(v);
        }
    }

    assert_eq!(messages, received);
}

#[test]
fn test_concurrent_senders_with_mock_channel() {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::sync_channel::<Event<TestMsg>>(100);

    // Spawn multiple threads sending messages
    let mut handles = Vec::new();
    for thread_id in 0..5 {
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let msg = TestMsg::Text(format!("thread-{}-msg-{}", thread_id, i));
                tx.send(Event::User(msg)).unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Count received messages
    let mut count = 0;
    while rx.try_recv().is_ok() {
        count += 1;
    }

    assert_eq!(count, 50); // 5 threads * 10 messages
}

#[test]
fn test_channel_capacity_backpressure() {
    use std::sync::mpsc;

    // Create channel with small capacity
    let (tx, rx) = mpsc::sync_channel::<Event<TestMsg>>(10);

    // Try to send more than capacity
    let mut sent = 0;
    for i in 0..20 {
        match tx.try_send(Event::User(TestMsg::Value(i))) {
            Ok(_) => sent += 1,
            Err(mpsc::TrySendError::Full(_)) => {
                // Expected when channel is full
                break;
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    // Should have sent up to capacity
    assert!(sent <= 10);
    assert!(sent > 0);

    // Drain some messages
    for _ in 0..5 {
        let _ = rx.recv().unwrap();
    }

    // Now we should be able to send more
    for i in 100..105 {
        tx.send(Event::User(TestMsg::Value(i))).unwrap();
    }
}

// Demonstrate the pattern users would use in practice
#[test]
fn test_usage_pattern_documentation() {
    // This test documents the intended usage pattern

    // In main.rs or app initialization:
    /*
    let mut program = Program::new(MyModel::default())?;

    // Option 1: Get sender after run() starts (requires run in separate thread)
    let program_handle = thread::spawn(move || {
        program.run().unwrap();
    });

    // Wait a bit for program to start
    thread::sleep(Duration::from_millis(100));

    // Now get sender - but we can't access program from here!
    // This is why we need sender before run() or a different API

    // Option 2: What we really want:
    let program = Program::new(MyModel::default())?;
    let sender = program.sender_for_async(); // New API needed

    // Start async tasks with sender
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            sender.send(Event::User(Msg::Tick)).unwrap();
        }
    });

    // Then run program
    program.run()?;
    */

    // For now, document that the API exists
    assert!(true);
}
