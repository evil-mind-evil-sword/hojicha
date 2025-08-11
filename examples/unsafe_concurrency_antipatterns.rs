//! Examples of UNSAFE concurrency anti-patterns to AVOID
//! 
//! ⚠️ WARNING: This file demonstrates what NOT to do! ⚠️
//! 
//! These patterns can lead to:
//! - Race conditions
//! - Deadlocks
//! - Data corruption
//! - Non-deterministic behavior
//! 
//! Each anti-pattern is followed by the correct approach.

#![allow(dead_code, unused_variables)]

use hojicha::{commands::*, core::*, event::*, prelude::*};
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

// ============================================================================
// ANTI-PATTERN 1: Shared Mutable State with Arc<Mutex<T>>
// ============================================================================

/// ❌ DON'T: Use Arc<Mutex<T>> for shared state
mod antipattern_shared_state {
    use super::*;
    
    struct BadModel {
        // ❌ This invites race conditions!
        shared_data: Arc<Mutex<Vec<i32>>>,
        shared_counter: Arc<Mutex<i32>>,
    }
    
    impl Model for BadModel {
        type Message = ();
        
        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }
        
        fn update(&mut self, _event: Event<Self::Message>) -> Cmd<Self::Message> {
            // ❌ Multiple tasks modifying shared state = race conditions
            let data1 = self.shared_data.clone();
            let data2 = self.shared_data.clone();
            let counter = self.shared_counter.clone();
            
            batch(vec![
                custom_async(move || async move {
                    // Task 1: Add items
                    for i in 0..100 {
                        let mut data = data1.lock().unwrap();
                        data.push(i);  // Racing with task 2!
                    }
                    None
                }),
                custom_async(move || async move {
                    // Task 2: Clear and modify
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    let mut data = data2.lock().unwrap();
                    data.clear();  // Might clear task 1's data!
                    data.push(-1);
                    None
                }),
                custom_async(move || async move {
                    // Task 3: Increment counter while reading data
                    let mut count = counter.lock().unwrap();
                    *count += 1;  // Non-deterministic final value!
                    None
                }),
            ])
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    }
}

/// ✅ DO: Use message passing instead
mod correct_message_passing {
    use super::*;
    
    #[derive(Clone)]
    enum Msg {
        AddItem(i32),
        ClearItems,
        IncrementCounter,
    }
    
    struct GoodModel {
        // ✅ All state owned by model, no sharing needed
        data: Vec<i32>,
        counter: i32,
    }
    
    impl Model for GoodModel {
        type Message = Msg;
        
        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }
        
        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(Msg::AddItem(item)) => {
                    // ✅ Deterministic state update
                    self.data.push(item);
                    Cmd::none()
                }
                Event::User(Msg::ClearItems) => {
                    // ✅ Clear happens in defined order
                    self.data.clear();
                    Cmd::none()
                }
                Event::User(Msg::IncrementCounter) => {
                    // ✅ No race conditions
                    self.counter += 1;
                    Cmd::none()
                }
                _ => Cmd::none(),
            }
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    }
}

// ============================================================================
// ANTI-PATTERN 2: Interior Mutability with Rc<RefCell<T>>
// ============================================================================

/// ❌ DON'T: Use Rc<RefCell<T>> for interior mutability
mod antipattern_interior_mutability {
    use super::*;
    
    struct BadModel {
        // ❌ Runtime borrow checking = potential panics!
        data: Rc<RefCell<Vec<String>>>,
        cache: Rc<RefCell<HashMap<String, String>>>,
    }
    
    impl Model for BadModel {
        type Message = ();
        
        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }
        
        fn update(&mut self, _event: Event<Self::Message>) -> Cmd<Self::Message> {
            // ❌ Can panic at runtime if already borrowed!
            let mut data = self.data.borrow_mut();
            data.push("item".into());
            
            // ❌ Nested borrows can cause panic
            self.process_data();  // Might try to borrow data again!
            
            Cmd::none()
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {
            // ❌ Can panic if data is already mutably borrowed
            let data = self.data.borrow();
            // render data...
        }
    }
    
    impl BadModel {
        fn process_data(&self) {
            // ❌ This will panic if called while data is already borrowed!
            let data = self.data.borrow_mut();
            // process...
        }
    }
    
    use std::collections::HashMap;
}

/// ✅ DO: Use explicit ownership and state updates
mod correct_ownership {
    use super::*;
    use std::collections::HashMap;
    
    struct GoodModel {
        // ✅ Direct ownership, no interior mutability
        data: Vec<String>,
        cache: HashMap<String, String>,
    }
    
    impl Model for GoodModel {
        type Message = ();
        
        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }
        
        fn update(&mut self, _event: Event<Self::Message>) -> Cmd<Self::Message> {
            // ✅ Direct mutation, no runtime borrow checking
            self.data.push("item".into());
            
            // ✅ Can safely call methods
            self.process_data();
            
            Cmd::none()
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {
            // ✅ Always safe to read
            // render &self.data...
        }
    }
    
    impl GoodModel {
        fn process_data(&mut self) {
            // ✅ Explicit mutable access, checked at compile time
            // process self.data...
        }
    }
}

// ============================================================================
// ANTI-PATTERN 3: Untracked Async Operations
// ============================================================================

/// ❌ DON'T: Fire and forget async operations without tracking
mod antipattern_untracked_async {
    use super::*;
    
    struct BadModel {
        result: Option<String>,
    }
    
    impl Model for BadModel {
        type Message = String;
        
        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }
        
        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(result) => {
                    // ❌ Which request is this from? Is it still relevant?
                    self.result = Some(result);
                    
                    // ❌ Starting more operations without cancelling previous
                    batch(vec![
                        custom_async(|| async { Some("result1".into()) }),
                        custom_async(|| async { Some("result2".into()) }),
                        custom_async(|| async { Some("result3".into()) }),
                    ])
                    // Results arrive in random order!
                }
                _ => Cmd::none(),
            }
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    }
}

/// ✅ DO: Track async operations with request IDs
mod correct_tracked_async {
    use super::*;
    use std::collections::HashMap;
    
    #[derive(Clone)]
    enum Msg {
        StartRequest,
        RequestComplete(u64, String),
    }
    
    struct GoodModel {
        next_request_id: u64,
        pending_requests: HashMap<u64, RequestInfo>,
        results: Vec<(u64, String)>,
    }
    
    struct RequestInfo {
        started_at: std::time::Instant,
    }
    
    impl Model for GoodModel {
        type Message = Msg;
        
        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }
        
        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(Msg::StartRequest) => {
                    // ✅ Track the request
                    let id = self.next_request_id;
                    self.next_request_id += 1;
                    self.pending_requests.insert(id, RequestInfo {
                        started_at: std::time::Instant::now(),
                    });
                    
                    // ✅ Include ID in response
                    custom_async(move || async move {
                        let result = format!("Result for request {}", id);
                        Some(Msg::RequestComplete(id, result))
                    })
                }
                Event::User(Msg::RequestComplete(id, result)) => {
                    // ✅ Verify request is still valid
                    if self.pending_requests.remove(&id).is_some() {
                        self.results.push((id, result));
                    }
                    // Ignore results from cancelled requests
                    Cmd::none()
                }
                _ => Cmd::none(),
            }
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    }
}

// ============================================================================
// ANTI-PATTERN 4: Global Mutable State
// ============================================================================

/// ❌ DON'T: Use global mutable state
mod antipattern_global_state {
    use super::*;
    
    // ❌ Global mutable state is unsafe and not thread-safe!
    static mut GLOBAL_COUNTER: i32 = 0;
    static mut GLOBAL_CACHE: Option<Vec<String>> = None;
    
    struct BadModel;
    
    impl Model for BadModel {
        type Message = ();
        
        fn init(&mut self) -> Cmd<Self::Message> {
            unsafe {
                // ❌ Requires unsafe, not thread-safe!
                GLOBAL_COUNTER = 0;
                GLOBAL_CACHE = Some(Vec::new());
            }
            Cmd::none()
        }
        
        fn update(&mut self, _event: Event<Self::Message>) -> Cmd<Self::Message> {
            unsafe {
                // ❌ Race conditions in async contexts!
                GLOBAL_COUNTER += 1;
                
                if let Some(ref mut cache) = GLOBAL_CACHE {
                    cache.push(format!("Item {}", GLOBAL_COUNTER));
                }
            }
            Cmd::none()
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    }
}

/// ✅ DO: Keep all state in the Model
mod correct_local_state {
    use super::*;
    
    struct GoodModel {
        // ✅ All state is local to the model
        counter: i32,
        cache: Vec<String>,
    }
    
    impl Model for GoodModel {
        type Message = ();
        
        fn init(&mut self) -> Cmd<Self::Message> {
            // ✅ Initialize local state
            self.counter = 0;
            self.cache = Vec::new();
            Cmd::none()
        }
        
        fn update(&mut self, _event: Event<Self::Message>) -> Cmd<Self::Message> {
            // ✅ Safe, deterministic updates
            self.counter += 1;
            self.cache.push(format!("Item {}", self.counter));
            Cmd::none()
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {
            // ✅ Safe to read local state
            // render self.counter and self.cache...
        }
    }
}

// ============================================================================
// ANTI-PATTERN 5: Deadlock-Prone Lock Ordering
// ============================================================================

/// ❌ DON'T: Use multiple locks with inconsistent ordering
mod antipattern_deadlock {
    use super::*;
    
    struct BadModel {
        // ❌ Multiple locks = potential deadlock!
        lock_a: Arc<Mutex<i32>>,
        lock_b: Arc<Mutex<i32>>,
    }
    
    impl Model for BadModel {
        type Message = ();
        
        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }
        
        fn update(&mut self, _event: Event<Self::Message>) -> Cmd<Self::Message> {
            let lock_a1 = self.lock_a.clone();
            let lock_b1 = self.lock_b.clone();
            let lock_a2 = self.lock_a.clone();
            let lock_b2 = self.lock_b.clone();
            
            batch(vec![
                custom_async(move || async move {
                    // ❌ Thread 1: Lock A then B
                    let _a = lock_a1.lock().unwrap();
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    let _b = lock_b1.lock().unwrap();
                    None
                }),
                custom_async(move || async move {
                    // ❌ Thread 2: Lock B then A = DEADLOCK!
                    let _b = lock_b2.lock().unwrap();
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    let _a = lock_a2.lock().unwrap();
                    None
                }),
            ])
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    }
}

/// ✅ DO: Use message passing to avoid locks entirely
mod correct_no_locks {
    use super::*;
    
    #[derive(Clone)]
    enum Msg {
        UpdateA(i32),
        UpdateB(i32),
        UpdateBoth(i32, i32),
    }
    
    struct GoodModel {
        // ✅ No locks needed!
        value_a: i32,
        value_b: i32,
    }
    
    impl Model for GoodModel {
        type Message = Msg;
        
        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }
        
        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(Msg::UpdateA(val)) => {
                    // ✅ No locks, no deadlock possible
                    self.value_a = val;
                    Cmd::none()
                }
                Event::User(Msg::UpdateB(val)) => {
                    self.value_b = val;
                    Cmd::none()
                }
                Event::User(Msg::UpdateBoth(a, b)) => {
                    // ✅ Atomic update of both values
                    self.value_a = a;
                    self.value_b = b;
                    Cmd::none()
                }
                _ => Cmd::none(),
            }
        }
        
        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    }
}

fn main() {
    println!("This file demonstrates anti-patterns - DO NOT run!");
    println!("See examples/safe_concurrency.rs for correct patterns.");
}