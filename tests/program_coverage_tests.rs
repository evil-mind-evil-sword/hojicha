//! Additional tests to improve coverage of program.rs and related modules

use hojicha::{
    commands,
    core::{Cmd, Model},
    event::{Event, Key, KeyEvent, KeyModifiers},
    program::{MouseMode, Program, ProgramOptions},
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use std::thread;
use std::time::Duration;

// Test model that can be controlled externally
#[derive(Clone)]
struct ControlledModel {
    update_count: Arc<AtomicU32>,
    should_quit_after: Arc<AtomicU32>,
    init_called: Arc<AtomicBool>,
    view_called: Arc<AtomicBool>,
    events_received: Arc<Mutex<Vec<String>>>,
}

impl ControlledModel {
    fn new(quit_after: u32) -> Self {
        Self {
            update_count: Arc::new(AtomicU32::new(0)),
            should_quit_after: Arc::new(AtomicU32::new(quit_after)),
            init_called: Arc::new(AtomicBool::new(false)),
            view_called: Arc::new(AtomicBool::new(false)),
            events_received: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Model for ControlledModel {
    type Message = String;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        self.init_called.store(true, Ordering::SeqCst);
        // Return a command that sends a message
        Some(commands::custom(|| Some("init_message".to_string())))
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        let count = self.update_count.fetch_add(1, Ordering::SeqCst);

        // Record the event
        let event_str = format!("{:?}", event);
        self.events_received.lock().unwrap().push(event_str);

        // Check if we should quit
        if count >= self.should_quit_after.load(Ordering::SeqCst) {
            return None; // Quit
        }

        match event {
            Event::User(msg) if msg == "generate_tick" => {
                Some(commands::tick(Duration::from_millis(1), || {
                    "tick_message".to_string()
                }))
            }
            Event::User(msg) if msg == "batch_test" => commands::batch(vec![
                Some(commands::custom(|| Some("batch1".to_string()))),
                Some(commands::custom(|| Some("batch2".to_string()))),
            ]),
            Event::User(msg) if msg == "sequence_test" => commands::sequence(vec![
                Some(commands::custom(|| Some("seq1".to_string()))),
                Some(commands::custom(|| Some("seq2".to_string()))),
            ]),
            Event::Quit => None,
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {
        self.view_called.store(true, Ordering::SeqCst);
    }
}

#[test]
fn test_program_full_lifecycle() {
    let model = ControlledModel::new(3);
    let init_called = Arc::clone(&model.init_called);
    let update_count = Arc::clone(&model.update_count);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    // Use run_until to run deterministically
    let program = Program::with_options(model, options).unwrap();
    
    program.run_until(|model| {
        model.update_count.load(Ordering::SeqCst) >= 3
    }).unwrap();

    // Check that init was called
    assert!(init_called.load(Ordering::SeqCst));

    // Check that update was called the expected number of times
    assert!(update_count.load(Ordering::SeqCst) >= 3);
}

#[test]
fn test_program_with_filter() {
    let model = ControlledModel::new(5);
    let events = Arc::clone(&model.events_received);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options)
        .unwrap()
        .with_filter(|_model, event| {
            // Filter out tick messages
            match &event {
                Event::User(msg) if msg.contains("tick") => None,
                _ => Some(event),
            }
        });

    // Send test messages
    let sender = program.init_async_bridge();
    sender.send(Event::User("normal".to_string())).unwrap();
    sender.send(Event::User("tick_message".to_string())).unwrap();
    sender.send(Event::User("another".to_string())).unwrap();
    
    // Run with short timeout
    program.run_with_timeout(Duration::from_millis(50)).unwrap();

    // Check that tick message was filtered
    let received = events.lock().unwrap();
    assert!(received.iter().any(|e| e.contains("normal")));
    assert!(received.iter().any(|e| e.contains("another")));
    assert!(!received.iter().any(|e| e.contains("tick_message")));
}

#[test]
fn test_program_message_sending() {
    let model = ControlledModel::new(10);
    let events = Arc::clone(&model.events_received);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();
    let sender = program.init_async_bridge();

    // Send messages
    for i in 0..5 {
        sender.send(Event::User(format!("msg_{}", i))).unwrap();
    }
    
    // Schedule quit after a short delay to ensure messages are processed
    let quit_sender = sender.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        quit_sender.send(Event::Quit).unwrap();
    });

    // Run until quit
    program.run().unwrap();

    // Verify all messages were received
    let received = events.lock().unwrap();
    for i in 0..5 {
        let msg = format!("msg_{}", i);
        assert!(received.iter().any(|e| e.contains(&msg)), 
                "Should have received message: {}", msg);
    }
}

#[test]
fn test_program_command_execution() {
    let model = ControlledModel::new(20);
    let events = Arc::clone(&model.events_received);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();
    let sender = program.init_async_bridge();

    // Test batch command
    sender.send(Event::User("batch_test".to_string())).unwrap();
    
    // Test sequence command
    sender.send(Event::User("sequence_test".to_string())).unwrap();
    
    // Schedule quit after a short delay
    let quit_sender = sender.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        quit_sender.send(Event::Quit).unwrap();
    });
    
    program.run().unwrap();

    // Check that batch and sequence commands were executed
    let received = events.lock().unwrap();
    assert!(received.iter().any(|e| e.contains("batch1")));
    assert!(received.iter().any(|e| e.contains("batch2")));
    assert!(received.iter().any(|e| e.contains("seq1")));
    assert!(received.iter().any(|e| e.contains("seq2")));
}

// Test model that sends events to itself
#[derive(Clone)]
struct SelfSendingModel {
    counter: Arc<AtomicU32>,
    sender: Arc<Mutex<Option<std::sync::mpsc::SyncSender<Event<i32>>>>>,
}

impl Model for SelfSendingModel {
    type Message = i32;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        // Send initial message
        Some(commands::custom(|| Some(1)))
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(n) if n < 5 => {
                self.counter.fetch_add(1, Ordering::SeqCst);
                // Send next message
                Some(commands::custom(move || Some(n + 1)))
            }
            Event::User(5) => {
                self.counter.fetch_add(1, Ordering::SeqCst);
                None // Quit after 5
            }
            Event::Quit => None,
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

#[test]
fn test_self_sending_messages() {
    let model = SelfSendingModel {
        counter: Arc::new(AtomicU32::new(0)),
        sender: Arc::new(Mutex::new(None)),
    };
    let counter = Arc::clone(&model.counter);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let program = Program::with_options(model, options).unwrap();

    // Run until completion
    program.run().unwrap();

    // Should have processed 5 messages
    assert_eq!(counter.load(Ordering::SeqCst), 5);
}

// Test model for priority testing
#[derive(Clone)]
struct PriorityTestModel {
    high_priority_count: Arc<AtomicU32>,
    normal_priority_count: Arc<AtomicU32>,
    low_priority_count: Arc<AtomicU32>,
    total_count: Arc<AtomicU32>,
}

impl Model for PriorityTestModel {
    type Message = String;

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        let total = self.total_count.fetch_add(1, Ordering::SeqCst);
        
        if total >= 10 {
            return None; // Quit after 10 events
        }

        match event {
            Event::Quit => {
                self.high_priority_count.fetch_add(1, Ordering::SeqCst);
                Cmd::none()
            }
            Event::Key(_) => {
                self.high_priority_count.fetch_add(1, Ordering::SeqCst);
                Cmd::none()
            }
            Event::User(_) => {
                self.normal_priority_count.fetch_add(1, Ordering::SeqCst);
                Cmd::none()
            }
            Event::Tick => {
                self.low_priority_count.fetch_add(1, Ordering::SeqCst);
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

#[test]
fn test_event_priority_processing() {
    use hojicha::program::PriorityConfig;

    let model = PriorityTestModel {
        high_priority_count: Arc::new(AtomicU32::new(0)),
        normal_priority_count: Arc::new(AtomicU32::new(0)),
        low_priority_count: Arc::new(AtomicU32::new(0)),
        total_count: Arc::new(AtomicU32::new(0)),
    };

    let high_count = Arc::clone(&model.high_priority_count);
    let normal_count = Arc::clone(&model.normal_priority_count);
    let low_count = Arc::clone(&model.low_priority_count);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let config = PriorityConfig {
        max_queue_size: 100,
        log_drops: false,
        priority_mapper: None,
    };

    let mut program = Program::with_options(model, options)
        .unwrap()
        .with_priority_config(config);

    let sender = program.init_async_bridge();

    // Send events of different priorities
    for _ in 0..3 {
        sender.send(Event::Tick).unwrap(); // Low priority
    }
    for i in 0..3 {
        sender.send(Event::User(format!("msg_{}", i))).unwrap(); // Normal priority
    }
    for _ in 0..2 {
        sender.send(Event::Key(KeyEvent::new(Key::Enter, KeyModifiers::empty()))).unwrap(); // High priority
    }

    // Run with timeout
    program.run_with_timeout(Duration::from_millis(100)).unwrap();

    // High priority events should be processed first
    assert!(high_count.load(Ordering::SeqCst) > 0);
}

// Test model for testing all ProgramOptions
#[derive(Clone, Default)]
struct OptionsTestModel {
    render_count: Arc<AtomicU32>,
}

impl Model for OptionsTestModel {
    type Message = ();

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::Quit => None,
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {
        self.render_count.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn test_program_options_all_combinations() {
    // Test with various option combinations
    let test_cases = vec![
        ProgramOptions::default()
            .with_alt_screen(false)
            .headless(),
        ProgramOptions::default()
            .with_mouse_mode(MouseMode::CellMotion)
            .headless(),
        ProgramOptions::default()
            .with_bracketed_paste(true)
            .headless(),
        ProgramOptions::default()
            .with_focus_reporting(true)
            .headless(),
        ProgramOptions::default()
            .with_fps(120)
            .headless(),
        ProgramOptions::default()
            .without_renderer()
            .headless(),
    ];

    for options in test_cases {
        let model = OptionsTestModel::default();
        let mut program = Program::with_options(model, options).unwrap();
        
        // Send quit immediately
        let sender = program.init_async_bridge();
        sender.send(Event::Quit).unwrap();
        
        // Should run and exit quickly
        let result = program.run_with_timeout(Duration::from_millis(50));
        assert!(result.is_ok());
    }
}

// Test model for stats and metrics
#[derive(Clone, Default)]
struct MetricsTestModel {
    message_count: Arc<AtomicU32>,
}

impl Model for MetricsTestModel {
    type Message = String;

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(_) => {
                let count = self.message_count.fetch_add(1, Ordering::SeqCst);
                if count >= 10 {
                    None // Quit after 10 messages
                } else {
                    Cmd::none()
                }
            }
            Event::Quit => None,
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

#[test]
fn test_program_stats_and_metrics() {
    let model = MetricsTestModel::default();
    
    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();
    let sender = program.init_async_bridge();

    // Send some messages
    for i in 0..5 {
        sender.send(Event::User(format!("msg_{}", i))).unwrap();
    }
    
    // Get stats before running
    let stats_before = program.event_stats();
    assert_eq!(stats_before.total_events, 0);

    // Send quit
    sender.send(Event::Quit).unwrap();
    
    // Run program
    program.run().unwrap();

    // Note: Can't get stats after run() consumes the program
    // This test mainly ensures the stats methods compile and don't panic
}

// Test async handle functionality
#[derive(Clone, Default)]
struct AsyncTestModel {
    async_messages: Arc<Mutex<Vec<String>>>,
}

impl Model for AsyncTestModel {
    type Message = String;

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(msg) => {
                self.async_messages.lock().unwrap().push(msg.clone());
                if msg == "quit" {
                    None
                } else {
                    Cmd::none()
                }
            }
            Event::Quit => None,
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

#[test]
fn test_program_async_bridge() {
    let model = AsyncTestModel::default();
    let messages = Arc::clone(&model.async_messages);

    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();
    
    // Initialize async bridge
    let sender = program.init_async_bridge();
    
    // Send messages from "external" source
    sender.send(Event::User("async1".to_string())).unwrap();
    sender.send(Event::User("async2".to_string())).unwrap();
    sender.send(Event::User("async3".to_string())).unwrap();
    
    // Schedule quit after a short delay
    let quit_sender = sender.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        quit_sender.send(Event::User("quit".to_string())).unwrap();
    });
    
    // Run program
    program.run().unwrap();
    
    // Verify all messages were received
    let received = messages.lock().unwrap();
    assert_eq!(received.len(), 4);
    assert!(received.contains(&"async1".to_string()));
    assert!(received.contains(&"async2".to_string()));
    assert!(received.contains(&"async3".to_string()));
    assert!(received.contains(&"quit".to_string()));
}

// Test terminal control methods
#[test]
fn test_terminal_control_methods() {
    let model = OptionsTestModel::default();
    
    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();
    
    // Test terminal control methods (should not panic in headless mode)
    let _ = program.release_terminal();
    let _ = program.restore_terminal();
    
    // Methods should work without error in headless mode
}

// Test program kill functionality
#[test]
fn test_program_kill() {
    let model = ControlledModel::new(100); // Would run for a long time
    
    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let program = Program::with_options(model, options).unwrap();
    
    // Kill immediately
    program.kill();
    
    // wait() should return quickly since we killed it
    program.wait();
    
    // Program should have stopped
}

// Test queue resizing
#[test]
fn test_queue_resize() {
    let model = OptionsTestModel::default();
    
    let options = ProgramOptions::default()
        .headless()
        .without_signal_handler();

    let mut program = Program::with_options(model, options).unwrap();
    
    // Test queue capacity methods
    let initial_capacity = program.queue_capacity();
    assert!(initial_capacity > 0);
    
    // Resize queue
    let result = program.resize_queue(initial_capacity * 2);
    assert!(result.is_ok());
    
    assert_eq!(program.queue_capacity(), initial_capacity * 2);
    
    // Send quit and run
    let sender = program.init_async_bridge();
    sender.send(Event::Quit).unwrap();
    program.run().unwrap();
}