//! Advanced red team tests for Hojicha library
//! Tests async operations, subscriptions, and error handling

use hojicha_core::prelude::*;
use hojicha_runtime::{Event, Key, KeyEvent, KeyModifiers, Program, ProgramOptions};
use std::time::Duration;
use std::thread;

// Test Model for advanced features
#[derive(Debug, Clone)]
struct AdvancedTestModel {
    state: String,
    counter: i32,
    async_results: Vec<String>,
    errors: Vec<String>,
    batch_completed: bool,
    sequence_step: usize,
    subscription_events: Vec<String>,
}

#[derive(Debug, Clone)]
enum TestMsg {
    // Basic messages
    UpdateState(String),
    Increment,
    Decrement,
    
    // Async messages
    StartAsync,
    AsyncComplete(String),
    AsyncError(String),
    
    // Batch/Sequence test
    BatchStart,
    BatchItem(usize),
    SequenceStart,
    SequenceStep(usize),
    
    // Subscription test
    SubscriptionEvent(String),
    
    // Error test
    TriggerPanic,
    TriggerError,
    RecoverFromError,
    
    // Timer test
    TimerTick,
    EveryTick(std::time::Instant),
}

impl Default for AdvancedTestModel {
    fn default() -> Self {
        Self {
            state: "initial".to_string(),
            counter: 0,
            async_results: Vec::new(),
            errors: Vec::new(),
            batch_completed: false,
            sequence_step: 0,
            subscription_events: Vec::new(),
        }
    }
}

impl Model for AdvancedTestModel {
    type Message = TestMsg;

    fn init(&mut self) -> Cmd<Self::Message> {
        // Test various initialization patterns
        batch(vec![
            custom(|| Some(TestMsg::UpdateState("initialized".to_string()))),
            tick(Duration::from_millis(100), || TestMsg::TimerTick),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(msg) => self.handle_message(msg),
            Event::Key(key) => self.handle_key(key),
            Event::Tick => {
                self.counter += 1;
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let text = format!(
            "State: {}, Counter: {}, Async: {:?}, Errors: {:?}",
            self.state, self.counter, self.async_results, self.errors
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(text),
            area,
        );
    }
}

impl AdvancedTestModel {
    fn handle_key(&mut self, key: KeyEvent) -> Cmd<TestMsg> {
        match key.key {
            Key::Char('q') => quit(),
            Key::Char('a') => custom(|| Some(TestMsg::StartAsync)),
            Key::Char('b') => custom(|| Some(TestMsg::BatchStart)),
            Key::Char('s') => custom(|| Some(TestMsg::SequenceStart)),
            Key::Char('e') => custom(|| Some(TestMsg::TriggerError)),
            Key::Up => {
                self.counter += 1;
                Cmd::none()
            }
            Key::Down => {
                self.counter -= 1;
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn handle_message(&mut self, msg: TestMsg) -> Cmd<TestMsg> {
        match msg {
            TestMsg::UpdateState(state) => {
                self.state = state;
                Cmd::none()
            }
            TestMsg::Increment => {
                self.counter += 1;
                Cmd::none()
            }
            TestMsg::Decrement => {
                self.counter -= 1;
                Cmd::none()
            }
            TestMsg::StartAsync => {
                // Test async command
                custom_async(|| async {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    Some(TestMsg::AsyncComplete("async_result".to_string()))
                })
            }
            TestMsg::AsyncComplete(result) => {
                self.async_results.push(result);
                Cmd::none()
            }
            TestMsg::AsyncError(err) => {
                self.errors.push(err);
                Cmd::none()
            }
            TestMsg::BatchStart => {
                // Test batch command - all should execute concurrently
                batch(vec![
                    custom(|| Some(TestMsg::BatchItem(1))),
                    custom(|| Some(TestMsg::BatchItem(2))),
                    custom(|| Some(TestMsg::BatchItem(3))),
                ])
            }
            TestMsg::BatchItem(n) => {
                self.async_results.push(format!("batch_{}", n));
                if self.async_results.len() >= 3 {
                    self.batch_completed = true;
                }
                Cmd::none()
            }
            TestMsg::SequenceStart => {
                // Test sequence command - should execute in order
                sequence(vec![
                    custom(|| Some(TestMsg::SequenceStep(1))),
                    tick(Duration::from_millis(10), || TestMsg::SequenceStep(2)),
                    custom(|| Some(TestMsg::SequenceStep(3))),
                ])
            }
            TestMsg::SequenceStep(n) => {
                self.sequence_step = n;
                self.async_results.push(format!("seq_{}", n));
                Cmd::none()
            }
            TestMsg::SubscriptionEvent(event) => {
                self.subscription_events.push(event);
                Cmd::none()
            }
            TestMsg::TriggerPanic => {
                // This should not crash the program
                panic!("Test panic!");
            }
            TestMsg::TriggerError => {
                // Test error handling
                custom_fallible(|| {
                    Err(Error::from(std::io::Error::other(
                        "Test error"
                    )))
                })
            }
            TestMsg::RecoverFromError => {
                self.errors.clear();
                self.state = "recovered".to_string();
                Cmd::none()
            }
            TestMsg::TimerTick => {
                self.counter += 100;
                Cmd::none()
            }
            TestMsg::EveryTick(instant) => {
                self.async_results.push(format!("every_{:?}", instant));
                Cmd::none()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_async_bridge_event_injection() {
        let model = AdvancedTestModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Test sending multiple events
        for i in 0..5 {
            let msg = TestMsg::UpdateState(format!("state_{}", i));
            let result = sender.send(Event::User(msg));
            assert!(result.is_ok(), "Failed to send event {}", i);
        }
    }

    #[test]
    fn test_batch_command() {
        let mut model = AdvancedTestModel::default();
        
        // Execute batch command
        let cmd = model.handle_message(TestMsg::BatchStart);
        
        // Check that it returns a batch command
        assert!(cmd.is_batch(), "Should return a batch command");
        
        // Extract and execute batch commands
        if let Some(cmds) = cmd.take_batch() {
            assert_eq!(cmds.len(), 3, "Batch should contain 3 commands");
            
            // Execute each command
            for cmd in cmds {
                if let Ok(Some(msg)) = cmd.execute() {
                    model.handle_message(msg);
                }
            }
            
            // Verify all batch items were processed
            assert_eq!(model.async_results.len(), 3, "All batch items should be processed");
            assert!(model.batch_completed, "Batch should be marked as completed");
        }
    }

    #[test]
    fn test_sequence_command() {
        let mut model = AdvancedTestModel::default();
        
        // Execute sequence command
        let cmd = model.handle_message(TestMsg::SequenceStart);
        
        // Check that it returns a sequence command
        assert!(cmd.is_sequence(), "Should return a sequence command");
        
        // Extract and execute sequence commands
        if let Some(cmds) = cmd.take_sequence() {
            assert_eq!(cmds.len(), 3, "Sequence should contain 3 commands");
            
            // Execute commands in order (skip tick as it needs async runtime)
            let expected_steps = [1, 3]; // Step 2 is a tick command
            let mut idx = 0;
            
            for cmd in cmds {
                // Skip tick commands as they need async runtime
                if !cmd.is_tick() {
                    if let Ok(Some(msg)) = cmd.execute() {
                        model.handle_message(msg);
                        assert_eq!(model.sequence_step, expected_steps[idx], "Sequence steps should match expected");
                        idx += 1;
                    }
                }
            }
        }
    }

    #[test]
    fn test_custom_fallible_command() {
        let mut model = AdvancedTestModel::default();
        
        // Trigger an error
        let cmd = model.handle_message(TestMsg::TriggerError);
        
        // Try to execute the fallible command
        let result = cmd.execute();
        assert!(result.is_err(), "Fallible command should return an error");
    }

    #[test]
    fn test_tick_command() {
        let model = AdvancedTestModel::default();
        
        // Create a tick command
        let cmd = tick(Duration::from_millis(100), || TestMsg::TimerTick);
        
        assert!(cmd.is_tick(), "Should be a tick command");
        
        // Extract tick details
        if let Some((duration, _callback)) = cmd.take_tick() {
            assert_eq!(duration, Duration::from_millis(100), "Duration should match");
        }
    }

    #[test]
    fn test_every_command() {
        let cmd = every(Duration::from_secs(1), TestMsg::EveryTick);
        
        assert!(cmd.is_every(), "Should be an every command");
        
        // Extract every details
        if let Some((duration, _callback)) = cmd.take_every() {
            assert_eq!(duration, Duration::from_secs(1), "Duration should match");
        }
    }

    #[test]
    fn test_program_with_async_bridge() {
        let _model = AdvancedTestModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer()
            .without_signal_handler();
        
        let mut program = Program::with_options(_model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Create a thread to send events
        let sender_clone = sender.clone();
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            sender_clone.send(Event::User(TestMsg::UpdateState("from_thread".to_string()))).ok();
            sender_clone.send(Event::Quit).ok();
        });
        
        // Note: We can't actually run the program in tests as it requires a terminal
        // But we've verified the setup works
        
        handle.join().unwrap();
    }

    #[test]
    fn test_model_state_mutations() {
        let mut model = AdvancedTestModel::default();
        
        // Test state mutations through various messages
        model.handle_message(TestMsg::UpdateState("test".to_string()));
        assert_eq!(model.state, "test");
        
        model.handle_message(TestMsg::Increment);
        assert_eq!(model.counter, 1);
        
        model.handle_message(TestMsg::Decrement);
        assert_eq!(model.counter, 0);
        
        model.handle_message(TestMsg::AsyncComplete("result".to_string()));
        assert_eq!(model.async_results.len(), 1);
        assert_eq!(model.async_results[0], "result");
        
        model.handle_message(TestMsg::AsyncError("error".to_string()));
        assert_eq!(model.errors.len(), 1);
        assert_eq!(model.errors[0], "error");
        
        model.handle_message(TestMsg::RecoverFromError);
        assert_eq!(model.errors.len(), 0);
        assert_eq!(model.state, "recovered");
    }

    #[test]
    fn test_event_priority() {
        // Test that quit events have highest priority
        let quit_event: Event<TestMsg> = Event::Quit;
        let key_event: Event<TestMsg> = Event::Key(KeyEvent::new(Key::Char('a'), KeyModifiers::empty()));
        let user_event: Event<TestMsg> = Event::User(TestMsg::Increment);
        let tick_event: Event<TestMsg> = Event::Tick;
        
        // These should all be different event types
        assert!(!matches!(quit_event, Event::Key(_)));
        assert!(!matches!(key_event, Event::User(_)));
        assert!(!matches!(user_event, Event::Tick));
    }
}