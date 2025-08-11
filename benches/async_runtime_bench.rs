//! Benchmark to measure the performance impact of shared vs new runtime creation

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use tokio::runtime::Runtime;

fn benchmark_new_runtime_creation(c: &mut Criterion) {
    c.bench_function("create_new_runtime_per_operation", |b| {
        b.iter(|| {
            // This is what custom_async used to do
            let runtime = Runtime::new().unwrap();
            runtime.block_on(async {
                tokio::time::sleep(Duration::from_micros(1)).await;
                black_box(42)
            })
        });
    });
}

fn benchmark_shared_runtime(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    
    c.bench_function("use_shared_runtime", |b| {
        b.iter(|| {
            // This is what we do now
            runtime.block_on(async {
                tokio::time::sleep(Duration::from_micros(1)).await;
                black_box(42)
            })
        });
    });
}

fn benchmark_runtime_creation_only(c: &mut Criterion) {
    c.bench_function("runtime_new_only", |b| {
        b.iter(|| {
            let _runtime = black_box(Runtime::new().unwrap());
        });
    });
}

fn benchmark_spawn_on_shared_runtime(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    
    c.bench_function("spawn_on_shared_runtime", |b| {
        b.iter(|| {
            let handle = runtime.spawn(async {
                tokio::time::sleep(Duration::from_micros(1)).await;
                42
            });
            runtime.block_on(handle).unwrap()
        });
    });
}

// Benchmark the actual implementation
fn benchmark_hojicha_async_commands(c: &mut Criterion) {
    use hojicha_core::commands;
    use hojicha_core::core::{Cmd, Model};
    use hojicha_core::event::Event;
    
    use ratatui::layout::Rect;
    use ratatui::Frame;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[derive(Clone)]
    struct BenchModel {
        completed: Arc<AtomicBool>,
    }

    #[derive(Debug, Clone)]
    enum BenchMsg {
        Complete,
    }

    impl Model for BenchModel {
        type Message = BenchMsg;

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(BenchMsg::Complete) => {
                    self.completed.store(true, Ordering::SeqCst);
                    Cmd::none()
                }
                _ => Cmd::none(),
            }
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    c.bench_function("hojicha_custom_async", |b| {
        b.iter(|| {
            let model = BenchModel {
                completed: Arc::new(AtomicBool::new(false)),
            };
            let completed = model.completed.clone();
            
            // Test custom_async performance
            let cmd = commands::custom_async(|| async {
                tokio::time::sleep(Duration::from_micros(1)).await;
                Some(BenchMsg::Complete)
            });
            
            // We need to execute the command somehow
            // Since we can't easily run the full runtime, let's measure command creation
            black_box(cmd);
            
            // Mark as completed for measurement
            completed.store(true, Ordering::SeqCst);
        });
    });

    c.bench_function("hojicha_spawn", |b| {
        b.iter(|| {
            let model = BenchModel {
                completed: Arc::new(AtomicBool::new(false)),
            };
            let completed = model.completed.clone();
            
            // Test spawn performance
            let cmd = commands::spawn(async {
                tokio::time::sleep(Duration::from_micros(1)).await;
                Some(BenchMsg::Complete)
            });
            
            black_box(cmd);
            completed.store(true, Ordering::SeqCst);
        });
    });
}

// Compare old vs new implementation approach
fn benchmark_old_vs_new_approach(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_implementation");
    
    // Old approach: create runtime every time
    group.bench_function("old_approach_with_new_runtime", |b| {
        b.iter(|| {
            // Simulate what custom_async used to do
            let result = (move || {
                let runtime = tokio::runtime::Runtime::new().ok()?;
                runtime.block_on(async {
                    tokio::time::sleep(Duration::from_micros(1)).await;
                    Some(42)
                })
            })();
            black_box(result)
        });
    });
    
    // New approach: use shared runtime
    let shared_runtime = Runtime::new().unwrap();
    group.bench_function("new_approach_shared_runtime", |b| {
        b.iter(|| {
            // Simulate what custom_async does now
            let future = async {
                tokio::time::sleep(Duration::from_micros(1)).await;
                Some(42)
            };
            
            let result = shared_runtime.block_on(future);
            black_box(result)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_new_runtime_creation,
    benchmark_shared_runtime,
    benchmark_runtime_creation_only,
    benchmark_spawn_on_shared_runtime,
    benchmark_hojicha_async_commands,
    benchmark_old_vs_new_approach
);
criterion_main!(benches);