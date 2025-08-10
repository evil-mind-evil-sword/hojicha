use hojicha::{
    commands,
    core::{Cmd, Model},
    event::Event,
    program::{Program, ProgramOptions},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Duration;

#[derive(Clone)]
struct StreamDemo {
    messages: Vec<String>,
    count: usize,
}

#[derive(Debug, Clone)]
enum Msg {
    StreamValue(String),
}

impl Model for StreamDemo {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        self.messages.push("Stream demo started!".to_string());
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(Msg::StreamValue(val)) => {
                self.count += 1;
                self.messages.push(format!("#{}: {}", self.count, val));

                // Keep only last 20 messages
                if self.messages.len() > 20 {
                    self.messages.remove(0);
                }

                // Quit after 50 messages
                if self.count >= 50 {
                    return commands::quit();
                }

                Cmd::none()
            }
            Event::Quit => commands::quit(),
            Event::Key(key) if key.key == hojicha::event::Key::Char('q') => commands::quit(),
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let title = Paragraph::new(format!("Stream Demo - Received {} messages", self.count))
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        let messages_text = self.messages.join("\n");
        let messages_widget = Paragraph::new(messages_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Stream Messages"),
            )
            .style(Style::default().fg(Color::White));
        frame.render_widget(messages_widget, chunks[1]);

        let help = Paragraph::new("Press 'q' to quit | Receiving stream data...")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[2]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = StreamDemo {
        messages: Vec::new(),
        count: 0,
    };

    let options = ProgramOptions::default()
        .with_alt_screen(true)
        .with_mouse_mode(hojicha::program::MouseMode::CellMotion);

    let mut program = Program::with_options(model, options)?;

    // Use the built-in stream subscription system
    use futures::stream::StreamExt;
    use hojicha::stream_builders::interval_stream;

    // Create Stream 1: Fast updates every 100ms
    let stream1 = interval_stream(Duration::from_millis(100))
        .take(20)
        .enumerate()
        .map(|(i, _)| Msg::StreamValue(format!("Stream 1: Value {}", i)));

    // Create Stream 2: Medium updates every 150ms
    let stream2 = interval_stream(Duration::from_millis(150))
        .take(15)
        .enumerate()
        .map(|(i, _)| Msg::StreamValue(format!("Stream 2: Data {}", i * 2)));

    // Create Stream 3: Slow updates every 200ms
    let stream3 = interval_stream(Duration::from_millis(200))
        .take(15)
        .enumerate()
        .map(|(i, _)| Msg::StreamValue(format!("Stream 3: Item {}", i * 3)));

    // Subscribe to all streams
    program.subscribe(stream1);
    program.subscribe(stream2);
    program.subscribe(stream3);

    // Run the program
    program.run()?;

    println!("Stream demo completed!");
    Ok(())
}
