//! Example demonstrating headless mode
//!
//! This example shows how to run a Hojicha program in headless mode,
//! which is useful for testing or running TUI logic without actual rendering.

use hojicha::{
    commands::{self, quit},
    core::{Cmd, Model},
    event::Event,
    program::{Program, ProgramOptions},
};
use ratatui::{layout::Rect, Frame};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    Complete,
}

struct HeadlessModel {
    counter: i32,
    log: Arc<Mutex<Vec<String>>>,
}

impl HeadlessModel {
    fn new(log: Arc<Mutex<Vec<String>>>) -> Self {
        Self { counter: 0, log }
    }

    fn log(&self, msg: String) {
        if let Ok(mut log) = self.log.lock() {
            log.push(msg);
            // Also print to stdout for visibility
            println!("{}", log.last().unwrap());
        }
    }
}

impl Model for HeadlessModel {
    type Message = Message;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        self.log("Model initialized".to_string());

        // Start a sequence of operations
        commands::sequence(vec![
            Some(commands::tick(Duration::from_millis(100), || {
                Message::Increment
            })),
            Some(commands::tick(Duration::from_millis(200), || {
                Message::Increment
            })),
            Some(commands::tick(Duration::from_millis(300), || {
                Message::Decrement
            })),
            Some(commands::tick(Duration::from_millis(400), || {
                Message::Complete
            })),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(Message::Increment) => {
                self.counter += 1;
                self.log(format!("Counter incremented to {}", self.counter));
                None
            }
            Event::User(Message::Decrement) => {
                self.counter -= 1;
                self.log(format!("Counter decremented to {}", self.counter));
                None
            }
            Event::User(Message::Complete) => {
                self.log(format!("Complete! Final counter value: {}", self.counter));
                Some(quit())
            }
            _ => None,
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {
        // In headless mode, this won't be called
        // But we still need to implement it to satisfy the trait
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting headless example...");

    // Shared log for capturing what happens
    let log = Arc::new(Mutex::new(Vec::new()));

    // Create model with shared log
    let model = HeadlessModel::new(log.clone());

    // Create program with headless mode enabled
    let options = ProgramOptions::default()
        .without_renderer() // This enables headless mode
        .without_signal_handler(); // Also disable signals for cleaner testing

    let program = Program::with_options(model, options)?;

    println!("Running program in headless mode...");

    // Run the program
    program.run()?;

    println!("\nProgram completed. Event log:");
    if let Ok(log) = log.lock() {
        for (i, entry) in log.iter().enumerate() {
            println!("  {}. {}", i + 1, entry);
        }
    }

    Ok(())
}
