//! Example demonstrating HTTP request helpers
//!
//! This example shows how to use the high-level async helpers for HTTP requests.

use hojicha_core::{
    async_helpers::{http_get, http_post, HttpError},
    commands,
    core::{Cmd, Model},
    event::{Event, Key},
};
use hojicha_runtime::{program::Program, ProgramOptions};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::time::Duration;

/// HTTP demo application
#[derive(Clone)]
struct HttpDemo {
    /// Current API endpoint
    endpoint: String,
    /// Request status
    status: RequestStatus,
    /// Response data
    response: Option<String>,
    /// Error message if any
    error: Option<String>,
    /// Request history
    history: Vec<String>,
}

#[derive(Clone)]
enum RequestStatus {
    Idle,
    Loading,
    Success,
    Error,
}

#[derive(Clone)]
enum Msg {
    /// Fetch data from API
    FetchData,
    /// Post data to API
    PostData,
    /// Data loaded successfully
    DataLoaded(String),
    /// Request failed
    RequestError(String),
    /// Clear response
    Clear,
}

impl Model for HttpDemo {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        self.log("HTTP Demo started. Press 'g' to GET, 'p' to POST");
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(msg) => match msg {
                Msg::FetchData => {
                    self.status = RequestStatus::Loading;
                    self.error = None;
                    self.log("Fetching data...");
                    
                    // Use the high-level HTTP helper
                    http_get(&self.endpoint, |result| {
                        match result {
                            Ok(response) => Msg::DataLoaded(response.body),
                            Err(e) => Msg::RequestError(e.to_string()),
                        }
                    })
                }
                Msg::PostData => {
                    self.status = RequestStatus::Loading;
                    self.error = None;
                    self.log("Posting data...");
                    
                    let json_body = r#"{"message": "Hello from Hojicha!"}"#;
                    
                    // Use the high-level POST helper
                    http_post(&self.endpoint, json_body, |result| {
                        match result {
                            Ok(response) => {
                                Msg::DataLoaded(format!("POST successful: {}", response.body))
                            }
                            Err(e) => Msg::RequestError(e.to_string()),
                        }
                    })
                }
                Msg::DataLoaded(data) => {
                    self.status = RequestStatus::Success;
                    self.response = Some(data.clone());
                    self.log(&format!("Data loaded: {} bytes", data.len()));
                    Cmd::none()
                }
                Msg::RequestError(error) => {
                    self.status = RequestStatus::Error;
                    self.error = Some(error.clone());
                    self.log(&format!("Request failed: {}", error));
                    Cmd::none()
                }
                Msg::Clear => {
                    self.response = None;
                    self.error = None;
                    self.status = RequestStatus::Idle;
                    self.log("Cleared response");
                    Cmd::none()
                }
            },
            Event::Key(key_event) => match key_event.key {
                Key::Char('g') | Key::Char('G') => {
                    self.update(Event::User(Msg::FetchData))
                }
                Key::Char('p') | Key::Char('P') => {
                    self.update(Event::User(Msg::PostData))
                }
                Key::Char('c') | Key::Char('C') => {
                    self.update(Event::User(Msg::Clear))
                }
                Key::Char('1') => {
                    self.endpoint = "https://api.example.com/users".to_string();
                    self.log("Endpoint changed to /users");
                    Cmd::none()
                }
                Key::Char('2') => {
                    self.endpoint = "https://api.example.com/posts".to_string();
                    self.log("Endpoint changed to /posts");
                    Cmd::none()
                }
                Key::Char('3') => {
                    self.endpoint = "https://api.example.com/comments".to_string();
                    self.log("Endpoint changed to /comments");
                    Cmd::none()
                }
                Key::Char('q') | Key::Esc => {
                    self.log("Exiting...");
                    commands::quit()
                }
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),     // Title
                Constraint::Length(4),     // Status
                Constraint::Min(8),        // Response
                Constraint::Length(8),     // History
                Constraint::Length(3),     // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new("HTTP Request Demo")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Status
        let status_color = match self.status {
            RequestStatus::Idle => Color::Gray,
            RequestStatus::Loading => Color::Yellow,
            RequestStatus::Success => Color::Green,
            RequestStatus::Error => Color::Red,
        };
        
        let status_text = vec![
            Line::from(vec![
                Span::raw("Endpoint: "),
                Span::styled(&self.endpoint, Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::raw("Status: "),
                Span::styled(
                    match self.status {
                        RequestStatus::Idle => "Ready",
                        RequestStatus::Loading => "Loading...",
                        RequestStatus::Success => "Success",
                        RequestStatus::Error => "Error",
                    },
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                ),
            ]),
        ];
        
        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title("Status"));
        frame.render_widget(status, chunks[1]);

        // Response/Error
        let response_title = if self.error.is_some() { "Error" } else { "Response" };
        let response_content = if let Some(error) = &self.error {
            vec![Line::from(Span::styled(
                error,
                Style::default().fg(Color::Red),
            ))]
        } else if let Some(data) = &self.response {
            // Split long responses into lines
            data.chars()
                .collect::<Vec<_>>()
                .chunks(chunks[2].width as usize - 4)
                .map(|chunk| Line::from(chunk.iter().collect::<String>()))
                .collect()
        } else {
            vec![Line::from(Span::styled(
                "No data yet. Press 'g' to GET or 'p' to POST",
                Style::default().fg(Color::DarkGray),
            ))]
        };
        
        let response = Paragraph::new(response_content)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title(response_title));
        frame.render_widget(response, chunks[2]);

        // History
        let history_items: Vec<ListItem> = self
            .history
            .iter()
            .rev()
            .take(5)
            .map(|entry| ListItem::new(entry.as_str()))
            .collect();
        
        let history = List::new(history_items)
            .block(Block::default().borders(Borders::ALL).title("History"));
        frame.render_widget(history, chunks[3]);

        // Help
        let help = Paragraph::new(vec![
            Line::from("g=GET | p=POST | c=Clear | 1/2/3=Change endpoint | q=Quit"),
        ])
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[4]);
    }
}

impl HttpDemo {
    fn new() -> Self {
        Self {
            endpoint: "https://api.example.com/data".to_string(),
            status: RequestStatus::Idle,
            response: None,
            error: None,
            history: Vec::new(),
        }
    }

    fn log(&mut self, message: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        self.history.push(format!("[{}] {}", timestamp, message));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("HTTP Request Demo");
    println!("=================");
    println!();
    println!("This demo shows how to use the high-level HTTP helpers.");
    println!("Note: Requests return mock data for demonstration.");
    println!();

    let model = HttpDemo::new();
    let program = Program::with_options(model, ProgramOptions::default())?;
    program.run()?;
    Ok(())
}