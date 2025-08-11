//! Red team issue discovery test
//! This test explores edge cases and documents issues found

use hojicha_core::prelude::*;
use hojicha_runtime::{Event, Key, KeyEvent, KeyModifiers, Program, ProgramOptions};
use std::time::Duration;

// Issue #1: Testing what happens with None vs Cmd::none()
#[derive(Debug, Clone)]
struct NoneTestModel {
    counter: i32,
}

#[derive(Debug, Clone, PartialEq)]
enum NoneMsg {
    Test,
}

impl Model for NoneTestModel {
    type Message = NoneMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::Key(key) if key.key == Key::Char('q') => quit(),
            Event::User(NoneMsg::Test) => {
                self.counter += 1;
                // Documentation says Cmd::none() continues running
                // But what if we return a command that produces None?
                Cmd::new(|| None) // Does this behave the same as Cmd::none()?
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format!("Counter: {}", self.counter)),
            area,
        );
    }
}

// Issue #2: Testing command execution order and timing
#[derive(Debug, Clone)]
struct TimingTestModel {
    events: Vec<String>,
}

#[derive(Debug, Clone)]
enum TimingMsg {
    Event(String),
    StartTest,
}

impl Model for TimingTestModel {
    type Message = TimingMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(TimingMsg::Event(name)) => {
                self.events.push(name);
                Cmd::none()
            }
            Event::User(TimingMsg::StartTest) => {
                // What order do these execute in?
                batch(vec![
                    tick(Duration::from_millis(10), || TimingMsg::Event("tick_10".to_string())),
                    custom(|| Some(TimingMsg::Event("immediate".to_string()))),
                    tick(Duration::from_millis(5), || TimingMsg::Event("tick_5".to_string())),
                ])
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format!("Events: {:?}", self.events)),
            area,
        );
    }
}

// Issue #3: Testing recursive commands
#[derive(Debug, Clone)]
struct RecursiveTestModel {
    depth: usize,
    max_depth: usize,
}

#[derive(Debug, Clone)]
enum RecursiveMsg {
    Recurse,
}

impl Model for RecursiveTestModel {
    type Message = RecursiveMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(RecursiveMsg::Recurse) => {
                self.depth += 1;
                if self.depth < self.max_depth {
                    // Does this cause stack overflow or handle gracefully?
                    custom(|| Some(RecursiveMsg::Recurse))
                } else {
                    Cmd::none()
                }
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format!("Depth: {}/{}", self.depth, self.max_depth)),
            area,
        );
    }
}

// Issue #4: Testing error propagation
#[derive(Debug, Clone)]
struct ErrorTestModel {
    errors: Vec<String>,
}

#[derive(Debug, Clone)]
enum ErrorMsg {
    TriggerError,
    TriggerPanic,
}

impl Model for ErrorTestModel {
    type Message = ErrorMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(ErrorMsg::TriggerError) => {
                // What happens to errors in fallible commands?
                custom_fallible(|| {
                    Err(Error::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Test error - does this get logged?"
                    )))
                })
            }
            Event::User(ErrorMsg::TriggerPanic) => {
                // What happens if we panic in a command?
                custom(|| {
                    panic!("Test panic in command!");
                })
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format!("Errors: {:?}", self.errors)),
            area,
        );
    }
}

// Issue #5: Testing memory leaks with large batch operations
#[derive(Debug, Clone)]
struct MemoryTestModel {
    data: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
enum MemoryMsg {
    Allocate(usize),
    Clear,
}

impl Model for MemoryTestModel {
    type Message = MemoryMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(MemoryMsg::Allocate(size)) => {
                // Allocate memory
                self.data.push(vec![0u8; size]);
                
                // Create many commands - does this cause issues?
                let cmds: Vec<Cmd<Self::Message>> = (0..1000)
                    .map(|_| custom(|| Some(MemoryMsg::Clear)))
                    .collect();
                
                batch(cmds)
            }
            Event::User(MemoryMsg::Clear) => {
                if !self.data.is_empty() {
                    self.data.pop();
                }
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format!("Allocations: {}", self.data.len())),
            area,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_vs_cmd_none() {
        let mut model = NoneTestModel { counter: 0 };
        
        // Test Cmd::none()
        let cmd1: Cmd<NoneMsg> = Cmd::none();
        assert!(cmd1.is_noop(), "Cmd::none() should be a no-op");
        
        // Test Cmd::new(|| None)
        let cmd2: Cmd<NoneMsg> = Cmd::new(|| None);
        assert!(!cmd2.is_noop(), "Cmd::new(|| None) is NOT a no-op");
        
        // Execute both
        let result1 = cmd1.execute();
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), None);
        
        let result2 = cmd2.execute();
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), None);
        
        // ISSUE: Cmd::new(|| None) and Cmd::none() behave differently internally
        // but both return None when executed. This could be confusing.
    }

    #[test]
    fn test_batch_timing_order() {
        let mut model = TimingTestModel { events: Vec::new() };
        
        // Test batch command with mixed timing
        let cmd = model.update(Event::User(TimingMsg::StartTest));
        
        // Extract batch commands
        if let Some(cmds) = cmd.take_batch() {
            // Check what types of commands are in the batch
            let mut has_tick = false;
            let mut has_immediate = false;
            
            for cmd in cmds {
                if cmd.is_tick() {
                    has_tick = true;
                } else {
                    has_immediate = true;
                    // Execute immediate commands
                    if let Ok(Some(msg)) = cmd.execute() {
                        model.update(Event::User(msg));
                    }
                }
            }
            
            assert!(has_tick, "Batch should contain tick commands");
            assert!(has_immediate, "Batch should contain immediate commands");
            
            // ISSUE: Tick commands can't be executed synchronously in tests
            // This makes testing timing-dependent behavior difficult
        }
    }

    #[test]
    fn test_recursive_commands() {
        let mut model = RecursiveTestModel {
            depth: 0,
            max_depth: 100,
        };
        
        // Start recursion
        let mut cmd = model.update(Event::User(RecursiveMsg::Recurse));
        
        // Execute recursively (simulate what the runtime would do)
        let mut iterations = 0;
        while !cmd.is_noop() && iterations < 200 {
            if let Ok(Some(msg)) = cmd.execute() {
                cmd = model.update(Event::User(msg));
                iterations += 1;
            } else {
                break;
            }
        }
        
        assert_eq!(model.depth, model.max_depth, "Should reach max depth");
        assert!(iterations < 200, "Should not infinite loop");
        
        // ISSUE: No built-in protection against deep recursion
        // Could potentially cause stack overflow in real usage
    }

    #[test]
    fn test_error_handling() {
        let mut model = ErrorTestModel { errors: Vec::new() };
        
        // Test error in fallible command
        let cmd = model.update(Event::User(ErrorMsg::TriggerError));
        let result = cmd.execute();
        
        assert!(result.is_err(), "Should return an error");
        
        // ISSUE: Errors from fallible commands are not automatically handled
        // The application needs to explicitly handle them or they're lost
    }

    #[test]
    #[should_panic(expected = "Test panic in command!")]
    fn test_panic_in_command() {
        let mut model = ErrorTestModel { errors: Vec::new() };
        
        // This should panic
        let cmd = model.update(Event::User(ErrorMsg::TriggerPanic));
        cmd.execute().unwrap();
        
        // ISSUE: Panics in commands are not caught
        // This could crash the entire application
    }

    #[test]
    fn test_large_batch_performance() {
        let mut model = MemoryTestModel { data: Vec::new() };
        
        // Create a large batch
        let cmd = model.update(Event::User(MemoryMsg::Allocate(1024)));
        
        if let Some(cmds) = cmd.take_batch() {
            assert_eq!(cmds.len(), 1000, "Should create 1000 commands");
            
            // Try to execute all commands
            let start = std::time::Instant::now();
            for cmd in cmds.into_iter().take(100) { // Only execute 100 to save time
                if let Ok(Some(msg)) = cmd.execute() {
                    model.update(Event::User(msg));
                }
            }
            let elapsed = start.elapsed();
            
            // ISSUE: Large batches might cause performance issues
            // No built-in limit on batch size
            assert!(elapsed < Duration::from_secs(1), "Should execute quickly");
        }
    }

    #[test]
    fn test_cmd_is_methods() {
        // Test all the is_* methods on Cmd
        let noop = Cmd::<NoneMsg>::none();
        assert!(noop.is_noop());
        assert!(!noop.is_quit());
        assert!(!noop.is_batch());
        assert!(!noop.is_sequence());
        assert!(!noop.is_tick());
        assert!(!noop.is_every());
        
        let quit_cmd = quit::<NoneMsg>();
        assert!(!quit_cmd.is_noop());
        assert!(quit_cmd.is_quit());
        
        // ISSUE: batch() optimizes single-element batches to just return the element
        // This means batch(vec![cmd]) does NOT return a batch command!
        let batch_cmd = batch::<NoneMsg>(vec![Cmd::none(), Cmd::none()]);
        assert!(batch_cmd.is_batch(), "Two-element batch should be a batch");
        
        let single_batch = batch::<NoneMsg>(vec![Cmd::none()]);
        assert!(!single_batch.is_batch(), "Single-element batch is optimized away");
        
        // ISSUE: sequence() has similar optimization
        let seq_cmd = sequence::<NoneMsg>(vec![Cmd::none(), Cmd::none()]);
        assert!(seq_cmd.is_sequence(), "Two-element sequence should be a sequence");
        
        let tick_cmd = tick(Duration::from_secs(1), || NoneMsg::Test);
        assert!(tick_cmd.is_tick());
        
        let every_cmd = every(Duration::from_secs(1), |_| NoneMsg::Test);
        assert!(every_cmd.is_every());
    }
}