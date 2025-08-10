//! Tests for bracketed paste functionality

use hojicha::commands;
use hojicha::prelude::*;
use std::sync::{Arc, Mutex};

#[test]
fn test_paste_event() {
    #[derive(Clone)]
    struct PasteModel {
        pastes: Arc<Mutex<Vec<String>>>,
    }

    #[derive(Debug, Clone)]
    enum Msg {}

    impl Model for PasteModel {
        type Message = Msg;

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::Paste(text) => {
                    self.pastes.lock().unwrap().push(text);
                }
                Event::Quit => return commands::quit(),
                _ => {}
            }
            Cmd::none()
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = PasteModel {
        pastes: Arc::new(Mutex::new(Vec::new())),
    };

    // Test paste event
    let mut test_model = model.clone();
    test_model.update(Event::Paste("Hello, World!".to_string()));

    let pastes = model.pastes.lock().unwrap();
    assert_eq!(pastes.len(), 1);
    assert_eq!(pastes[0], "Hello, World!");
}

#[test]
fn test_bracketed_paste_commands() {
    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {}

    // The commands should compile and be creatable
    let _enable = enable_bracketed_paste::<TestMsg>();
    let _disable = disable_bracketed_paste::<TestMsg>();

    // They should return None when executed (handled by runtime)
    let enable_cmd = enable_bracketed_paste::<TestMsg>();
    assert!(enable_cmd.test_execute().unwrap().is_none());

    let disable_cmd = disable_bracketed_paste::<TestMsg>();
    assert!(disable_cmd.test_execute().unwrap().is_none());
}

#[test]
fn test_program_options_bracketed_paste() {
    let opts = ProgramOptions::default().with_bracketed_paste(true);
    assert!(opts.bracketed_paste);

    let default_opts = ProgramOptions::default();
    assert!(!default_opts.bracketed_paste);
}
