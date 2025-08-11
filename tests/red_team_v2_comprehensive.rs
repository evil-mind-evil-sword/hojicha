//! Red Team V2: Comprehensive testing of improvements
//! Finding edge cases in error handling, async, and command execution

use hojicha_core::prelude::*;
use hojicha_core::event::{Event, Key, KeyEvent, KeyModifiers};
use hojicha_runtime::{Program, ProgramOptions};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::{Duration, Instant};
use std::thread;

// ============== Error Handling Tests ==============

#[derive(Debug, Clone)]
struct ErrorHandlingModel {
    errors_caught: Vec<String>,
    panics_caught: usize,
    state: String,
}

#[derive(Debug, Clone)]
enum ErrorMsg {
    TriggerSimpleError,
    TriggerChainedErrors(usize),
    TriggerPanicInUpdate,
    TriggerPanicInCommand,
    TriggerPanicInAsync,
    TriggerDeepStackError(usize),
    ErrorReceived(String),
}

impl Default for ErrorHandlingModel {
    fn default() -> Self {
        Self {
            errors_caught: Vec::new(),
            panics_caught: 0,
            state: "initial".to_string(),
        }
    }
}

impl Model for ErrorHandlingModel {
    type Message = ErrorMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(msg) => match msg {
                ErrorMsg::TriggerSimpleError => {
                    custom_fallible(|| {
                        Err(Error::from(std::io::Error::other("Simple test error")))
                    })
                }
                ErrorMsg::TriggerChainedErrors(n) => {
                    // Create a chain of fallible commands
                    let mut cmds = vec![];
                    for i in 0..n {
                        cmds.push(custom_fallible(move || {
                            if i % 2 == 0 {
                                Ok(Some(ErrorMsg::ErrorReceived(format!("ok_{}", i))))
                            } else {
                                Err(Error::from(std::io::Error::other(format!("error_{}", i))))
                            }
                        }));
                    }
                    batch(cmds)
                }
                ErrorMsg::TriggerPanicInUpdate => {
                    panic!("Panic in update!");
                }
                ErrorMsg::TriggerPanicInCommand => {
                    custom(|| panic!("Panic in command!"))
                }
                ErrorMsg::TriggerPanicInAsync => {
                    spawn(async {
                        panic!("Panic in async!");
                    })
                }
                ErrorMsg::TriggerDeepStackError(depth) => {
                    if depth > 0 {
                        custom_fallible(move || {
                            // Create deep stack
                            let _big_array = [0u8; 1024 * 100]; // 100KB on stack
                            Ok(Some(ErrorMsg::TriggerDeepStackError(depth - 1)))
                        })
                    } else {
                        custom_fallible(|| {
                            Err(Error::from(std::io::Error::other("Deep stack error")))
                        })
                    }
                }
                ErrorMsg::ErrorReceived(err) => {
                    self.errors_caught.push(err);
                    Cmd::none()
                }
            },
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

// ============== Async/Stream Tests ==============

#[derive(Debug, Clone)]
struct AsyncStreamModel {
    events_received: Vec<String>,
    active_streams: usize,
    cancelled_count: usize,
}

#[derive(Debug, Clone)]
enum AsyncMsg {
    StartStream(usize),
    StreamEvent(usize, String),
    StartManyStreams(usize),
    CancelStream(usize),
    StartRaceCondition,
    AsyncTimeout,
    StartInfiniteStream,
    TestBackpressure(usize),
}

impl Default for AsyncStreamModel {
    fn default() -> Self {
        Self {
            events_received: Vec::new(),
            active_streams: 0,
            cancelled_count: 0,
        }
    }
}

impl Model for AsyncStreamModel {
    type Message = AsyncMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(msg) => match msg {
                AsyncMsg::StreamEvent(id, data) => {
                    self.events_received.push(format!("stream_{}: {}", id, data));
                    Cmd::none()
                }
                AsyncMsg::StartStream(id) => {
                    self.active_streams += 1;
                    spawn(async move {
                        for i in 0..5 {
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            // What if we don't return?
                            if i < 4 {
                                return Some(AsyncMsg::StreamEvent(id, format!("event_{}", i)));
                            }
                        }
                        None
                    })
                }
                AsyncMsg::StartManyStreams(count) => {
                    // Test resource limits
                    let mut cmds = vec![];
                    for i in 0..count {
                        cmds.push(spawn(async move {
                            tokio::time::sleep(Duration::from_millis(1)).await;
                            Some(AsyncMsg::StreamEvent(i, "flood".to_string()))
                        }));
                    }
                    batch(cmds)
                }
                AsyncMsg::StartRaceCondition => {
                    // Multiple async operations modifying shared state
                    let shared = Arc::new(Mutex::new(0));
                    let mut cmds = vec![];
                    
                    for i in 0..10 {
                        let shared_clone = shared.clone();
                        cmds.push(spawn(async move {
                            let mut val = shared_clone.lock().unwrap();
                            *val += 1;
                            drop(val);
                            tokio::time::sleep(Duration::from_millis(1)).await;
                            Some(AsyncMsg::StreamEvent(i, "race".to_string()))
                        }));
                    }
                    batch(cmds)
                }
                AsyncMsg::StartInfiniteStream => {
                    // Stream that never ends - memory leak?
                    spawn(async move {
                        loop {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            // Never returns
                        }
                    })
                }
                AsyncMsg::TestBackpressure(rate) => {
                    // Generate events faster than they can be processed
                    spawn(async move {
                        for i in 0..rate {
                            // No delay - flood the system
                            if i % 100 == 0 {
                                tokio::task::yield_now().await;
                            }
                        }
                        Some(AsyncMsg::StreamEvent(0, format!("backpressure_{}", rate)))
                    })
                }
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

// ============== Command Execution Edge Cases ==============

#[derive(Debug, Clone)]
struct CommandEdgeModel {
    execution_order: Vec<String>,
    command_count: usize,
}

#[derive(Debug, Clone)]
enum CommandMsg {
    TestEmptyBatch,
    TestSingleBatch,
    TestNestedBatch(usize),
    TestMixedCommands,
    TestRecursiveCommand(usize),
    TestCyclicBatch,
    TestCommandThatReturnsNone,
    TestManySmallCommands(usize),
    RecordExecution(String),
}

impl Default for CommandEdgeModel {
    fn default() -> Self {
        Self {
            execution_order: Vec::new(),
            command_count: 0,
        }
    }
}

impl Model for CommandEdgeModel {
    type Message = CommandMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        self.command_count += 1;
        
        match event {
            Event::User(msg) => match msg {
                CommandMsg::TestEmptyBatch => {
                    // What happens with empty batch?
                    batch(vec![])
                }
                CommandMsg::TestSingleBatch => {
                    // Single item batch optimization
                    batch(vec![
                        custom(|| Some(CommandMsg::RecordExecution("single".to_string())))
                    ])
                }
                CommandMsg::TestNestedBatch(depth) => {
                    // Deeply nested batches
                    let mut cmd = custom(|| Some(CommandMsg::RecordExecution("leaf".to_string())));
                    for i in 0..depth {
                        cmd = batch(vec![
                            cmd,
                            custom(move || Some(CommandMsg::RecordExecution(format!("level_{}", i)))),
                        ]);
                    }
                    cmd
                }
                CommandMsg::TestMixedCommands => {
                    // Mix of all command types
                    batch(vec![
                        custom(|| Some(CommandMsg::RecordExecution("custom".to_string()))),
                        tick(Duration::from_millis(1), || CommandMsg::RecordExecution("tick".to_string())),
                        spawn(async {
                            Some(CommandMsg::RecordExecution("async".to_string()))
                        }),
                        sequence(vec![
                            custom(|| Some(CommandMsg::RecordExecution("seq1".to_string()))),
                            custom(|| Some(CommandMsg::RecordExecution("seq2".to_string()))),
                        ]),
                        custom_fallible(|| Ok(Some(CommandMsg::RecordExecution("fallible".to_string())))),
                    ])
                }
                CommandMsg::TestRecursiveCommand(n) => {
                    if n > 0 {
                        sequence(vec![
                            custom(move || Some(CommandMsg::RecordExecution(format!("rec_{}", n)))),
                            custom(move || Some(CommandMsg::TestRecursiveCommand(n - 1))),
                        ])
                    } else {
                        Cmd::none()
                    }
                }
                CommandMsg::TestCyclicBatch => {
                    // Commands that create more commands in a cycle
                    if self.command_count < 100 {
                        batch(vec![
                            custom(|| Some(CommandMsg::RecordExecution("cycle".to_string()))),
                            custom(|| Some(CommandMsg::TestCyclicBatch)),
                        ])
                    } else {
                        Cmd::none()
                    }
                }
                CommandMsg::TestCommandThatReturnsNone => {
                    // Explicitly test Cmd::new(|| None)
                    Cmd::new(|| None)
                }
                CommandMsg::TestManySmallCommands(n) => {
                    // Stress test with many tiny commands
                    let cmds: Vec<_> = (0..n)
                        .map(|i| custom(move || Some(CommandMsg::RecordExecution(format!("cmd_{}", i)))))
                        .collect();
                    batch(cmds)
                }
                CommandMsg::RecordExecution(s) => {
                    self.execution_order.push(s);
                    Cmd::none()
                }
            },
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

// ============== Performance and Resource Limits ==============

#[derive(Debug, Clone)]
struct PerformanceModel {
    allocations: Vec<Vec<u8>>,
    thread_count: usize,
    message_count: usize,
}

#[derive(Debug, Clone)]
enum PerfMsg {
    AllocateLargeMemory(usize),
    SpawnManyThreads(usize),
    CreateDeepRecursion(usize),
    FloodWithMessages(usize),
    TestQueueResize,
    StressTestPriority,
    CreateMemoryLeak,
}

impl Default for PerformanceModel {
    fn default() -> Self {
        Self {
            allocations: Vec::new(),
            thread_count: 0,
            message_count: 0,
        }
    }
}

impl Model for PerformanceModel {
    type Message = PerfMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(msg) => match msg {
                PerfMsg::AllocateLargeMemory(mb) => {
                    // Allocate large chunks of memory
                    self.allocations.push(vec![0u8; mb * 1024 * 1024]);
                    Cmd::none()
                }
                PerfMsg::SpawnManyThreads(n) => {
                    // Spawn many async tasks
                    let mut cmds = vec![];
                    for i in 0..n {
                        cmds.push(spawn(async move {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            None
                        }));
                    }
                    batch(cmds)
                }
                PerfMsg::CreateDeepRecursion(depth) => {
                    fn recursive_alloc(n: usize) -> Vec<u8> {
                        let mut v = vec![0u8; 1024];
                        if n > 0 {
                            v.extend(recursive_alloc(n - 1));
                        }
                        v
                    }
                    
                    custom(move || {
                        let _ = recursive_alloc(depth);
                        None
                    })
                }
                PerfMsg::FloodWithMessages(n) => {
                    let cmds: Vec<_> = (0..n)
                        .map(|_| custom(|| None))
                        .collect();
                    batch(cmds)
                }
                PerfMsg::CreateMemoryLeak => {
                    // Intentionally leak memory
                    let leaked = Box::new(vec![0u8; 1024 * 1024]);
                    Box::leak(leaked);
                    Cmd::none()
                }
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_chaining() {
        let model = ErrorHandlingModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Send chained errors
        sender.send(Event::User(ErrorMsg::TriggerChainedErrors(10))).unwrap();
        
        // Give time for errors to process
        thread::sleep(Duration::from_millis(100));
        
        // Check if errors are handled properly
        // ISSUE: How do we verify error handling without access to model?
    }

    #[test]
    fn test_async_flood() {
        let model = AsyncStreamModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Flood with async operations
        sender.send(Event::User(AsyncMsg::StartManyStreams(1000))).unwrap();
        
        thread::sleep(Duration::from_millis(500));
        
        // ISSUE: Can the system handle 1000 concurrent async operations?
    }

    #[test]
    fn test_command_edge_cases() {
        let model = CommandEdgeModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Test empty batch
        sender.send(Event::User(CommandMsg::TestEmptyBatch)).unwrap();
        
        // Test single batch optimization
        sender.send(Event::User(CommandMsg::TestSingleBatch)).unwrap();
        
        // Test deeply nested batches
        sender.send(Event::User(CommandMsg::TestNestedBatch(100))).unwrap();
        
        thread::sleep(Duration::from_millis(100));
        
        // ISSUE: How are edge cases handled?
    }

    #[test]
    fn test_performance_limits() {
        let model = PerformanceModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Test with many small messages
        let start = Instant::now();
        for _ in 0..10000 {
            sender.send(Event::User(PerfMsg::FloodWithMessages(1))).unwrap();
        }
        let elapsed = start.elapsed();
        
        println!("Sending 10000 messages took: {:?}", elapsed);
        
        // ISSUE: What's the throughput limit?
    }

    #[test]
    fn test_recursive_command_limit() {
        let model = CommandEdgeModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Test deep recursion
        sender.send(Event::User(CommandMsg::TestRecursiveCommand(1000))).unwrap();
        
        thread::sleep(Duration::from_millis(500));
        
        // ISSUE: Is there a recursion limit?
    }

    #[test]
    fn test_panic_recovery() {
        let model = ErrorHandlingModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Send panic command
        sender.send(Event::User(ErrorMsg::TriggerPanicInCommand)).unwrap();
        
        // Wait a bit
        thread::sleep(Duration::from_millis(100));
        
        // Try to send another message - does it still work?
        let result = sender.send(Event::User(ErrorMsg::ErrorReceived("after_panic".to_string())));
        
        // ISSUE: Does the program recover from panics?
        println!("Send after panic: {:?}", result);
    }

    #[test]
    fn test_backpressure() {
        let model = AsyncStreamModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Generate events faster than processing
        sender.send(Event::User(AsyncMsg::TestBackpressure(100000))).unwrap();
        
        // ISSUE: How is backpressure handled?
    }

    #[test]
    fn test_mixed_command_ordering() {
        let model = CommandEdgeModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Send mixed command types
        sender.send(Event::User(CommandMsg::TestMixedCommands)).unwrap();
        
        thread::sleep(Duration::from_millis(200));
        
        // ISSUE: What's the execution order for mixed command types?
    }
}