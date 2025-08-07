//! Tests for the Stream subscription API

use hojicha::{
    core::{Cmd, Model},
    event::Event,
    program::{Program, ProgramOptions},
};
use futures::stream::{self, StreamExt};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

#[derive(Clone)]
struct StreamTestModel {
    received: Arc<AtomicUsize>,
}

#[derive(Debug, Clone)]
enum TestMsg {
    StreamValue(i32),
    Quit,
}

impl Model for StreamTestModel {
    type Message = TestMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(TestMsg::StreamValue(_)) => {
                let count = self.received.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= 10 {
                    None // Quit after 10 messages
                } else {
                    Cmd::none()
                }
            }
            Event::User(TestMsg::Quit) => None,
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

#[tokio::test]
async fn test_stream_subscription_basic() {
    let model = StreamTestModel {
        received: Arc::new(AtomicUsize::new(0)),
    };
    let counter = Arc::clone(&model.received);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();

    // Create a stream of values with small delays to simulate async behavior
    let stream = stream::iter(0..10).then(|i| async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        TestMsg::StreamValue(i)
    });

    // Subscribe to the stream
    let subscription = program.subscribe(stream);

    // Give the stream task time to start
    std::thread::sleep(Duration::from_millis(100));

    // Check if subscription is active
    assert!(subscription.is_active(), "Subscription should be active");

    // Run the program in a blocking task
    tokio::task::spawn_blocking(move || {
        let _ = program.run_with_timeout(Duration::from_secs(2));
    })
    .await
    .unwrap();

    // Check that we received all values
    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[test]
fn test_stream_subscription_cancellation() {
    let model = StreamTestModel {
        received: Arc::new(AtomicUsize::new(0)),
    };
    let counter = Arc::clone(&model.received);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();

    // Create an infinite stream
    let stream = stream::repeat(1).map(TestMsg::StreamValue);

    // Subscribe and immediately cancel
    let subscription = program.subscribe(stream);
    subscription.cancel();

    // Run the program briefly
    let _ = program.run_with_timeout(Duration::from_millis(100));

    // Should receive very few or no values due to cancellation
    let count = counter.load(Ordering::SeqCst);
    assert!(
        count < 5,
        "Should receive very few values after cancellation, got {}",
        count
    );
}

#[test]
fn test_multiple_stream_subscriptions() {
    let model = StreamTestModel {
        received: Arc::new(AtomicUsize::new(0)),
    };
    let counter = Arc::clone(&model.received);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();

    // Subscribe to multiple streams
    let stream1 = stream::iter(0..5).map(TestMsg::StreamValue);
    let stream2 = stream::iter(5..10).map(TestMsg::StreamValue);

    let _sub1 = program.subscribe(stream1);
    let _sub2 = program.subscribe(stream2);

    // Run the program
    let _ = program.run_with_timeout(Duration::from_secs(2));

    // Should receive values from both streams
    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[tokio::test]
async fn test_async_stream_subscription() {
    use tokio_stream::wrappers::IntervalStream;

    let model = StreamTestModel {
        received: Arc::new(AtomicUsize::new(0)),
    };
    let counter = Arc::clone(&model.received);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();

    // Create an async interval stream
    let interval = tokio::time::interval(Duration::from_millis(10));
    let stream = IntervalStream::new(interval)
        .take(10)
        .map(|_| TestMsg::StreamValue(1));

    // Subscribe to the async stream
    let _subscription = program.subscribe(stream);

    // Run the program in a blocking task
    tokio::task::spawn_blocking(move || {
        let _ = program.run_with_timeout(Duration::from_secs(3));
    })
    .await
    .unwrap();

    // Should receive 10 ticks
    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[test]
fn test_stream_with_delay() {
    let model = StreamTestModel {
        received: Arc::new(AtomicUsize::new(0)),
    };
    let counter = Arc::clone(&model.received);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();

    // Create a stream with delays between items
    let stream = stream::iter(0..5).then(|i| async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        TestMsg::StreamValue(i)
    });

    // Subscribe to the stream
    let _subscription = program.subscribe(stream);

    // Run the program
    let _ = program.run_with_timeout(Duration::from_secs(2));

    // Should receive all values despite delays
    let count = counter.load(Ordering::SeqCst);
    assert!(
        count >= 5,
        "Should receive at least 5 values, got {}",
        count
    );
}
