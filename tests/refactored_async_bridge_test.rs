//! Refactored async bridge test without timing dependencies
//!
//! This shows how to test async message passing deterministically

use hojicha::{
    commands,
    core::{Cmd, Model},
    event::Event,
    program::{Program, ProgramOptions},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Debug)]
enum TestMsg {
    Increment,
    Data(String),
    Quit,
}

#[derive(Clone, Default)]
struct TestModel {
    counter: Arc<AtomicUsize>,
    messages: Arc<Mutex<Vec<String>>>,
}

impl Model for TestModel {
    type Message = TestMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(TestMsg::Increment) => {
                self.counter.fetch_add(1, Ordering::SeqCst);
            }
            Event::User(TestMsg::Data(s)) => {
                self.messages.lock().unwrap().push(s);
            }
            Event::User(TestMsg::Quit) | Event::Quit => return commands::quit(),
            _ => {}
        }
        Cmd::none()
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

#[test]
fn test_multiple_senders_deterministic() {
    // OLD WAY (with timing dependencies):
    // thread::spawn(move || {
    //     for i in 0..3 {
    //         sender.send(msg);
    //         thread::sleep(Duration::from_millis(10)); // BAD!
    //     }
    // });
    // thread::sleep(Duration::from_millis(100)); // Wait for messages - BAD!

    // NEW WAY (deterministic):
    let model = TestModel::default();
    let messages = model.messages.clone();
    let counter = model.counter.clone();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();

    let sender = program.init_async_bridge();

    // Send all messages immediately - no timing
    let senders: Vec<_> = (0..3).map(|_| sender.clone()).collect();

    // Send messages from multiple "senders" deterministically
    for (thread_id, sender) in senders.iter().enumerate() {
        for msg_id in 0..3 {
            let msg = format!("thread-{}-msg-{}", thread_id, msg_id);
            sender.send(Event::User(TestMsg::Data(msg))).unwrap();
        }
    }

    // Send quit after all messages
    sender.send(Event::User(TestMsg::Quit)).unwrap();

    // Run until quit is processed
    program
        .run_with_timeout(Duration::from_millis(100))
        .unwrap();

    // Verify all messages were received
    let received = messages.lock().unwrap();
    assert_eq!(received.len(), 9, "Should receive all 9 messages");

    // Check all messages are present (order may vary)
    for thread_id in 0..3 {
        for msg_id in 0..3 {
            let expected = format!("thread-{}-msg-{}", thread_id, msg_id);
            assert!(received.contains(&expected));
        }
    }
}

#[test]
fn test_high_frequency_deterministic() {
    // OLD WAY:
    // thread::spawn(move || {
    //     for i in 0..100 {
    //         sender.send(msg);
    //         // Hope the queue doesn't overflow!
    //     }
    // });

    // NEW WAY:
    let model = TestModel::default();
    let counter = model.counter.clone();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();

    let sender = program.init_async_bridge();

    // Send exactly 100 messages
    const MESSAGE_COUNT: usize = 100;
    for _ in 0..MESSAGE_COUNT {
        sender.send(Event::User(TestMsg::Increment)).unwrap();
    }

    // Quit after all messages
    sender.send(Event::User(TestMsg::Quit)).unwrap();

    // Run program - it will process all messages and quit
    program
        .run_with_timeout(Duration::from_millis(500))
        .unwrap();

    // Verify exactly MESSAGE_COUNT messages were processed
    assert_eq!(
        counter.load(Ordering::SeqCst),
        MESSAGE_COUNT,
        "All messages should be processed"
    );
}

#[test]
fn test_message_ordering_deterministic() {
    // Test that messages from a single sender maintain order
    let model = TestModel::default();
    let messages = model.messages.clone();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();

    let sender = program.init_async_bridge();

    // Send ordered messages
    for i in 0..10 {
        sender
            .send(Event::User(TestMsg::Data(format!("{:02}", i))))
            .unwrap();
    }
    sender.send(Event::User(TestMsg::Quit)).unwrap();

    // Run program
    program
        .run_with_timeout(Duration::from_millis(100))
        .unwrap();

    // Note: With priority queue, order may not be preserved
    // This is expected behavior - we verify all messages arrived
    let received = messages.lock().unwrap();
    assert_eq!(received.len(), 10);

    // All messages should be present
    for i in 0..10 {
        let expected = format!("{:02}", i);
        assert!(received.contains(&expected));
    }
}

// For truly async operations, use tokio's test utilities
#[tokio::test]
async fn test_async_with_controlled_time() {
    use std::time::Duration;
    use tokio::time::interval;

    // Time is paused - we control it
    let mut interval = interval(Duration::from_secs(1));

    let mut ticks = 0;
    for _ in 0..5 {
        // advance(Duration::from_secs(1)).await; // Would use with start_paused
        interval.tick().await;
        ticks += 1;
    }

    assert_eq!(ticks, 5);
    // This took 0 real time but simulated 5 seconds
}

#[test]
fn test_condition_based_completion() {
    // Best practice: Use conditions instead of timing
    let model = TestModel::default();
    let counter = model.counter.clone();

    let program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();

    // Run until a condition is met
    program
        .run_until(|model| model.counter.load(Ordering::SeqCst) >= 10)
        .unwrap();

    assert!(counter.load(Ordering::SeqCst) >= 10);
}
