use hojicha::{
    commands,
    core::{Cmd, Model},
    event::Event,
    program::{Program, ProgramOptions},
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct EventLoopTestModel {
    events_received: Arc<Mutex<Vec<String>>>,
    update_count: Arc<Mutex<usize>>,
    should_quit_after: Option<usize>,
    start_time: Instant,
}

#[derive(Debug, Clone)]
enum TestMsg {
    Tick,
    AsyncComplete(String),
    ExternalMessage(String),
    Quit,
}

impl Model for EventLoopTestModel {
    type Message = TestMsg;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        let tick_cmd = commands::tick(Duration::from_millis(10), || TestMsg::Tick);
        Some(tick_cmd)
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        let mut count = self.update_count.lock().unwrap();
        *count += 1;
        let current_count = *count;

        let mut events = self.events_received.lock().unwrap();
        events.push(format!("{:?}", event));

        // Quit after specified number of updates
        if let Some(quit_after) = self.should_quit_after {
            if current_count >= quit_after {
                return None; // Quit
            }
        }

        match event {
            Event::User(TestMsg::Tick) => {
                // Schedule next tick
                Some(commands::tick(Duration::from_millis(10), || TestMsg::Tick))
            }
            Event::User(TestMsg::Quit) => None,
            Event::User(TestMsg::AsyncComplete(msg)) => {
                events.push(format!("Async: {}", msg));
                Cmd::none()
            }
            Event::User(TestMsg::ExternalMessage(msg)) => {
                events.push(format!("External: {}", msg));
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
}

#[test]
fn test_event_loop_basic_operation() {
    let model = EventLoopTestModel {
        events_received: Arc::new(Mutex::new(Vec::new())),
        update_count: Arc::new(Mutex::new(0)),
        should_quit_after: Some(5), // Quit after 5 events
        start_time: Instant::now(),
    };

    let events_clone = Arc::clone(&model.events_received);
    let count_clone = Arc::clone(&model.update_count);

    let options = ProgramOptions::default().headless();
    let program = Program::with_options(model, options).unwrap();

    program
        .run_with_timeout(Duration::from_millis(100))
        .unwrap();

    let final_count = *count_clone.lock().unwrap();
    assert!(
        final_count >= 5,
        "Should have processed at least 5 events, got {}",
        final_count
    );

    let events = events_clone.lock().unwrap();
    assert!(!events.is_empty(), "Should have received events");
}

#[test]
fn test_event_loop_with_async_bridge() {
    let model = EventLoopTestModel {
        events_received: Arc::new(Mutex::new(Vec::new())),
        update_count: Arc::new(Mutex::new(0)),
        should_quit_after: None,
        start_time: Instant::now(),
    };

    let events_clone = Arc::clone(&model.events_received);
    let count_clone = Arc::clone(&model.update_count);

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    // Initialize async bridge
    program.init_async_bridge();
    let sender = program
        .sender()
        .expect("async bridge should be initialized");

    // Send messages immediately
    for i in 0..5 {
        let _ = sender.send(Event::User(TestMsg::ExternalMessage(format!("msg_{}", i))));
    }
    let _ = sender.send(Event::User(TestMsg::Quit));

    // Run the program
    program
        .run_with_timeout(Duration::from_millis(100))
        .unwrap();

    let events = events_clone.lock().unwrap();
    let external_msgs: Vec<_> = events.iter().filter(|e| e.contains("External")).collect();

    assert!(
        external_msgs.len() >= 5,
        "Should have received all external messages"
    );
}

#[test]
fn test_event_loop_concurrent_senders() {
    let model = EventLoopTestModel {
        events_received: Arc::new(Mutex::new(Vec::new())),
        update_count: Arc::new(Mutex::new(0)),
        should_quit_after: None,
        start_time: Instant::now(),
    };

    let events_clone = Arc::clone(&model.events_received);
    let count_clone = Arc::clone(&model.update_count);

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    program.init_async_bridge();
    let sender = program
        .sender()
        .expect("async bridge should be initialized");

    // Use a barrier to coordinate thread starts
    use std::sync::Barrier;
    let barrier = Arc::new(Barrier::new(4)); // 3 sender threads + main

    // Spawn multiple sender threads
    let mut handles = vec![];
    for thread_id in 0..3 {
        let sender_clone = sender.clone();
        let barrier_clone = barrier.clone();
        let handle = thread::spawn(move || {
            barrier_clone.wait(); // Synchronize start
            for i in 0..10 {
                let msg = format!("t{}_m{}", thread_id, i);
                let _ = sender_clone.send(Event::User(TestMsg::ExternalMessage(msg)));
                thread::yield_now(); // Yield instead of sleep
            }
        });
        handles.push(handle);
    }

    // Start all threads simultaneously
    barrier.wait();

    // Wait for all senders to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Send quit after all messages
    let _ = sender.send(Event::User(TestMsg::Quit));

    program
        .run_with_timeout(Duration::from_millis(200))
        .unwrap();

    let events = events_clone.lock().unwrap();
    let external_msgs: Vec<_> = events.iter().filter(|e| e.contains("External")).collect();

    assert!(
        external_msgs.len() >= 20,
        "Should receive messages from all threads"
    );
}

#[test]
fn test_event_loop_command_execution_order() {
    let model = EventLoopTestModel {
        events_received: Arc::new(Mutex::new(Vec::new())),
        update_count: Arc::new(Mutex::new(0)),
        should_quit_after: Some(10),
        start_time: Instant::now(),
    };

    let events_clone = Arc::clone(&model.events_received);

    let options = ProgramOptions::default().headless();
    let program = Program::with_options(model, options).unwrap();

    program
        .run_with_timeout(Duration::from_millis(500))
        .unwrap();

    let events = events_clone.lock().unwrap();

    // Should have tick events from init
    let tick_events: Vec<_> = events.iter().filter(|e| e.contains("Tick")).collect();

    assert!(!tick_events.is_empty(), "Should have received tick events");
}

#[test]
fn test_event_loop_graceful_shutdown() {
    let model = EventLoopTestModel {
        events_received: Arc::new(Mutex::new(Vec::new())),
        update_count: Arc::new(Mutex::new(0)),
        should_quit_after: None,
        start_time: Instant::now(),
    };

    let count_clone = Arc::clone(&model.update_count);

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    program.init_async_bridge();
    let sender = program
        .sender()
        .expect("async bridge should be initialized");

    // Send quit immediately
    let _ = sender.send(Event::User(TestMsg::Quit));

    let start = Instant::now();
    program.run().unwrap();
    let elapsed = start.elapsed();

    // Should quit promptly after receiving quit message
    assert!(elapsed < Duration::from_millis(100), "Should quit promptly");

    let final_count = *count_clone.lock().unwrap();
    assert!(
        final_count > 0,
        "Should have processed some events before quitting"
    );
}

#[test]
fn test_event_loop_high_frequency_messages() {
    let model = EventLoopTestModel {
        events_received: Arc::new(Mutex::new(Vec::new())),
        update_count: Arc::new(Mutex::new(0)),
        should_quit_after: None,
        start_time: Instant::now(),
    };

    let count_clone = Arc::clone(&model.update_count);

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    program.init_async_bridge();
    let sender = program
        .sender()
        .expect("async bridge should be initialized");

    // Blast messages as fast as possible
    let sender_clone = sender.clone();
    thread::spawn(move || {
        for i in 0..100 {
            let _ = sender_clone.send(Event::User(TestMsg::ExternalMessage(format!("{}", i))));
        }
        let _ = sender_clone.send(Event::User(TestMsg::Quit));
    });

    program.run_with_timeout(Duration::from_secs(1)).unwrap();

    let final_count = *count_clone.lock().unwrap();
    assert!(final_count >= 100, "Should handle high-frequency messages");
}
