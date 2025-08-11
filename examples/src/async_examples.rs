// Async Programming Examples
//
// This example demonstrates Hojicha's async capabilities:
// - Async commands and tasks
// - Stream subscriptions
// - Cancellable operations
// - External event injection
//
// Press Tab to switch between examples.

use hojicha_core::{
    commands::{self, spawn, tick},
    core::{Cmd, Model},
    event::{Event, Key},
};
use hojicha_pearls::{
    components,
    style::{ColorProfile, Theme},
};
use hojicha_runtime::program::Program;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Example {
    Timer,
    Stream,
    Cancellable,
    External,
}

impl Example {
    fn next(self) -> Self {
        match self {
            Self::Timer => Self::Stream,
            Self::Stream => Self::Cancellable,
            Self::Cancellable => Self::External,
            Self::External => Self::Timer,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Timer => "Async Timer",
            Self::Stream => "Stream Subscription",
            Self::Cancellable => "Cancellable Task",
            Self::External => "External Events",
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    StreamData(String),
    TaskComplete(String),
    External(String),
    StartTask,
    CancelTask,
    NextExample,
}

struct AsyncExamples {
    current_example: Example,

    // Timer state
    timer_count: u32,
    timer_running: bool,

    // Stream state
    stream_messages: Vec<String>,
    stream_active: bool,

    // Cancellable task state
    task_id: Option<String>,
    task_status: String,
    task_progress: f64,

    // External events
    external_messages: Vec<String>,

    // Theme
    theme: Theme,
    color_profile: ColorProfile,
}

impl AsyncExamples {
    fn new() -> Self {
        Self {
            current_example: Example::Timer,
            timer_count: 0,
            timer_running: false,
            stream_messages: Vec::new(),
            stream_active: false,
            task_id: None,
            task_status: "No task running".to_string(),
            task_progress: 0.0,
            external_messages: Vec::new(),
            theme: Theme::default(),
            color_profile: ColorProfile::default(),
        }
    }

    fn render_timer(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let status = if self.timer_running {
            format!("Timer Running: {} seconds", self.timer_count)
        } else {
            "Timer Stopped".to_string()
        };

        let timer_display = Paragraph::new(status)
            .style(ratatui::style::Style::default().fg(if self.timer_running {
                ratatui::style::Color::Green
            } else {
                ratatui::style::Color::Yellow
            }))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Timer "));
        frame.render_widget(timer_display, chunks[0]);

        let help = Paragraph::new("Press SPACE to start/stop timer").alignment(Alignment::Center);
        frame.render_widget(help, chunks[1]);

        let code = Paragraph::new(
            "// Using tick command:\n\
             tick(Duration::from_secs(1), || Message::Tick)",
        )
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Code "));
        frame.render_widget(code, chunks[2]);
    }

    fn render_stream(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        // Stream messages
        let items: Vec<ListItem> = self
            .stream_messages
            .iter()
            .rev()
            .take(10)
            .map(|msg| ListItem::new(msg.as_str()))
            .collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(format!(
            " Stream Messages (Active: {}) ",
            self.stream_active
        )));
        frame.render_widget(list, chunks[0]);

        let help = Paragraph::new("Press SPACE to toggle stream subscription")
            .alignment(Alignment::Center);
        frame.render_widget(help, chunks[1]);
    }

    fn render_cancellable(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        // Task status
        let status = Paragraph::new(self.task_status.clone())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Task Status "),
            );
        frame.render_widget(status, chunks[0]);

        // Progress bar
        let progress = Gauge::default()
            .percent((self.task_progress * 100.0) as u16)
            .label(format!("{:.0}%", self.task_progress * 100.0))
            .gauge_style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));
        frame.render_widget(progress, chunks[1]);

        let help =
            Paragraph::new("Press SPACE to start task, C to cancel").alignment(Alignment::Center);
        frame.render_widget(help, chunks[2]);
    }

    fn render_external(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        // External messages
        let items: Vec<ListItem> = self
            .external_messages
            .iter()
            .rev()
            .take(10)
            .map(|msg| ListItem::new(msg.as_str()))
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" External Events "),
        );
        frame.render_widget(list, chunks[0]);

        let help = Paragraph::new("External events are automatically received")
            .alignment(Alignment::Center);
        frame.render_widget(help, chunks[1]);
    }
}

impl Model for AsyncExamples {
    type Message = Message;

    fn init(&mut self) -> Cmd<Self::Message> {
        // Start external event simulation
        spawn(async {
            tokio::time::sleep(Duration::from_secs(3)).await;
            Some(Message::External("External event #1".to_string()))
        })
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(msg) => match msg {
                Message::Tick => {
                    if self.timer_running {
                        self.timer_count += 1;
                        tick(Duration::from_secs(1), || Message::Tick)
                    } else {
                        Cmd::none()
                    }
                }
                Message::StreamData(data) => {
                    self.stream_messages.push(data);
                    if self.stream_messages.len() > 50 {
                        self.stream_messages.remove(0);
                    }
                    Cmd::none()
                }
                Message::TaskComplete(result) => {
                    self.task_status = result;
                    self.task_progress = 1.0;
                    self.task_id = None;
                    Cmd::none()
                }
                Message::External(msg) => {
                    self.external_messages.push(msg);
                    if self.external_messages.len() > 50 {
                        self.external_messages.remove(0);
                    }
                    // Continue receiving external events
                    spawn(async {
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        Some(Message::External("External event".to_string()))
                    })
                }
                Message::StartTask => Cmd::none(),
                Message::CancelTask => Cmd::none(),
                Message::NextExample => {
                    self.current_example = self.current_example.next();
                    Cmd::none()
                }
            },
            Event::Key(key) => match key.key {
                Key::Char('q') | Key::Esc => commands::quit(),
                Key::Tab => {
                    self.current_example = self.current_example.next();
                    Cmd::none()
                }
                Key::Char(' ') => {
                    match self.current_example {
                        Example::Timer => {
                            self.timer_running = !self.timer_running;
                            if self.timer_running {
                                tick(Duration::from_secs(1), || Message::Tick)
                            } else {
                                Cmd::none()
                            }
                        }
                        Example::Stream => {
                            self.stream_active = !self.stream_active;
                            if self.stream_active {
                                // Start streaming data
                                tick(Duration::from_millis(500), || {
                                    Message::StreamData("Stream message".to_string())
                                })
                            } else {
                                Cmd::none()
                            }
                        }
                        Example::Cancellable => {
                            self.task_status = "Task running...".to_string();
                            self.task_progress = 0.0;
                            spawn(async {
                                for _i in 1..=10 {
                                    tokio::time::sleep(Duration::from_millis(500)).await;
                                }
                                Some(Message::TaskComplete(
                                    "Task completed successfully!".to_string(),
                                ))
                            })
                        }
                        _ => Cmd::none(),
                    }
                }
                Key::Char('c') if self.current_example == Example::Cancellable => {
                    if self.task_id.is_some() {
                        self.task_status = "Task cancelled".to_string();
                        self.task_progress = 0.0;
                        self.task_id = None;
                    }
                    Cmd::none()
                }
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Header
        let header = Paragraph::new(format!("Async Examples - {}", self.current_example.name()))
            .style(
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Cyan)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(header, chunks[0]);

        // Content based on current example
        match self.current_example {
            Example::Timer => self.render_timer(frame, chunks[1]),
            Example::Stream => self.render_stream(frame, chunks[1]),
            Example::Cancellable => self.render_cancellable(frame, chunks[1]),
            Example::External => self.render_external(frame, chunks[1]),
        }
    }
}

fn main() -> hojicha_core::Result<()> {
    let program = Program::new(AsyncExamples::new())?;
    program.run()
}
