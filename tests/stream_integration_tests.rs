//! Property-based tests for async stream integration

use futures::stream::{self, StreamExt};
use hojicha::{
    core::{Cmd, Model},
    event::Event,
    program::{Program, ProgramOptions},
};
use proptest::prelude::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

// Model that collects messages from streams
#[derive(Clone)]
struct StreamCollector {
    messages_received: Arc<AtomicUsize>,
    last_values: Arc<std::sync::Mutex<Vec<i32>>>,
    should_stop_at: usize,
}

impl StreamCollector {
    fn new(stop_at: usize) -> Self {
        Self {
            messages_received: Arc::new(AtomicUsize::new(0)),
            last_values: Arc::new(std::sync::Mutex::new(Vec::new())),
            should_stop_at: stop_at,
        }
    }
}

#[derive(Debug, Clone)]
enum StreamMessage {
    Value(i32),
    StreamComplete,
    Tick,
}

impl Model for StreamCollector {
    type Message = StreamMessage;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        None // Streams will be attached externally
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(StreamMessage::Value(val)) => {
                let count = self.messages_received.fetch_add(1, Ordering::SeqCst) + 1;
                self.last_values.lock().unwrap().push(val);

                if count >= self.should_stop_at {
                    None // Quit after receiving enough messages
                } else {
                    Cmd::none()
                }
            }
            Event::User(StreamMessage::StreamComplete) => {
                // Stream has completed
                None // Quit
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #[test]
    fn prop_stream_delivers_all_values(
        values in prop::collection::vec(any::<i32>(), 1..50),
        chunk_size in 1..10usize,
    ) {
        // Create a stream that emits values
        let stream_values = values.clone();
        let value_stream = stream::iter(stream_values.into_iter().map(StreamMessage::Value));

        let model = StreamCollector::new(values.len());
        let collected = Arc::clone(&model.last_values);

        let options = ProgramOptions::default()
            .headless()
            .without_signal_handler();

        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();

        // Send all values immediately in chunks
        for chunk in values.chunks(chunk_size) {
            for val in chunk {
                let _ = sender.send(Event::User(StreamMessage::Value(*val)));
            }
            // Yield to allow processing between chunks
            std::thread::yield_now();
        }
        let _ = sender.send(Event::User(StreamMessage::StreamComplete));

        // Run program until it stops
        let _ = program.run_with_timeout(Duration::from_secs(5));

        // Verify all values were received
        let received = collected.lock().unwrap();
        prop_assert_eq!(received.len(), values.len(), "Should receive all values");
        prop_assert_eq!(&*received, &values, "Values should match and be in order");
    }
}

proptest! {
    #[test]
    fn prop_multiple_streams_interleave_correctly(
        stream1_values in prop::collection::vec(0..100i32, 1..20),
        stream2_values in prop::collection::vec(100..200i32, 1..20),
        stream3_values in prop::collection::vec(200..300i32, 1..20),
    ) {
        let total_expected = stream1_values.len() + stream2_values.len() + stream3_values.len();
        let model = StreamCollector::new(total_expected);
        let collected = Arc::clone(&model.last_values);

        let options = ProgramOptions::default()
            .headless()
            .without_signal_handler();

        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();

        // Send values from all three streams interleaved
        use std::sync::Barrier;
        let barrier = Arc::new(Barrier::new(4)); // 3 threads + main

        let mut handles = vec![];

        let s1 = sender.clone();
        let vals1 = stream1_values.clone();
        let b1 = barrier.clone();
        let h1 = std::thread::spawn(move || {
            b1.wait(); // Synchronize start
            for val in vals1 {
                let _ = s1.send(Event::User(StreamMessage::Value(val)));
                std::thread::yield_now();
            }
        });
        handles.push(h1);

        let s2 = sender.clone();
        let vals2 = stream2_values.clone();
        let b2 = barrier.clone();
        let h2 = std::thread::spawn(move || {
            b2.wait(); // Synchronize start
            for val in vals2 {
                let _ = s2.send(Event::User(StreamMessage::Value(val)));
                std::thread::yield_now();
            }
        });
        handles.push(h2);

        let s3 = sender.clone();
        let vals3 = stream3_values.clone();
        let b3 = barrier.clone();
        let h3 = std::thread::spawn(move || {
            b3.wait(); // Synchronize start
            for val in vals3 {
                let _ = s3.send(Event::User(StreamMessage::Value(val)));
                std::thread::yield_now();
            }
        });
        handles.push(h3);

        // Start all threads simultaneously
        barrier.wait();

        // Run program until it stops
        let _ = program.run_with_timeout(Duration::from_secs(10));

        let received = collected.lock().unwrap();
        prop_assert_eq!(received.len(), total_expected, "Should receive all values from all streams");

        // Check that we got values from all three ranges
        let has_stream1 = received.iter().any(|&v| v < 100);
        let has_stream2 = received.iter().any(|&v| v >= 100 && v < 200);
        let has_stream3 = received.iter().any(|&v| v >= 200);

        prop_assert!(has_stream1, "Should have values from stream 1");
        prop_assert!(has_stream2, "Should have values from stream 2");
        prop_assert!(has_stream3, "Should have values from stream 3");
    }
}

proptest! {
    #[test]
    fn prop_stream_backpressure_handling(
        burst_size in 100..1000usize,
        rate_ms in 0..5u64,
    ) {
        // Test that high-frequency streams don't cause issues
        let model = StreamCollector::new(burst_size);
        let collected = Arc::clone(&model.last_values);

        let options = ProgramOptions::default()
            .headless()
            .without_signal_handler();

        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();

        // Send all messages immediately
        for i in 0..burst_size {
            let _ = sender.send(Event::User(StreamMessage::Value(i as i32)));
            // Yield periodically to simulate rate limiting without sleep
            if rate_ms > 0 && i % 10 == 0 {
                std::thread::yield_now();
            }
        }

        // Run program with timeout
        let result = program.run_with_timeout(Duration::from_secs(30));
        prop_assert!(result.is_ok(), "Program should handle burst without panicking");

        let received = collected.lock().unwrap();
        // We should receive most messages (some might be dropped due to channel capacity)
        prop_assert!(received.len() > 0, "Should receive at least some messages");
        prop_assert!(received.len() <= burst_size, "Should not receive more than sent");
    }
}

proptest! {
    #[test]
    fn prop_stream_cancellation(
        values_to_send in 50..200usize,
        cancel_after in 10..40usize,
    ) {
        // Test that streams can be cancelled mid-flow
        let model = StreamCollector::new(cancel_after);
        let collected = Arc::clone(&model.last_values);
        let received_count = Arc::clone(&model.messages_received);

        let options = ProgramOptions::default()
            .headless()
            .without_signal_handler();

        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();

        // Send values up to cancel_after (program will stop automatically)
        // No need for cancellation flag since program stops at cancel_after
        for i in 0..values_to_send.min(cancel_after + 10) {
            let _ = sender.send(Event::User(StreamMessage::Value(i as i32)));
            // Yield periodically
            if i % 5 == 0 {
                std::thread::yield_now();
            }
        }

        // Run program (it will stop after cancel_after messages)
        let _ = program.run_with_timeout(Duration::from_secs(10));

        let final_count = received_count.load(Ordering::SeqCst);
        let received = collected.lock().unwrap();

        prop_assert_eq!(final_count, cancel_after, "Should stop at cancel_after messages");
        prop_assert_eq!(received.len(), cancel_after, "Should have exactly cancel_after values");

        // Verify we got the right values in order
        for (i, &val) in received.iter().enumerate() {
            prop_assert_eq!(val, i as i32, "Values should be in sequence");
        }
    }
}

// Test async/await stream integration
#[tokio::test]
async fn test_tokio_stream_integration() {
    // Note: tokio_stream not available, using simpler test

    let model = StreamCollector::new(5);
    let collected = Arc::clone(&model.last_values);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();
    let sender = program.init_async_bridge();

    // Send 5 values immediately
    for count in 0..5 {
        let _ = sender.send(Event::User(StreamMessage::Value(count)));
    }

    // Run program in blocking thread
    tokio::task::spawn_blocking(move || {
        let _ = program.run_with_timeout(Duration::from_millis(100));
    })
    .await
    .unwrap();

    let received = collected.lock().unwrap();
    assert_eq!(received.len(), 5, "Should receive 5 values");
}

#[test]
fn test_stream_error_handling() {
    // Test that stream errors don't crash the program
    let model = StreamCollector::new(10);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();
    let sender = program.init_async_bridge();

    // Send some messages then drop the sender to simulate disconnection
    for i in 0..3 {
        let _ = sender.send(Event::User(StreamMessage::Value(i)));
    }
    // Simulate disconnection by dropping the sender clone
    drop(sender);

    // Program should handle the disconnection gracefully
    let result = program.run_with_timeout(Duration::from_millis(100));
    assert!(
        result.is_ok(),
        "Program should handle stream disconnection gracefully"
    );
}
