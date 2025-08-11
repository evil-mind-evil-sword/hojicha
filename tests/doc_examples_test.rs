// Test all documentation examples to ensure they compile and work

#[cfg(test)]
mod readme_example {
    use hojicha_core::prelude::*;
    use hojicha_runtime::prelude::*;
    use ratatui::widgets::{Block, Borders, Paragraph};

    struct Counter {
        value: i32,
    }

    impl Model for Counter {
        type Message = ();

        fn update(&mut self, event: Event<()>) -> Cmd<()> {
            match event {
                Event::Key(key) => match key.key {
                    Key::Up => self.value += 1,
                    Key::Down => self.value -= 1,
                    Key::Char('q') => return commands::quit(),
                    _ => {}
                },
                _ => {}
            }
            Cmd::none()
        }

        fn view(&self, frame: &mut Frame, area: Rect) {
            let text = format!("Counter: {}\n\nUp/Down: change | q: quit", self.value);
            let widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Counter"));
            frame.render_widget(widget, area);
        }
    }

    #[test]
    fn test_readme_counter_compiles() {
        let _model = Counter { value: 0 };
        // This test just ensures the example compiles
    }
}