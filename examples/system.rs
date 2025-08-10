//! System integration example
//!
//! This example demonstrates system-level features:
//! - External command execution
//! - Suspend/resume with Ctrl+Z
//! - Error handling and recovery
//! - Async operations
//! - Window management

use hojicha::commands;
use hojicha::event::Key;
use hojicha::prelude::*;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
enum SystemEvent {
    CommandOutput {
        command: String,
        #[allow(dead_code)]
        output: String,
        exit_code: i32,
    },
    Error {
        context: String,
        error: String,
    },
    Suspended,
    Resumed,
    WindowResized {
        width: u16,
        height: u16,
    },
}

struct App {
    events: VecDeque<(Instant, SystemEvent)>,
    current_command: Option<String>,
    is_executing: bool,
    suspend_count: usize,
    error_count: usize,
    window_size: (u16, u16),
    start_time: Instant,
    last_error: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    ExecuteCommand(String),
    CommandComplete {
        command: String,
        output: String,
        exit_code: i32,
    },
    ErrorOccurred {
        context: String,
        error: String,
    },
    SimulateError,
    ClearLog,
    #[allow(dead_code)]
    Quit,
}

impl App {
    fn new() -> Self {
        Self {
            events: VecDeque::new(),
            current_command: None,
            is_executing: false,
            suspend_count: 0,
            error_count: 0,
            window_size: (80, 24),
            start_time: Instant::now(),
            last_error: None,
        }
    }

    fn log_event(&mut self, event: SystemEvent) {
        self.events.push_front((Instant::now(), event));
        while self.events.len() > 20 {
            self.events.pop_back();
        }
    }

    fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{secs}s")
        } else {
            format!("{}m {}s", secs / 60, secs % 60)
        }
    }
}

impl Model for App {
    type Message = Message;

    fn init(&mut self) -> Cmd<Self::Message> {
        send(Message::ExecuteCommand(
            "echo 'Hojicha system example started!'".to_string(),
        ))
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(Message::ExecuteCommand(cmd)) => {
                if self.is_executing {
                    self.last_error = Some("Already executing a command".to_string());
                    return Cmd::none();
                }

                self.current_command = Some(cmd.clone());
                self.is_executing = true;

                let cmd_for_closure = cmd.clone();
                commands::exec("sh", vec!["-c", &cmd], move |exit_code| {
                    let code = exit_code.unwrap_or(-1);
                    let output = format!("Command exited with code: {code}");
                    Message::CommandComplete {
                        command: cmd_for_closure.clone(),
                        output,
                        exit_code: code,
                    }
                })
            }
            Event::User(Message::CommandComplete {
                command,
                output,
                exit_code,
            }) => {
                self.is_executing = false;
                self.current_command = None;
                self.log_event(SystemEvent::CommandOutput {
                    command,
                    output,
                    exit_code,
                });
                Cmd::none()
            }
            Event::User(Message::ErrorOccurred { context, error }) => {
                self.error_count += 1;
                self.last_error = Some(error.clone());
                self.log_event(SystemEvent::Error { context, error });
                Cmd::none()
            }
            Event::User(Message::SimulateError) => {
                // Simulate an error
                send(Message::ErrorOccurred {
                    context: "SimulateError".to_string(),
                    error: "Simulated error for demonstration".to_string(),
                })
            }
            Event::User(Message::ClearLog) => {
                self.events.clear();
                self.last_error = None;
                Cmd::none()
            }
            Event::User(Message::Quit) => commands::quit(),
            Event::Key(key) => match key.key {
                Key::Char('q') if key.modifiers.is_empty() => commands::quit(),
                Key::Esc => commands::quit(),
                Key::Char('c') if key.modifiers.is_empty() => send(Message::ClearLog),
                Key::Char('e') if key.modifiers.is_empty() => send(Message::SimulateError),
                Key::Char('1') if key.modifiers.is_empty() => {
                    send(Message::ExecuteCommand("ls -la".to_string()))
                }
                Key::Char('2') if key.modifiers.is_empty() => {
                    send(Message::ExecuteCommand("pwd".to_string()))
                }
                Key::Char('3') if key.modifiers.is_empty() => {
                    send(Message::ExecuteCommand("date".to_string()))
                }
                Key::Char('4') if key.modifiers.is_empty() => {
                    send(Message::ExecuteCommand("echo $SHELL".to_string()))
                }
                Key::Char('5') if key.modifiers.is_empty() => {
                    send(Message::ExecuteCommand("false".to_string()))
                }
                Key::Char('6') if key.modifiers.is_empty() => send(Message::ExecuteCommand(
                    "sleep 2 && echo 'Done sleeping'".to_string(),
                )),
                _ => Cmd::none(),
            },
            Event::Suspend => {
                self.suspend_count += 1;
                self.log_event(SystemEvent::Suspended);
                Cmd::none()
            }
            Event::Resume => {
                self.log_event(SystemEvent::Resumed);
                Cmd::none()
            }
            Event::Resize { width, height } => {
                self.window_size = (width, height);
                self.log_event(SystemEvent::WindowResized { width, height });
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Min(10),
                Constraint::Length(7),
            ])
            .split(area);

        // Status panel
        self.render_status(frame, chunks[0]);

        // Event log
        self.render_events(frame, chunks[1]);

        // Commands panel
        self.render_commands(frame, chunks[2]);
    }
}

impl App {
    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let status_block = Block::default()
            .title(" System Status ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let uptime = Self::format_duration(self.start_time.elapsed());

        let status_lines = vec![
            Line::from(vec![
                Span::raw("Window: "),
                Span::styled(
                    format!("{}x{}", self.window_size.0, self.window_size.1),
                    Style::default().fg(Color::Magenta),
                ),
                Span::raw("  |  Uptime: "),
                Span::styled(uptime, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("Suspends: "),
                Span::styled(
                    format!("{}", self.suspend_count),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  |  Errors: "),
                Span::styled(
                    format!("{}", self.error_count),
                    Style::default().fg(if self.error_count > 0 {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                ),
            ]),
            Line::from(vec![
                Span::raw("Status: "),
                if self.is_executing {
                    Span::styled(
                        "Executing command...",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled("Idle", Style::default().fg(Color::Green))
                },
            ]),
            if let Some(cmd) = &self.current_command {
                Line::from(vec![
                    Span::raw("Command: "),
                    Span::styled(cmd, Style::default().fg(Color::Blue)),
                ])
            } else {
                Line::from("")
            },
            if let Some(error) = &self.last_error {
                Line::from(vec![
                    Span::styled(
                        "Last Error: ",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(error, Style::default().fg(Color::Red)),
                ])
            } else {
                Line::from("")
            },
        ];

        let status = Paragraph::new(status_lines)
            .block(status_block)
            .alignment(Alignment::Left);
        frame.render_widget(status, area);
    }

    fn render_events(&self, frame: &mut Frame, area: Rect) {
        let events_block = Block::default()
            .title(" System Events (newest first) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let event_items: Vec<ListItem> = self
            .events
            .iter()
            .map(|(timestamp, event)| {
                let elapsed = timestamp.duration_since(self.start_time);
                let time_str = Self::format_duration(elapsed);

                let (icon, color, text) = match event {
                    SystemEvent::CommandOutput {
                        command, exit_code, ..
                    } => {
                        let icon = if *exit_code == 0 { "✓" } else { "✗" };
                        let color = if *exit_code == 0 {
                            Color::Green
                        } else {
                            Color::Red
                        };
                        (
                            icon,
                            color,
                            format!("Command '{command}' (exit: {exit_code})"),
                        )
                    }
                    SystemEvent::Error { context, error } => {
                        ("⚠", Color::Red, format!("{context}: {error}"))
                    }
                    SystemEvent::Suspended => {
                        ("⏸", Color::Yellow, "Process suspended (Ctrl+Z)".to_string())
                    }
                    SystemEvent::Resumed => ("▶", Color::Green, "Process resumed".to_string()),
                    SystemEvent::WindowResized { width, height } => (
                        "⬚",
                        Color::Cyan,
                        format!("Window resized to {width}x{height}"),
                    ),
                };

                let content = format!("[{time_str:>6}] {icon} {text}");
                ListItem::new(content).style(Style::default().fg(color))
            })
            .collect();

        let events_list = List::new(event_items).block(events_block);
        frame.render_widget(events_list, area);
    }

    fn render_commands(&self, frame: &mut Frame, area: Rect) {
        let commands_block = Block::default()
            .title(" Commands ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));

        let commands = vec![
            Line::from(vec![Span::styled(
                "Quick Commands: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled(
                    "1",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - ls -la  |  "),
                Span::styled(
                    "2",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - pwd  |  "),
                Span::styled(
                    "3",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - date  |  "),
                Span::styled(
                    "4",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - echo $SHELL"),
            ]),
            Line::from(vec![
                Span::styled(
                    "5",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - false (exits with error)  |  "),
                Span::styled(
                    "6",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - sleep 2 (long running)"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "System: ",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Ctrl+Z",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Suspend  |  "),
                Span::styled(
                    "e",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Simulate error  |  "),
                Span::styled(
                    "c",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Clear  |  "),
                Span::styled(
                    "q",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Quit"),
            ]),
        ];

        let commands_widget = Paragraph::new(commands)
            .block(commands_block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_widget(commands_widget, area);
    }
}

fn send<M: hojicha::core::Message>(msg: M) -> Cmd<M> {
    Cmd::new(move || Some(msg))
}

fn main() -> anyhow::Result<()> {
    eprintln!("Starting system integration demo...");
    eprintln!("This example demonstrates command execution, suspend/resume, and error handling.");
    eprintln!();
    eprintln!(
        "Try pressing number keys to execute commands, Ctrl+Z to suspend, 'e' to simulate errors."
    );
    eprintln!();

    let options = ProgramOptions::default().with_fps(100);

    let program = Program::with_options(App::new(), options)?;
    program.run()?;
    Ok(())
}
