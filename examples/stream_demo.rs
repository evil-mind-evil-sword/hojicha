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

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        self.messages.push("Stream demo started!".to_string());
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
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
                    return Some(commands::quit());
                }

                Cmd::none()
            }
            Event::Quit => None,
            Event::Key(key) if key.key == hojicha::event::Key::Char('q') => Some(commands::quit()),
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
    
    // Initialize async bridge for external events
    let sender = program.init_async_bridge();
    
    // Spawn async tasks that send messages through the bridge
    std::thread::spawn(move || {
        // Use a local tokio runtime for the streams
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Create multiple streams
            let sender1 = sender.clone();
            let sender2 = sender.clone();
            let sender3 = sender.clone();
            
            // Stream 1
            tokio::spawn(async move {
                for i in 0..20 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let _ = sender1.send(Event::User(Msg::StreamValue(format!("Stream 1: Value {}", i))));
                }
            });
            
            // Stream 2
            tokio::spawn(async move {
                for i in 0..15 {
                    tokio::time::sleep(Duration::from_millis(150)).await;
                    let _ = sender2.send(Event::User(Msg::StreamValue(format!("Stream 2: Data {}", i * 2))));
                }
            });
            
            // Stream 3
            tokio::spawn(async move {
                for i in 0..15 {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    let _ = sender3.send(Event::User(Msg::StreamValue(format!("Stream 3: Item {}", i * 3))));
                }
            });
            
            // Keep runtime alive for the duration
            tokio::time::sleep(Duration::from_secs(10)).await;
        });
    });

    // Run the program
    program.run()?;

    println!("Stream demo completed!");
    Ok(())
}
