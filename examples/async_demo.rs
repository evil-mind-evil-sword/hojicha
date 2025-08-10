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
use std::time::{Duration, Instant};

#[derive(Clone)]
struct AsyncDemo {
    messages: Vec<String>,
    start_time: Instant,
    tick_count: u32,
}

#[derive(Debug, Clone)]
enum Msg {
    Tick,
    AsyncComplete(String),
    Quit,
}

impl Model for AsyncDemo {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        self.messages.push("Starting async demo...".to_string());

        // Demonstrate async execution with batch
        commands::batch(vec![
            // This will complete immediately
            commands::custom(|| Some(Msg::AsyncComplete("Immediate task completed".to_string()))),
            // This will complete after 1 second
            commands::tick(Duration::from_secs(1), || {
                Msg::AsyncComplete("1 second delay completed".to_string())
            }),
            // This will complete after 2 seconds
            commands::tick(Duration::from_secs(2), || {
                Msg::AsyncComplete("2 second delay completed".to_string())
            }),
            // Start a recurring tick every 500ms
            commands::every(Duration::from_millis(500), |_| Msg::Tick),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(Msg::Tick) => {
                self.tick_count += 1;
                let elapsed = self.start_time.elapsed();
                self.messages.push(format!(
                    "[{:.1}s] Tick #{}",
                    elapsed.as_secs_f32(),
                    self.tick_count
                ));

                // Stop after 10 ticks
                if self.tick_count >= 10 {
                    return commands::quit();
                }

                // Schedule next tick
                commands::tick(Duration::from_millis(500), || Msg::Tick)
            }
            Event::User(Msg::AsyncComplete(msg)) => {
                let elapsed = self.start_time.elapsed();
                self.messages
                    .push(format!("[{:.1}s] {}", elapsed.as_secs_f32(), msg));
                Cmd::none()
            }
            Event::User(Msg::Quit) | Event::Quit => commands::quit(),
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

        let title = Paragraph::new("Async Command Demo")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        let messages: Vec<String> = self
            .messages
            .iter()
            .rev()
            .take(chunks[1].height as usize - 2)
            .cloned()
            .collect();
        let messages_text = messages.join("\n");

        let messages_widget = Paragraph::new(messages_text)
            .block(Block::default().borders(Borders::ALL).title("Messages"))
            .style(Style::default().fg(Color::White));
        frame.render_widget(messages_widget, chunks[1]);

        let help = Paragraph::new("Press 'q' to quit | Watch async commands execute!")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[2]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = AsyncDemo {
        messages: Vec::new(),
        start_time: Instant::now(),
        tick_count: 0,
    };

    let options = ProgramOptions::default()
        .with_alt_screen(true)
        .with_mouse_mode(hojicha::program::MouseMode::CellMotion);

    let program = Program::with_options(model, options)?;
    program.run()?;

    Ok(())
}
