// Example demonstrating error handling patterns in Hojicha
//
// This example shows:
// - Using the FallibleModel trait for error handling
// - Converting errors to messages
// - Displaying errors in the UI
// - Recovering from errors

use hojicha_core::prelude::*;
use hojicha_core::{
    commands,
    fallible::{FallibleModel, FallibleModelExt},
};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use std::fs;
use std::io;

#[derive(Clone, Debug)]
enum Msg {
    LoadFile(String),
    FileLoaded(String),
    SaveFile(String),
    FileSaved,
    SimulateError,
    ClearError,
    ErrorOccurred(String),
    Quit,
}

struct ErrorHandlingApp {
    file_content: Option<String>,
    error_message: Option<String>,
    success_message: Option<String>,
    operation_log: Vec<String>,
    files_to_try: Vec<String>,
}

impl ErrorHandlingApp {
    fn new() -> Self {
        Self {
            file_content: None,
            error_message: None,
            success_message: None,
            operation_log: vec!["App started".to_string()],
            files_to_try: vec![
                "Cargo.toml".to_string(),
                "nonexistent.txt".to_string(),
                "/root/protected.txt".to_string(),
            ],
        }
    }

    fn log_operation(&mut self, msg: String) {
        self.operation_log.push(msg);
        // Keep only last 10 operations
        if self.operation_log.len() > 10 {
            self.operation_log.remove(0);
        }
    }

    fn load_file(&mut self, path: &str) -> io::Result<String> {
        self.log_operation(format!("Attempting to load: {}", path));
        fs::read_to_string(path)
    }

    fn save_file(&mut self, path: &str, content: &str) -> io::Result<()> {
        self.log_operation(format!("Attempting to save: {}", path));
        fs::write(path, content)
    }
}

impl Model for ErrorHandlingApp {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        // Try to load a file on startup using fallible command
        commands::fallible_with_error(
            || {
                let content = fs::read_to_string("README.md")?;
                Ok(Some(Msg::FileLoaded(content)))
            },
            |err| Msg::ErrorOccurred(format!("Failed to load README.md: {}", err)),
        )
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        // Delegate to the fallible update method
        self.update_with_error_handling(event)
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(5),     // Main content
                Constraint::Length(6),  // Error/Success messages
                Constraint::Length(12), // Operation log
                Constraint::Length(3),  // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Error Handling Demo")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Main content area
        let content = if let Some(ref text) = self.file_content {
            let truncated = if text.len() > 200 {
                format!("{}...\n\n(File truncated for display)", &text[..200])
            } else {
                text.clone()
            };
            Paragraph::new(truncated)
                .block(Block::default().borders(Borders::ALL).title("File Content"))
                .wrap(Wrap { trim: true })
        } else {
            Paragraph::new("No file loaded.\n\nPress 1/2/3 to try loading different files.")
                .block(Block::default().borders(Borders::ALL).title("File Content"))
        };
        frame.render_widget(content, chunks[1]);

        // Error/Success message area
        let message_area = if let Some(ref error) = self.error_message {
            Paragraph::new(format!("❌ Error: {}", error))
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).title("Status"))
                .wrap(Wrap { trim: true })
        } else if let Some(ref success) = self.success_message {
            Paragraph::new(format!("✅ Success: {}", success))
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL).title("Status"))
        } else {
            Paragraph::new("Ready").block(Block::default().borders(Borders::ALL).title("Status"))
        };
        frame.render_widget(message_area, chunks[2]);

        // Operation log
        let log_items: Vec<ListItem> = self
            .operation_log
            .iter()
            .enumerate()
            .map(|(i, msg)| {
                let style = if msg.contains("Error") || msg.contains("Failed") {
                    Style::default().fg(Color::Red)
                } else if msg.contains("Success") {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                };
                ListItem::new(format!("{:2}. {}", i + 1, msg)).style(style)
            })
            .collect();

        let log = List::new(log_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Operation Log"),
        );
        frame.render_widget(log, chunks[3]);

        // Help
        let help = Paragraph::new(
            "Keys: 1-3: Load files | s: Save test file | e: Simulate error | c: Clear error | q: Quit"
        )
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[4]);
    }
}

impl FallibleModel for ErrorHandlingApp {
    fn try_update(
        &mut self,
        event: Event<Self::Message>,
    ) -> hojicha::error::Result<Cmd<Self::Message>> {
        // Clear previous messages
        self.success_message = None;

        match event {
            Event::Key(key) => match key.key {
                Key::Char('q') => Ok(commands::quit()),
                Key::Char('1') => Ok(commands::custom(|| {
                    Some(Msg::LoadFile("Cargo.toml".to_string()))
                })),
                Key::Char('2') => Ok(commands::custom(|| {
                    Some(Msg::LoadFile("nonexistent.txt".to_string()))
                })),
                Key::Char('3') => Ok(commands::custom(|| {
                    Some(Msg::LoadFile("/root/protected.txt".to_string()))
                })),
                Key::Char('s') => Ok(commands::custom(|| {
                    Some(Msg::SaveFile("test_output.txt".to_string()))
                })),
                Key::Char('e') => Ok(commands::custom(|| Some(Msg::SimulateError))),
                Key::Char('c') => Ok(commands::custom(|| Some(Msg::ClearError))),
                _ => Ok(Cmd::none()),
            },
            Event::User(msg) => match msg {
                Msg::LoadFile(path) => {
                    // Use fallible_with_error to convert errors to messages
                    let path_for_closure = path.clone();
                    Ok(commands::fallible_with_error(
                        move || {
                            let content = fs::read_to_string(&path_for_closure)?;
                            Ok(Some(Msg::FileLoaded(content)))
                        },
                        move |err| {
                            Msg::ErrorOccurred(format!("Failed to load '{}': {}", path, err))
                        },
                    ))
                }
                Msg::FileLoaded(content) => {
                    self.file_content = Some(content.clone());
                    self.success_message = Some(format!(
                        "File loaded successfully ({} bytes)",
                        content.len()
                    ));
                    self.log_operation("File loaded successfully".to_string());
                    self.error_message = None;
                    Ok(Cmd::none())
                }
                Msg::SaveFile(path) => {
                    if let Some(ref content) = self.file_content {
                        let content = content.clone();
                        let path_for_closure = path.clone();
                        Ok(commands::fallible_with_error(
                            move || {
                                fs::write(&path_for_closure, content)?;
                                Ok(Some(Msg::FileSaved))
                            },
                            move |err| {
                                Msg::ErrorOccurred(format!("Failed to save '{}': {}", path, err))
                            },
                        ))
                    } else {
                        self.error_message = Some("No content to save".to_string());
                        self.log_operation("Error: No content to save".to_string());
                        Ok(Cmd::none())
                    }
                }
                Msg::FileSaved => {
                    self.success_message = Some("File saved successfully".to_string());
                    self.log_operation("File saved successfully".to_string());
                    Ok(Cmd::none())
                }
                Msg::SimulateError => {
                    // Deliberately cause an error to demonstrate error handling
                    Err(hojicha::error::Error::Model(
                        "Simulated error for demonstration".to_string(),
                    ))
                }
                Msg::ClearError => {
                    self.error_message = None;
                    self.log_operation("Errors cleared".to_string());
                    Ok(Cmd::none())
                }
                Msg::ErrorOccurred(error) => {
                    self.error_message = Some(error.clone());
                    self.log_operation(format!("Error: {}", error));
                    Ok(Cmd::none())
                }
                Msg::Quit => Ok(commands::quit()),
            },
            _ => Ok(Cmd::none()),
        }
    }

    fn handle_error(&mut self, error: hojicha::error::Error) -> Cmd<Self::Message> {
        // Custom error handling
        let error_msg = format!("{}", error);
        self.error_message = Some(error_msg.clone());
        self.log_operation(format!("Handled error: {}", error_msg));

        // We could also convert the error to a message for further processing
        commands::custom(move || Some(Msg::ErrorOccurred(error_msg)))
    }
}

fn main() -> anyhow::Result<()> {
    let app = ErrorHandlingApp::new();
    let program = Program::new(app)?;
    program.run()?;
    Ok(())
}
