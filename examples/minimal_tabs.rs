//! Minimal test to isolate Tabs issue

use hojicha::{
    commands,
    core::{Cmd, Model},
    event::{Event, Key},
    program::Program,
};
use ratatui::{layout::Rect, widgets::Paragraph, Frame};

struct MinimalTabs {
    counter: usize,
    selected_tab: usize,
}

impl Model for MinimalTabs {
    type Message = ();

    fn init(&mut self) -> Cmd<Self::Message> {
        eprintln!("Init called");
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        eprintln!("Update called with event: {:?}", event);

        match event {
            Event::Key(key) => {
                eprintln!("Key event: {:?}", key);
                match key.key {
                    Key::Char('q') => {
                        eprintln!("Quit requested");
                        commands::quit()
                    }
                    Key::Tab => {
                        eprintln!("Tab pressed - before: selected_tab={}", self.selected_tab);
                        self.selected_tab = (self.selected_tab + 1) % 3;
                        eprintln!("Tab pressed - after: selected_tab={}", self.selected_tab);
                        self.counter += 1;
                        Cmd::none()
                    }
                    _ => {
                        eprintln!("Other key: {:?}", key.key);
                        Cmd::none()
                    }
                }
            }
            _ => {
                eprintln!("Non-key event: {:?}", event);
                Cmd::none()
            }
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        eprintln!(
            "View called - selected_tab={}, counter={}",
            self.selected_tab, self.counter
        );

        let text = format!(
            "Minimal Tabs Test\n\n\
             Selected Tab: {}\n\
             Tab presses: {}\n\n\
             Press Tab to switch tabs\n\
             Press q to quit\n\n\
             If this quits when you press Tab,\n\
             the issue is NOT in the Tabs component.",
            self.selected_tab, self.counter
        );

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }
}

fn main() -> hojicha::Result<()> {
    eprintln!("=== Starting Minimal Tabs Test ===");

    let model = MinimalTabs {
        counter: 0,
        selected_tab: 0,
    };

    let result = Program::new(model)?.run();

    eprintln!("=== Program exited ===");
    eprintln!("Result: {:?}", result);

    result
}
