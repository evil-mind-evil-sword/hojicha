//! Testing utilities for boba applications
//!
//! This module provides tools for testing TUI applications in headless mode.

pub mod event_recorder;
pub mod event_test_harness;
pub mod mock_terminal;
pub mod test_backend;
pub mod test_runner;
pub mod time_control;

pub use event_recorder::{EventRecorder, RecordedEvent};
pub use event_test_harness::{EventTestHarness, PriorityEventTestHarness};
pub use mock_terminal::MockTerminal;
pub use test_backend::TestBackend;
pub use test_runner::TestRunner;

use crate::{
    core::{Cmd, Model},
    event::Event,
};
use std::sync::{Arc, Mutex};

/// Test harness for running models in headless mode
pub struct TestHarness<M: Model> {
    model: M,
    events: Vec<Event<M::Message>>,
    outputs: Arc<Mutex<Vec<String>>>,
}

impl<M: Model> TestHarness<M> {
    /// Create a new test harness
    pub fn new(model: M) -> Self {
        Self {
            model,
            events: Vec::new(),
            outputs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add an event to be processed
    pub fn send_event(mut self, event: Event<M::Message>) -> Self {
        self.events.push(event);
        self
    }

    /// Add multiple events
    pub fn send_events(mut self, events: Vec<Event<M::Message>>) -> Self {
        self.events.extend(events);
        self
    }

    /// Run the test and collect outputs
    pub fn run(mut self) -> TestResult<M>
    where
        M::Message: std::fmt::Debug,
        Cmd<M::Message>: std::fmt::Debug,
    {
        let outputs = Arc::clone(&self.outputs);
        let mut commands_executed = Vec::new();

        // Run init
        if let Some(cmd) = self.model.init() {
            commands_executed.push(format!("Init command: {cmd:?}"));
        }

        // Process each event
        for event in self.events {
            let event_str = format!("{event:?}");
            if let Some(cmd) = self.model.update(event) {
                commands_executed.push(format!("Command from {event_str}: {cmd:?}"));
            }
        }

        let final_outputs = outputs.lock().unwrap().clone();
        TestResult {
            model: self.model,
            outputs: final_outputs,
            commands_executed,
        }
    }
}

/// Result of a test run
pub struct TestResult<M: Model> {
    pub model: M,
    pub outputs: Vec<String>,
    pub commands_executed: Vec<String>,
}

impl<M: Model> TestResult<M> {
    /// Assert that a specific output was produced
    pub fn assert_output_contains(&self, expected: &str) -> &Self {
        assert!(
            self.outputs.iter().any(|o| o.contains(expected)),
            "Expected output to contain '{}', but got: {:?}",
            expected,
            self.outputs
        );
        self
    }

    /// Assert that a command was executed
    pub fn assert_command_executed(&self, command_substr: &str) -> &Self {
        assert!(
            self.commands_executed
                .iter()
                .any(|c| c.contains(command_substr)),
            "Expected command containing '{}' to be executed, but got: {:?}",
            command_substr,
            self.commands_executed
        );
        self
    }

    /// Get the final model state
    pub fn into_model(self) -> M {
        self.model
    }
}

/// Macro for easily creating test events
#[macro_export]
macro_rules! test_events {
    ($($event:expr),* $(,)?) => {
        vec![$($event),*]
    };
}

/// Macro for key events
#[macro_export]
macro_rules! key {
    (Char($ch:expr)) => {
        Event::Key($crate::event::KeyEvent::new(
            $crate::event::Key::Char($ch),
            $crate::event::KeyModifiers::empty(),
        ))
    };
    (Enter) => {
        Event::Key($crate::event::KeyEvent::new(
            $crate::event::Key::Enter,
            $crate::event::KeyModifiers::empty(),
        ))
    };
    (Esc) => {
        Event::Key($crate::event::KeyEvent::new(
            $crate::event::Key::Esc,
            $crate::event::KeyModifiers::empty(),
        ))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestModel {
        counter: i32,
    }

    impl Model for TestModel {
        type Message = i32;

        fn init(&mut self) -> Option<Cmd<Self::Message>> {
            None
        }

        fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
            match event {
                Event::User(n) => {
                    self.counter += n;
                    None
                }
                _ => None,
            }
        }

        fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {
            // Not used in tests
        }
    }

    #[test]
    fn test_harness_basic() {
        let model = TestModel { counter: 0 };

        let result = TestHarness::new(model)
            .send_event(Event::User(5))
            .send_event(Event::User(3))
            .run();

        assert_eq!(result.model.counter, 8);
    }
}
