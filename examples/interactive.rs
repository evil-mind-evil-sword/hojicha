//! Interactive input handling example
//!
//! This example demonstrates all input handling features:
//! - Keyboard input with modifiers
//! - Mouse tracking modes (none, cell motion, all motion)
//! - Bracketed paste detection
//! - Focus/blur events
//! - Window resize handling

use hojicha::commands;
use hojicha::event::{Key, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use hojicha::prelude::*;
use hojicha::program::{MouseMode, ProgramOptions};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::collections::VecDeque;
use std::time::Instant;

const MAX_EVENTS: usize = 30;

#[derive(Debug, Clone)]
struct EventLog {
    timestamp: Instant,
    description: String,
    event_type: EventType,
}

#[derive(Debug, Clone)]
enum EventType {
    Keyboard,
    Mouse,
    Paste,
    Focus,
    Resize,
    System,
}

struct App {
    events: VecDeque<EventLog>,
    mouse_mode: MouseMode,
    mouse_position: (u16, u16),
    window_size: (u16, u16),
    has_focus: bool,
    #[allow(dead_code)]
    paste_mode_enabled: bool,
    last_key: Option<String>,
    instructions_visible: bool,
    start_time: Instant,
}

#[derive(Debug, Clone)]
enum Message {
    ToggleMouseMode,
    ToggleInstructions,
    ClearEvents,
    #[allow(dead_code)]
    Quit,
}

impl App {
    fn new() -> Self {
        Self {
            events: VecDeque::new(),
            mouse_mode: MouseMode::None,
            mouse_position: (0, 0),
            window_size: (80, 24),
            has_focus: true,
            paste_mode_enabled: true,
            last_key: None,
            instructions_visible: true,
            start_time: Instant::now(),
        }
    }

    fn log_event(&mut self, description: String, event_type: EventType) {
        self.events.push_front(EventLog {
            timestamp: Instant::now(),
            description,
            event_type,
        });

        while self.events.len() > MAX_EVENTS {
            self.events.pop_back();
        }
    }

    fn format_key(key: &KeyEvent) -> String {
        let mut parts = Vec::new();

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if key.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }
        if key.modifiers.contains(KeyModifiers::SUPER) {
            parts.push("Super");
        }
        if key.modifiers.contains(KeyModifiers::HYPER) {
            parts.push("Hyper");
        }
        if key.modifiers.contains(KeyModifiers::META) {
            parts.push("Meta");
        }

        let key_str = match key.key {
            Key::Char(c) => format!("{c}"),
            Key::F(n) => format!("F{n}"),
            Key::Backspace => "Backspace".to_string(),
            Key::Enter => "Enter".to_string(),
            Key::Left => "←".to_string(),
            Key::Right => "→".to_string(),
            Key::Up => "↑".to_string(),
            Key::Down => "↓".to_string(),
            Key::Tab => "Tab".to_string(),
            Key::Delete => "Delete".to_string(),
            Key::Insert => "Insert".to_string(),
            Key::Home => "Home".to_string(),
            Key::End => "End".to_string(),
            Key::PageUp => "PageUp".to_string(),
            Key::PageDown => "PageDown".to_string(),
            Key::Esc => "Esc".to_string(),
            Key::Null => "Null".to_string(),
            _ => format!("{:?}", key.key),
        };

        if !parts.is_empty() {
            format!("{}+{}", parts.join("+"), key_str)
        } else {
            key_str
        }
    }

    fn format_mouse(mouse: &MouseEvent) -> String {
        let kind_str = match mouse.kind {
            MouseEventKind::Down(button) => format!("Down({button:?})"),
            MouseEventKind::Up(button) => format!("Up({button:?})"),
            MouseEventKind::Drag(button) => format!("Drag({button:?})"),
            MouseEventKind::Moved => "Moved".to_string(),
            MouseEventKind::ScrollDown => "ScrollDown".to_string(),
            MouseEventKind::ScrollUp => "ScrollUp".to_string(),
            MouseEventKind::ScrollLeft => "ScrollLeft".to_string(),
            MouseEventKind::ScrollRight => "ScrollRight".to_string(),
        };

        format!("{} at ({}, {})", kind_str, mouse.column, mouse.row)
    }

    fn get_mouse_mode_name(&self) -> &str {
        match self.mouse_mode {
            MouseMode::None => "Disabled",
            MouseMode::CellMotion => "Cell Motion (clicks + cell-level motion)",
            MouseMode::AllMotion => "All Motion (all mouse events)",
        }
    }
}

impl Model for App {
    type Message = Message;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        self.log_event("Application started".to_string(), EventType::System);
        // Window size command would go here
        None
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(Message::ToggleMouseMode) => {
                self.mouse_mode = match self.mouse_mode {
                    MouseMode::None => MouseMode::CellMotion,
                    MouseMode::CellMotion => MouseMode::AllMotion,
                    MouseMode::AllMotion => MouseMode::None,
                };
                self.log_event(
                    format!("Mouse mode: {}", self.get_mouse_mode_name()),
                    EventType::System,
                );
                None
            }
            Event::User(Message::ToggleInstructions) => {
                self.instructions_visible = !self.instructions_visible;
                self.log_event(
                    format!(
                        "Instructions {}",
                        if self.instructions_visible {
                            "shown"
                        } else {
                            "hidden"
                        }
                    ),
                    EventType::System,
                );
                None
            }
            Event::User(Message::ClearEvents) => {
                self.events.clear();
                self.log_event("Event log cleared".to_string(), EventType::System);
                None
            }
            Event::User(Message::Quit) => Some(commands::quit()), // Using explicit quit command
            Event::Key(key) => {
                // Check for quit keys first - demonstrate both patterns
                if key.key == Key::Char('q') && key.modifiers.is_empty() {
                    return Some(commands::quit()); // Using explicit quit command
                } else if key.key == Key::Esc {
                    return None; // Traditional pattern still works
                }

                let formatted = Self::format_key(&key);
                self.last_key = Some(formatted.clone());
                self.log_event(format!("Key: {formatted}"), EventType::Keyboard);

                match key.key {
                    Key::Char('m') if key.modifiers.is_empty() => {
                        Some(send(Message::ToggleMouseMode))
                    }
                    Key::Char('i') if key.modifiers.is_empty() => {
                        Some(send(Message::ToggleInstructions))
                    }
                    Key::Char('c') if key.modifiers.is_empty() => Some(send(Message::ClearEvents)),
                    _ => None,
                }
            }
            Event::Mouse(mouse) => {
                self.mouse_position = (mouse.column, mouse.row);
                let formatted = Self::format_mouse(&mouse);
                self.log_event(format!("Mouse: {formatted}"), EventType::Mouse);
                None
            }
            Event::Paste(text) => {
                let preview = if text.len() > 50 {
                    format!("{}... ({} chars)", &text[..50], text.len())
                } else {
                    text.clone()
                };
                self.log_event(
                    format!("Paste: \"{}\"", preview.replace('\n', "\\n")),
                    EventType::Paste,
                );
                None
            }
            Event::Focus => {
                self.has_focus = true;
                self.log_event("Window focused".to_string(), EventType::Focus);
                None
            }
            Event::Blur => {
                self.has_focus = false;
                self.log_event("Window blurred".to_string(), EventType::Focus);
                None
            }
            Event::Resize { width, height } => {
                self.window_size = (width, height);
                self.log_event(
                    format!("Window resized to {width}x{height}"),
                    EventType::Resize,
                );
                None
            }
            Event::Suspend => {
                self.log_event(
                    "Application suspended (Ctrl+Z)".to_string(),
                    EventType::System,
                );
                None
            }
            Event::Resume => {
                self.log_event("Application resumed".to_string(), EventType::System);
                None
            }
            _ => None,
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Min(10),
                Constraint::Length(if self.instructions_visible { 8 } else { 1 }),
            ])
            .split(area);

        // Status panel
        let status_block = Block::default()
            .title(" Status ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let uptime = self.start_time.elapsed().as_secs();
        let status_text = vec![
            Line::from(vec![
                Span::raw("Focus: "),
                Span::styled(
                    if self.has_focus { "Yes" } else { "No" },
                    Style::default().fg(if self.has_focus {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
                Span::raw("  |  Mouse: "),
                Span::styled(
                    self.get_mouse_mode_name(),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Window: "),
                Span::styled(
                    format!("{}x{}", self.window_size.0, self.window_size.1),
                    Style::default().fg(Color::Magenta),
                ),
                Span::raw("  |  Mouse Pos: "),
                Span::styled(
                    format!("({}, {})", self.mouse_position.0, self.mouse_position.1),
                    Style::default().fg(Color::Blue),
                ),
            ]),
            Line::from(vec![
                Span::raw("Last Key: "),
                Span::styled(
                    self.last_key.as_deref().unwrap_or("None"),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Uptime: "),
                Span::styled(
                    format!("{}m {}s", uptime / 60, uptime % 60),
                    Style::default().fg(Color::Gray),
                ),
                Span::raw("  |  Events: "),
                Span::styled(
                    format!("{}", self.events.len()),
                    Style::default().fg(Color::White),
                ),
            ]),
        ];

        let status = Paragraph::new(status_text)
            .block(status_block)
            .alignment(Alignment::Left);
        frame.render_widget(status, chunks[0]);

        // Event log
        let events_block = Block::default()
            .title(" Event Log (newest first) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let event_items: Vec<ListItem> = self
            .events
            .iter()
            .map(|log| {
                let elapsed = log.timestamp.duration_since(self.start_time).as_secs();
                let color = match log.event_type {
                    EventType::Keyboard => Color::Green,
                    EventType::Mouse => Color::Blue,
                    EventType::Paste => Color::Yellow,
                    EventType::Focus => Color::Magenta,
                    EventType::Resize => Color::Cyan,
                    EventType::System => Color::Gray,
                };

                let content = format!("[{:04}s] {}", elapsed, log.description);

                ListItem::new(content).style(Style::default().fg(color))
            })
            .collect();

        let events_list = List::new(event_items).block(events_block);
        frame.render_widget(events_list, chunks[1]);

        // Instructions
        if self.instructions_visible {
            let instructions_block = Block::default()
                .title(" Instructions (press 'i' to toggle) ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::DarkGray));

            let instructions = vec![
                Line::from(vec![
                    Span::styled(
                        "m",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" - Toggle mouse mode  |  "),
                    Span::styled(
                        "c",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" - Clear event log  |  "),
                    Span::styled(
                        "q",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" - Quit"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Try: "),
                    Span::styled("• Type any key  ", Style::default().fg(Color::Green)),
                    Span::styled("• Click/drag mouse  ", Style::default().fg(Color::Blue)),
                    Span::styled("• Paste text  ", Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::raw("     "),
                    Span::styled("• Resize window  ", Style::default().fg(Color::Cyan)),
                    Span::styled("• Focus/blur window  ", Style::default().fg(Color::Magenta)),
                    Span::styled("• Press Ctrl+Z", Style::default().fg(Color::Red)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Modifier keys work too: "),
                    Span::styled(
                        "Ctrl+A, Alt+Tab, Shift+F1",
                        Style::default().fg(Color::White),
                    ),
                ]),
            ];

            let instructions_widget = Paragraph::new(instructions)
                .block(instructions_block)
                .alignment(Alignment::Left);
            frame.render_widget(instructions_widget, chunks[2]);
        } else {
            let hint = Paragraph::new("Press 'i' to show instructions")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(hint, chunks[2]);
        }
    }
}

fn send<M: hojicha::core::Message>(msg: M) -> Cmd<M> {
    Cmd::new(move || Some(msg))
}

fn main() -> anyhow::Result<()> {
    eprintln!("Starting interactive input demo...");
    eprintln!("This example demonstrates all input handling features.");
    eprintln!("Check the terminal for the interactive UI.");

    let options = ProgramOptions::default()
        .with_mouse_mode(MouseMode::CellMotion)
        .with_bracketed_paste(true)
        .with_focus_reporting(true);

    let program = Program::with_options(App::new(), options)?;
    program.run()?;
    Ok(())
}
