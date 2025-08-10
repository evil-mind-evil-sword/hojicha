use hojicha::commands;
use hojicha::prelude::*;
use hojicha::{Event, Model, Program, ProgramOptions};
use std::io::{Cursor, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Test model that responds to input
struct InputTestModel {
    received_keys: Vec<char>,
    quit_on: Option<char>,
}

impl Model for InputTestModel {
    type Message = ();

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        if let Event::Key(key) = event {
            if let Key::Char(c) = key.key {
                self.received_keys.push(c);
                if Some(c) == self.quit_on {
                    return commands::quit();
                }
            }
        }
        Cmd::none()
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {
        // Simple view for testing
    }
}

#[test]
fn test_custom_input_source() {
    // Create a custom input source with predefined key events
    let input_data = "abc\x1b"; // 'a', 'b', 'c', then ESC
    let input_source = Cursor::new(input_data.as_bytes().to_vec());

    let model = InputTestModel {
        received_keys: vec![],
        quit_on: Some('c'),
    };

    // This should fail to compile initially since with_input doesn't exist yet
    let options = ProgramOptions::default()
        .headless()
        .with_input(Box::new(input_source));

    let program = Program::with_options(model, options).unwrap();
    program
        .run_with_timeout(Duration::from_millis(100))
        .unwrap();

    // Verify the model received the expected input
    // We'll need a way to inspect the final model state
}

#[test]
fn test_input_from_string() {
    // Test helper method for string input
    let model = InputTestModel {
        received_keys: vec![],
        quit_on: Some('q'),
    };

    let options = ProgramOptions::default()
        .headless()
        .with_input_string("testq"); // Helper for common case

    let program = Program::with_options(model, options).unwrap();
    program
        .run_with_timeout(Duration::from_millis(100))
        .unwrap();
}

// Custom writer to capture log output
struct LogCapture {
    logs: Arc<Mutex<Vec<String>>>,
}

impl Write for LogCapture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let msg = String::from_utf8_lossy(buf).to_string();
        self.logs.lock().unwrap().push(msg);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
#[ignore = "Test needs headless mode configuration"]
fn test_log_to_file() {
    use std::fs;
    use std::path::Path;

    let log_path = "/tmp/hojicha_test_log.txt";

    // Initialize file logger
    hojicha::logging::init_file_logger(log_path).unwrap();

    // Log some messages
    hojicha::logging::debug("Debug message");
    hojicha::logging::info("Info message");
    hojicha::logging::warn("Warning message");
    hojicha::logging::error("Error message");

    // Verify log file exists and contains expected content
    assert!(Path::new(log_path).exists());
    let content = fs::read_to_string(log_path).unwrap();
    assert!(content.contains("Debug message"));
    assert!(content.contains("Info message"));
    assert!(content.contains("Warning message"));
    assert!(content.contains("Error message"));

    // Clean up
    fs::remove_file(log_path).ok();
}

#[test]
#[ignore = "Test needs headless mode configuration"]
fn test_log_commands() {
    struct LogTestModel {
        log_count: usize,
    }

    impl Model for LogTestModel {
        type Message = String;

        fn init(&mut self) -> Cmd<Self::Message> {
            // Test logging from commands
            commands::batch(vec![
                Cmd::new(|| {
                    println!("DEBUG: Initializing model");
                    None
                }),
                Cmd::new(|| {
                    println!("INFO: Model initialized");
                    None
                }),
                commands::custom(|| Some("logged".to_string())),
            ])
        }

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(msg) if msg == "logged" => {
                    self.log_count += 1;
                    commands::quit()
                }
                _ => commands::quit(),
            }
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let capture = Arc::new(Mutex::new(vec![]));
    let log_capture = LogCapture {
        logs: capture.clone(),
    };

    // Configure logging to use our capture
    hojicha::logging::init_with_writer(Box::new(log_capture)).unwrap();

    let model = LogTestModel { log_count: 0 };
    let program = Program::new(model).unwrap();
    program
        .run_with_timeout(Duration::from_millis(100))
        .unwrap();

    // Verify logs were captured
    let logs = capture.lock().unwrap();
    assert!(logs.iter().any(|log| log.contains("Initializing model")));
    assert!(logs.iter().any(|log| log.contains("Model initialized")));
}

#[test]
fn test_log_levels() {
    let capture = Arc::new(Mutex::new(vec![]));
    let log_capture = LogCapture {
        logs: capture.clone(),
    };

    // Set log level to INFO (should exclude DEBUG)
    hojicha::logging::init_with_writer_and_level(
        Box::new(log_capture),
        hojicha::logging::LogLevel::Info,
    )
    .unwrap();

    hojicha::logging::debug("Should not appear");
    hojicha::logging::info("Should appear");
    hojicha::logging::warn("Should also appear");

    let logs = capture.lock().unwrap();
    assert!(!logs.iter().any(|log| log.contains("Should not appear")));
    assert!(logs.iter().any(|log| log.contains("Should appear")));
    assert!(logs.iter().any(|log| log.contains("Should also appear")));
}
