// Comprehensive test of all documentation examples

#[cfg(test)]
mod readme_examples {
    use hojicha_core::prelude::*;
    use hojicha_runtime::prelude::*;
    use ratatui::widgets::{Block, Borders, Paragraph};
    use std::time::Duration;

    // Test the main counter example
    #[test]
    fn test_counter_example() {
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
                        Key::Char('q') => return quit(),
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

        let mut counter = Counter { value: 0 };
        assert_eq!(counter.value, 0);
        counter.update(Event::Key(KeyEvent::new(Key::Up, KeyModifiers::empty())));
        assert_eq!(counter.value, 1);
    }

    // Test async operations example
    #[test]
    fn test_async_operations() {
        enum Msg {
            FetchData,
            DataLoaded(String),
        }

        struct App;

        impl Model for App {
            type Message = Msg;

            fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
                match event {
                    Event::User(Msg::FetchData) => {
                        spawn(async {
                            // Simulate fetch_api()
                            Some(Msg::DataLoaded("test".to_string()))
                        })
                    }
                    _ => Cmd::none()
                }
            }

            fn view(&self, _frame: &mut Frame, _area: Rect) {}
        }

        let mut app = App;
        let _cmd = app.update(Event::User(Msg::FetchData));
    }

    // Test timer example
    #[test]
    fn test_timer_example() {
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

        let mut app = App;
        let _cmd = app.init();
    }
}

#[cfg(test)]
mod common_patterns_examples {
    use hojicha_core::prelude::*;
    use hojicha_runtime::prelude::*;
    use ratatui::prelude::*;
    use ratatui::widgets::{Block, Borders, Paragraph};
    use std::time::Duration;
    use std::collections::HashMap;

    // Test basic application structure
    #[test]
    fn test_basic_app() {
        struct App {}
        enum Msg {}

        impl Model for App {
            type Message = Msg;
            
            fn init(&mut self) -> Cmd<Msg> {
                Cmd::none()
            }
            
            fn update(&mut self, _event: Event<Msg>) -> Cmd<Msg> {
                Cmd::none()
            }
            
            fn view(&self, _frame: &mut Frame, _area: Rect) {}
        }

        let _app = App {};
    }

    // Test loading states pattern
    #[test]
    fn test_loading_states() {
        enum LoadingState<T> {
            _NotStarted,
            Loading,
            Loaded(T),
            Failed(String),
        }

        struct Item;

        struct App {
            data: LoadingState<Vec<Item>>,
        }

        enum Msg {
            DataLoaded(Vec<Item>),
            LoadFailed(String),
        }

        impl Model for App {
            type Message = Msg;
            
            fn init(&mut self) -> Cmd<Msg> {
                self.data = LoadingState::Loading;
                spawn(async {
                    // Simulate fetch
                    Some(Msg::DataLoaded(vec![]))
                })
            }
            
            fn view(&self, frame: &mut Frame, area: Rect) {
                match &self.data {
                    LoadingState::Loading => {
                        let widget = Paragraph::new("Loading...");
                        frame.render_widget(widget, area);
                    }
                    LoadingState::Loaded(_data) => {
                        // Render data
                    }
                    LoadingState::Failed(error) => {
                        let widget = Paragraph::new(error.as_str());
                        frame.render_widget(widget, area);
                    }
                    _ => {}
                }
            }

            fn update(&mut self, _event: Event<Msg>) -> Cmd<Msg> {
                Cmd::none()
            }
        }

        let mut app = App { 
            data: LoadingState::Loading 
        };
        let _cmd = app.init();
    }

    // Test form validation
    #[test]
    fn test_form_state() {
        struct FormState {
            username: String,
            email: String,
            errors: HashMap<String, String>,
            _submitting: bool,
        }

        impl FormState {
            fn validate(&mut self) -> bool {
                self.errors.clear();
                
                if self.username.is_empty() {
                    self.errors.insert("username".into(), "Required".into());
                }
                
                if !self.email.contains('@') {
                    self.errors.insert("email".into(), "Invalid email".into());
                }
                
                self.errors.is_empty()
            }
        }

        let mut form = FormState {
            username: String::new(),
            email: String::new(),
            errors: HashMap::new(),
            _submitting: false,
        };

        assert!(!form.validate());
        assert!(form.errors.contains_key("username"));
        assert!(form.errors.contains_key("email"));

        form.username = "user".to_string();
        form.email = "user@example.com".to_string();
        assert!(form.validate());
        assert!(form.errors.is_empty());
    }

    // Test keyboard shortcuts
    #[test]
    fn test_keyboard_shortcuts() {
        enum Mode {
            Normal,
            _Insert,
            _Command,
        }

        struct App {
            mode: Mode,
        }

        enum Msg {
            Save,
            Open,
        }

        impl App {
            fn save(&self) -> Cmd<Msg> {
                Cmd::new(|| Some(Msg::Save))
            }

            fn open(&self) -> Cmd<Msg> {
                Cmd::new(|| Some(Msg::Open))
            }

            fn handle_normal_mode(&mut self, _key: KeyEvent) -> Cmd<Msg> {
                Cmd::none()
            }
        }

        impl Model for App {
            type Message = Msg;

            fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
                match event {
                    Event::Key(key) => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            match key.key {
                                Key::Char('s') => return self.save(),
                                Key::Char('o') => return self.open(),
                                Key::Char('q') => return quit(),
                                _ => {}
                            }
                        }
                        
                        match self.mode {
                            Mode::Normal => self.handle_normal_mode(key),
                            _ => Cmd::none(),
                        }
                    }
                    _ => Cmd::none()
                }
            }

            fn view(&self, _frame: &mut Frame, _area: Rect) {}
        }

        let mut app = App { mode: Mode::Normal };
        let cmd = app.update(Event::Key(KeyEvent::new(
            Key::Char('s'), 
            KeyModifiers::CONTROL
        )));
        assert!(!cmd.is_noop());
    }

    // Test error handling
    #[test]
    fn test_error_handling() {
        struct App {
            data: String,
        }

        enum Msg {
            SaveFile,
            SaveSuccess,
            SaveError(String),
        }

        impl Model for App {
            type Message = Msg;

            fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
                match event {
                    Event::User(Msg::SaveFile) => {
                        let data = self.data.clone();
                        custom_fallible(move || {
                            match std::fs::write("/tmp/test.json", &data) {
                                Ok(_) => Ok(Some(Msg::SaveSuccess)),
                                Err(e) => Ok(Some(Msg::SaveError(e.to_string()))),
                            }
                        })
                    }
                    _ => Cmd::none()
                }
            }

            fn view(&self, _frame: &mut Frame, _area: Rect) {}
        }

        let mut app = App { data: "test".to_string() };
        let _cmd = app.update(Event::User(Msg::SaveFile));
    }

    // Test error recovery with retry
    #[test]
    fn test_error_recovery() {
        struct App {
            last_error: Option<String>,
            retry_count: u32,
        }

        enum Msg {
            Error(String),
            Retry,
        }

        impl Model for App {
            type Message = Msg;

            fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
                match event {
                    Event::User(Msg::Error(e)) => {
                        self.last_error = Some(e);
                        self.retry_count += 1;
                        
                        if self.retry_count < 3 {
                            let delay = Duration::from_secs(2_u64.pow(self.retry_count));
                            tick(delay, || Msg::Retry)
                        } else {
                            Cmd::none()
                        }
                    }
                    _ => Cmd::none()
                }
            }

            fn view(&self, _frame: &mut Frame, _area: Rect) {}
        }

        let mut app = App { 
            last_error: None, 
            retry_count: 0 
        };
        
        let cmd = app.update(Event::User(Msg::Error("test".to_string())));
        assert!(!cmd.is_noop());
        assert_eq!(app.retry_count, 1);
    }
}