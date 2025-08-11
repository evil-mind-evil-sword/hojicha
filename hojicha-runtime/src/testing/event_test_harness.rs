//! Enhanced event test harness for deterministic testing

use hojicha_core::core::{Cmd, Model};
use hojicha_core::event::{Event, Key, KeyEvent};
use std::collections::VecDeque;
use std::time::Duration;

/// Enhanced event test harness with better control
pub struct EventTestHarness<M: Model> {
    model: M,
    event_queue: VecDeque<Event<M::Message>>,
    message_history: Vec<M::Message>,
    command_history: Vec<String>,
    tick_callbacks: Vec<(Duration, Box<dyn FnOnce() -> M::Message>)>,
    every_callbacks: Vec<(Duration, Box<dyn FnOnce(std::time::Instant) -> M::Message>)>,
}

impl<M> EventTestHarness<M>
where
    M: Model,
    M::Message: Clone + std::fmt::Debug,
{
    /// Create a new event test harness
    pub fn new(mut model: M) -> Self {
        // Initialize the model
        let init_cmd = model.init();
        
        let mut harness = Self {
            model,
            event_queue: VecDeque::new(),
            message_history: Vec::new(),
            command_history: Vec::new(),
            tick_callbacks: Vec::new(),
            every_callbacks: Vec::new(),
        };
        
        // Process initial command
        if !init_cmd.is_noop() {
            harness.process_command(init_cmd);
        }
        
        harness
    }
    
    /// Send a key event
    pub fn send_key(&mut self, key: Key) {
        use hojicha_core::event::KeyModifiers;
        let event = Event::Key(KeyEvent {
            key,
            modifiers: KeyModifiers::empty(),
        });
        self.send_event(event);
    }
    
    /// Send a character key
    pub fn send_char(&mut self, c: char) {
        self.send_key(Key::Char(c));
    }
    
    /// Send an event
    pub fn send_event(&mut self, event: Event<M::Message>) {
        self.event_queue.push_back(event);
        self.process_events();
    }
    
    /// Send a user message
    pub fn send_message(&mut self, message: M::Message) {
        self.send_event(Event::User(message));
    }
    
    /// Process all queued events
    fn process_events(&mut self) {
        while let Some(event) = self.event_queue.pop_front() {
            // Log user messages
            if let Event::User(ref msg) = event {
                self.message_history.push(msg.clone());
            }
            
            // Update model
            let cmd = self.model.update(event);
            
            // Process resulting command
            if !cmd.is_noop() {
                self.process_command(cmd);
            }
        }
    }
    
    /// Process a command and extract its effects
    fn process_command(&mut self, cmd: Cmd<M::Message>) {
        if cmd.is_quit() {
            self.command_history.push("quit".to_string());
        } else if cmd.is_tick() {
            self.command_history.push("tick".to_string());
            // Store tick callback for manual triggering
            if let Some((duration, callback)) = cmd.take_tick() {
                self.tick_callbacks.push((duration, Box::new(callback)));
            }
        } else if cmd.is_every() {
            self.command_history.push("every".to_string());
            // Store every callback for manual triggering
            if let Some((duration, callback)) = cmd.take_every() {
                self.every_callbacks.push((duration, Box::new(callback)));
            }
        } else if cmd.is_batch() {
            self.command_history.push("batch".to_string());
            if let Some(cmds) = cmd.take_batch() {
                for subcmd in cmds {
                    self.process_command(subcmd);
                }
            }
        } else if cmd.is_sequence() {
            self.command_history.push("sequence".to_string());
            if let Some(cmds) = cmd.take_sequence() {
                for subcmd in cmds {
                    self.process_command(subcmd);
                }
            }
        } else if cmd.is_async() {
            self.command_history.push("async".to_string());
            // For testing, we can't easily execute async commands synchronously
            // This would need special handling based on test requirements
        } else {
            // Try to execute regular commands
            if let Ok(Some(msg)) = cmd.test_execute() {
                self.send_message(msg);
            }
        }
    }
    
    /// Manually trigger all tick callbacks
    pub fn trigger_ticks(&mut self) {
        let callbacks = std::mem::take(&mut self.tick_callbacks);
        for (_, callback) in callbacks {
            let msg = callback();
            self.send_message(msg);
        }
    }
    
    /// Manually trigger tick callbacks for a specific duration
    pub fn trigger_ticks_for(&mut self, target_duration: Duration) {
        let mut triggered = Vec::new();
        
        for (i, (duration, _)) in self.tick_callbacks.iter().enumerate() {
            if *duration <= target_duration {
                triggered.push(i);
            }
        }
        
        // Execute and remove triggered callbacks
        for i in triggered.into_iter().rev() {
            let (_, callback) = self.tick_callbacks.remove(i);
            let msg = callback();
            self.send_message(msg);
        }
    }
    
    /// Manually trigger every callbacks (can only trigger once due to FnOnce)
    pub fn trigger_every(&mut self, _count: usize) {
        let now = std::time::Instant::now();
        
        // Take ownership since we can only call FnOnce once
        let callbacks = std::mem::take(&mut self.every_callbacks);
        for (_, callback) in callbacks {
            let msg = callback(now);
            self.send_message(msg);
        }
    }
    
    /// Check if a message was received
    pub fn received(&self, message: &M::Message) -> bool
    where
        M::Message: PartialEq,
    {
        self.message_history.iter().any(|m| m == message)
    }
    
    /// Get all received messages
    pub fn messages(&self) -> &[M::Message] {
        &self.message_history
    }
    
    /// Get the last received message
    pub fn last_message(&self) -> Option<&M::Message> {
        self.message_history.last()
    }
    
    /// Check if a command was executed
    pub fn executed_command(&self, command_type: &str) -> bool {
        self.command_history.iter().any(|c| c == command_type)
    }
    
    /// Get the command history
    pub fn command_history(&self) -> &[String] {
        &self.command_history
    }
    
    /// Clear the message history
    pub fn clear_history(&mut self) {
        self.message_history.clear();
        self.command_history.clear();
    }
    
    /// Get a reference to the model
    pub fn model(&self) -> &M {
        &self.model
    }
    
    /// Get a mutable reference to the model
    pub fn model_mut(&mut self) -> &mut M {
        &mut self.model
    }
    
    /// Assert that the model satisfies a predicate
    pub fn assert_model<F>(&self, predicate: F) -> bool
    where
        F: FnOnce(&M) -> bool,
    {
        predicate(&self.model)
    }
}

/// Builder for creating test scenarios with events
pub struct EventScenarioBuilder<M: Model> {
    harness: EventTestHarness<M>,
    steps: Vec<EventTestStep<M::Message>>,
}

enum EventTestStep<M> {
    SendKey(Key),
    SendMessage(M),
    TriggerTicks,
    TriggerEvery(usize),
    AssertReceived(M),
}

impl<M> EventScenarioBuilder<M>
where
    M: Model,
    M::Message: Clone + PartialEq + std::fmt::Debug,
{
    /// Create a new scenario builder
    pub fn new(model: M) -> Self {
        Self {
            harness: EventTestHarness::new(model),
            steps: Vec::new(),
        }
    }
    
    /// Add a key press
    pub fn key(mut self, key: Key) -> Self {
        self.steps.push(EventTestStep::SendKey(key));
        self
    }
    
    /// Add a character key press
    pub fn char(self, c: char) -> Self {
        self.key(Key::Char(c))
    }
    
    /// Send a message
    pub fn message(mut self, msg: M::Message) -> Self {
        self.steps.push(EventTestStep::SendMessage(msg));
        self
    }
    
    /// Trigger all tick callbacks
    pub fn trigger_ticks(mut self) -> Self {
        self.steps.push(EventTestStep::TriggerTicks);
        self
    }
    
    /// Trigger every callbacks N times
    pub fn trigger_every(mut self, count: usize) -> Self {
        self.steps.push(EventTestStep::TriggerEvery(count));
        self
    }
    
    /// Assert a message was received
    pub fn expect_message(mut self, msg: M::Message) -> Self {
        self.steps.push(EventTestStep::AssertReceived(msg));
        self
    }
    
    /// Assert on model state (currently unsupported due to ownership constraints)
    pub fn expect_model<F>(self, _predicate: F) -> Self
    where
        F: FnOnce(&M) -> bool + 'static,
    {
        // Model assertions require rethinking due to ownership
        eprintln!("Warning: Model assertions not yet fully supported in builder");
        self
    }
    
    /// Run the scenario
    pub fn run(mut self) -> Result<EventTestHarness<M>, String> {
        for (i, step) in self.steps.into_iter().enumerate() {
            match step {
                EventTestStep::SendKey(key) => {
                    self.harness.send_key(key);
                }
                EventTestStep::SendMessage(msg) => {
                    self.harness.send_message(msg);
                }
                EventTestStep::TriggerTicks => {
                    self.harness.trigger_ticks();
                }
                EventTestStep::TriggerEvery(count) => {
                    self.harness.trigger_every(count);
                }
                EventTestStep::AssertReceived(expected) => {
                    if !self.harness.received(&expected) {
                        return Err(format!(
                            "Step {}: Expected message {:?} was not received. History: {:?}",
                            i + 1,
                            expected,
                            self.harness.messages()
                        ));
                    }
                }
            }
        }
        
        Ok(self.harness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hojicha_core::commands;
    
    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {
        Increment,
        Decrement,
        TimerFired,
        Reset,
    }
    
    struct CounterModel {
        value: i32,
    }
    
    impl Model for CounterModel {
        type Message = TestMsg;
        
        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }
        
        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(TestMsg::Increment) => {
                    self.value += 1;
                    commands::tick(Duration::from_secs(1), || TestMsg::TimerFired)
                }
                Event::User(TestMsg::Decrement) => {
                    self.value -= 1;
                    Cmd::none()
                }
                Event::User(TestMsg::TimerFired) => {
                    self.value *= 2;
                    Cmd::none()
                }
                Event::User(TestMsg::Reset) => {
                    self.value = 0;
                    Cmd::none()
                }
                Event::Key(key) => match key.key {
                    Key::Up => self.update(Event::User(TestMsg::Increment)),
                    Key::Down => self.update(Event::User(TestMsg::Decrement)),
                    Key::Char('r') => self.update(Event::User(TestMsg::Reset)),
                    _ => Cmd::none(),
                },
                _ => Cmd::none(),
            }
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    }
    
    #[test]
    fn test_event_harness() {
        let mut harness = EventTestHarness::new(CounterModel { value: 0 });
        
        // Send increment message
        harness.send_message(TestMsg::Increment);
        assert_eq!(harness.model().value, 1);
        assert!(harness.executed_command("tick"));
        
        // Trigger the tick callback
        harness.trigger_ticks();
        assert!(harness.received(&TestMsg::TimerFired));
        assert_eq!(harness.model().value, 2);
    }
    
    #[test]
    fn test_scenario_builder() {
        let result = EventScenarioBuilder::new(CounterModel { value: 0 })
            .message(TestMsg::Increment)
            .trigger_ticks()
            .expect_message(TestMsg::TimerFired)
            .run();
        
        assert!(result.is_ok());
        let harness = result.unwrap();
        assert_eq!(harness.model().value, 2);
    }
}