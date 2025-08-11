//! Test that async commands use shared runtime instead of creating new ones

use hojicha_core::commands;
use hojicha_core::core::{Cmd, Model};
use hojicha_core::event::Event;
use hojicha_runtime::testing::TestRunner;
use ratatui::layout::Rect;
use ratatui::Frame;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct AsyncTestModel {
    async_count: Arc<AtomicUsize>,
    completed: bool,
    initial_cmd: Option<Cmd<AsyncMsg>>,
}

impl Clone for AsyncTestModel {
    fn clone(&self) -> Self {
        Self {
            async_count: self.async_count.clone(),
            completed: self.completed,
            initial_cmd: None, // Don't clone the command
        }
    }
}

#[derive(Debug, Clone)]
enum AsyncMsg {
    AsyncComplete,
    BatchComplete,
}

impl Model for AsyncTestModel {
    type Message = AsyncMsg;

    fn init(&mut self) -> Cmd<Self::Message> {
        self.initial_cmd.take().unwrap_or_else(Cmd::none)
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(AsyncMsg::AsyncComplete) => {
                self.async_count.fetch_add(1, Ordering::SeqCst);
                
                // Test that we can spawn multiple async commands efficiently
                if self.async_count.load(Ordering::SeqCst) < 10 {
                    commands::spawn(async {
                        // Small delay to simulate async work
                        tokio::time::sleep(Duration::from_millis(1)).await;
                        Some(AsyncMsg::AsyncComplete)
                    })
                } else {
                    self.completed = true;
                    Cmd::none()
                }
            }
            Event::User(AsyncMsg::BatchComplete) => {
                self.completed = true;
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

#[test]
fn test_async_commands_use_shared_runtime() {
    // This test verifies that async commands complete quickly
    // If they were creating new runtimes, this would be much slower
    
    let model = AsyncTestModel {
        async_count: Arc::new(AtomicUsize::new(0)),
        completed: false,
        initial_cmd: Some(commands::spawn(async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            Some(AsyncMsg::AsyncComplete)
        })),
    };
    
    let start = Instant::now();
    
    let runner = TestRunner::new(model)
        .unwrap()
        .with_timeout(Duration::from_secs(1));
    
    runner.run_until(|m| m.completed).unwrap();
    
    let elapsed = start.elapsed();
    
    // If we're creating new runtimes, 10 async operations would take
    // significantly longer (>100ms due to runtime creation overhead)
    // With shared runtime, should complete in <50ms
    assert!(elapsed < Duration::from_millis(100), 
            "Async operations took too long: {:?}", elapsed);
}

#[test]
fn test_batch_async_commands_performance() {
    // Test that batched async commands also use shared runtime
    
    let model = AsyncTestModel {
        async_count: Arc::new(AtomicUsize::new(0)),
        completed: false,
        initial_cmd: Some(commands::batch(vec![
            commands::spawn(async {
                tokio::time::sleep(Duration::from_millis(5)).await;
                None
            }),
            commands::spawn(async {
                tokio::time::sleep(Duration::from_millis(5)).await;
                None
            }),
            commands::spawn(async {
                tokio::time::sleep(Duration::from_millis(5)).await;
                Some(AsyncMsg::BatchComplete)
            }),
        ])),
    };
    
    let start = Instant::now();
    
    let runner = TestRunner::new(model)
        .unwrap()
        .with_timeout(Duration::from_secs(1));
    
    runner.run_until(|m| m.completed).unwrap();
    
    let elapsed = start.elapsed();
    
    // Batched commands should run concurrently
    // Should complete in ~5ms (concurrent) not 15ms (sequential)
    // And definitely not >100ms (new runtime creation)
    assert!(elapsed < Duration::from_millis(50), 
            "Batch async operations took too long: {:?}", elapsed);
}

#[test]
fn test_custom_async_uses_shared_runtime() {
    // Test that custom_async also benefits from shared runtime
    
    let model = AsyncTestModel {
        async_count: Arc::new(AtomicUsize::new(0)),
        completed: false,
        initial_cmd: Some(commands::custom_async(|| async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            Some(AsyncMsg::BatchComplete)
        })),
    };
    
    let start = Instant::now();
    
    let runner = TestRunner::new(model)
        .unwrap()
        .with_timeout(Duration::from_millis(100));
    
    runner.run_until(|m| m.completed).unwrap();
    
    let elapsed = start.elapsed();
    
    // Should complete quickly with shared runtime
    assert!(elapsed < Duration::from_millis(50), 
            "custom_async took too long: {:?}", elapsed);
}