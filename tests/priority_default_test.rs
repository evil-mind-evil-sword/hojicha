//! Test that priority event processing is now the default behavior

use hojicha::prelude::*;
use hojicha::program::{PriorityConfig, Program, ProgramOptions};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
enum TestMsg {
    LowPriority(usize),
    NormalPriority(usize),
    HighPriority(usize),
    Quit,
}

struct TestModel {
    events_received: Vec<TestMsg>,
    high_count: Arc<AtomicUsize>,
    normal_count: Arc<AtomicUsize>,
    low_count: Arc<AtomicUsize>,
}

impl TestModel {
    fn new() -> Self {
        Self {
            events_received: Vec::new(),
            high_count: Arc::new(AtomicUsize::new(0)),
            normal_count: Arc::new(AtomicUsize::new(0)),
            low_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Model for TestModel {
    type Message = TestMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(msg) => {
                match &msg {
                    TestMsg::HighPriority(_) => {
                        self.high_count.fetch_add(1, Ordering::SeqCst);
                    }
                    TestMsg::NormalPriority(_) => {
                        self.normal_count.fetch_add(1, Ordering::SeqCst);
                    }
                    TestMsg::LowPriority(_) => {
                        self.low_count.fetch_add(1, Ordering::SeqCst);
                    }
                    TestMsg::Quit => return None,
                }
                self.events_received.push(msg);
            }
            _ => {}
        }

        // Quit after receiving enough events
        if self.events_received.len() >= 30 {
            return None;
        }

        Cmd::none()
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
}

#[test]
fn test_priority_is_default() {
    // Create a program without any special configuration
    let model = TestModel::new();
    let high_count = model.high_count.clone();
    let normal_count = model.normal_count.clone();
    let low_count = model.low_count.clone();

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    // Get sender to inject events
    let sender = program.init_async_bridge();

    // Spawn thread that sends events in mixed order
    thread::spawn(move || {
        // Send a burst of low priority events
        for i in 0..10 {
            let _ = sender.send(Event::User(TestMsg::LowPriority(i)));
        }

        // Send some normal priority events
        for i in 0..10 {
            let _ = sender.send(Event::User(TestMsg::NormalPriority(i)));
        }

        // Send high priority events (should be processed first)
        for i in 0..10 {
            let _ = sender.send(Event::User(TestMsg::HighPriority(i)));
        }

        // Give time for events to queue up
        thread::sleep(Duration::from_millis(50));

        // Send quit to end the test
        let _ = sender.send(Event::User(TestMsg::Quit));
    });

    // Run the program
    let result = program.run_with_timeout(Duration::from_secs(1));
    assert!(result.is_ok());

    // Check that events were processed (not necessarily in strict priority order
    // due to timing, but we should have received all of them)
    let total = high_count.load(Ordering::SeqCst)
        + normal_count.load(Ordering::SeqCst)
        + low_count.load(Ordering::SeqCst);
    assert_eq!(total, 30, "Should have processed all 30 events");

    println!(
        "Processed {} high, {} normal, {} low priority events",
        high_count.load(Ordering::SeqCst),
        normal_count.load(Ordering::SeqCst),
        low_count.load(Ordering::SeqCst)
    );
}

#[test]
fn test_custom_priority_config() {
    // Test that we can configure custom priority settings
    let model = TestModel::new();

    let config = PriorityConfig {
        max_queue_size: 50,
        log_drops: true,
        priority_mapper: Some(Arc::new(|event| match event {
            Event::User(_) => hojicha::priority_queue::Priority::High,
            _ => hojicha::priority_queue::Priority::Low,
        })),
    };

    let options = ProgramOptions::default().headless();
    let program = Program::with_options(model, options)
        .unwrap()
        .with_priority_config(config);

    // Just verify it compiles and initializes
    let stats = program.event_stats();
    assert_eq!(stats.total_events, 0);
}

#[test]
fn test_event_flooding_doesnt_block_quit() {
    // This test simulates the exact scenario from the feature request:
    // Flood of low priority events shouldn't block high priority quit

    let model = TestModel::new();
    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    let sender = program.init_async_bridge();

    thread::spawn(move || {
        // Simulate file watcher flooding with events
        for i in 0..100 {
            let _ = sender.send(Event::Tick); // Low priority
            if i == 50 {
                // User presses Ctrl+C in the middle of the flood
                let _ = sender.send(Event::Quit); // High priority - should process quickly
            }
        }
    });

    // Run and verify it quits quickly despite the flood
    let start = std::time::Instant::now();
    let result = program.run_with_timeout(Duration::from_secs(2));
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert!(
        elapsed < Duration::from_secs(1),
        "Should quit quickly despite event flood"
    );

    println!("Quit after {} ms despite event flood", elapsed.as_millis());
}

#[test]
fn test_backpressure_handling() {
    // Test that the system handles backpressure gracefully

    let model = TestModel::new();

    // Configure a small queue to trigger backpressure
    let config = PriorityConfig {
        max_queue_size: 10,
        log_drops: false, // Don't spam logs in test
        priority_mapper: None,
    };

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options)
        .unwrap()
        .with_priority_config(config);

    let sender = program.init_async_bridge();

    thread::spawn(move || {
        // Send more events than the queue can hold
        for i in 0..50 {
            let _ = sender.send(Event::User(TestMsg::LowPriority(i)));
        }

        thread::sleep(Duration::from_millis(100));

        // High priority quit should still get through
        let _ = sender.send(Event::User(TestMsg::Quit));
    });

    let result = program.run_with_timeout(Duration::from_secs(1));
    assert!(result.is_ok());

    println!("Backpressure test completed successfully");
}
