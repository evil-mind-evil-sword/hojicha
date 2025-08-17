# Concurrency Safety Guide

This guide covers best practices for safe concurrent programming in Hojicha applications.

## Table of Contents
- [Core Principles](#core-principles)
- [Common Pitfalls](#common-pitfalls)
- [Safe Patterns](#safe-patterns)
- [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
- [Testing Concurrent Code](#testing-concurrent-code)
- [Debugging Tips](#debugging-tips)

## Core Principles

### 1. Message Passing Over Shared State

The Elm Architecture encourages message passing as the primary communication mechanism:

✅ **DO: Use messages to communicate between concurrent operations**
```rust
// Good: Message passing
fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match event {
        Event::User(Msg::StartFetch) => {
            async_cmd(async {
                let data = fetch_data().await;
                Some(Msg::DataFetched(data))  // Send result as message
            })
        }
        Event::User(Msg::DataFetched(data)) => {
            self.data = data;  // Update state in response to message
            Cmd::none()
        }
        _ => Cmd::none()
    }
}
```

❌ **DON'T: Share mutable state between tasks**
```rust
// Bad: Shared mutable state
struct Model {
    shared_data: Arc<Mutex<Vec<Data>>>,  // Danger!
}

fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    let data = self.shared_data.clone();
    async_cmd(async move {
        let mut data = data.lock().unwrap();
        data.push(new_item);  // Race condition!
        None
    })
}
```

### 2. Immutability by Default

Keep your model data immutable where possible:

✅ **DO: Use immutable data structures**
```rust
use im::Vector;  // Immutable vector

struct Model {
    items: Vector<Item>,  // Efficient immutable updates
}

fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match event {
        Event::User(Msg::AddItem(item)) => {
            self.items = self.items.push_back(item);  // Creates new vector
            Cmd::none()
        }
        _ => Cmd::none()
    }
}
```

### 3. Clear State Transitions

Design your state as a finite state machine with clear transitions:

✅ **DO: Use enums for state machines**
```rust
enum AppState {
    Idle,
    Loading { request_id: u64 },
    Loaded { data: Data },
    Error { message: String },
}

struct Model {
    state: AppState,
}

fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match (&self.state, event) {
        (AppState::Idle, Event::User(Msg::StartLoad)) => {
            self.state = AppState::Loading { request_id: generate_id() };
            // Start async operation
        }
        (AppState::Loading { request_id }, Event::User(Msg::LoadComplete(id, data))) 
            if *request_id == id => {
            self.state = AppState::Loaded { data };
            Cmd::none()
        }
        _ => Cmd::none()  // Invalid transitions ignored
    }
}
```

## Common Pitfalls

### 1. Race Conditions with Shared State

**Problem:**
```rust
// Two concurrent tasks modifying the same data
let state1 = self.shared_vec.clone();
let state2 = self.shared_vec.clone();

combine(vec![
    async_cmd(async move {
        state1.lock().unwrap().push(1);  // Task 1 modifies
    }),
    async_cmd(async move {
        state2.lock().unwrap().clear();  // Task 2 clears - race!
    }),
])
```

**Solution:**
```rust
// Use sequential messages instead
combine(vec![
    async_cmd(async { Some(Msg::AddItem(1)) }),
    async_cmd(async { Some(Msg::ClearItems) }),
])
```

### 2. Deadlocks from Lock Ordering

**Problem:**
```rust
// Potential deadlock from inconsistent lock ordering
let lock_a = self.lock_a.clone();
let lock_b = self.lock_b.clone();

async_cmd(async move {
    let _a = lock_a.lock();  // Thread 1: locks A then B
    let _b = lock_b.lock();
})

// Elsewhere...
async_cmd(async move {
    let _b = lock_b.lock();  // Thread 2: locks B then A - DEADLOCK!
    let _a = lock_a.lock();
})
```

**Solution:**
Don't use multiple locks. Use message passing instead.

### 3. Non-Deterministic Updates

**Problem:**
```rust
// Order of updates is non-deterministic
combine(vec![
    async_cmd(async { Some(Msg::SetValue(1)) }),
    async_cmd(async { Some(Msg::SetValue(2)) }),
    async_cmd(async { Some(Msg::SetValue(3)) }),
])
// Final value could be 1, 2, or 3!
```

**Solution:**
```rust
// Use chain for deterministic ordering
chain(vec![
    async_cmd(async { Some(Msg::SetValue(1)) }),
    async_cmd(async { Some(Msg::SetValue(2)) }),
    async_cmd(async { Some(Msg::SetValue(3)) }),
])
// Final value will always be 3
```

## Safe Patterns

### 1. Actor Pattern with Channels

Use channels to implement actor-like patterns:

```rust
use tokio::sync::mpsc;

enum ActorMsg {
    Add(i32),
    Remove(i32),
    GetSum(mpsc::Sender<i32>),
}

struct Model {
    actor_tx: mpsc::Sender<ActorMsg>,
}

impl Model {
    fn new() -> (Self, Cmd<Msg>) {
        let (tx, mut rx) = mpsc::channel(100);
        
        // Spawn actor task
        let cmd = async_cmd(async move {
            let mut state = Vec::new();
            
            while let Some(msg) = rx.recv().await {
                match msg {
                    ActorMsg::Add(val) => state.push(val),
                    ActorMsg::Remove(val) => state.retain(|&x| x != val),
                    ActorMsg::GetSum(reply) => {
                        let sum = state.iter().sum();
                        let _ = reply.send(sum).await;
                    }
                }
            }
            None
        });
        
        (Self { actor_tx: tx }, cmd)
    }
}
```

### 2. Request-Response Pattern

Track requests to handle concurrent operations safely:

```rust
struct Model {
    next_request_id: u64,
    pending_requests: HashMap<u64, RequestInfo>,
}

fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match event {
        Event::User(Msg::StartRequest) => {
            let id = self.next_request_id;
            self.next_request_id += 1;
            self.pending_requests.insert(id, RequestInfo::new());
            
            async_cmd(async move {
                let result = perform_request().await;
                Some(Msg::RequestComplete(id, result))
            })
        }
        Event::User(Msg::RequestComplete(id, result)) => {
            if self.pending_requests.remove(&id).is_some() {
                // Handle valid response
                self.process_result(result);
            }
            // Ignore responses for unknown requests
            Cmd::none()
        }
        _ => Cmd::none()
    }
}
```

### 3. Cancellation Tokens

Use cancellation tokens to manage long-running operations:

```rust
use tokio_util::sync::CancellationToken;

struct Model {
    current_operation: Option<CancellationToken>,
}

fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match event {
        Event::User(Msg::StartOperation) => {
            // Cancel previous operation if any
            if let Some(token) = &self.current_operation {
                token.cancel();
            }
            
            let token = CancellationToken::new();
            self.current_operation = Some(token.clone());
            
            async_cmd(async move {
                tokio::select! {
                    _ = token.cancelled() => None,
                    result = long_operation() => Some(Msg::OperationComplete(result)),
                }
            })
        }
        _ => Cmd::none()
    }
}
```

## Anti-Patterns to Avoid

### ❌ Arc<Mutex<T>> in Model

```rust
// DON'T DO THIS
struct Model {
    shared_state: Arc<Mutex<State>>,  // Invites race conditions
}
```

**Why it's bad:**
- Encourages shared mutable state
- Makes race conditions likely
- Hard to reason about
- Difficult to test

**Alternative:** Use message passing

### ❌ Interior Mutability (Rc<RefCell<T>>)

```rust
// DON'T DO THIS
struct Model {
    data: Rc<RefCell<Data>>,  // Runtime borrow checking
}
```

**Why it's bad:**
- Can panic at runtime
- Not thread-safe
- Hides mutability
- Breaks Rust's ownership model

**Alternative:** Use explicit state updates in `update()`

### ❌ Global Mutable State

```rust
// DON'T DO THIS
static mut GLOBAL_STATE: Option<State> = None;  // Unsafe!
```

**Why it's bad:**
- Requires unsafe code
- Not thread-safe without synchronization
- Hidden dependencies
- Makes testing impossible

**Alternative:** Keep all state in the Model

## Testing Concurrent Code

### 1. Use Deterministic Test Harnesses

```rust
#[test]
fn test_concurrent_updates() {
    let mut harness = EventTestHarness::new(Model::default());
    
    // Send concurrent operations
    harness.send_message(Msg::StartOp1);
    harness.send_message(Msg::StartOp2);
    
    // Manually trigger completion in deterministic order
    harness.send_message(Msg::Op1Complete(result1));
    harness.send_message(Msg::Op2Complete(result2));
    
    // Assert final state
    assert_eq!(harness.model().state, expected_state);
}
```

### 2. Test Race Conditions

```rust
#[test]
fn test_no_race_conditions() {
    let model = Model::default();
    
    // Run many concurrent operations
    for _ in 0..100 {
        let mut harness = EventTestHarness::new(model.clone());
        
        // Send many concurrent messages
        for i in 0..10 {
            harness.send_message(Msg::ConcurrentOp(i));
        }
        
        // State should always be consistent
        assert!(harness.model().is_consistent());
    }
}
```

### 3. Test Cancellation

```rust
#[test]
fn test_operation_cancellation() {
    let mut harness = TimeControlledHarness::new(Model::default());
    
    // Start operation
    harness.send_message(Msg::StartLongOp);
    
    // Start another before first completes
    harness.send_message(Msg::StartLongOp);
    
    // Only the second should complete
    harness.advance_time(Duration::from_secs(10));
    assert_eq!(harness.model().completed_ops, 1);
}
```

## Debugging Tips

### 1. Add Logging to Track Concurrency

```rust
fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    log::debug!("Update called with: {:?}, current state: {:?}", event, self.state);
    
    let result = match event {
        // ... handle events
    };
    
    log::debug!("Update returning: {:?}, new state: {:?}", result, self.state);
    result
}
```

### 2. Use Debug Assertions

```rust
fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    debug_assert!(self.invariants_hold(), "State invariants violated!");
    
    let result = match event {
        // ... handle events
    };
    
    debug_assert!(self.invariants_hold(), "State invariants violated after update!");
    result
}
```

### 3. Visualize State Transitions

```rust
impl Model {
    fn debug_state_transition(&self, event: &Event<Msg>) {
        eprintln!("┌─────────────────────────");
        eprintln!("│ State: {:?}", self.state);
        eprintln!("│ Event: {:?}", event);
        eprintln!("│ Time:  {:?}", std::time::Instant::now());
        eprintln!("└─────────────────────────");
    }
}
```

## Best Practices Summary

1. **Use message passing** instead of shared mutable state
2. **Keep Model immutable** where possible
3. **Design clear state machines** with explicit transitions
4. **Track async operations** with request IDs
5. **Test concurrent scenarios** deterministically
6. **Avoid Arc<Mutex<T>>** and other interior mutability
7. **Use cancellation tokens** for long-running operations
8. **Log state transitions** for debugging
9. **Assert invariants** in debug builds
10. **Document concurrency assumptions** in your code

## Further Reading

- [The Elm Architecture Guide](https://guide.elm-lang.org/architecture/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Fearless Concurrency in Rust](https://doc.rust-lang.org/book/ch16-00-concurrency.html)