//! Example demonstrating async event injection using a timer
//!
//! This shows how to use the async bridge to send messages from external threads.

use hojicha::commands;
use hojicha::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Default)]
struct TimerApp {
    events: Vec<String>,
    start_time: Option<Instant>,
    tick_count: u32,
    external_count: u32,
}

#[derive(Clone, Debug)]
enum Msg {
    Tick,                 // Timer tick every second
    ExternalData(String), // Data from external source
    FastTick,             // Fast timer every 100ms
}

impl Model for TimerApp {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        self.start_time = Some(Instant::now());
        self.events.push("App initialized".to_string());
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(Msg::Tick) => {
                self.tick_count += 1;
                let elapsed = self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);
                self.events
                    .push(format!("[{}s] Timer tick #{}", elapsed, self.tick_count));

                // Keep only last 20 events
                if self.events.len() > 20 {
                    self.events.remove(0);
                }
            }
            Event::User(Msg::ExternalData(data)) => {
                self.external_count += 1;
                let elapsed = self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);
                self.events
                    .push(format!("[{}s] External: {}", elapsed, data));

                if self.events.len() > 20 {
                    self.events.remove(0);
                }
            }
            Event::User(Msg::FastTick) => {
                // Just count fast ticks, don't log them all
                // This demonstrates high-frequency messages
            }
            Event::Key(key) => match key.key {
                Key::Char('q') | Key::Esc => return commands::quit(),
                Key::Char('c') => {
                    self.events.clear();
                    self.events.push("Events cleared".to_string());
                }
                _ => {}
            },
            _ => {}
        }
        Cmd::none()
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(6),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(area);

        // Stats panel
        let stats = format!(
            "Timer Ticks: {}\nExternal Messages: {}\nElapsed: {}s\n\nPress 'q' to quit, 'c' to clear",
            self.tick_count,
            self.external_count,
            self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0)
        );
        let stats_widget = Paragraph::new(stats).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Async Timer Demo"),
        );
        frame.render_widget(stats_widget, chunks[0]);

        // Event log
        let items: Vec<ListItem> = self
            .events
            .iter()
            .map(|e| ListItem::new(e.as_str()))
            .collect();
        let list =
            List::new(items).block(Block::default().borders(Borders::ALL).title("Event Log"));
        frame.render_widget(list, chunks[1]);
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create the program and initialize async bridge
    let mut program = Program::new(TimerApp::default())?;
    let sender = program.init_async_bridge();

    // Start a timer thread that sends a tick every second
    let timer_sender = sender.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            if timer_sender.send(Event::User(Msg::Tick)).is_err() {
                // Program has shut down
                break;
            }
        }
    });

    // Start another thread simulating external data
    let external_sender = sender.clone();
    thread::spawn(move || {
        let messages = vec![
            "Connection established",
            "Data received from server",
            "Processing update",
            "Cache refreshed",
            "Background task completed",
        ];

        for (i, msg) in messages.iter().cycle().enumerate() {
            thread::sleep(Duration::from_millis(2500));
            let data = format!("{} ({})", msg, i + 1);
            if external_sender
                .send(Event::User(Msg::ExternalData(data)))
                .is_err()
            {
                break;
            }
        }
    });

    // Start a fast timer to demonstrate handling high-frequency messages
    let fast_sender = sender.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(100));
        if fast_sender.send(Event::User(Msg::FastTick)).is_err() {
            break;
        }
    });

    // Run the program
    program.run()?;

    Ok(())
}
