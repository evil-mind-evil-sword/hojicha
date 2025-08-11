//! Common test utilities and models shared across integration tests

use hojicha_core::{
    commands,
    core::{Cmd, Model},
    event::Event,
};
use ratatui::{layout::Rect, Frame};
use std::sync::{Arc, Mutex};

/// A simple test model that's reused across multiple tests
#[derive(Debug, Clone)]
pub struct SimpleTestModel {
    pub counter: i32,
    pub messages: Vec<String>,
    pub state: String,
}

impl Default for SimpleTestModel {
    fn default() -> Self {
        Self {
            counter: 0,
            messages: Vec::new(),
            state: "initial".to_string(),
        }
    }
}

impl Model for SimpleTestModel {
    type Message = TestMsg;

    fn init(&mut self) -> Cmd<Self::Message> {
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        if let Some(msg) = event.into_user() {
            match msg {
                TestMsg::Increment => {
                    self.counter += 1;
                    Cmd::none()
                }
                TestMsg::Decrement => {
                    self.counter -= 1;
                    Cmd::none()
                }
                TestMsg::SetState(state) => {
                    self.state = state;
                    Cmd::none()
                }
                TestMsg::AddMessage(msg) => {
                    self.messages.push(msg);
                    Cmd::none()
                }
                TestMsg::Clear => {
                    self.counter = 0;
                    self.messages.clear();
                    self.state = "cleared".to_string();
                    Cmd::none()
                }
                TestMsg::Batch => {
                    commands::batch(vec![
                        commands::custom(|| Some(TestMsg::Increment)),
                        commands::custom(|| Some(TestMsg::AddMessage("batched".to_string()))),
                    ])
                }
                TestMsg::Sequence => {
                    commands::sequence(vec![
                        commands::custom(|| Some(TestMsg::Increment)),
                        commands::custom(|| Some(TestMsg::Increment)),
                    ])
                }
                TestMsg::Quit => commands::quit(),
                TestMsg::Tick => {
                    self.messages.push("tick".to_string());
                    Cmd::none()
                }
            }
        } else {
            Cmd::none()
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {
        // Not used in tests
    }
}

/// Common test message types
#[derive(Debug, Clone, PartialEq)]
pub enum TestMsg {
    Increment,
    Decrement,
    SetState(String),
    AddMessage(String),
    Clear,
    Batch,
    Sequence,
    Quit,
    Tick,
}

/// A stress test model for performance testing
#[derive(Debug, Clone)]
pub struct StressTestModel {
    pub event_count: usize,
    pub messages: Arc<Mutex<Vec<String>>>,
}

impl Default for StressTestModel {
    fn default() -> Self {
        Self {
            event_count: 0,
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Model for StressTestModel {
    type Message = StressMsg;

    fn init(&mut self) -> Cmd<Self::Message> {
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        self.event_count += 1;
        
        if let Some(msg) = event.into_user() {
            match msg {
                StressMsg::Ping(id) => {
                    self.messages.lock().unwrap().push(format!("ping-{}", id));
                    commands::custom(move || Some(StressMsg::Pong(id)))
                }
                StressMsg::Pong(id) => {
                    self.messages.lock().unwrap().push(format!("pong-{}", id));
                    if id < 100 {
                        commands::custom(move || Some(StressMsg::Ping(id + 1)))
                    } else {
                        Cmd::none()
                    }
                }
                StressMsg::Spam(count) => {
                    let cmds: Vec<_> = (0..count)
                        .map(|i| commands::custom(move || Some(StressMsg::Message(i))))
                        .collect();
                    commands::batch(cmds)
                }
                StressMsg::Message(id) => {
                    self.messages.lock().unwrap().push(format!("msg-{}", id));
                    Cmd::none()
                }
            }
        } else {
            Cmd::none()
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {
        // Not used in tests
    }
}

/// Stress test message types
#[derive(Debug, Clone, PartialEq)]
pub enum StressMsg {
    Ping(usize),
    Pong(usize),
    Spam(usize),
    Message(usize),
}

/// Test utility functions
pub mod utils {
    use std::time::{Duration, Instant};
    
    /// Wait for a condition with timeout
    pub fn wait_for<F>(condition: F, timeout: Duration) -> bool
    where
        F: Fn() -> bool,
    {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if condition() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    }
    
    /// Create a test message vec
    pub fn test_messages(count: usize) -> Vec<String> {
        (0..count).map(|i| format!("test-{}", i)).collect()
    }
}