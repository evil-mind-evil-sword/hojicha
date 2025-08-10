use hojicha::{
    async_handle::AsyncHandle,
    commands,
    core::{Cmd, Model},
    event::Event,
    program::{Program, ProgramOptions},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone)]
struct CancellableDemo {
    messages: Vec<String>,
    active_operations: Vec<(usize, String)>,
    operation_counter: usize,
    cancelled_count: usize,
    completed_count: usize,
}

#[derive(Debug, Clone)]
enum Msg {
    StartOperation,
    OperationComplete(usize, String),
    CancelOperation(usize),
    CancelAll,
}

impl Model for CancellableDemo {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        self.messages
            .push("Cancellable Operations Demo Started!".to_string());
        self.messages.push(
            "Press 's' to start operation, 'c' to cancel last, 'x' to cancel all".to_string(),
        );

        // Store handles in the model to actually demonstrate cancellation
        // This would be stored in a field if we were tracking real handles
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(Msg::StartOperation) => {
                let id = self.operation_counter;
                self.operation_counter += 1;

                let operation_name = format!("Operation #{}", id);
                self.active_operations.push((id, operation_name.clone()));
                self.messages.push(format!("Started: {}", operation_name));

                // Use spawn to create a cancellable async operation
                // In a real app, we'd store the handle returned by program.spawn_cancellable()
                let delay = Duration::from_secs(3 + ((id % 3) as u64));
                commands::spawn(async move {
                    tokio::time::sleep(delay).await;
                    Some(Msg::OperationComplete(
                        id,
                        format!(
                            "Operation #{} completed after {:.1}s",
                            id,
                            delay.as_secs_f32()
                        ),
                    ))
                })
            }
            Event::User(Msg::OperationComplete(id, result)) => {
                self.active_operations.retain(|(op_id, _)| *op_id != id);
                self.completed_count += 1;
                self.messages.push(format!("✓ {}", result));
                Cmd::none()
            }
            Event::User(Msg::CancelOperation(id)) => {
                if let Some(pos) = self
                    .active_operations
                    .iter()
                    .position(|(op_id, _)| *op_id == id)
                {
                    let (_, name) = self.active_operations.remove(pos);
                    self.cancelled_count += 1;
                    self.messages.push(format!("✗ Cancelled: {}", name));
                    // In a real implementation, we'd call handle.cancel() here
                }
                Cmd::none()
            }
            Event::User(Msg::CancelAll) => {
                let count = self.active_operations.len();
                self.cancelled_count += count;
                for (_, name) in self.active_operations.drain(..) {
                    self.messages.push(format!("✗ Cancelled: {}", name));
                }
                // In a real implementation, we'd cancel all handles here
                Cmd::none()
            }
            Event::Key(key) => match key.key {
                hojicha::event::Key::Char('q') => commands::quit(),
                hojicha::event::Key::Char('s') => {
                    // Start a new operation
                    self.update(Event::User(Msg::StartOperation))
                }
                hojicha::event::Key::Char('c') => {
                    // Cancel the last operation
                    if let Some((id, _)) = self.active_operations.last() {
                        let id = *id;
                        self.update(Event::User(Msg::CancelOperation(id)))
                    } else {
                        Cmd::none()
                    }
                }
                hojicha::event::Key::Char('x') => {
                    // Cancel all operations
                    self.update(Event::User(Msg::CancelAll))
                }
                _ => Cmd::none(),
            },
            Event::Quit => commands::quit(),
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Percentage(30),
                Constraint::Min(0),
                Constraint::Length(4),
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Cancellable Operations Demo")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Active operations
        let active_items: Vec<ListItem> = self
            .active_operations
            .iter()
            .map(|(id, name)| {
                ListItem::new(format!("[{}] {}", id, name))
                    .style(Style::default().fg(Color::Yellow))
            })
            .collect();

        let active_list =
            List::new(active_items).block(Block::default().borders(Borders::ALL).title(format!(
                "Active Operations ({})",
                self.active_operations.len()
            )));
        frame.render_widget(active_list, chunks[1]);

        // Messages/Log
        let messages: Vec<String> = self
            .messages
            .iter()
            .rev()
            .take(chunks[2].height as usize - 2)
            .cloned()
            .collect();
        let messages_text = messages.join("\n");

        let messages_widget = Paragraph::new(messages_text)
            .block(Block::default().borders(Borders::ALL).title(format!(
                "Log (Completed: {}, Cancelled: {})",
                self.completed_count, self.cancelled_count
            )))
            .style(Style::default().fg(Color::White));
        frame.render_widget(messages_widget, chunks[2]);

        // Help
        let help = Paragraph::new(
            "Keys: [s] Start operation | [c] Cancel last | [x] Cancel all | [q] Quit",
        )
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[3]);
    }
}

// Advanced implementation with actual cancellation support
struct AdvancedCancellableDemo {
    messages: Vec<String>,
    active_operations: HashMap<usize, (String, AsyncHandle<String>)>,
    operation_counter: usize,
    cancelled_count: usize,
    completed_count: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Note: This is a simplified demo. For real cancellation support,
    // you would need to:
    // 1. Store AsyncHandle instances returned by program.spawn_cancellable()
    // 2. Call handle.cancel() when cancelling operations
    // 3. Use tokio::select! in async tasks to respond to cancellation

    let model = CancellableDemo {
        messages: Vec::new(),
        active_operations: Vec::new(),
        operation_counter: 0,
        cancelled_count: 0,
        completed_count: 0,
    };

    let options = ProgramOptions::default()
        .with_alt_screen(true)
        .with_mouse_mode(hojicha::program::MouseMode::CellMotion);

    let mut program = Program::with_options(model, options)?;

    // For a real implementation with cancellation, you would use:
    // let handle = program.spawn_cancellable(|token| async move {
    //     tokio::select! {
    //         _ = token.cancelled() => {
    //             return "Operation cancelled".to_string();
    //         }
    //         _ = tokio::time::sleep(Duration::from_secs(5)) => {
    //             return "Operation completed".to_string();
    //         }
    //     }
    // });
    //
    // Then later: handle.cancel().await;

    program.run()?;

    println!("Demo completed!");

    Ok(())
}
