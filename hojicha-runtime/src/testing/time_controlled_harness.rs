//! Time-controlled test harness for deterministic testing of async and timing operations

use crate::program::{CommandExecutor, Program, ProgramOptions};
use hojicha_core::core::{Cmd, Model};
use hojicha_core::event::Event;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

/// A test harness with full control over time advancement
pub struct TimeControlledHarness<M: Model> {
    model: M,
    runtime: Runtime,
    executor: CommandExecutor<M::Message>,
    message_queue: Arc<Mutex<VecDeque<M::Message>>>,
    current_time: RefCell<Instant>,
    is_paused: RefCell<bool>,
    event_log: RefCell<Vec<Event<M::Message>>>,
}

impl<M> TimeControlledHarness<M>
where
    M: Model + Clone,
    M::Message: Clone + Send + 'static,
{
    /// Create a new time-controlled test harness
    pub fn new(model: M) -> Self {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("Failed to create runtime");
        
        let executor = CommandExecutor::new().expect("Failed to create executor");
        
        Self {
            model,
            runtime,
            executor,
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
            current_time: RefCell::new(Instant::now()),
            is_paused: RefCell::new(false),
            event_log: RefCell::new(Vec::new()),
        }
    }
    
    /// Pause time advancement (for documentation purposes)
    pub fn pause_time(&self) {
        *self.is_paused.borrow_mut() = true;
    }
    
    /// Resume time advancement (for documentation purposes)
    pub fn resume_time(&self) {
        *self.is_paused.borrow_mut() = false;
    }
    
    /// Advance time by the specified duration
    pub fn advance_time(&mut self, duration: Duration) {
        if !*self.is_paused.borrow() {
            self.pause_time();
        }
        
        *self.current_time.borrow_mut() += duration;
        
        // Simulate time passing by processing any pending timer-based messages
        // In a real implementation, we'd integrate with a mock timer
        self.process_pending_messages();
    }
    
    /// Set the current time to a specific instant
    pub fn set_time(&mut self, instant: Instant) {
        let current = *self.current_time.borrow();
        if instant > current {
            let duration = instant - current;
            self.advance_time(duration);
        }
    }
    
    /// Send an event to the model
    pub fn send_event(&mut self, event: Event<M::Message>) {
        self.event_log.borrow_mut().push(event.clone());
        
        // Process the event through the model
        let cmd = self.model.update(event);
        
        if !cmd.is_noop() {
            self.execute_command(cmd);
        }
    }
    
    /// Send a user message
    pub fn send_message(&mut self, message: M::Message) {
        self.send_event(Event::User(message));
    }
    
    /// Execute a command with time control
    pub fn execute_command(&self, cmd: Cmd<M::Message>) {
        let (tx, rx) = mpsc::sync_channel(100);
        let queue = self.message_queue.clone();
        
        // Execute the command
        self.executor.execute(cmd, tx);
        
        // Collect messages in background
        std::thread::spawn(move || {
            while let Ok(Event::User(msg)) = rx.recv() {
                queue.lock().unwrap().push_back(msg);
            }
        });
    }
    
    /// Process any pending messages in the queue
    fn process_pending_messages(&mut self) {
        let messages: Vec<_> = {
            let mut queue = self.message_queue.lock().unwrap();
            queue.drain(..).collect()
        };
        
        for msg in messages {
            self.send_event(Event::User(msg));
        }
    }
    
    /// Assert that a specific message was received
    pub fn assert_received(&self, expected: M::Message) -> bool
    where
        M::Message: PartialEq + std::fmt::Debug,
    {
        self.event_log
            .borrow()
            .iter()
            .any(|event| matches!(event, Event::User(msg) if msg == &expected))
    }
    
    /// Assert that a message was NOT received
    pub fn assert_not_received(&self, unexpected: M::Message) -> bool
    where
        M::Message: PartialEq + std::fmt::Debug,
    {
        !self.assert_received(unexpected)
    }
    
    /// Assert on the current model state
    pub fn assert_state<F>(&self, predicate: F) -> bool
    where
        F: FnOnce(&M) -> bool,
    {
        predicate(&self.model)
    }
    
    /// Run until a condition is met
    pub fn run_until<F>(&mut self, condition: F, timeout: Duration) -> bool
    where
        F: Fn(&M) -> bool,
    {
        let start = Instant::now();
        
        while start.elapsed() < timeout {
            if condition(&self.model) {
                return true;
            }
            
            // Advance time in small increments
            self.advance_time(Duration::from_millis(10));
            self.process_pending_messages();
        }
        
        false
    }
    
    /// Run for a specific duration
    pub fn run_for(&mut self, duration: Duration) {
        let steps = duration.as_millis() / 10;
        
        for _ in 0..steps {
            self.advance_time(Duration::from_millis(10));
            self.process_pending_messages();
        }
    }
    
    /// Get all received messages
    pub fn received_messages(&self) -> Vec<M::Message> {
        self.event_log
            .borrow()
            .iter()
            .filter_map(|event| {
                if let Event::User(msg) = event {
                    Some(msg.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Clear the event log
    pub fn clear_log(&self) {
        self.event_log.borrow_mut().clear();
    }
    
    /// Get the current model
    pub fn model(&self) -> &M {
        &self.model
    }
    
    /// Get a mutable reference to the model
    pub fn model_mut(&mut self) -> &mut M {
        &mut self.model
    }
}

/// Builder for creating test scenarios
pub struct TestScenarioBuilder<M: Model> {
    harness: TimeControlledHarness<M>,
    steps: Vec<TestStep<M::Message>>,
}

#[derive(Clone)]
enum TestStep<M> {
    SendMessage(M),
    AdvanceTime(Duration),
    AssertReceived(M),
    WaitFor(Duration),
}

impl<M> TestScenarioBuilder<M>
where
    M: Model + Clone,
    M::Message: Clone + Send + PartialEq + std::fmt::Debug + 'static,
{
    /// Create a new test scenario
    pub fn new(model: M) -> Self {
        Self {
            harness: TimeControlledHarness::new(model),
            steps: Vec::new(),
        }
    }
    
    /// Add a step to send a message
    pub fn send(mut self, message: M::Message) -> Self {
        self.steps.push(TestStep::SendMessage(message));
        self
    }
    
    /// Add a step to advance time
    pub fn advance(mut self, duration: Duration) -> Self {
        self.steps.push(TestStep::AdvanceTime(duration));
        self
    }
    
    /// Add a step to assert a message was received
    pub fn expect(mut self, message: M::Message) -> Self {
        self.steps.push(TestStep::AssertReceived(message));
        self
    }
    
    /// Add a step to wait for a duration
    pub fn wait(mut self, duration: Duration) -> Self {
        self.steps.push(TestStep::WaitFor(duration));
        self
    }
    
    /// Run the test scenario
    pub fn run(mut self) -> Result<(), String> {
        self.harness.pause_time();
        
        for step in self.steps {
            match step {
                TestStep::SendMessage(msg) => {
                    self.harness.send_message(msg);
                }
                TestStep::AdvanceTime(duration) => {
                    self.harness.advance_time(duration);
                }
                TestStep::AssertReceived(expected) => {
                    if !self.harness.assert_received(expected.clone()) {
                        return Err(format!("Expected message {:?} was not received", expected));
                    }
                }
                TestStep::WaitFor(duration) => {
                    self.harness.run_for(duration);
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hojicha_core::commands;
    
    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {
        StartTimer,
        TimerFired,
        StartRecurring,
        Recurring(u32),
    }
    
    #[derive(Clone)]
    struct TestModel {
        counter: u32,
    }
    
    impl Model for TestModel {
        type Message = TestMsg;
        
        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }
        
        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(TestMsg::StartTimer) => {
                    commands::tick(Duration::from_secs(1), || TestMsg::TimerFired)
                }
                Event::User(TestMsg::StartRecurring) => {
                    let counter = self.counter;
                    self.counter += 1;
                    commands::every(Duration::from_secs(1), move |_| {
                        TestMsg::Recurring(counter)
                    })
                }
                Event::User(TestMsg::TimerFired) => {
                    self.counter += 1;
                    Cmd::none()
                }
                _ => Cmd::none(),
            }
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    }
    
    #[test]
    fn test_timer_behavior() {
        let mut harness = TimeControlledHarness::new(TestModel { counter: 0 });
        
        // Pause time for deterministic testing
        harness.pause_time();
        
        // Send event that triggers a tick command
        harness.send_message(TestMsg::StartTimer);
        
        // Timer shouldn't fire immediately
        assert!(!harness.assert_received(TestMsg::TimerFired));
        
        // Advance time explicitly
        harness.advance_time(Duration::from_secs(1));
        
        // Check that timer fired
        assert!(harness.assert_received(TestMsg::TimerFired));
        assert_eq!(harness.model().counter, 1);
    }
    
    #[test]
    fn test_scenario_builder() {
        let result = TestScenarioBuilder::new(TestModel { counter: 0 })
            .send(TestMsg::StartTimer)
            .advance(Duration::from_millis(500))
            .advance(Duration::from_millis(500))
            .expect(TestMsg::TimerFired)
            .run();
        
        assert!(result.is_ok());
    }
}