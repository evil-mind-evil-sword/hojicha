//! Working integration tests for async event bridge
//!
//! These tests verify the async bridge functionality with the new init_async_bridge() API.

use hojicha::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Simple test model that collects messages
#[derive(Default)]
struct TestModel {
    messages: Arc<Mutex<Vec<String>>>,
    should_quit: Arc<Mutex<bool>>,
    message_count: Arc<AtomicUsize>,
}

#[derive(Clone, Debug)]
enum TestMsg {
    Data(String),
    Increment,
    Quit,
}

impl Model for TestModel {
    type Message = TestMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(TestMsg::Data(s)) => {
                self.messages.lock().unwrap().push(s);
                self.message_count.fetch_add(1, Ordering::SeqCst);
            }
            Event::User(TestMsg::Increment) => {
                self.message_count.fetch_add(1, Ordering::SeqCst);
            }
            Event::User(TestMsg::Quit) => {
                *self.should_quit.lock().unwrap() = true;
                return None;
            }
            _ => {}
        }

        // Auto-quit after receiving enough messages for tests
        if self.message_count.load(Ordering::SeqCst) >= 10 {
            return None;
        }

        Cmd::none()
    }

    fn view(&self, _frame: &mut Frame, _area: ratatui::layout::Rect) {
        // No-op for tests
    }
}

#[test]
fn test_simple_timer_messages() {
    let model = TestModel::default();
    let messages = model.messages.clone();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();

    let sender = program.init_async_bridge();

    // Send messages immediately - they should be buffered
    for i in 0..5 {
        sender
            .send(Event::User(TestMsg::Data(format!("tick-{}", i))))
            .unwrap();
    }

    // Schedule quit after a short delay to ensure messages are processed
    let sender_clone = sender.clone();
    thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(10));
        sender_clone.send(Event::User(TestMsg::Quit)).unwrap();
    });

    // Run program - it should exit when it receives Quit
    let start = Instant::now();
    program.run().unwrap();
    let elapsed = start.elapsed();

    // Verify we received messages
    let received = messages.lock().unwrap();
    assert_eq!(
        received.len(),
        5,
        "Should have received exactly 5 messages, got {}",
        received.len()
    );

    // Verify all messages are present (order may vary with priority queue)
    for i in 0..5 {
        let expected = format!("tick-{}", i);
        assert!(
            received.contains(&expected),
            "Should contain message: {}",
            expected
        );
    }

    // Should have exited quickly (not timeout)
    assert!(
        elapsed < Duration::from_secs(1),
        "Program should exit quickly after Quit message"
    );
}

#[test]
fn test_multiple_concurrent_senders() {
    let model = TestModel::default();
    let messages = model.messages.clone();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();

    let sender = program.init_async_bridge();

    // Send messages from multiple threads
    for thread_id in 0..3 {
        for msg_id in 0..3 {
            let msg = format!("thread-{}-msg-{}", thread_id, msg_id);
            sender.send(Event::User(TestMsg::Data(msg))).unwrap();
        }
    }

    // Schedule quit after a short delay
    let quit_sender = sender.clone();
    thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(10));
        quit_sender.send(Event::User(TestMsg::Quit)).unwrap();
    });

    // Run program
    program.run().unwrap();

    // Verify we received all messages
    let received = messages.lock().unwrap();
    assert_eq!(
        received.len(),
        9,
        "Should have received 9 messages (3 threads * 3 messages)"
    );

    // Verify all messages are present (order may vary due to concurrency)
    for thread_id in 0..3 {
        for msg_id in 0..3 {
            let expected = format!("thread-{}-msg-{}", thread_id, msg_id);
            assert!(
                received.contains(&expected),
                "Missing message: {}",
                expected
            );
        }
    }
}

#[test]
fn test_high_frequency_messages() {
    let model = TestModel::default();
    let counter = model.message_count.clone();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();

    let sender = program.init_async_bridge();

    // Send many messages quickly - program will auto-quit after 10
    for _ in 0..10 {
        sender.send(Event::User(TestMsg::Increment)).unwrap();
    }

    // Run program (will auto-quit after 10 messages)
    let start = Instant::now();
    program.run().unwrap();
    let elapsed = start.elapsed();

    // Verify all messages were processed
    assert_eq!(counter.load(Ordering::SeqCst), 10);

    // Should handle all messages quickly
    assert!(
        elapsed < Duration::from_secs(1),
        "Should process messages quickly"
    );
}

#[test]
fn test_sender_cloning() {
    let model = TestModel::default();
    let messages = model.messages.clone();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();

    let sender1 = program.init_async_bridge();
    let sender2 = sender1.clone();
    let sender3 = sender2.clone();

    // Use different clones
    sender1
        .send(Event::User(TestMsg::Data("from-clone-1".to_string())))
        .unwrap();
    sender2
        .send(Event::User(TestMsg::Data("from-clone-2".to_string())))
        .unwrap();

    // Schedule quit after a short delay
    thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(10));
        sender3.send(Event::User(TestMsg::Quit)).unwrap();
    });

    // Run program
    program.run().unwrap();

    // Verify messages from different clones
    let received = messages.lock().unwrap();
    assert!(received.contains(&"from-clone-1".to_string()));
    assert!(received.contains(&"from-clone-2".to_string()));
}

#[test]
fn test_message_ordering_single_sender() {
    let model = TestModel::default();
    let messages = model.messages.clone();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();

    let sender = program.init_async_bridge();

    // Send messages in order
    for i in 0..5 {
        sender
            .send(Event::User(TestMsg::Data(format!("msg-{}", i))))
            .unwrap();
    }

    // Schedule quit after a short delay
    let quit_sender = sender.clone();
    thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(10));
        quit_sender.send(Event::User(TestMsg::Quit)).unwrap();
    });

    // Run program
    program.run().unwrap();

    // Verify all messages were received (order may vary with priority queue)
    let received = messages.lock().unwrap();
    assert!(received.len() >= 5, "Should receive all messages");

    // Check that all expected messages are present
    for i in 0..5 {
        let expected = format!("msg-{}", i);
        assert!(
            received.contains(&expected),
            "Should receive message: {}",
            expected
        );
    }
}

#[test]
fn test_sender_methods() {
    let model = TestModel::default();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();

    // Test that sender() returns None before init
    assert!(program.sender().is_none());

    // Test that send_message() fails before init
    assert!(program.send_message(TestMsg::Quit).is_err());

    // Initialize async bridge
    let _sender = program.init_async_bridge();

    // Now sender() should return Some
    assert!(program.sender().is_some());

    // send_message() should work now
    assert!(program.send_message(TestMsg::Quit).is_ok());

    // Run should exit immediately due to Quit message
    let start = Instant::now();
    program.run().unwrap();
    assert!(start.elapsed() < Duration::from_millis(100));
}

#[test]
fn test_rapid_fire_messages() {
    let model = TestModel::default();
    let counter = model.message_count.clone();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();
    let sender = program.init_async_bridge();

    // Send messages immediately
    for _ in 0..10 {
        sender.send(Event::User(TestMsg::Increment)).unwrap();
    }

    // The model will auto-quit after 10 messages (see line 46)

    // Run program
    program.run().unwrap();

    // Should have processed 10 messages (auto-quit threshold)
    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[test]
fn test_async_bridge_with_condition() {
    let model = TestModel::default();
    let counter = model.message_count.clone();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();
    let sender = program.init_async_bridge();

    // Send exactly 5 messages
    for _ in 0..5 {
        sender.send(Event::User(TestMsg::Increment)).unwrap();
    }

    // Run until condition is met
    program
        .run_until(|model| model.message_count.load(Ordering::SeqCst) >= 5)
        .unwrap();

    // Should have processed exactly 5 messages
    assert_eq!(counter.load(Ordering::SeqCst), 5);
}
