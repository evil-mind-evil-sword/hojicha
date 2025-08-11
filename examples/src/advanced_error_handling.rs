// Advanced error handling example
//
// This example demonstrates advanced error handling patterns:
// - Custom error handlers for fallible commands
// - Converting errors to user-facing messages
// - Error recovery strategies
// - Logging and debugging errors

use hojicha_core::commands;
use hojicha_core::error::Error;
use hojicha_core::event::{Event, Key};
use hojicha_core::prelude::*;
use hojicha_runtime::program::error_handler::EventErrorHandler;
use hojicha_runtime::program::{CommandExecutor, Program, ProgramOptions};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Debug)]
enum Msg {
    LoadFile(String),
    FileLoaded { path: String, content: String },
    SaveFile { path: String, content: String },
    FileSaved(String),
    SimulateError,
    ClearErrors,
    ErrorOccurred { context: String, error: String },
    Quit,
}

struct AdvancedErrorApp {
    current_file: Option<String>,
    file_content: Option<String>,
    error_log: Vec<(String, String)>, // (context, error)
    success_log: Vec<String>,
    command_executor: Option<Arc<CommandExecutor<Msg>>>,
}

impl AdvancedErrorApp {
    fn new() -> Self {
        // Create a custom command executor with error handler
        let error_handler = EventErrorHandler::new(|err| {
            Msg::ErrorOccurred {
                context: "Command execution".to_string(),
                error: err.to_string(),
            }
        });
        
        let executor = Arc::new(
            CommandExecutor::with_error_handler(error_handler)
                .expect("Failed to create command executor")
        );
        
        Self {
            current_file: None,
            file_content: None,
            error_log: Vec::new(),
            success_log: vec!["Application started".to_string()],
            command_executor: Some(executor),
        }
    }

    fn log_error(&mut self, context: String, error: String) {
        self.error_log.push((context, error));
        // Keep only last 10 errors
        if self.error_log.len() > 10 {
            self.error_log.remove(0);
        }
    }

    fn log_success(&mut self, message: String) {
        self.success_log.push(message);
        // Keep only last 10 successes
        if self.success_log.len() > 10 {
            self.success_log.remove(0);
        }
    }
}

impl Model for AdvancedErrorApp {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        // Try to load a default file on startup using fallible command
        commands::custom_fallible(|| {
            // This might fail if the file doesn't exist
            match fs::read_to_string("README.md") {
                Ok(content) => Ok(Some(Msg::FileLoaded {
                    path: "README.md".to_string(),
                    content,
                })),
                Err(err) => Err(Error::Io(err)),
            }
        })
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::Key(key) => match key.key {
                Key::Char('q') if key.modifiers.is_empty() => commands::quit(),
                Key::Esc => commands::quit(),
                Key::Char('1') => {
                    // Load a file that should exist
                    commands::custom(|| Some(Msg::LoadFile("Cargo.toml".to_string())))
                }
                Key::Char('2') => {
                    // Try to load a file that probably doesn't exist
                    commands::custom(|| Some(Msg::LoadFile("nonexistent.txt".to_string())))
                }
                Key::Char('3') => {
                    // Try to load from a protected directory
                    commands::custom(|| Some(Msg::LoadFile("/root/protected.txt".to_string())))
                }
                Key::Char('s') if key.modifiers.is_empty() => {
                    // Save current content
                    if let Some(content) = &self.file_content {
                        commands::custom({
                            let content = content.clone();
                            move || Some(Msg::SaveFile {
                                path: "output.txt".to_string(),
                                content,
                            })
                        })
                    } else {
                        commands::custom(|| Some(Msg::ErrorOccurred {
                            context: "Save".to_string(),
                            error: "No content to save".to_string(),
                        }))
                    }
                }
                Key::Char('e') => commands::custom(|| Some(Msg::SimulateError)),
                Key::Char('c') => commands::custom(|| Some(Msg::ClearErrors)),
                _ => Cmd::none(),
            },
            Event::User(msg) => match msg {
                Msg::LoadFile(path) => {
                    self.log_success(format!("Attempting to load: {}", path));
                    let path_for_closure = path.clone();
                    
                    // Use fallible command with automatic error handling
                    commands::custom_fallible(move || {
                        let content = fs::read_to_string(&path_for_closure)?;
                        Ok(Some(Msg::FileLoaded {
                            path: path_for_closure,
                            content,
                        }))
                    })
                }
                Msg::FileLoaded { path, content } => {
                    self.current_file = Some(path.clone());
                    self.file_content = Some(content.clone());
                    self.log_success(format!("Successfully loaded {} ({} bytes)", path, content.len()));
                    Cmd::none()
                }
                Msg::SaveFile { path, content } => {
                    let path_for_closure = path.clone();
                    commands::custom_fallible(move || {
                        fs::write(&path_for_closure, content)?;
                        Ok(Some(Msg::FileSaved(path_for_closure)))
                    })
                }
                Msg::FileSaved(path) => {
                    self.log_success(format!("Successfully saved: {}", path));
                    Cmd::none()
                }
                Msg::SimulateError => {
                    // Deliberately create a fallible command that fails
                    commands::custom_fallible(|| {
                        Err(Error::Model("Simulated error for demonstration".to_string()))
                    })
                }
                Msg::ClearErrors => {
                    self.error_log.clear();
                    self.log_success("Error log cleared".to_string());
                    Cmd::none()
                }
                Msg::ErrorOccurred { context, error } => {
                    self.log_error(context, error);
                    Cmd::none()
                }
                Msg::Quit => commands::quit(),
            },
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Min(5),      // Content
                Constraint::Length(10),  // Error log
                Constraint::Length(8),   // Success log
                Constraint::Length(3),   // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Advanced Error Handling Demo")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Content area
        let content_text = if let Some(ref file) = self.current_file {
            if let Some(ref content) = self.file_content {
                let truncated = if content.len() > 500 {
                    format!("File: {}\n\n{}...\n\n(Truncated)", file, &content[..500])
                } else {
                    format!("File: {}\n\n{}", file, content)
                };
                truncated
            } else {
                format!("Loading: {}", file)
            }
        } else {
            "No file loaded.\n\nPress 1/2/3 to load files.".to_string()
        };

        let content = Paragraph::new(content_text)
            .block(Block::default().borders(Borders::ALL).title("File Content"))
            .wrap(Wrap { trim: true });
        frame.render_widget(content, chunks[1]);

        // Error log
        let error_items: Vec<ListItem> = self
            .error_log
            .iter()
            .rev()
            .map(|(context, error)| {
                ListItem::new(format!("[{}] {}", context, error))
                    .style(Style::default().fg(Color::Red))
            })
            .collect();

        let error_list = List::new(error_items)
            .block(Block::default().borders(Borders::ALL).title(format!("Error Log ({})", self.error_log.len())));
        frame.render_widget(error_list, chunks[2]);

        // Success log
        let success_items: Vec<ListItem> = self
            .success_log
            .iter()
            .rev()
            .take(5)
            .map(|msg| ListItem::new(msg.as_str()).style(Style::default().fg(Color::Green)))
            .collect();

        let success_list = List::new(success_items)
            .block(Block::default().borders(Borders::ALL).title("Success Log"));
        frame.render_widget(success_list, chunks[3]);

        // Help
        let help = Paragraph::new(
            "Keys: 1: Load Cargo.toml | 2: Load nonexistent | 3: Load protected | s: Save | e: Simulate error | c: Clear | q: Quit"
        )
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[4]);
    }
}

fn main() -> anyhow::Result<()> {
    eprintln!("Starting advanced error handling demo...");
    eprintln!("This example demonstrates:");
    eprintln!("- Automatic error handling for fallible commands");
    eprintln!("- Converting errors to user messages");
    eprintln!("- Error logging and recovery");
    eprintln!();

    let options = ProgramOptions::default().with_fps(100);

    let program = Program::with_options(AdvancedErrorApp::new(), options)?;
    program.run()?;
    Ok(())
}