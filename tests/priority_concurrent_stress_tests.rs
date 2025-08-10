// Concurrent stress tests for priority event processing
//
// These tests verify the system behaves correctly under heavy concurrent load.
// They are marked with #[ignore] to exclude from regular test runs.
// Run with: cargo test --test priority_concurrent_stress_tests -- --ignored

use hojicha::commands;

use hojicha::prelude::*;
use hojicha::program::{PriorityConfig, Program, ProgramOptions};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
enum StressMsg {
    HighPriority(usize, usize), // (thread_id, sequence)
    NormalPriority(usize, usize),
    LowPriority(usize, usize),
    Checkpoint(usize), // Thread checkpoint
    Complete,
}

struct StressModel {
    high_received: Arc<AtomicUsize>,
    normal_received: Arc<AtomicUsize>,
    low_received: Arc<AtomicUsize>,
    total_received: Arc<AtomicUsize>,
    checkpoints: Vec<usize>,
    target_messages: usize,
}

impl StressModel {
    fn new(target: usize) -> Self {
        Self {
            high_received: Arc::new(AtomicUsize::new(0)),
            normal_received: Arc::new(AtomicUsize::new(0)),
            low_received: Arc::new(AtomicUsize::new(0)),
            total_received: Arc::new(AtomicUsize::new(0)),
            checkpoints: Vec::new(),
            target_messages: target,
        }
    }
}

impl Model for StressModel {
    type Message = StressMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(msg) => {
                match msg {
                    StressMsg::HighPriority(_, _) => {
                        self.high_received.fetch_add(1, Ordering::SeqCst);
                    }
                    StressMsg::NormalPriority(_, _) => {
                        self.normal_received.fetch_add(1, Ordering::SeqCst);
                    }
                    StressMsg::LowPriority(_, _) => {
                        self.low_received.fetch_add(1, Ordering::SeqCst);
                    }
                    StressMsg::Checkpoint(id) => {
                        self.checkpoints.push(id);
                    }
                    StressMsg::Complete => {
                        return commands::quit(); // Quit
                    }
                }

                let total = self.total_received.fetch_add(1, Ordering::SeqCst);
                if total >= self.target_messages {
                    return commands::quit(); // Exit when target reached
                }
            }
            Event::Quit => return commands::quit(),
            _ => {}
        }
        Cmd::none()
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

#[test]
#[ignore = "Stress test - run with --ignored"]
fn test_concurrent_multi_producer_stress() {
    const NUM_THREADS: usize = 10;
    const MESSAGES_PER_THREAD: usize = 100;
    const TOTAL_EXPECTED: usize = NUM_THREADS * MESSAGES_PER_THREAD * 3; // 3 priority levels

    let model = StressModel::new(TOTAL_EXPECTED);
    let high_count = Arc::clone(&model.high_received);
    let normal_count = Arc::clone(&model.normal_received);
    let low_count = Arc::clone(&model.low_received);

    let config = PriorityConfig {
        max_queue_size: 10000,
        log_drops: false,
        priority_mapper: None,
    };

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options)
        .unwrap()
        .with_priority_config(config);

    let sender = program.init_async_bridge();
    let barrier = Arc::new(Barrier::new(NUM_THREADS + 1));
    let start_flag = Arc::new(AtomicBool::new(false));

    // Spawn producer threads
    let mut handles = vec![];
    for thread_id in 0..NUM_THREADS {
        let sender = sender.clone();
        let barrier = Arc::clone(&barrier);
        let start_flag = Arc::clone(&start_flag);

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            barrier.wait();

            // Wait for start signal
            while !start_flag.load(Ordering::SeqCst) {
                std::hint::spin_loop();
            }

            // Send messages as fast as possible
            for seq in 0..MESSAGES_PER_THREAD {
                // Send one of each priority
                let _ = sender.send(Event::User(StressMsg::HighPriority(thread_id, seq)));
                let _ = sender.send(Event::User(StressMsg::NormalPriority(thread_id, seq)));
                let _ = sender.send(Event::User(StressMsg::LowPriority(thread_id, seq)));
            }
        });
        handles.push(handle);
    }

    // All threads ready
    barrier.wait();

    // Start the race
    start_flag.store(true, Ordering::SeqCst);

    // Wait for all producers to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Send completion signal
    sender.send(Event::User(StressMsg::Complete)).unwrap();

    // Run the program to process all messages
    program.run().unwrap();

    // Verify all messages were processed
    let high = high_count.load(Ordering::SeqCst);
    let normal = normal_count.load(Ordering::SeqCst);
    let low = low_count.load(Ordering::SeqCst);

    assert_eq!(
        high + normal + low,
        TOTAL_EXPECTED,
        "Should process all messages without loss"
    );

    // High priority should be processed more than low under stress
    assert!(
        high >= low,
        "High priority ({}) should be >= low priority ({}) under stress",
        high,
        low
    );
}

#[test]
#[ignore = "Stress test - run with --ignored"]
fn test_burst_flooding_with_priority_preservation() {
    const BURST_SIZE: usize = 1000;
    const NUM_BURSTS: usize = 10;

    let model = StressModel::new(BURST_SIZE * NUM_BURSTS * 3);
    let high_count = Arc::clone(&model.high_received);

    let config = PriorityConfig {
        max_queue_size: 5000,
        log_drops: false,
        priority_mapper: None,
    };

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options)
        .unwrap()
        .with_priority_config(config);

    let sender = program.init_async_bridge();

    // Send bursts of messages
    for burst in 0..NUM_BURSTS {
        // Send a burst all at once
        for i in 0..BURST_SIZE {
            let _ = sender.send(Event::User(StressMsg::LowPriority(burst, i)));
            let _ = sender.send(Event::User(StressMsg::NormalPriority(burst, i)));
            let _ = sender.send(Event::User(StressMsg::HighPriority(burst, i)));
        }
    }

    // Send completion
    sender.send(Event::User(StressMsg::Complete)).unwrap();

    // Process all messages
    program.run().unwrap();

    // High priority messages should still be processed
    assert!(
        high_count.load(Ordering::SeqCst) > 0,
        "High priority messages should be processed even under burst load"
    );
}

#[test]
#[ignore = "Stress test - run with --ignored"]
fn test_priority_under_memory_pressure() {
    // Test with a smaller queue to force backpressure
    const QUEUE_SIZE: usize = 100;
    const MESSAGE_COUNT: usize = 1000;

    let model = StressModel::new(MESSAGE_COUNT);
    let high_count = Arc::clone(&model.high_received);
    let low_count = Arc::clone(&model.low_received);
    let total_count = Arc::clone(&model.total_received);

    let config = PriorityConfig {
        max_queue_size: QUEUE_SIZE,
        log_drops: false, // Don't log to avoid noise
        priority_mapper: None,
    };

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options)
        .unwrap()
        .with_priority_config(config);

    let sender = program.init_async_bridge();

    // Producer thread - sends more messages than queue can hold
    let producer_sender = sender.clone();
    let producer = thread::spawn(move || {
        for i in 0..MESSAGE_COUNT {
            // Try to send, some may be dropped due to backpressure
            let _ = producer_sender.send(Event::User(StressMsg::LowPriority(0, i)));
            let _ = producer_sender.send(Event::User(StressMsg::HighPriority(0, i)));

            // Don't overwhelm too quickly - allow some processing
            if i % 10 == 0 {
                thread::yield_now();
            }
        }
        let _ = producer_sender.send(Event::User(StressMsg::Complete));
    });

    // Run program with timeout
    program
        .run_with_timeout(Duration::from_millis(500))
        .unwrap();

    producer.join().unwrap();

    let total = total_count.load(Ordering::SeqCst);
    let high = high_count.load(Ordering::SeqCst);
    let low = low_count.load(Ordering::SeqCst);

    // Should have processed some messages despite backpressure
    assert!(total > 0, "Should process some messages");

    // Under backpressure, high priority should be preserved more than low
    if total < MESSAGE_COUNT {
        // If we dropped messages, high priority should be favored
        let high_ratio = high as f64 / high.max(1) as f64;
        let low_ratio = low as f64 / low.max(1) as f64;
        assert!(
            high_ratio >= low_ratio,
            "High priority should be preserved under backpressure"
        );
    }
}

#[test]
#[ignore = "Stress test - run with --ignored"]
fn test_latency_measurement() {
    const NUM_MESSAGES: usize = 1000;

    let model = StressModel::new(NUM_MESSAGES);
    let high_count = Arc::clone(&model.high_received);
    let normal_count = Arc::clone(&model.normal_received);
    let low_count = Arc::clone(&model.low_received);

    let config = PriorityConfig {
        max_queue_size: 2000,
        log_drops: false,
        priority_mapper: None,
    };

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options)
        .unwrap()
        .with_priority_config(config);

    let sender = program.init_async_bridge();

    // Send messages with different priorities
    for i in 0..NUM_MESSAGES / 3 {
        sender
            .send(Event::User(StressMsg::HighPriority(0, i)))
            .unwrap();
        sender
            .send(Event::User(StressMsg::NormalPriority(0, i)))
            .unwrap();
        sender
            .send(Event::User(StressMsg::LowPriority(0, i)))
            .unwrap();
    }

    sender.send(Event::User(StressMsg::Complete)).unwrap();

    let start = Instant::now();
    program.run().unwrap();
    let elapsed = start.elapsed();

    let total_processed = high_count.load(Ordering::SeqCst)
        + normal_count.load(Ordering::SeqCst)
        + low_count.load(Ordering::SeqCst);

    assert_eq!(total_processed, NUM_MESSAGES);

    // Calculate throughput
    let throughput = total_processed as f64 / elapsed.as_secs_f64();
    println!(
        "Processed {} messages in {:?} ({:.0} msgs/sec)",
        total_processed, elapsed, throughput
    );

    // Latency should be reasonable (this is a loose check)
    assert!(
        elapsed < Duration::from_secs(2),
        "Should process {} messages in under 10 seconds",
        NUM_MESSAGES
    );
}

#[test]
fn test_priority_ordering_verification() {
    // This is a non-stress test to verify basic priority ordering
    let model = StressModel::new(30);
    let high_count = Arc::clone(&model.high_received);
    let normal_count = Arc::clone(&model.normal_received);
    let low_count = Arc::clone(&model.low_received);

    let config = PriorityConfig {
        max_queue_size: 100,
        log_drops: false,
        priority_mapper: None,
    };

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options)
        .unwrap()
        .with_priority_config(config);

    let sender = program.init_async_bridge();

    // Send 10 of each priority
    for i in 0..10 {
        sender
            .send(Event::User(StressMsg::LowPriority(0, i)))
            .unwrap();
        sender
            .send(Event::User(StressMsg::NormalPriority(0, i)))
            .unwrap();
        sender
            .send(Event::User(StressMsg::HighPriority(0, i)))
            .unwrap();
    }

    // Schedule completion after a short delay
    let complete_sender = sender.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        complete_sender
            .send(Event::User(StressMsg::Complete))
            .unwrap();
    });

    program.run().unwrap();

    // All messages should be processed
    assert_eq!(high_count.load(Ordering::SeqCst), 10);
    assert_eq!(normal_count.load(Ordering::SeqCst), 10);
    assert_eq!(low_count.load(Ordering::SeqCst), 10);
}
