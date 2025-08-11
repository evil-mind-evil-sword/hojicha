//! Example demonstrating debugging and tracing features
//!
//! Run with debugging enabled:
//! ```bash
//! HOJICHA_DEBUG=1 cargo run --example debugging
//! HOJICHA_TRACE=all cargo run --example debugging
//! HOJICHA_METRICS=1 cargo run --example debugging
//! ```

use hojicha_core::{
    commands,
    core::{Cmd, Model},
    debug::{DebugContext, TraceEvent},
    event::{Event, Key},
};
use hojicha_runtime::{program::Program, ProgramOptions};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

/// A demo app that showcases debugging features
#[derive(Clone)]
struct DebugDemo {
    counter: i32,
    messages: Vec<String>,
    debug_context: DebugContext,
    last_command: String,
    metrics_display: String,
}

#[derive(Clone, Debug)]
enum Msg {
    Increment,
    Decrement,
    Tick,
    AsyncComplete(String),
    ToggleDebug,
    ClearMessages,
}

impl Model for DebugDemo {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        self.log("Application initialized");
        
        // Trace the initialization
        self.debug_context.trace_event(TraceEvent::Custom {
            label: "INIT".to_string(),
            data: "Starting debug demo".to_string(),
        });
        
        // Return a tick command that we can inspect
        commands::tick(Duration::from_secs(1), || Msg::Tick)
            .inspect(|cmd| {
                eprintln!("[INIT] Creating tick command: {}", cmd.debug_name());
            })
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        // Record metrics
        if let Some(mut metrics) = self.debug_context.get_metrics() {
            metrics.record_event();
        }

        match event {
            Event::User(msg) => {
                // Trace message reception
                self.debug_context.trace_event(TraceEvent::MessageReceived {
                    id: 0,
                    message: format!("{:?}", msg),
                    timestamp: Instant::now(),
                });

                match msg {
                    Msg::Increment => {
                        self.counter += 1;
                        self.last_command = "Increment".to_string();
                        self.log(&format!("Counter incremented to {}", self.counter));
                        
                        // Demonstrate command inspection
                        Cmd::none()
                            .inspect(|_| eprintln!("[UPDATE] Increment handled, returning NoOp"))
                    }
                    Msg::Decrement => {
                        self.counter -= 1;
                        self.last_command = "Decrement".to_string();
                        self.log(&format!("Counter decremented to {}", self.counter));
                        
                        Cmd::none()
                            .inspect(|_| eprintln!("[UPDATE] Decrement handled, returning NoOp"))
                    }
                    Msg::Tick => {
                        self.log("Tick received");
                        self.update_metrics_display();
                        
                        // Continue ticking
                        commands::tick(Duration::from_secs(1), || Msg::Tick)
                            .inspect_if(self.debug_context.is_enabled(), |cmd| {
                                eprintln!("[TICK] Scheduling next tick: {}", cmd.debug_name());
                            })
                    }
                    Msg::AsyncComplete(data) => {
                        self.log(&format!("Async operation completed: {}", data));
                        self.last_command = "AsyncComplete".to_string();
                        Cmd::none()
                    }
                    Msg::ToggleDebug => {
                        // In a real app, you might want to dynamically toggle debugging
                        self.log("Debug toggle requested (requires restart with env vars)");
                        Cmd::none()
                    }
                    Msg::ClearMessages => {
                        self.messages.clear();
                        self.log("Messages cleared");
                        Cmd::none()
                    }
                }
            }
            Event::Key(key_event) => {
                // Trace key events
                self.debug_context.trace_event(TraceEvent::EventProcessed {
                    event_type: format!("Key({:?})", key_event.key),
                    timestamp: Instant::now(),
                });

                match key_event.key {
                    Key::Char('+') | Key::Char('=') => {
                        self.update(Event::User(Msg::Increment))
                    }
                    Key::Char('-') | Key::Char('_') => {
                        self.update(Event::User(Msg::Decrement))
                    }
                    Key::Char('a') => {
                        // Demonstrate async command with inspection
                        self.log("Starting async operation...");
                        
                        commands::spawn(async {
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            Some(Msg::AsyncComplete("Data loaded!".to_string()))
                        })
                        .inspect(|cmd| {
                            eprintln!("[ASYNC] Spawning async command: {}", cmd.debug_name());
                        })
                    }
                    Key::Char('c') => {
                        self.update(Event::User(Msg::ClearMessages))
                    }
                    Key::Char('d') => {
                        self.update(Event::User(Msg::ToggleDebug))
                    }
                    Key::Char('q') | Key::Esc => {
                        self.log("Quitting...");
                        commands::quit()
                    }
                    _ => Cmd::none(),
                }
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Record frame start
        if let Some(mut metrics) = self.debug_context.get_metrics() {
            metrics.start_view();
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),     // Title
                Constraint::Length(5),     // Counter
                Constraint::Min(5),        // Messages
                Constraint::Length(6),     // Debug info
                Constraint::Length(3),     // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Debug Demo - Hojicha Debugging Features")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Counter display
        let counter_text = vec![
            Line::from(vec![
                Span::raw("Counter: "),
                Span::styled(
                    self.counter.to_string(),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Last Command: "),
                Span::styled(&self.last_command, Style::default().fg(Color::Green)),
            ]),
        ];
        let counter = Paragraph::new(counter_text)
            .block(Block::default().borders(Borders::ALL).title("State"));
        frame.render_widget(counter, chunks[1]);

        // Messages log
        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .rev()
            .take(10)
            .map(|msg| ListItem::new(msg.as_str()))
            .collect();
        let messages_list = List::new(messages)
            .block(Block::default().borders(Borders::ALL).title("Message Log"));
        frame.render_widget(messages_list, chunks[2]);

        // Debug info
        let debug_info = if self.debug_context.is_enabled() {
            vec![
                Line::from(vec![
                    Span::raw("Debug: "),
                    Span::styled("ENABLED", Style::default().fg(Color::Green)),
                ]),
                Line::from(format!("Trace Level: {:?}", self.debug_context.trace_level())),
                Line::from(self.metrics_display.as_str()),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::raw("Debug: "),
                    Span::styled("DISABLED", Style::default().fg(Color::Red)),
                ]),
                Line::from("Set HOJICHA_DEBUG=1 to enable"),
                Line::from("Set HOJICHA_TRACE=all for full tracing"),
            ]
        };
        let debug_panel = Paragraph::new(debug_info)
            .block(Block::default().borders(Borders::ALL).title("Debug Info"));
        frame.render_widget(debug_panel, chunks[3]);

        // Help
        let help = Paragraph::new(vec![
            Line::from("Keys: +/- = Adjust | a = Async | c = Clear | d = Debug | q = Quit"),
        ])
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[4]);

        // Record frame end
        if let Some(mut metrics) = self.debug_context.get_metrics() {
            metrics.end_view();
        }
    }
}

impl DebugDemo {
    fn new() -> Self {
        let debug_context = DebugContext::new();
        
        // Log initial debug state
        if debug_context.is_enabled() {
            eprintln!("=== Debug Mode Enabled ===");
            eprintln!("Trace Level: {:?}", debug_context.trace_level());
        }
        
        Self {
            counter: 0,
            messages: Vec::new(),
            debug_context,
            last_command: "None".to_string(),
            metrics_display: String::new(),
        }
    }

    fn log(&mut self, message: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        self.messages.push(format!("[{}] {}", timestamp, message));
        
        // Also trace as a custom event
        if self.debug_context.is_enabled() {
            self.debug_context.trace_event(TraceEvent::Custom {
                label: "LOG".to_string(),
                data: message.to_string(),
            });
        }
    }

    fn update_metrics_display(&mut self) {
        if let Some(metrics) = self.debug_context.get_metrics() {
            let summary = metrics.summary();
            self.metrics_display = format!(
                "FPS: {:.1} | Events: {} | Commands: {}",
                summary.average_fps,
                summary.total_events,
                summary.total_commands
            );
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up environment for demo if not already set
    if std::env::var("HOJICHA_DEBUG").is_err() {
        eprintln!("Tip: Run with HOJICHA_DEBUG=1 to see debug output");
        eprintln!("     HOJICHA_TRACE=all for full tracing");
        eprintln!("     HOJICHA_METRICS=1 for performance metrics");
        eprintln!();
    }

    let model = DebugDemo::new();
    let program = Program::with_options(model, ProgramOptions::default())?;
    program.run()?;
    Ok(())
}