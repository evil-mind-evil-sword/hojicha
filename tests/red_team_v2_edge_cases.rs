//! Red Team V2: Finding remaining edge cases after improvements
//! Focus on subtle bugs, race conditions, and API inconsistencies

use hojicha_core::prelude::*;
use hojicha_core::event::{Event, Key, KeyEvent};
use hojicha_runtime::{Program, ProgramOptions};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::thread;

// ========== Issue: Event ordering and priority ===========
#[derive(Debug, Clone)]
struct EventOrderModel {
    events: Vec<String>,
}

#[derive(Debug, Clone)]
enum OrderMsg {
    RecordEvent(String),
    TriggerMixedPriority,
}

impl Default for EventOrderModel {
    fn default() -> Self {
        Self { events: Vec::new() }
    }
}

impl Model for EventOrderModel {
    type Message = OrderMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(OrderMsg::RecordEvent(s)) => {
                self.events.push(s);
                Cmd::none()
            }
            Event::User(OrderMsg::TriggerMixedPriority) => {
                // Send events with different priorities
                batch(vec![
                    tick(Duration::from_millis(1), || OrderMsg::RecordEvent("tick".to_string())),
                    custom(|| Some(OrderMsg::RecordEvent("custom".to_string()))),
                    spawn(async {
                        Some(OrderMsg::RecordEvent("async".to_string()))
                    }),
                ])
            }
            Event::Key(_) => {
                self.events.push("key".to_string());
                Cmd::none()
            }
            Event::Resize { .. } => {
                self.events.push("resize".to_string());
                Cmd::none()
            }
            Event::Tick => {
                self.events.push("tick_event".to_string());
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

// ========== Issue: Resource exhaustion ===========
#[derive(Debug, Clone)]
struct ResourceModel {
    allocation_count: Arc<AtomicUsize>,
}

#[derive(Debug, Clone)]
enum ResourceMsg {
    AllocateForever,
    SpawnForever,
    RecurseForever(usize),
}

impl Default for ResourceModel {
    fn default() -> Self {
        Self {
            allocation_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Model for ResourceModel {
    type Message = ResourceMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(ResourceMsg::AllocateForever) => {
                // Keep allocating memory
                let count = self.allocation_count.clone();
                spawn(async move {
                    loop {
                        let _leak = Box::leak(Box::new(vec![0u8; 1024 * 1024])); // 1MB leak
                        count.fetch_add(1, Ordering::Relaxed);
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                })
            }
            Event::User(ResourceMsg::SpawnForever) => {
                // Keep spawning tasks
                batch(vec![
                    spawn(async {
                        tokio::time::sleep(Duration::from_secs(3600)).await; // Sleep 1 hour
                        None
                    }),
                    custom(|| Some(ResourceMsg::SpawnForever)), // Recurse
                ])
            }
            Event::User(ResourceMsg::RecurseForever(n)) => {
                // Infinite recursion with growing stack
                let _big_array = [0u8; 1024 * 10]; // 10KB on stack
                custom(move || Some(ResourceMsg::RecurseForever(n + 1)))
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

// ========== Issue: API inconsistencies ===========
#[derive(Debug, Clone)]
struct ApiModel {
    results: Vec<String>,
}

#[derive(Debug, Clone)]
enum ApiMsg {
    TestBatchVsStrict,
    TestSequenceVsStrict,
    TestEveryVsTick,
    TestAsyncConsistency,
    Record(String),
}

impl Default for ApiModel {
    fn default() -> Self {
        Self { results: Vec::new() }
    }
}

impl Model for ApiModel {
    type Message = ApiMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(ApiMsg::TestBatchVsStrict) => {
                // Test if batch and batch_strict behave differently
                batch(vec![
                    batch(vec![]), // Empty batch - optimized away?
                    batch_strict(vec![]), // Empty strict batch - kept?
                    custom(|| Some(ApiMsg::Record("batch_test".to_string()))),
                ])
            }
            Event::User(ApiMsg::TestSequenceVsStrict) => {
                // Test sequence optimization edge cases
                sequence(vec![
                    sequence(vec![Cmd::none()]), // Single-element nested
                    sequence_strict(vec![Cmd::none()]), // Strict version
                    custom(|| Some(ApiMsg::Record("sequence_test".to_string()))),
                ])
            }
            Event::User(ApiMsg::TestEveryVsTick) => {
                // Test timing command differences
                batch(vec![
                    tick(Duration::from_millis(10), || ApiMsg::Record("tick".to_string())),
                    every(Duration::from_millis(10), |_| ApiMsg::Record("every".to_string())),
                ])
            }
            Event::User(ApiMsg::TestAsyncConsistency) => {
                // Test different async patterns
                batch(vec![
                    spawn(async { Some(ApiMsg::Record("spawn".to_string())) }),
                    custom_async(|| async { Some(ApiMsg::Record("custom_async".to_string())) }),
                    // Are these equivalent?
                ])
            }
            Event::User(ApiMsg::Record(s)) => {
                self.results.push(s);
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

// ========== Issue: State corruption ===========
#[derive(Debug, Clone)]
struct StateModel {
    shared_state: Arc<Mutex<Vec<usize>>>,
    local_state: Vec<usize>,
}

#[derive(Debug, Clone)]
enum StateMsg {
    CorruptSharedState,
    ModifyDuringUpdate,
    TestMoveSemantics,
}

impl Default for StateModel {
    fn default() -> Self {
        Self {
            shared_state: Arc::new(Mutex::new(Vec::new())),
            local_state: Vec::new(),
        }
    }
}

impl Model for StateModel {
    type Message = StateMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(StateMsg::CorruptSharedState) => {
                // Multiple threads modifying shared state
                let state1 = self.shared_state.clone();
                let state2 = self.shared_state.clone();
                
                batch(vec![
                    spawn(async move {
                        for i in 0..100 {
                            if let Ok(mut s) = state1.lock() {
                                s.push(i);
                                // No proper synchronization
                                s.sort_unstable();
                            }
                            tokio::task::yield_now().await;
                        }
                        None
                    }),
                    spawn(async move {
                        for i in 100..200 {
                            if let Ok(mut s) = state2.lock() {
                                s.push(i);
                                // Concurrent modification
                                s.retain(|&x| x % 2 == 0);
                            }
                            tokio::task::yield_now().await;
                        }
                        None
                    }),
                ])
            }
            Event::User(StateMsg::ModifyDuringUpdate) => {
                // Try to modify model during command execution
                self.local_state.push(1);
                
                let state_ptr = &mut self.local_state as *mut Vec<usize>;
                custom(move || {
                    // UNSAFE: Modifying model from command
                    unsafe {
                        (*state_ptr).push(2);
                    }
                    None
                })
            }
            Event::User(StateMsg::TestMoveSemantics) => {
                // Test if model can be moved/cloned incorrectly
                let cloned = self.local_state.clone();
                custom(move || {
                    let _ = cloned; // Move captured clone
                    None
                })
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_priority_ordering() {
        // ISSUE: Is event priority consistent across all scenarios?
        let model = EventOrderModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Send events with different priorities
        sender.send(Event::Resize { width: 100, height: 50 }).unwrap();
        sender.send(Event::Key(KeyEvent::new(Key::Char('a'), Default::default()))).unwrap();
        sender.send(Event::Tick).unwrap();
        sender.send(Event::User(OrderMsg::TriggerMixedPriority)).unwrap();
        
        thread::sleep(Duration::from_millis(100));
        
        // What order are they processed in?
        // Documentation says: Quit > Key > User > Tick > Resize
        // But is this always true?
    }

    #[test]
    #[ignore] // This test would exhaust resources
    fn test_resource_exhaustion() {
        let model = ResourceModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Try to exhaust resources
        sender.send(Event::User(ResourceMsg::SpawnForever)).unwrap();
        
        thread::sleep(Duration::from_secs(1));
        
        // ISSUE: No resource limits?
    }

    #[test]
    fn test_api_consistency() {
        // ISSUE: Different APIs for similar functionality
        
        // batch vs batch_strict
        let b1 = batch::<ApiMsg>(vec![]);
        let b2 = batch_strict::<ApiMsg>(vec![]);
        assert_ne!(b1.is_batch(), b2.is_batch(), "Inconsistent empty batch behavior");
        
        // spawn vs custom_async - are they really different?
        // Both create async commands but with different signatures
    }

    #[test]
    fn test_state_safety() {
        let model = StateModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Try to corrupt state
        sender.send(Event::User(StateMsg::CorruptSharedState)).unwrap();
        
        thread::sleep(Duration::from_millis(200));
        
        // ISSUE: Shared mutable state can lead to race conditions
    }

    #[test]
    fn test_command_execution_fairness() {
        // ISSUE: Are commands executed fairly or can some starve?
        let model = ApiModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Flood with one type of message
        for _ in 0..1000 {
            sender.send(Event::User(ApiMsg::Record("flood".to_string()))).unwrap();
        }
        
        // Then send a different message
        sender.send(Event::User(ApiMsg::TestBatchVsStrict)).unwrap();
        
        thread::sleep(Duration::from_millis(100));
        
        // Does the second message ever get processed?
    }

    #[test]
    fn test_panic_recovery_completeness() {
        // Test if ALL panic scenarios are handled
        
        // We know commands have panic recovery, but what about:
        // 1. Panics in Model::init()?
        // 2. Panics in Model::view()?
        // 3. Panics in Model::update() itself (not in commands)?
        // 4. Panics in custom error handlers?
    }

    #[test]
    fn test_memory_leak_in_subscriptions() {
        // ISSUE: Do cancelled subscriptions properly clean up?
        // If we create and cancel many subscriptions, is memory freed?
    }

    #[test]
    fn test_command_result_ordering() {
        // ISSUE: If commands in a batch complete at different times,
        // are their results processed in completion order or submission order?
        
        let model = ApiModel::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Send commands with different delays
        let cmd = batch(vec![
            spawn(async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Some(ApiMsg::Record("slow".to_string()))
            }),
            spawn(async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Some(ApiMsg::Record("fast".to_string()))
            }),
        ]);
        
        // What order are results processed?
    }
}