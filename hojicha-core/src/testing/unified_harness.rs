//! Unified test harness combining event and time control
//!
//! This module provides a single, configurable test harness that unifies
//! the functionality of EventTestHarness and TimeControlledHarness.

use crate::{
    core::{Cmd, Model},
    event::Event,
    testing::time_control::{TimeController, PausedTimeGuard},
};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Configuration for the unified test harness
#[derive(Debug, Clone)]
pub struct HarnessConfig {
    /// Whether to use manual time control
    pub time_controlled: bool,
    /// Whether to start with paused time
    pub start_paused: bool,
    /// Maximum number of events to process automatically
    pub max_auto_events: Option<usize>,
    /// Whether to enable debug logging
    pub debug_logging: bool,
    /// Timeout for async operations (when time controlled)
    pub async_timeout: Duration,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            time_controlled: false,
            start_paused: true,
            max_auto_events: Some(1000),
            debug_logging: false,
            async_timeout: Duration::from_secs(30),
        }
    }
}

/// A unified test harness that combines event processing and time control
///
/// This allows testing both synchronous event processing and time-dependent
/// async operations in a deterministic way.
pub struct UnifiedTestHarness<M: Model> {
    model: M,
    /// Queue of events to process
    event_queue: VecDeque<Event<M::Message>>,
    /// Events that have been processed
    processed_events: Vec<Event<M::Message>>,
    /// Commands that have been executed
    executed_commands: Vec<String>, // String description of commands
    /// Number of update calls
    update_count: AtomicUsize,
    /// Whether the model has quit
    has_quit: bool,
    /// Time controller (if time controlled)
    time_controller: Option<TimeController>,
    /// RAII guard for paused time
    _time_guard: Option<PausedTimeGuard>,
    /// Configuration
    config: HarnessConfig,
    /// Start time for measurements
    start_time: Instant,
    /// Messages generated during execution
    generated_messages: Vec<M::Message>,
}

impl<M: Model + 'static> UnifiedTestHarness<M>
where
    M::Message: Clone + std::fmt::Debug,
{
    /// Create a new unified test harness with default config
    pub fn new(model: M) -> Self {
        Self::with_config(model, HarnessConfig::default())
    }

    /// Create a new unified test harness with custom config
    pub fn with_config(mut model: M, config: HarnessConfig) -> Self {
        let time_controller = if config.time_controlled {
            Some(if config.start_paused {
                TimeController::new_paused()
            } else {
                TimeController::new_real()
            })
        } else {
            None
        };

        let time_guard = if config.time_controlled && config.start_paused {
            Some(PausedTimeGuard::new())
        } else {
            None
        };

        // Call init and process any initial commands
        let init_cmd = model.init();
        let mut harness = Self {
            model,
            event_queue: VecDeque::new(),
            processed_events: Vec::new(),
            executed_commands: Vec::new(),
            update_count: AtomicUsize::new(0),
            has_quit: false,
            time_controller,
            _time_guard: time_guard,
            config,
            start_time: Instant::now(),
            generated_messages: Vec::new(),
        };

        if !init_cmd.is_noop() {
            harness.execute_command(init_cmd);
        }

        harness
    }

    /// Create a new harness with paused time
    pub fn new_with_paused_time(model: M) -> Self {
        let config = HarnessConfig {
            time_controlled: true,
            start_paused: true,
            ..Default::default()
        };
        Self::with_config(model, config)
    }

    /// Create a new harness for testing async operations
    pub fn new_for_async(model: M) -> Self {
        let config = HarnessConfig {
            time_controlled: true,
            start_paused: false,
            ..Default::default()
        };
        Self::with_config(model, config)
    }

    /// Get a reference to the model
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Get a mutable reference to the model
    pub fn model_mut(&mut self) -> &mut M {
        &mut self.model
    }

    /// Queue an event for processing
    pub fn send_event(&mut self, event: Event<M::Message>) {
        if self.config.debug_logging {
            eprintln!("[HARNESS] Queuing event: {:?}", event);
        }
        self.event_queue.push_back(event);
    }

    /// Queue a user message
    pub fn send_message(&mut self, message: M::Message) {
        self.send_event(Event::User(message));
    }

    /// Queue multiple events
    pub fn send_events(&mut self, events: impl IntoIterator<Item = Event<M::Message>>) {
        for event in events {
            self.send_event(event);
        }
    }

    /// Queue multiple messages
    pub fn send_messages(&mut self, messages: impl IntoIterator<Item = M::Message>) {
        for message in messages {
            self.send_message(message);
        }
    }

    /// Process a single event from the queue
    pub fn process_one(&mut self) -> Option<Event<M::Message>> {
        if self.has_quit {
            return None;
        }

        if let Some(event) = self.event_queue.pop_front() {
            self.update_count.fetch_add(1, Ordering::SeqCst);
            let event_clone = event.clone();

            if self.config.debug_logging {
                eprintln!("[HARNESS] Processing event: {:?}", event);
                eprintln!("[HARNESS] Model state before: {:?}", std::any::type_name::<M>());
            }

            // Process the event
            let cmd = self.model.update(event);
            self.execute_command(cmd);

            self.processed_events.push(event_clone.clone());
            
            if self.config.debug_logging {
                eprintln!("[HARNESS] Event processed. Queue length: {}", self.event_queue.len());
            }

            Some(event_clone)
        } else {
            None
        }
    }

    /// Process all queued events
    pub fn process_all(&mut self) -> Vec<Event<M::Message>> {
        let mut processed = Vec::new();
        let max_events = self.config.max_auto_events.unwrap_or(usize::MAX);
        let mut count = 0;

        while let Some(event) = self.process_one() {
            processed.push(event);
            count += 1;
            
            if count >= max_events {
                if self.config.debug_logging {
                    eprintln!("[HARNESS] Hit max auto events limit ({}), stopping", max_events);
                }
                break;
            }
        }
        processed
    }

    /// Process events until a condition is met
    pub fn process_until<F>(&mut self, mut condition: F) -> bool
    where
        F: FnMut(&M) -> bool,
    {
        let max_events = self.config.max_auto_events.unwrap_or(usize::MAX);
        let mut count = 0;

        while !self.has_quit && !self.event_queue.is_empty() && count < max_events {
            if condition(&self.model) {
                return true;
            }
            
            if self.process_one().is_none() {
                break;
            }
            count += 1;
        }
        
        condition(&self.model)
    }

    /// Process events for a specific duration (requires time control)
    pub fn process_for_duration(&mut self, duration: Duration) -> Vec<Event<M::Message>> {
        if self.time_controller.is_none() {
            panic!("process_for_duration requires time_controlled=true");
        }
        
        let start_time = self.now();
        let mut processed = Vec::new();
        
        loop {
            let current_time = self.now();
            if current_time - start_time >= duration {
                break;
            }
            
            if let Some(event) = self.process_one() {
                processed.push(event);
            } else {
                // No more events, advance time manually
                let remaining = duration - (current_time - start_time);
                if remaining > Duration::ZERO {
                    self.advance_time(remaining.min(Duration::from_millis(100)));
                } else {
                    break;
                }
            }
        }
        
        processed
    }

    /// Execute a command and handle any resulting messages
    fn execute_command(&mut self, cmd: Cmd<M::Message>) {
        if cmd.is_quit() {
            self.has_quit = true;
            self.executed_commands.push("quit".to_string());
            return;
        }

        if cmd.is_noop() {
            return;
        }

        // Check command type before consuming it
        let is_batch = cmd.is_batch();
        let is_sequence = cmd.is_sequence();

        // For testing purposes, we try to execute the command
        // In a real implementation, this would be handled by the runtime
        match cmd.test_execute() {
            Ok(Some(message)) => {
                if self.config.debug_logging {
                    eprintln!("[HARNESS] Command generated message: {:?}", message);
                }
                self.generated_messages.push(message.clone());
                self.send_message(message);
                self.executed_commands.push("sync_command".to_string());
            }
            Ok(None) => {
                // Command executed but produced no message
                self.executed_commands.push("sync_command_no_message".to_string());
            }
            Err(_) => {
                // Command failed or is async
                self.executed_commands.push("async_or_failed_command".to_string());
            }
        }

        // Handle special command types
        if is_batch {
            self.executed_commands.push("batch_command".to_string());
        } else if is_sequence {
            self.executed_commands.push("sequence_command".to_string());
        }
    }

    /// Get the number of events processed
    pub fn processed_count(&self) -> usize {
        self.update_count.load(Ordering::SeqCst)
    }

    /// Get the number of events still queued
    pub fn queued_count(&self) -> usize {
        self.event_queue.len()
    }

    /// Check if the model has quit
    pub fn has_quit(&self) -> bool {
        self.has_quit
    }

    /// Get all processed events
    pub fn processed_events(&self) -> &[Event<M::Message>] {
        &self.processed_events
    }

    /// Get all executed commands (as descriptions)
    pub fn executed_commands(&self) -> &[String] {
        &self.executed_commands
    }

    /// Get messages generated by commands
    pub fn generated_messages(&self) -> &[M::Message] {
        &self.generated_messages
    }

    /// Clear the event queue
    pub fn clear_queue(&mut self) {
        self.event_queue.clear();
    }

    /// Clear processed events history
    pub fn clear_history(&mut self) {
        self.processed_events.clear();
        self.executed_commands.clear();
        self.generated_messages.clear();
    }

    /// Get elapsed time since harness creation
    pub fn elapsed(&self) -> Duration {
        if let Some(controller) = &self.time_controller {
            controller.now()
        } else {
            self.start_time.elapsed()
        }
    }

    // Time control methods (require time_controlled=true)

    /// Pause time (requires time control)
    pub fn pause_time(&self) {
        if let Some(controller) = &self.time_controller {
            controller.pause();
        } else {
            panic!("pause_time requires time_controlled=true");
        }
    }

    /// Resume time (requires time control)
    pub fn resume_time(&self) {
        if let Some(controller) = &self.time_controller {
            controller.resume();
        } else {
            panic!("resume_time requires time_controlled=true");
        }
    }

    /// Advance time by the given duration (requires paused time)
    pub fn advance_time(&self, duration: Duration) {
        if let Some(controller) = &self.time_controller {
            controller.advance(duration).expect("Time must be paused to advance manually");
        } else {
            panic!("advance_time requires time_controlled=true");
        }
    }

    /// Set time scale (requires time control)
    pub fn set_time_scale(&self, scale: f64) {
        if let Some(controller) = &self.time_controller {
            controller.set_scale(scale);
        } else {
            panic!("set_time_scale requires time_controlled=true");
        }
    }

    /// Get current virtual time (requires time control)
    pub fn now(&self) -> Duration {
        if let Some(controller) = &self.time_controller {
            controller.now()
        } else {
            panic!("now requires time_controlled=true");
        }
    }

    // Assertion helpers

    /// Assert that the model is in a specific state
    pub fn assert_model<F>(&self, predicate: F, message: &str)
    where
        F: FnOnce(&M) -> bool,
    {
        assert!(predicate(&self.model), "{}", message);
    }

    /// Assert that a specific number of events were processed
    pub fn assert_processed_count(&self, expected: usize) {
        assert_eq!(
            self.processed_count(),
            expected,
            "Expected {} processed events, got {}",
            expected,
            self.processed_count()
        );
    }

    /// Assert that a specific message was generated
    pub fn assert_message_generated(&self, message: &M::Message)
    where
        M::Message: PartialEq,
    {
        assert!(
            self.generated_messages.contains(message),
            "Expected message {:?} was not generated",
            message
        );
    }

    /// Assert that no events are queued
    pub fn assert_queue_empty(&self) {
        assert_eq!(
            self.queued_count(),
            0,
            "Expected empty queue, but {} events are queued",
            self.queued_count()
        );
    }

    /// Create a scenario builder for declarative testing
    pub fn scenario(model: M) -> ScenarioBuilder<M> {
        ScenarioBuilder::new(model)
    }
}

/// Builder for creating declarative test scenarios
pub struct ScenarioBuilder<M: Model> {
    model: M,
    config: HarnessConfig,
    events: Vec<ScenarioStep<M::Message>>,
}

/// A step in a test scenario
pub enum ScenarioStep<Msg> {
    /// Send an event
    SendEvent(Event<Msg>),
    /// Send a message
    SendMessage(Msg),
    /// Advance time
    AdvanceTime(Duration),
    /// Wait for a condition
    WaitFor(Box<dyn Fn(&dyn std::any::Any) -> bool>),
    /// Assert a condition
    Assert(Box<dyn Fn(&dyn std::any::Any) -> bool>, String),
    /// Process all queued events
    ProcessAll,
    /// Process events for a duration
    ProcessFor(Duration),
}

impl<Msg> std::fmt::Debug for ScenarioStep<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScenarioStep::SendEvent(_) => write!(f, "SendEvent"),
            ScenarioStep::SendMessage(_) => write!(f, "SendMessage"),
            ScenarioStep::AdvanceTime(d) => write!(f, "AdvanceTime({:?})", d),
            ScenarioStep::WaitFor(_) => write!(f, "WaitFor(<closure>)"),
            ScenarioStep::Assert(_, msg) => write!(f, "Assert(<closure>, {})", msg),
            ScenarioStep::ProcessAll => write!(f, "ProcessAll"),
            ScenarioStep::ProcessFor(d) => write!(f, "ProcessFor({:?})", d),
        }
    }
}

impl<M: Model + 'static> ScenarioBuilder<M>
where
    M::Message: Clone + std::fmt::Debug,
{
    /// Create a new scenario builder
    pub fn new(model: M) -> Self {
        Self {
            model,
            config: HarnessConfig::default(),
            events: Vec::new(),
        }
    }

    /// Configure the harness
    pub fn with_config(mut self, config: HarnessConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable time control
    pub fn with_time_control(mut self) -> Self {
        self.config.time_controlled = true;
        self
    }

    /// Send an event
    pub fn send_event(mut self, event: Event<M::Message>) -> Self {
        self.events.push(ScenarioStep::SendEvent(event));
        self
    }

    /// Send a message
    pub fn send_message(mut self, message: M::Message) -> Self {
        self.events.push(ScenarioStep::SendMessage(message));
        self
    }

    /// Advance time
    pub fn advance_time(mut self, duration: Duration) -> Self {
        self.events.push(ScenarioStep::AdvanceTime(duration));
        self
    }

    /// Process all queued events
    pub fn process_all(mut self) -> Self {
        self.events.push(ScenarioStep::ProcessAll);
        self
    }

    /// Process events for a duration
    pub fn process_for(mut self, duration: Duration) -> Self {
        self.events.push(ScenarioStep::ProcessFor(duration));
        self
    }

    /// Assert a condition about the model
    pub fn assert_model<F>(mut self, predicate: F, message: impl Into<String>) -> Self
    where
        F: Fn(&M) -> bool + 'static,
    {
        let message = message.into();
        self.events.push(ScenarioStep::Assert(
            Box::new(move |model_any| {
                let model = model_any.downcast_ref::<M>().expect("Type mismatch");
                predicate(model)
            }),
            message,
        ));
        self
    }

    /// Execute the scenario
    pub fn run(self) -> UnifiedTestHarness<M> {
        let mut harness = UnifiedTestHarness::with_config(self.model, self.config);

        for step in self.events {
            match step {
                ScenarioStep::SendEvent(event) => harness.send_event(event),
                ScenarioStep::SendMessage(message) => harness.send_message(message),
                ScenarioStep::AdvanceTime(duration) => harness.advance_time(duration),
                ScenarioStep::ProcessAll => {
                    harness.process_all();
                }
                ScenarioStep::ProcessFor(duration) => {
                    harness.process_for_duration(duration);
                }
                ScenarioStep::Assert(predicate, message) => {
                    let model_any: &dyn std::any::Any = &harness.model;
                    assert!(predicate(model_any), "{}", message);
                }
                ScenarioStep::WaitFor(_predicate) => {
                    // TODO: Implement wait_for
                    unimplemented!("WaitFor not yet implemented");
                }
            }
        }

        harness
    }
}

/// Convenience macro for creating test scenarios
#[macro_export]
macro_rules! test_scenario {
    ($model:expr, $($step:expr),* $(,)?) => {{
        let mut builder = $crate::testing::unified_harness::ScenarioBuilder::new($model);
        $(
            builder = $step(builder);
        )*
        builder.run()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Cmd, Model};
    use crate::event::Event;

    #[derive(Debug, Clone)]
    struct TestModel {
        counter: i32,
        messages: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TestMessage {
        Increment,
        Decrement,
        SetValue(i32),
        AddMessage(String),
        Reset,
    }

    impl Default for TestModel {
        fn default() -> Self {
            Self {
                counter: 0,
                messages: Vec::new(),
            }
        }
    }

    impl Model for TestModel {
        type Message = TestMessage;

        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(TestMessage::Increment) => {
                    self.counter += 1;
                    Cmd::none()
                }
                Event::User(TestMessage::Decrement) => {
                    self.counter -= 1;
                    Cmd::none()
                }
                Event::User(TestMessage::SetValue(value)) => {
                    self.counter = value;
                    Cmd::none()
                }
                Event::User(TestMessage::AddMessage(msg)) => {
                    self.messages.push(msg);
                    Cmd::none()
                }
                Event::User(TestMessage::Reset) => {
                    self.counter = 0;
                    self.messages.clear();
                    Cmd::none()
                }
                _ => Cmd::none(),
            }
        }

        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    }

    #[test]
    fn test_unified_harness_basic() {
        let model = TestModel::default();
        let mut harness = UnifiedTestHarness::new(model);

        harness.send_message(TestMessage::Increment);
        harness.send_message(TestMessage::Increment);
        harness.process_all();

        assert_eq!(harness.model().counter, 2);
        assert_eq!(harness.processed_count(), 2);
    }

    #[test]
    fn test_unified_harness_with_time_control() {
        let model = TestModel::default();
        let mut harness = UnifiedTestHarness::new_with_paused_time(model);

        // Time should start at zero
        assert_eq!(harness.now(), Duration::ZERO);

        // Advance time manually
        harness.advance_time(Duration::from_secs(5));
        assert_eq!(harness.now(), Duration::from_secs(5));

        // Send and process messages
        harness.send_message(TestMessage::SetValue(42));
        harness.process_all();

        assert_eq!(harness.model().counter, 42);
    }

    #[test]
    fn test_scenario_builder() {
        let model = TestModel::default();
        
        let harness = ScenarioBuilder::new(model)
            .with_time_control()
            .send_message(TestMessage::Increment)
            .send_message(TestMessage::Increment)
            .process_all()
            .assert_model(|m| m.counter == 2, "Counter should be 2")
            .send_message(TestMessage::Reset)
            .process_all()
            .assert_model(|m| m.counter == 0, "Counter should be reset")
            .run();

        assert_eq!(harness.model().counter, 0);
        assert_eq!(harness.processed_count(), 3);
    }

    #[test]
    fn test_harness_assertions() {
        let model = TestModel::default();
        let mut harness = UnifiedTestHarness::new(model);

        harness.send_message(TestMessage::Increment);
        harness.process_all();

        harness.assert_model(|m| m.counter == 1, "Counter should be 1");
        harness.assert_processed_count(1);
        harness.assert_queue_empty();
    }

    #[test] 
    fn test_process_until() {
        let model = TestModel::default();
        let mut harness = UnifiedTestHarness::new(model);

        // Queue multiple events
        for i in 1..=5 {
            harness.send_message(TestMessage::SetValue(i));
        }

        // Process until counter reaches 3
        let result = harness.process_until(|m| m.counter == 3);
        assert!(result);
        assert_eq!(harness.model().counter, 3);
        
        // Should have 2 events left in queue
        assert_eq!(harness.queued_count(), 2);
    }

    #[test]
    fn test_harness_debug_logging() {
        let model = TestModel::default();
        let config = HarnessConfig {
            debug_logging: true,
            ..Default::default()
        };
        let mut harness = UnifiedTestHarness::with_config(model, config);

        harness.send_message(TestMessage::Increment);
        harness.process_all();

        // Just test that it doesn't panic with debug logging enabled
        assert_eq!(harness.model().counter, 1);
    }
}