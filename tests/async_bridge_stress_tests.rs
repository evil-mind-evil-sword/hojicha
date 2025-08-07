//! Stress tests for async bridge - marked with #[ignore] by default
//!
//! Run with: cargo test --test async_bridge_stress_tests -- --ignored

use hojicha::{
    core::{Cmd, Model},
    event::Event,
    program::{Program, ProgramOptions},
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Barrier,
};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct StressTestModel {
    messages_received: Arc<AtomicUsize>,
    bytes_processed: Arc<AtomicUsize>,
    errors_count: Arc<AtomicUsize>,
    max_messages: usize,
}

#[derive(Debug, Clone)]
enum StressMsg {
    Data(Vec<u8>),
    Burst(usize),
    Error(String),
    Quit,
}

impl Model for StressTestModel {
    type Message = StressMsg;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(StressMsg::Data(bytes)) => {
                self.messages_received.fetch_add(1, Ordering::Relaxed);
                self.bytes_processed
                    .fetch_add(bytes.len(), Ordering::Relaxed);

                if self.messages_received.load(Ordering::Relaxed) >= self.max_messages {
                    return None; // Quit
                }
                Cmd::none()
            }
            Event::User(StressMsg::Burst(count)) => {
                self.messages_received.fetch_add(count, Ordering::Relaxed);
                Cmd::none()
            }
            Event::User(StressMsg::Error(_)) => {
                self.errors_count.fetch_add(1, Ordering::Relaxed);
                Cmd::none()
            }
            Event::User(StressMsg::Quit) => None,
            Event::Tick => {
                // Don't quit on tick events, just continue
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
}

#[test]
#[ignore = "Stress test - run with --ignored"]
fn test_async_bridge_sustained_load() {
    let model = StressTestModel {
        messages_received: Arc::new(AtomicUsize::new(0)),
        bytes_processed: Arc::new(AtomicUsize::new(0)),
        errors_count: Arc::new(AtomicUsize::new(0)),
        max_messages: 10000,
    };

    let messages_clone = Arc::clone(&model.messages_received);
    let bytes_clone = Arc::clone(&model.bytes_processed);

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    let sender = program.init_async_bridge();

    // Send all messages immediately without timing
    for i in 0..10000 {
        let data = vec![i as u8; 100]; // 100 bytes per message
        if sender.send(Event::User(StressMsg::Data(data))).is_err() {
            break;
        }

        // Yield occasionally to allow processing
        if i % 100 == 0 {
            thread::yield_now();
        }
    }

    // Run program - will auto-quit after max_messages
    let start = Instant::now();
    program.run_with_timeout(Duration::from_secs(5)).unwrap();
    let elapsed = start.elapsed();

    let total_messages = messages_clone.load(Ordering::Relaxed);
    let total_bytes = bytes_clone.load(Ordering::Relaxed);

    assert!(
        total_messages >= 5000,
        "Should process most messages under sustained load (got {})",
        total_messages
    );
    assert!(
        total_bytes >= 500_000,
        "Should process expected data volume (got {})",
        total_bytes
    );

    println!(
        "Processed {} messages ({} bytes) in {:?}",
        total_messages, total_bytes, elapsed
    );
}

#[test]
#[ignore = "Stress test - run with --ignored"]
fn test_async_bridge_burst_handling() {
    let model = StressTestModel {
        messages_received: Arc::new(AtomicUsize::new(0)),
        bytes_processed: Arc::new(AtomicUsize::new(0)),
        errors_count: Arc::new(AtomicUsize::new(0)),
        max_messages: 5000, // Auto-quit after expected messages
    };

    let messages_clone = Arc::clone(&model.messages_received);

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    let sender = program.init_async_bridge();

    // Use barrier to coordinate burst start
    let barrier = Arc::new(Barrier::new(6)); // 5 threads + main

    // Create burst senders
    let mut handles = vec![];
    for burst_id in 0..5 {
        let sender = sender.clone();
        let barrier = barrier.clone();
        let handle = thread::spawn(move || {
            barrier.wait(); // Synchronize start

            // Each thread sends a burst of 1000 messages
            for i in 0..1000 {
                let data = vec![(burst_id * 1000 + i) as u8; 50];
                let _ = sender.send(Event::User(StressMsg::Data(data)));
            }
        });
        handles.push(handle);
    }

    // Start all threads simultaneously
    barrier.wait();

    // Run program - will auto-quit after max_messages
    program.run_with_timeout(Duration::from_secs(2)).unwrap();

    // Wait for all burst senders
    for handle in handles {
        handle.join().unwrap();
    }

    let total_messages = messages_clone.load(Ordering::Relaxed);
    assert!(
        total_messages >= 2000,
        "Should handle burst traffic (got {})",
        total_messages
    );
}

#[test]
#[ignore = "Stress test - run with --ignored"]
fn test_async_bridge_many_concurrent_senders() {
    let model = StressTestModel {
        messages_received: Arc::new(AtomicUsize::new(0)),
        bytes_processed: Arc::new(AtomicUsize::new(0)),
        errors_count: Arc::new(AtomicUsize::new(0)),
        max_messages: 2000, // Auto-quit after expected messages
    };

    let messages_clone = Arc::clone(&model.messages_received);

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    let sender = program.init_async_bridge();

    // Create many concurrent senders
    let mut handles = vec![];
    let barrier = Arc::new(Barrier::new(21)); // 20 threads + main

    for sender_id in 0..20 {
        let sender = sender.clone();
        let barrier = barrier.clone();
        let handle = thread::spawn(move || {
            barrier.wait(); // Synchronize start

            for i in 0..100 {
                let data = vec![(sender_id * 100 + i) as u8; 10];
                let _ = sender.send(Event::User(StressMsg::Data(data)));
                // Yield instead of sleep
                if i % 10 == 0 {
                    thread::yield_now();
                }
            }
        });
        handles.push(handle);
    }

    // Start all threads simultaneously
    barrier.wait();

    // Run program - will auto-quit after max_messages
    program.run_with_timeout(Duration::from_secs(2)).unwrap();

    for handle in handles {
        handle.join().unwrap();
    }

    let total_messages = messages_clone.load(Ordering::Relaxed);
    assert!(
        total_messages >= 500,
        "Should handle many concurrent senders (got {})",
        total_messages
    );
}

#[test]
#[ignore = "Stress test - run with --ignored"]
fn test_async_bridge_memory_stability() {
    let model = StressTestModel {
        messages_received: Arc::new(AtomicUsize::new(0)),
        bytes_processed: Arc::new(AtomicUsize::new(0)),
        errors_count: Arc::new(AtomicUsize::new(0)),
        max_messages: 1000,
    };

    let messages_clone = Arc::clone(&model.messages_received);
    let bytes_clone = Arc::clone(&model.bytes_processed);

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    let sender = program.init_async_bridge();

    // Send all messages immediately
    for i in 0..1000 {
        // Vary message size to stress memory allocation
        let size = (i % 10) * 1000 + 100;
        let data = vec![(i % 256) as u8; size];
        let _ = sender.send(Event::User(StressMsg::Data(data)));

        // Yield periodically to allow processing
        if i % 50 == 0 {
            thread::yield_now();
        }
    }

    let start = Instant::now();
    program.run_with_timeout(Duration::from_secs(3)).unwrap();
    let elapsed = start.elapsed();

    let total_messages = messages_clone.load(Ordering::Relaxed);
    let total_bytes = bytes_clone.load(Ordering::Relaxed);

    assert!(
        total_messages >= 900,
        "Should handle varied message sizes (got {})",
        total_messages
    );
    assert!(
        total_bytes > 1_000_000,
        "Should process significant data volume (got {})",
        total_bytes
    );

    println!(
        "Processed {} messages ({} bytes) in {:?}",
        total_messages, total_bytes, elapsed
    );
}

#[test]
#[ignore = "Stress test - run with --ignored"]
fn test_async_bridge_channel_overflow_recovery() {
    let model = StressTestModel {
        messages_received: Arc::new(AtomicUsize::new(0)),
        bytes_processed: Arc::new(AtomicUsize::new(0)),
        errors_count: Arc::new(AtomicUsize::new(0)),
        max_messages: 5000, // Auto-quit after some messages
    };

    let messages_clone = Arc::clone(&model.messages_received);
    let errors_clone = Arc::clone(&model.errors_count);

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    let sender = program.init_async_bridge();

    // Send messages without any delay to test overflow handling
    let mut error_count = 0;
    for i in 0..10000 {
        let data = vec![i as u8; 1000];
        match sender.send(Event::User(StressMsg::Data(data))) {
            Ok(_) => {}
            Err(_) => {
                error_count += 1;
                // Try to send error message (may also fail)
                let _ = sender.send(Event::User(StressMsg::Error("Channel full".to_string())));
            }
        }

        // Yield occasionally
        if i % 1000 == 0 {
            thread::yield_now();
        }
    }

    // Run program - will auto-quit after max_messages
    program.run_with_timeout(Duration::from_secs(2)).unwrap();

    let total_messages = messages_clone.load(Ordering::Relaxed);

    // Should handle some messages even under extreme pressure
    assert!(
        total_messages > 0,
        "Should process messages despite pressure (got {})",
        total_messages
    );

    println!(
        "Processed {} messages, {} send errors",
        total_messages, error_count
    );
}
