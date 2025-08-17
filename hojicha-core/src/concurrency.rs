//! Concurrency safety utilities and patterns
//! 
//! This module provides utilities for safe concurrent programming in Hojicha applications.
//! 
//! ## Key Principles
//! 
//! 1. **Message passing over shared state**: Use messages to communicate between tasks
//! 2. **Request tracking**: Track async operations with unique IDs
//! 3. **Cancellation support**: Cancel operations when they're no longer needed
//! 4. **State machines**: Use enums to represent valid states and transitions

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Unique identifier for async requests
/// 
/// Use this to track async operations and match responses to requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(u64);

impl RequestId {
    /// Create a new unique request ID
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
    
    /// Get the underlying ID value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request#{}", self.0)
    }
}

/// Tracks pending async requests
/// 
/// Use this to manage concurrent operations and prevent processing
/// responses from cancelled or outdated requests.
/// 
/// # Example
/// ```
/// use hojicha_core::concurrency::{RequestTracker, RequestId};
/// 
/// let mut tracker = RequestTracker::new();
/// 
/// // Start a request
/// let id = RequestId::new();
/// tracker.track(id);
/// 
/// // Later, when response arrives
/// if tracker.complete(id) {
///     // Process valid response
/// } else {
///     // Ignore cancelled/unknown request
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RequestTracker {
    pending: HashMap<RequestId, RequestInfo>,
}

#[derive(Debug, Clone)]
struct RequestInfo {
    started_at: Instant,
}

impl RequestTracker {
    /// Create a new request tracker
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }
    
    /// Track a new request
    pub fn track(&mut self, id: RequestId) {
        self.pending.insert(id, RequestInfo {
            started_at: Instant::now(),
        });
    }
    
    /// Complete a request and return true if it was being tracked
    pub fn complete(&mut self, id: RequestId) -> bool {
        self.pending.remove(&id).is_some()
    }
    
    /// Check if a request is currently being tracked
    pub fn is_pending(&self, id: RequestId) -> bool {
        self.pending.contains_key(&id)
    }
    
    /// Cancel a specific request
    pub fn cancel(&mut self, id: RequestId) -> bool {
        self.pending.remove(&id).is_some()
    }
    
    /// Cancel all pending requests
    pub fn cancel_all(&mut self) {
        self.pending.clear();
    }
    
    /// Get the number of pending requests
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    
    /// Get how long a request has been pending
    pub fn elapsed(&self, id: RequestId) -> Option<std::time::Duration> {
        self.pending.get(&id).map(|info| info.started_at.elapsed())
    }
    
    /// Get all pending request IDs
    pub fn pending_ids(&self) -> Vec<RequestId> {
        self.pending.keys().copied().collect()
    }
}

impl Default for RequestTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Generator for unique request IDs
/// 
/// Use this when you need to generate multiple IDs in a structured way.
/// 
/// # Example
/// ```
/// use hojicha_core::concurrency::RequestIdGenerator;
/// 
/// let mut gen = RequestIdGenerator::new();
/// let id1 = gen.next();
/// let id2 = gen.next();
/// assert_ne!(id1, id2);
/// ```
#[derive(Debug, Clone)]
pub struct RequestIdGenerator {
    next_id: u64,
}

impl RequestIdGenerator {
    /// Create a new ID generator
    pub fn new() -> Self {
        Self { next_id: 1 }
    }
    
    /// Generate the next unique ID
    pub fn next(&mut self) -> RequestId {
        let id = RequestId(self.next_id);
        self.next_id += 1;
        id
    }
    
    /// Get the next ID without incrementing
    pub fn peek(&self) -> RequestId {
        RequestId(self.next_id)
    }
}

impl Default for RequestIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// State machine helper for managing application states
/// 
/// This trait helps enforce valid state transitions.
pub trait StateMachine: Sized {
    /// The event type that triggers state transitions
    type Event;
    
    /// Attempt to transition to a new state based on an event
    /// 
    /// Returns `Some(new_state)` if the transition is valid,
    /// or `None` if the transition is invalid.
    fn transition(self, event: Self::Event) -> Option<Self>;
    
    /// Check if a transition would be valid without consuming self
    fn can_transition(&self, event: &Self::Event) -> bool;
}

/// Example state machine implementation
/// 
/// ```
/// use hojicha_core::concurrency::StateMachine;
/// 
/// #[derive(Debug, Clone)]
/// enum AppState {
///     Idle,
///     Loading,
///     Ready,
///     Error,
/// }
/// 
/// #[derive(Debug)]
/// enum AppEvent {
///     StartLoad,
///     LoadSuccess,
///     LoadError,
///     Reset,
/// }
/// 
/// impl StateMachine for AppState {
///     type Event = AppEvent;
///     
///     fn transition(self, event: Self::Event) -> Option<Self> {
///         match (self, event) {
///             (AppState::Idle, AppEvent::StartLoad) => Some(AppState::Loading),
///             (AppState::Loading, AppEvent::LoadSuccess) => Some(AppState::Ready),
///             (AppState::Loading, AppEvent::LoadError) => Some(AppState::Error),
///             (_, AppEvent::Reset) => Some(AppState::Idle),
///             _ => None, // Invalid transition
///         }
///     }
///     
///     fn can_transition(&self, event: &Self::Event) -> bool {
///         matches!(
///             (self, event),
///             (AppState::Idle, AppEvent::StartLoad) |
///             (AppState::Loading, AppEvent::LoadSuccess) |
///             (AppState::Loading, AppEvent::LoadError) |
///             (_, AppEvent::Reset)
///         )
///     }
/// }
/// ```
pub struct StateTransition<S, E> {
    from: S,
    to: S,
    event: E,
}

impl<S, E> StateTransition<S, E> {
    /// Create a new state transition
    pub fn new(from: S, to: S, event: E) -> Self {
        Self { from, to, event }
    }
}

/// Safe message channel for actor-like patterns
/// 
/// This provides a type-safe way to implement actor patterns without
/// shared mutable state.
#[cfg(feature = "tokio")]
pub mod actor {
    use tokio::sync::mpsc;
    
    /// Actor handle for sending messages
    pub struct ActorHandle<M> {
        sender: mpsc::Sender<M>,
    }
    
    impl<M> ActorHandle<M> {
        /// Send a message to the actor
        pub async fn send(&self, msg: M) -> Result<(), mpsc::error::SendError<M>> {
            self.sender.send(msg).await
        }
        
        /// Try to send a message without blocking
        pub fn try_send(&self, msg: M) -> Result<(), mpsc::error::TrySendError<M>> {
            self.sender.try_send(msg)
        }
    }
    
    impl<M> Clone for ActorHandle<M> {
        fn clone(&self) -> Self {
            Self {
                sender: self.sender.clone(),
            }
        }
    }
    
    /// Create an actor channel
    pub fn channel<M>(buffer: usize) -> (ActorHandle<M>, mpsc::Receiver<M>) {
        let (tx, rx) = mpsc::channel(buffer);
        (ActorHandle { sender: tx }, rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_request_id_uniqueness() {
        let id1 = RequestId::new();
        let id2 = RequestId::new();
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_request_tracker() {
        let mut tracker = RequestTracker::new();
        let id = RequestId::new();
        
        // Track a request
        tracker.track(id);
        assert!(tracker.is_pending(id));
        assert_eq!(tracker.pending_count(), 1);
        
        // Complete the request
        assert!(tracker.complete(id));
        assert!(!tracker.is_pending(id));
        assert_eq!(tracker.pending_count(), 0);
        
        // Completing again returns false
        assert!(!tracker.complete(id));
    }
    
    #[test]
    fn test_request_id_generator() {
        let mut gen = RequestIdGenerator::new();
        
        let id1 = gen.next();
        let id2 = gen.next();
        let id3 = gen.next();
        
        assert_eq!(id1.value(), 1);
        assert_eq!(id2.value(), 2);
        assert_eq!(id3.value(), 3);
    }
    
    #[test]
    fn test_state_machine_example() {
        #[derive(Debug, Clone, PartialEq)]
        enum State {
            A,
            B,
            C,
        }
        
        #[derive(Debug)]
        enum Event {
            Next,
            Reset,
        }
        
        impl StateMachine for State {
            type Event = Event;
            
            fn transition(self, event: Self::Event) -> Option<Self> {
                match (self, event) {
                    (State::A, Event::Next) => Some(State::B),
                    (State::B, Event::Next) => Some(State::C),
                    (_, Event::Reset) => Some(State::A),
                    _ => None,
                }
            }
            
            fn can_transition(&self, event: &Self::Event) -> bool {
                matches!(
                    (self, event),
                    (State::A, Event::Next) |
                    (State::B, Event::Next) |
                    (_, Event::Reset)
                )
            }
        }
        
        let state = State::A;
        assert!(state.can_transition(&Event::Next));
        
        let state = state.transition(Event::Next).unwrap();
        assert_eq!(state, State::B);
        
        let state = state.transition(Event::Next).unwrap();
        assert_eq!(state, State::C);
        
        let state = state.transition(Event::Reset).unwrap();
        assert_eq!(state, State::A);
    }
}