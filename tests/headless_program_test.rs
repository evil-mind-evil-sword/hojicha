//! Integration tests for Program using headless mode

use hojicha::{
    commands,
    core::{Cmd, Model},
    event::{Event, Key},
    program::{Program, ProgramOptions},
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Test model that tracks all operations
struct TestModel {
    counter: i32,
    messages: Arc<Mutex<Vec<String>>>,
    commands_executed: Arc<Mutex<Vec<String>>>,
}

impl TestModel {
    fn new() -> (Self, Arc<Mutex<Vec<String>>>, Arc<Mutex<Vec<String>>>) {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let commands = Arc::new(Mutex::new(Vec::new()));

        let model = Self {
            counter: 0,
            messages: Arc::clone(&messages),
            commands_executed: Arc::clone(&commands),
        };

        (model, messages, commands)
    }

    fn log(&self, msg: String) {
        self.messages.lock().unwrap().push(msg);
    }
}

#[derive(Debug, Clone)]
enum TestMessage {
    Increment,
    Decrement,
    SetValue(i32),
    Tick,
}

impl Model for TestModel {
    type Message = TestMessage;

    fn init(&mut self) -> Cmd<Self::Message> {
        self.log("Model initialized".to_string());
        self.commands_executed
            .lock()
            .unwrap()
            .push("init".to_string());

        // Return a command that will produce a message
        commands::tick(Duration::from_millis(10), || TestMessage::Tick)
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(TestMessage::Increment) => {
                self.counter += 1;
                self.log(format!("Incremented to {}", self.counter));
                Cmd::none()
            }
            Event::User(TestMessage::Decrement) => {
                self.counter -= 1;
                self.log(format!("Decremented to {}", self.counter));
                Cmd::none()
            }
            Event::User(TestMessage::SetValue(v)) => {
                self.counter = v;
                self.log(format!("Set value to {}", v));
                Cmd::none()
            }
            Event::User(TestMessage::Tick) => {
                self.log("Tick received".to_string());
                self.commands_executed
                    .lock()
                    .unwrap()
                    .push("tick_handled".to_string());
                commands::quit()
            }
            Event::Key(key) if key.key == Key::Char('q') => {
                self.log("Quit key pressed".to_string());
                commands::quit()
            }
            Event::Quit => {
                self.log("Quit event received".to_string());
                commands::quit()
            }
            _ => commands::quit(),
        }
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {
        // Not needed for headless testing
    }
}

#[test]
fn test_program_headless_basic() {
    let (model, messages, commands) = TestModel::new();

    let options = ProgramOptions::default()
        .without_renderer() // Headless mode
        .without_signal_handler() // No signals in tests
        .with_fps(60); // Test FPS limiting

    let program = Program::with_options(model, options).unwrap();

    // The program will process events

    // The program will run until it receives quit from the tick command
    std::thread::spawn(move || {
        let _ = program.run();
    });

    // Give it time to process
    std::thread::sleep(Duration::from_millis(50));

    // Check that messages were processed
    let msgs = messages.lock().unwrap();
    assert!(msgs.contains(&"Model initialized".to_string()));
    assert!(msgs.contains(&"Tick received".to_string()));

    let cmds = commands.lock().unwrap();
    assert!(cmds.contains(&"init".to_string()));
}

#[test]
fn test_program_message_filtering() {
    let (model, messages, _) = TestModel::new();

    let options = ProgramOptions::default()
        .without_renderer()
        .without_signal_handler()
        .headless();

    let mut program = Program::with_options(model, options).unwrap();

    // Add a filter that blocks increment messages
    program = program.with_filter(|_model, event| {
        match event {
            Event::User(TestMessage::Increment) => None, // Filter out increments
            _ => Some(event),
        }
    });

    // Note: This test can't properly test filtering without actually running
    // the program and sending messages through the event system.
    // For now, just verify the program can be created with a filter.

    // This would need to be tested with actual event generation
    let _ = program.run_with_timeout(Duration::from_millis(10));

    // Just check that init ran
    let msgs = messages.lock().unwrap();
    assert!(msgs.contains(&"Model initialized".to_string()));
}

#[test]
fn test_program_with_custom_output() {
    use std::io::Write;

    struct CustomOutput {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for CustomOutput {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.buffer.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    // CustomOutput is already Send + Sync because Arc<Mutex<Vec<u8>>> is Send + Sync

    let output_buffer = Arc::new(Mutex::new(Vec::new()));
    let custom_output = CustomOutput {
        buffer: Arc::clone(&output_buffer),
    };

    let (model, _, _) = TestModel::new();

    let options = ProgramOptions::default()
        .with_output(Box::new(custom_output))
        .without_signal_handler()
        .headless(); // Add headless to prevent trying to interact with terminal

    let program = Program::with_options(model, options).unwrap();

    // Use printf/println
    program.printf(format_args!("Test {}", 123));
    program.println("Hello from test");

    // The program needs to be running for these to be processed properly
    // Since printf/println write to stderr (not the custom output), we need to check stderr
    // Note: The custom output is only used for the terminal rendering, not for printf/println
    // which always write to stderr. This test verifies the custom output can be set.
}

#[test]
fn test_program_fps_limiting() {
    let (model, _, _) = TestModel::new();

    // Test with very low FPS
    let options = ProgramOptions::default()
        .without_renderer()
        .without_signal_handler()
        .with_fps(2); // Only 2 FPS

    let _program = Program::with_options(model, options).unwrap();

    // In a real test, we'd measure the actual frame rate
    // For now, just verify it compiles and runs
}

#[test]
fn test_program_lifecycle() {
    let (model, messages, _) = TestModel::new();

    let options = ProgramOptions::default()
        .without_renderer()
        .without_signal_handler()
        .headless();

    let program = Program::with_options(model, options).unwrap();

    // Run briefly to trigger initialization
    let _ = program.run_with_timeout(Duration::from_millis(10));

    // Check that init was called
    let msgs = messages.lock().unwrap();
    assert!(msgs.contains(&"Model initialized".to_string()));
}
