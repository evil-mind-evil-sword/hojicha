use hojicha_runtime;
//! Example demonstrating debug features: Printf/Println and SetWindowTitle
//!
//! This example shows how to use the new debugging features:
//! - printf/println for debug output to stderr
//! - set_window_title to change the terminal window title
//! - FPS control to limit frame rate
//! - Message filtering to intercept and modify events

use hojicha_core::{
use hojicha_pearls::{components, style};    commands::{self, quit, set_window_title},
    core::{Cmd, Model},
    event::{Event, Key},
    hojicha_runtime::program::Program, ProgramOptions},
};
use ratatui::{
    layout::{Alignment, Rect},
    hojicha_pearls::style::{ColorProfile, Theme},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Duration;

#[derive(Debug, Clone)]
enum Message {
    Tick,
    DebugPrint,
}

struct DebugModel {
    counter: u32,
    debug_prints: u32,
}

impl DebugModel {
    fn new() -> Self {
        Self {
            counter: 0,
            debug_prints: 0,
        }
    }
}

impl Model for DebugModel {
    type Message = Message;

    fn init(&mut self) -> Cmd<Self::Message> {
        // Set the window title when the app starts
        commands::batch(vec![
            set_window_title("Hojicha Debug Features Demo"),
            commands::every(Duration::from_millis(1000), |_| Message::Tick),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(Message::Tick) => {
                self.counter += 1;

                // Every 3 ticks, print debug info to stderr
                if self.counter % 3 == 0 {
                    eprintln!("Debug: Counter reached {}", self.counter);
                }

                // Update window title with counter
                set_window_title(format!("Hojicha Debug - Count: {}", self.counter))
            }
            Event::User(Message::DebugPrint) => {
                self.debug_prints += 1;
                eprintln!("Debug print #{} triggered", self.debug_prints);
                Cmd::none()
            }
            Event::Key(key) => {
                match key.key {
                    Key::Char('q') => quit(),
                    Key::Char('d') => {
                        // Trigger a debug print
                        Cmd::new(|| Some(Message::DebugPrint))
                    }
                    _ => Cmd::none(),
                }
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let text = format!(
            "Debug Features Demo\n\n\
             Counter: {}\n\
             Debug Prints: {}\n\n\
             Press 'd' to trigger a debug print (check stderr)\n\
             Press 'q' to quit\n\n\
             Check your terminal title bar - it should update with the counter!\n\
             Debug output appears above the TUI in your terminal.",
            self.counter, self.debug_prints
        );

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Debug Features ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create model
    let model = DebugModel::new();

    // Create program with custom options
    let options = ProgramOptions::default().with_alt_screen(true).with_fps(30); // Limit to 30 FPS

    let mut program = Program::with_options(model, options)?;

    // Set filter on the program (not on options)
    program = program.with_filter(|_model, event| {
        // Example filter: log all events to stderr
        match &event {
            Event::Key(key) => {
                eprintln!("Filter: Key event - {:?}", key.key);
            }
            Event::User(Message::Tick) => {
                // Don't log tick events (too noisy)
            }
            _ => {
                eprintln!("Filter: Other event");
            }
        }
        Some(event) // Pass through all events
    });

    // Print initial debug message
    program.println("Debug Features Example Started!");
    program.println("All debug output will appear here above the TUI.");

    // Run the program
    Ok(program.run()?)
}
