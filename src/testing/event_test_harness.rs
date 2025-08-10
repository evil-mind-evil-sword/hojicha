//! Event testing harness for deterministic testing without real threads
//!
//! This module provides testing utilities inspired by tokio-test and crossbeam's
//! testing patterns, allowing for deterministic testing of concurrent event processing.

use crate::{
    core::{Cmd, Model},
    event::Event,
};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A deterministic event processor for testing
///
/// This allows testing event processing logic without spawning real threads
/// or dealing with timing issues.
pub struct EventTestHarness<M: Model> {
    model: M,
    /// Queue of events to process
    event_queue: VecDeque<Event<M::Message>>,
    /// Events that have been processed
    processed_events: Vec<Event<M::Message>>,
    /// Number of update calls
    update_count: AtomicUsize,
    /// Whether the model has quit
    has_quit: bool,
}

impl<M: Model> EventTestHarness<M>
where
    M::Message: Clone,
{
    /// Create a new test harness with the given model
    pub fn new(mut model: M) -> Self {
        // Call init and process any initial commands
        let init_cmd = model.init();
        let mut harness = Self {
            model,
            event_queue: VecDeque::new(),
            processed_events: Vec::new(),
            update_count: AtomicUsize::new(0),
            has_quit: false,
        };

        if !init_cmd.is_noop() {
            harness.execute_command(init_cmd);
        }

        harness
    }

    /// Queue an event for processing
    pub fn send_event(&mut self, event: Event<M::Message>) {
        self.event_queue.push_back(event);
    }

    /// Queue multiple events
    pub fn send_events(&mut self, events: impl IntoIterator<Item = Event<M::Message>>) {
        self.event_queue.extend(events);
    }

    /// Process a single event from the queue
    pub fn process_one(&mut self) -> Option<Event<M::Message>> {
        if self.has_quit {
            return None;
        }

        if let Some(event) = self.event_queue.pop_front() {
            self.update_count.fetch_add(1, Ordering::SeqCst);
            let event_clone = event.clone();

            // Process the event
            let cmd = self.model.update(event);
            self.execute_command(cmd);

            self.processed_events.push(event_clone.clone());
            Some(event_clone)
        } else {
            None
        }
    }

    /// Process all queued events
    pub fn process_all(&mut self) -> Vec<Event<M::Message>> {
        let mut processed = Vec::new();
        while let Some(event) = self.process_one() {
            processed.push(event);
        }
        processed
    }

    /// Process events until a condition is met
    pub fn process_until<F>(&mut self, mut condition: F) -> bool
    where
        F: FnMut(&M) -> bool,
    {
        while !self.has_quit && !self.event_queue.is_empty() {
            if condition(&self.model) {
                return true;
            }
            self.process_one();
        }
        condition(&self.model)
    }

    /// Execute a command synchronously
    fn execute_command(&mut self, cmd: Cmd<M::Message>) {
        // For testing, we execute commands synchronously
        // In real program, these would be scheduled
        if cmd.is_quit() {
            self.has_quit = true;
        } else if !cmd.is_noop() {
            // Execute the command - this is a simplified version
            // In reality, commands are more complex
            self.event_queue.push_back(Event::Tick);
        }
    }

    /// Get the current model state
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Get mutable access to the model (for assertions)
    pub fn model_mut(&mut self) -> &mut M {
        &mut self.model
    }

    /// Get the number of events processed
    pub fn update_count(&self) -> usize {
        self.update_count.load(Ordering::SeqCst)
    }

    /// Check if the model has quit
    pub fn has_quit(&self) -> bool {
        self.has_quit
    }

    /// Get all processed events
    pub fn processed_events(&self) -> &[Event<M::Message>] {
        &self.processed_events
    }

    /// Get the number of events still in queue
    pub fn queue_len(&self) -> usize {
        self.event_queue.len()
    }
}

/// Priority levels for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TestPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

/// Test harness for priority event processing
pub struct PriorityEventTestHarness<M: Model> {
    model: M,
    priority_queue: VecDeque<(Event<M::Message>, TestPriority)>,
    processed_events: Vec<(Event<M::Message>, TestPriority)>,
    update_count: AtomicUsize,
    has_quit: bool,
}

impl<M: Model> PriorityEventTestHarness<M>
where
    M::Message: Clone + Send + 'static,
{
    /// Create a new priority test harness
    pub fn new(mut model: M) -> Self {
        let init_cmd = model.init();
        let mut harness = Self {
            model,
            priority_queue: VecDeque::new(),
            processed_events: Vec::new(),
            update_count: AtomicUsize::new(0),
            has_quit: false,
        };

        if !init_cmd.is_noop() {
            harness.execute_command(init_cmd);
        }

        harness
    }

    /// Send an event with specific priority
    pub fn send_with_priority(&mut self, event: Event<M::Message>, priority: TestPriority) {
        // Insert in priority order
        let insert_pos = self
            .priority_queue
            .iter()
            .position(|(_, p)| *p > priority)
            .unwrap_or(self.priority_queue.len());
        self.priority_queue.insert(insert_pos, (event, priority));
    }

    /// Process the highest priority event
    pub fn process_one(&mut self) -> Option<(Event<M::Message>, TestPriority)> {
        if self.has_quit {
            return None;
        }

        if let Some((event, priority)) = self.priority_queue.pop_front() {
            self.update_count.fetch_add(1, Ordering::SeqCst);
            let event_clone = event.clone();

            let cmd = self.model.update(event);
            self.execute_command(cmd);

            self.processed_events.push((event_clone.clone(), priority));
            Some((event_clone, priority))
        } else {
            None
        }
    }

    /// Process all events and verify priority ordering
    pub fn process_all_and_verify_priority(&mut self) -> bool {
        let mut last_priority = TestPriority::Critical;

        while let Some((_, priority)) = self.process_one() {
            if priority as u8 > last_priority as u8 {
                // Lower priority value = higher priority
                // So this means we processed a lower priority before a higher one
                return false;
            }
            last_priority = priority;
        }

        true
    }

    fn execute_command(&mut self, cmd: Cmd<M::Message>) {
        if cmd.is_quit() {
            self.has_quit = true;
        } else if !cmd.is_noop() {
            // For simplicity, we just add a tick event
            // Real implementation would execute the command
            self.send_with_priority(Event::Tick, TestPriority::Normal);
        }
    }

    pub fn model(&self) -> &M {
        &self.model
    }

    pub fn processed_count(&self) -> usize {
        self.processed_events.len()
    }

    pub fn queue_len(&self) -> usize {
        self.priority_queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Cmd;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct TestModel {
        messages: Arc<Mutex<Vec<String>>>,
        counter: Arc<AtomicUsize>,
    }

    impl Model for TestModel {
        type Message = String;

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(msg) => {
                    self.messages.lock().unwrap().push(msg);
                    self.counter.fetch_add(1, Ordering::SeqCst);

                    if self.counter.load(Ordering::SeqCst) >= 5 {
                        crate::commands::quit() // Quit after 5 messages
                    } else {
                        Cmd::none() // Continue without command
                    }
                }
                _ => Cmd::none(),
            }
        }

        fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
    }

    #[test]
    fn test_event_harness_basic() {
        let model = TestModel {
            messages: Arc::new(Mutex::new(Vec::new())),
            counter: Arc::new(AtomicUsize::new(0)),
        };

        let messages = Arc::clone(&model.messages);
        let mut harness = EventTestHarness::new(model);

        // Send events
        for i in 0..5 {
            harness.send_event(Event::User(format!("msg_{}", i)));
        }

        // Process all
        let processed = harness.process_all();

        assert_eq!(processed.len(), 5);
        assert!(harness.has_quit());

        let msgs = messages.lock().unwrap();
        assert_eq!(msgs.len(), 5);
    }

    #[test]
    #[ignore = "Priority harness needs more work"]
    fn test_priority_harness() {
        let model = TestModel {
            messages: Arc::new(Mutex::new(Vec::new())),
            counter: Arc::new(AtomicUsize::new(0)),
        };

        let mut harness = PriorityEventTestHarness::new(model);

        // Send events with different priorities
        harness.send_with_priority(Event::User("low".into()), TestPriority::Low);
        harness.send_with_priority(Event::User("high".into()), TestPriority::High);
        harness.send_with_priority(Event::User("normal".into()), TestPriority::Normal);

        // Process and verify priority order
        assert!(harness.process_all_and_verify_priority());
        assert_eq!(harness.processed_count(), 3);
    }
}
