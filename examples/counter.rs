//! A simple counter example, similar to Bubbletea's examples

use hojicha::prelude::*;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

/// The model for our counter application
struct Counter {
    value: i32,
}

/// Messages that update our counter
/// Note: This example handles keyboard input directly in the update function,
/// but these messages could be sent via commands from other parts of the app
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Msg {
    Increment,
    Decrement,
    Reset,
}

impl Model for Counter {
    type Message = Msg;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        // No initial command
        None
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(msg) => match msg {
                Msg::Increment => self.value += 1,
                Msg::Decrement => self.value -= 1,
                Msg::Reset => self.value = 0,
            },
            Event::Key(key) => match key.key {
                Key::Char('q') | Key::Esc => return Some(quit()),
                Key::Up | Key::Char('k') => self.value += 1,
                Key::Down | Key::Char('j') => self.value -= 1,
                Key::Char('r') => self.value = 0,
                _ => {}
            },
            _ => {}
        }
        None
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(format!("Counter: {}", self.value)).centered(),
            Line::from(""),
            Line::from("↑/k: increment").centered(),
            Line::from("↓/j: decrement").centered(),
            Line::from("r: reset").centered(),
            Line::from("q/esc: quit").centered(),
        ];

        let widget = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Counter Example "),
        );

        frame.render_widget(widget, area);
    }
}

fn main() -> hojicha::Result<()> {
    let counter = Counter { value: 0 };
    let program = Program::new(counter)?;
    program.run()?;
    Ok(())
}
