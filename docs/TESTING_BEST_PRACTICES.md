# Testing Best Practices for Hojicha

## Overview

This document outlines testing best practices for the hojicha TUI framework, based on patterns from Tokio, Crossbeam, and other mature Rust async/concurrent libraries.

## Current Testing Architecture

### Strengths
- Headless testing with `ProgramOptions::headless()`
- Timeout-based bounded execution
- Atomic counters for verification
- Priority queue stress testing

### Testing Patterns We Use

#### 1. Concurrent Message Sending
```rust
// Good: Using barriers for synchronized starts
let barrier = Arc::new(Barrier::new(num_threads));
for thread_id in 0..num_threads {
    let barrier = barrier.clone();
    thread::spawn(move || {
        barrier.wait(); // All threads start together
        // Send messages...
    });
}
```

#### 2. Deterministic Message Verification
```rust
// Good: Count-based termination instead of time-based
program.run_until(|model| {
    model.message_count.load(Ordering::SeqCst) >= expected
}).unwrap();
```

## Recommended Improvements

### 1. Mock Event Processor (High Priority)

Create a test harness that doesn't require running the full program:

```rust
// src/testing/mock_processor.rs
pub struct MockEventProcessor<M: Model> {
    model: M,
    events: VecDeque<Event<M::Message>>,
    processed: Vec<Event<M::Message>>,
    wake_count: AtomicUsize,
}

impl<M: Model> MockEventProcessor<M> {
    pub fn process_events(&mut self) -> Vec<Event<M::Message>> {
        while let Some(event) = self.events.pop_front() {
            if let Some(cmd) = self.model.update(event.clone()) {
                // Process command synchronously
                self.execute_command(cmd);
            }
            self.processed.push(event);
        }
        self.processed.clone()
    }
}
```

### 2. Deterministic Time Control (High Priority)

Instead of real time, use virtual time:

```rust
// src/testing/time_control.rs
pub struct VirtualClock {
    now: Instant,
    pending_timers: BinaryHeap<Timer>,
}

impl VirtualClock {
    pub fn advance(&mut self, duration: Duration) {
        self.now += duration;
        // Fire any timers that are now ready
    }
    
    pub fn advance_to_next_timer(&mut self) {
        if let Some(timer) = self.pending_timers.peek() {
            self.now = timer.deadline;
        }
    }
}
```

### 3. Property-Based Testing (Medium Priority)

Use `proptest` or `quickcheck` for invariant testing:

```rust
proptest! {
    #[test]
    fn priority_ordering_preserved(
        events in prop::collection::vec(
            event_strategy(), 
            0..1000
        )
    ) {
        let mut processor = PriorityEventProcessor::new();
        processor.submit_all(events);
        
        let processed = processor.drain();
        // Verify high priority events come before low priority
        verify_priority_order(&processed);
    }
}
```

### 4. Stress Testing Patterns (Medium Priority)

Follow Crossbeam's approach for stress testing:

```rust
#[test]
#[ignore = "stress test"]
fn stress_concurrent_senders() {
    const THREADS: usize = 100;
    const MESSAGES_PER_THREAD: usize = 1000;
    
    let (sender, receiver) = channel();
    let barrier = Arc::new(Barrier::new(THREADS));
    
    // Spawn senders
    let handles: Vec<_> = (0..THREADS).map(|id| {
        let sender = sender.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            barrier.wait();
            for seq in 0..MESSAGES_PER_THREAD {
                sender.send(Message { id, seq }).unwrap();
            }
        })
    }).collect();
    
    // Verify all messages received
    let mut received = HashMap::new();
    for _ in 0..THREADS * MESSAGES_PER_THREAD {
        let msg = receiver.recv().unwrap();
        *received.entry(msg.id).or_insert(0) += 1;
    }
    
    // Check fairness
    for count in received.values() {
        assert_eq!(*count, MESSAGES_PER_THREAD);
    }
}
```

### 5. Loom Integration for Model Checking (Low Priority)

For critical concurrent code, use `loom` for exhaustive testing:

```rust
#[test]
#[cfg(loom)]
fn loom_priority_queue_safety() {
    loom::model(|| {
        let queue = Arc::new(PriorityQueue::new());
        
        let q1 = queue.clone();
        let t1 = loom::thread::spawn(move || {
            q1.push(Event::HighPriority);
        });
        
        let q2 = queue.clone();
        let t2 = loom::thread::spawn(move || {
            q2.push(Event::LowPriority);
        });
        
        t1.join().unwrap();
        t2.join().unwrap();
        
        // Verify queue state is consistent
        assert_eq!(queue.len(), 2);
    });
}
```

## Testing Anti-Patterns to Avoid

### ❌ Don't Use Real Time
```rust
// Bad
thread::sleep(Duration::from_millis(100));
assert!(something_happened);

// Good
model.wait_for_condition(|m| m.something_happened);
```

### ❌ Don't Test Implementation Details
```rust
// Bad: Testing internal queue structure
assert_eq!(program.internal_queue.capacity(), 1024);

// Good: Testing observable behavior
assert!(program.can_accept_events());
```

### ❌ Don't Use Unbounded Operations
```rust
// Bad
loop {
    if condition { break; }
    thread::sleep(Duration::from_millis(1));
}

// Good
program.run_with_timeout(Duration::from_secs(1))?;
```

## Test Organization

### Unit Tests
- Test individual components in isolation
- Use mock dependencies
- Should run in < 10ms each

### Integration Tests
- Test component interactions
- Use real implementations where possible
- Should run in < 100ms each

### Stress Tests
- Mark with `#[ignore]`
- Test with high volumes (10k+ operations)
- Verify performance characteristics

### Property Tests
- Test invariants across random inputs
- Use smaller iteration counts for CI (10-100)
- Use larger counts for local testing (1000+)

## Continuous Integration

### Test Execution Strategy
```bash
# Fast tests for every commit
cargo test --lib --all-features

# Integration tests for PRs
cargo test --all-features

# Stress tests nightly
cargo test -- --ignored

# Property tests with more iterations weekly
PROPTEST_CASES=10000 cargo test
```

## Metrics and Coverage

### Key Metrics to Track
- Test execution time
- Code coverage (target: 70%+)
- Flaky test rate (target: 0%)
- Performance regression detection

### Coverage Focus Areas
1. Event processing paths (critical)
2. Priority queue operations (critical)
3. Async bridge functionality (important)
4. Command execution (important)
5. UI rendering (nice-to-have)

## Future Improvements

1. **Test Harness Library**: Create `hojicha-test` crate with testing utilities
2. **Fuzzing**: Add fuzzing for event processing logic
3. **Benchmark Suite**: Track performance over time
4. **Mutation Testing**: Verify test effectiveness

## References

- [Tokio Testing Patterns](https://tokio.rs/tokio/topics/testing)
- [Crossbeam Channel Tests](https://github.com/crossbeam-rs/crossbeam/tree/master/crossbeam-channel/tests)
- [Loom Model Checking](https://github.com/tokio-rs/loom)
- [Property-based Testing in Rust](https://proptest-rs.github.io/proptest/)