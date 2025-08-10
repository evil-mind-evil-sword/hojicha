//! Simple test to verify async bridge functionality

use hojicha::commands;
use hojicha::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
struct SimpleModel {
    received: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Debug)]
enum Msg {
    Test(String),
}

impl Model for SimpleModel {
    type Message = Msg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(Msg::Test(s)) => {
                self.received.lock().unwrap().push(s);
                // Quit after receiving one message
                commands::quit()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut Frame, _: ratatui::layout::Rect) {}
}

#[test]
fn test_basic_async_bridge() {
    let model = SimpleModel::default();
    let received = model.received.clone();

    let mut program = Program::with_options(model, ProgramOptions::default().headless()).unwrap();

    // Initialize async bridge
    let sender = program.init_async_bridge();

    // Send a message
    sender
        .send(Event::User(Msg::Test("Hello".to_string())))
        .unwrap();

    // Run program - should quit after receiving the message
    program
        .run_with_timeout(Duration::from_millis(100))
        .unwrap();

    // Check message was received
    let msgs = received.lock().unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0], "Hello");
}
