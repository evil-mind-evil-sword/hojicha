// Test README examples compile correctly

#[cfg(test)]
mod readme_counter {
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
                    Key::Char('q') => return quit(), // Use quit from prelude
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
    fn test_counter_compiles() {
        let counter = Counter { value: 0 };
        // Test basic operations
        let mut counter = counter;
        let cmd = counter.update(Event::Key(KeyEvent::new(Key::Up, KeyModifiers::empty())));
        assert!(cmd.is_noop());
        assert_eq!(counter.value, 1);
        
        let cmd = counter.update(Event::Key(KeyEvent::new(Key::Down, KeyModifiers::empty())));
        assert!(cmd.is_noop());
        assert_eq!(counter.value, 0);
        
        let cmd = counter.update(Event::Key(KeyEvent::new(Key::Char('q'), KeyModifiers::empty())));
        assert!(cmd.is_quit());
    }
}

#[cfg(test)]
mod readme_async_operations {
    use hojicha_core::prelude::*;
    use hojicha_runtime::prelude::*;
    
    enum Msg {
        FetchData,
        DataLoaded(String),
    }
    
    struct App;
    
    async fn fetch_api() -> std::result::Result<String, Box<dyn std::error::Error>> {
        Ok("test data".to_string())
    }
    
    impl Model for App {
        type Message = Msg;
        
        fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
            match event {
                Event::User(Msg::FetchData) => {
                    // This doesn't exist in commands module
                    // commands::spawn(async {
                    //     let data = fetch_api().await.ok()?;
                    //     Some(Msg::DataLoaded(data))
                    // })
                    
                    // Use Cmd::async_cmd instead (but it's marked hidden!)
                    // Or use the proper pattern
                    Cmd::new(|| {
                        // Can't do async here directly
                        // This example is wrong!
                        None
                    })
                }
                _ => Cmd::none()
            }
        }
        
        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }
    
    #[test]
    fn test_async_pattern() {
        // This example in README is incorrect - spawn doesn't exist in commands
    }
}

#[cfg(test)]
mod readme_timers {
    use hojicha_core::prelude::*;
    use hojicha_runtime::prelude::*;
    use std::time::Duration;
    
    enum Msg {
        Tick,
    }
    
    struct App;
    
    impl Model for App {
        type Message = Msg;
        
        fn init(&mut self) -> Cmd<Msg> {
            every(Duration::from_secs(1), |_| Msg::Tick)
        }
        
        fn update(&mut self, _event: Event<Msg>) -> Cmd<Msg> {
            Cmd::none()
        }
        
        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }
    
    #[test]
    fn test_timer_compiles() {
        let mut app = App;
        let cmd = app.init();
        // Can't easily test if it's an Every command since those methods are hidden
    }
}