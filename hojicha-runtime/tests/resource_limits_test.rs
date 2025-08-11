//! Integration tests for resource limits

use hojicha_runtime::resource_limits::{ResourceLimits, ResourceMonitor};
use hojicha_runtime::program::{Program, ProgramOptions};
use hojicha_core::{commands, core::{Cmd, Model}, event::Event};
use ratatui::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
struct StressTestModel {
    counter: Arc<AtomicUsize>,
}

impl Model for StressTestModel {
    type Message = ();
    
    fn init(&mut self) -> Cmd<Self::Message> {
        Cmd::none()
    }
    
    fn update(&mut self, _event: Event<Self::Message>) -> Cmd<Self::Message> {
        // Spawn many async tasks
        let mut cmds = vec![];
        for _ in 0..100 {
            let counter = self.counter.clone();
            cmds.push(commands::custom_async(move || {
                let counter = counter.clone();
                async move {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    counter.fetch_add(1, Ordering::SeqCst);
                    Some(())
                }
            }));
        }
        commands::batch(cmds)
    }
    
    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

#[test]
fn test_async_task_limits_compile() {
    // This test just ensures the resource limits integration compiles
    // and can be configured through ProgramOptions
    let counter = Arc::new(AtomicUsize::new(0));
    let model = StressTestModel { counter: counter.clone() };
    
    // Set very low limit
    let limits = ResourceLimits::default()
        .with_max_tasks(10);
    
    let options = ProgramOptions::new()
        .with_resource_limits(limits)
        .without_renderer()
        .headless();
    
    // Create program but don't run (we're just testing the configuration)
    let _program = Program::with_options(model, options);
    
    // The resource monitor should prevent spawning too many tasks
    // This test mainly ensures the code compiles and doesn't panic
}

#[tokio::test]
async fn test_resource_monitor_stats() {
    let monitor = ResourceMonitor::with_limits(
        ResourceLimits::default().with_max_tasks(5)
    );
    
    // Acquire some permits
    let _p1 = monitor.try_acquire_task_permit().await.unwrap();
    let _p2 = monitor.try_acquire_task_permit().await.unwrap();
    
    let stats = monitor.stats();
    assert_eq!(stats.active_tasks, 2);
    assert_eq!(stats.total_spawned, 2);
    assert_eq!(stats.peak_concurrent, 2);
    
    // Try to exceed limit
    let _p3 = monitor.try_acquire_task_permit().await.unwrap();
    let _p4 = monitor.try_acquire_task_permit().await.unwrap();
    let _p5 = monitor.try_acquire_task_permit().await.unwrap();
    
    // This should fail
    let result = monitor.try_acquire_task_permit().await;
    assert!(result.is_err());
    
    let stats = monitor.stats();
    assert_eq!(stats.active_tasks, 5);
    assert_eq!(stats.total_rejected, 1);
}

#[test]
fn test_resource_limits_builder() {
    let limits = ResourceLimits::default()
        .with_max_tasks(100)
        .with_max_recursion(50);
    
    assert_eq!(limits.max_concurrent_tasks, 100);
    assert_eq!(limits.max_recursion_depth, 50);
    assert_eq!(limits.task_warning_threshold, 80); // 80% of 100
    
    let unlimited = ResourceLimits::unlimited();
    assert_eq!(unlimited.max_concurrent_tasks, usize::MAX);
    assert!(!unlimited.log_warnings);
    assert!(!unlimited.reject_when_full);
}