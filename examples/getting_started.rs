//! Getting Started - Your first hojicha application
//!
//! This example demonstrates the basics of the Elm Architecture in hojicha:
//! - Model: Application state (a counter and message)
//! - Update: Handle events and update state
//! - View: Render the UI
//! - Commands: Side effects (like timed messages)
//!
//! Controls:
//! - ↑/k: Increment counter
//! - ↓/j: Decrement counter
//! - Space: Toggle timer
//! - r: Reset everything
//! - m: Change message
//! - q: Quit

use hojicha::commands;
use hojicha::event::Key;
use hojicha::prelude::*;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// Our application state
struct GettingStarted {
    counter: i32,
    message: String,
    timer_running: bool,
    ticks: u32,
}

/// Messages that can be sent to update our state
#[derive(Clone)]
enum Msg {
    Increment,
    Decrement,
    Reset,
    ToggleTimer,
    Tick,
    ChangeMessage,
}

impl Model for GettingStarted {
    type Message = Msg;

    /// Initialize the model - called once at startup
    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        // We don't need any initial commands
        None
    }

    /// Handle events and update the model
    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            // Handle keyboard events
            Event::Key(key_event) => match key_event.key {
                Key::Char('q') | Key::Esc => {
                    // Returning None quits the application
                    return Some(commands::quit());
                }
                Key::Up | Key::Char('k') => {
                    self.counter += 1;
                    self.message = format!("Incremented to {}", self.counter);
                }
                Key::Down | Key::Char('j') => {
                    self.counter -= 1;
                    self.message = format!("Decremented to {}", self.counter);
                }
                Key::Char(' ') => {
                    self.timer_running = !self.timer_running;
                    if self.timer_running {
                        self.message = "Timer started! 🏃".to_string();
                        // Start sending tick messages every 100ms
                        return Some(tick(std::time::Duration::from_millis(100), || Msg::Tick));
                    } else {
                        self.message = "Timer stopped! ⏸️".to_string();
                    }
                }
                Key::Char('r') => {
                    self.counter = 0;
                    self.ticks = 0;
                    self.timer_running = false;
                    self.message = "Reset! Starting fresh 🌱".to_string();
                }
                Key::Char('m') => {
                    // Cycle through different messages
                    self.message = match self.message.as_str() {
                        "Hello, Hojicha! 🍵" => "Welcome to TUI development! 🎨".to_string(),
                        "Welcome to TUI development! 🎨" => {
                            "Elm Architecture is awesome! 🌳".to_string()
                        }
                        "Elm Architecture is awesome! 🌳" => "Happy coding! 💻".to_string(),
                        _ => "Hello, Hojicha! 🍵".to_string(),
                    };
                }
                _ => {}
            },

            // Handle custom messages
            Event::User(msg) => match msg {
                Msg::Tick => {
                    if self.timer_running {
                        self.ticks += 1;
                        // Continue ticking
                        return Some(tick(std::time::Duration::from_millis(100), || Msg::Tick));
                    }
                }
                _ => {} // Other messages handled above
            },

            // Handle mouse events (just for demonstration)
            Event::Mouse(mouse) => {
                self.message = format!("Mouse at ({}, {})", mouse.column, mouse.row);
            }

            _ => {}
        }

        // Continue running without side effects
        None
    }

    /// Render the UI
    fn view(&self, frame: &mut Frame, _area: ratatui::layout::Rect) {
        // Create the layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(7), // Counter display
                Constraint::Length(5), // Timer display
                Constraint::Length(3), // Message
                Constraint::Min(0),    // Spacer
                Constraint::Length(6), // Help
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new("🍵 Getting Started with Hojicha")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Counter display
        let counter_color = if self.counter > 0 {
            Color::Green
        } else if self.counter < 0 {
            Color::Red
        } else {
            Color::Yellow
        };

        let counter_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("Counter: "),
                Span::styled(
                    format!("{}", self.counter),
                    Style::default()
                        .fg(counter_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(format!("Press ↑/↓ to change")),
        ];

        let counter = Paragraph::new(counter_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Counter"));
        frame.render_widget(counter, chunks[1]);

        // Timer display
        let timer_status = if self.timer_running {
            ("Running", Color::Green, "⏱️")
        } else {
            ("Stopped", Color::Gray, "⏸️")
        };

        let timer_text = vec![
            Line::from(vec![
                Span::raw("Status: "),
                Span::styled(timer_status.0, Style::default().fg(timer_status.1)),
                Span::raw(format!(" {}", timer_status.2)),
            ]),
            Line::from(format!("Ticks: {}", self.ticks)),
            Line::from(format!("Time: {:.1}s", self.ticks as f32 / 10.0)),
        ];

        let timer = Paragraph::new(timer_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Timer"));
        frame.render_widget(timer, chunks[2]);

        // Message display
        let message = Paragraph::new(self.message.as_str())
            .style(Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Message"));
        frame.render_widget(message, chunks[3]);

        // Help text
        let help_text = vec![
            Line::from("Controls:"),
            Line::from("↑/k: +1  ↓/j: -1  Space: Timer  r: Reset"),
            Line::from("m: Change message  q: Quit"),
            Line::from(""),
            Line::from(Span::styled(
                "Try different keys and watch the state change!",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )),
        ];

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Help"));
        frame.render_widget(help, chunks[5]);
    }
}

impl GettingStarted {
    fn new() -> Self {
        Self {
            counter: 0,
            message: "Hello, Hojicha! 🍵".to_string(),
            timer_running: false,
            ticks: 0,
        }
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Configure the program
    let options = ProgramOptions::default()
        .with_alt_screen(true) // Use alternate screen buffer
        .with_mouse_mode(MouseMode::CellMotion); // Enable mouse support

    // Create and run the program
    let program = Program::with_options(GettingStarted::new(), options)?;
    program.run()?;

    Ok(())
}
