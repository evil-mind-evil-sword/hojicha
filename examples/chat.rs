//! Chat Application Example - A simple chat interface with channels
//!
//! This example demonstrates:
//! - Multiple channels/rooms
//! - Message history
//! - Input handling with TextArea component
//! - Scrollable message view with Viewport
//! - Real-time UI updates

use hojicha::commands;
use hojicha::components::{TextArea, Viewport};
use hojicha::event::{Event, Key};
use hojicha::prelude::*;
use hojicha::program::{MouseMode, ProgramOptions};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

#[derive(Debug, Clone)]
struct ChatMessage {
    author: String,
    content: String,
    timestamp: String,
}

impl ChatMessage {
    fn new(author: String, content: String) -> Self {
        let now = chrono::Local::now();
        Self {
            author,
            content,
            timestamp: now.format("%H:%M").to_string(),
        }
    }
}

struct ChatApp {
    // Channels
    channels: Vec<String>,
    current_channel: usize,

    // Messages per channel
    messages: HashMap<String, Vec<ChatMessage>>,

    // Input
    input: TextArea,

    // Display
    message_view: Viewport,

    // State
    username: String,
    tick_count: u64,
}

impl ChatApp {
    fn new() -> Self {
        let channels = vec![
            "# general".to_string(),
            "# random".to_string(),
            "# development".to_string(),
            "# help".to_string(),
        ];

        let mut messages = HashMap::new();

        // Add some initial messages
        messages.insert(
            "# general".to_string(),
            vec![
                ChatMessage::new("System".to_string(), "Welcome to the chat!".to_string()),
                ChatMessage::new("Alice".to_string(), "Hey everyone!".to_string()),
                ChatMessage::new("Bob".to_string(), "Hi Alice, how's it going?".to_string()),
            ],
        );

        messages.insert(
            "# random".to_string(),
            vec![
                ChatMessage::new(
                    "Charlie".to_string(),
                    "Check out this cool Rust trick!".to_string(),
                ),
                ChatMessage::new(
                    "Dave".to_string(),
                    "Nice! I didn't know about that".to_string(),
                ),
            ],
        );

        messages.insert(
            "# development".to_string(),
            vec![
                ChatMessage::new(
                    "Eve".to_string(),
                    "Anyone using boba for their TUI?".to_string(),
                ),
                ChatMessage::new("Frank".to_string(), "Yes! It's really nice".to_string()),
            ],
        );

        messages.insert(
            "# help".to_string(),
            vec![ChatMessage::new(
                "System".to_string(),
                "Ask your questions here".to_string(),
            )],
        );

        let mut input = TextArea::new();
        input.set_focused(true);

        let mut app = Self {
            channels,
            current_channel: 0,
            messages,
            input,
            message_view: Viewport::new(),
            username: "You".to_string(),
            tick_count: 0,
        };

        app.update_message_view();
        app
    }

    fn send_message(&mut self) {
        let content = self.input.value();
        if content.trim().is_empty() {
            return;
        }

        let channel = &self.channels[self.current_channel];
        let message = ChatMessage::new(self.username.clone(), content);

        self.messages
            .entry(channel.clone())
            .or_insert_with(Vec::new)
            .push(message);

        self.input.set_value("");
        self.update_message_view();
    }

    fn switch_channel(&mut self, index: usize) {
        if index < self.channels.len() {
            self.current_channel = index;
            self.update_message_view();
        }
    }

    fn update_message_view(&mut self) {
        let channel = &self.channels[self.current_channel];
        let messages = self.messages.get(channel).cloned().unwrap_or_default();

        let mut content = String::new();
        for msg in messages {
            content.push_str(&format!(
                "[{}] {}: {}\n",
                msg.timestamp, msg.author, msg.content
            ));
        }

        self.message_view.set_content(content);
        self.message_view.scroll_to_bottom();
    }

    fn simulate_message(&mut self) {
        // Simulate incoming messages occasionally
        if self.tick_count % 50 == 0 {
            let messages = [
                (
                    "# general",
                    "Alice",
                    "Anyone working on something interesting?",
                ),
                ("# random", "Charlie", "Just discovered a new crate!"),
                ("# development", "Eve", "The new features look great"),
                ("# help", "Grace", "How do I use the TextArea component?"),
            ];

            let (channel, author, content) =
                messages[(self.tick_count / 50) as usize % messages.len()];

            if channel != &self.channels[self.current_channel] {
                // Only add to other channels so user sees unread indicator
                let message = ChatMessage::new(author.to_string(), content.to_string());
                self.messages
                    .entry(channel.to_string())
                    .or_insert_with(Vec::new)
                    .push(message);
            }
        }
    }
}

impl Model for ChatApp {
    type Message = Message;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        Some(commands::every(Duration::from_millis(100), |_| {
            Message::Tick
        }))
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::Key(key) => match key.key {
                Key::Char('q') if !self.input.is_focused() => {
                    return Some(commands::quit());
                }
                Key::Tab => {
                    // Switch to next channel
                    let next = (self.current_channel + 1) % self.channels.len();
                    self.switch_channel(next);
                }
                Key::Enter => {
                    self.send_message();
                }
                Key::Esc => {
                    // Clear input or exit if empty
                    if self.input.value().is_empty() {
                        return Some(commands::quit());
                    } else {
                        self.input.set_value("");
                    }
                }
                Key::F(n) if n >= 1 && n <= 4 => {
                    // F1-F4 to switch channels
                    self.switch_channel((n - 1) as usize);
                }
                _ => {
                    // Pass other keys to input
                    self.input.handle_event(&key);
                }
            },
            Event::User(Message::Tick) => {
                self.tick_count += 1;
                self.simulate_message();
            }
            Event::Mouse(mouse) => {
                self.message_view.handle_mouse(&mouse);
            }
            _ => {}
        }

        None
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Main layout: channels sidebar | chat area
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20), // Channels
                Constraint::Min(40),    // Chat
            ])
            .split(area);

        // Render channels
        self.render_channels(frame, main_chunks[0]);

        // Chat area: messages | input
        let chat_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),    // Messages
                Constraint::Length(3), // Input
            ])
            .split(main_chunks[1]);

        self.render_messages(frame, chat_chunks[0]);
        self.render_input(frame, chat_chunks[1]);
    }
}

impl ChatApp {
    fn render_channels(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .channels
            .iter()
            .enumerate()
            .map(|(i, channel)| {
                let style = if i == self.current_channel {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // Check for unread messages (simplified - just check if other channels have messages)
                let unread = if i != self.current_channel {
                    self.messages
                        .get(channel)
                        .map(|msgs| msgs.len())
                        .unwrap_or(0)
                } else {
                    0
                };

                let text = if unread > 0 {
                    format!("{} ({})", channel, unread)
                } else {
                    channel.clone()
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let channels_list = List::new(items)
            .block(
                Block::default()
                    .title(" Channels ")
                    .borders(Borders::ALL)
                    .border_style(Style::default()),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(channels_list, area);
    }

    fn render_messages(&self, frame: &mut Frame, area: Rect) {
        let channel = &self.channels[self.current_channel];
        let title = format!(" {} ", channel);

        let block = Block::default().title(title).borders(Borders::ALL);

        // Use the viewport to render scrollable messages
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Get messages and render them
        let messages = self.messages.get(channel).cloned().unwrap_or_default();
        let mut lines = vec![];

        for msg in messages {
            // Timestamp and author
            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{}] ", msg.timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{}: ", msg.author),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(msg.content.clone()),
            ]));
        }

        // Add padding if few messages
        if lines.is_empty() {
            lines.push(Line::from(Span::styled(
                "No messages yet. Start the conversation!",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )));
        }

        let messages_widget = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(messages_widget, inner);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let input_block = Block::default()
            .title(format!(
                " Message ({}@{}) ",
                self.username, self.channels[self.current_channel]
            ))
            .borders(Borders::ALL)
            .border_style(if self.input.is_focused() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let inner = input_block.inner(area);
        frame.render_widget(input_block, area);

        // Render the actual text content
        let input_text =
            Paragraph::new(self.input.value()).style(Style::default().fg(Color::White));

        frame.render_widget(input_text, inner);

        // Show cursor if focused
        if self.input.is_focused() {
            // Position cursor at end of text
            let text_len = self.input.value().len() as u16;
            frame.set_cursor_position((
                inner.x + text_len.min(inner.width.saturating_sub(1)),
                inner.y,
            ));
        }
    }
}

// Add chrono for timestamps
use chrono;

fn main() -> hojicha::Result<()> {
    let options = ProgramOptions::default()
        .with_alt_screen(true)
        .with_mouse_mode(MouseMode::CellMotion);

    let program = Program::with_options(ChatApp::new(), options)?;
    program.run()
}
