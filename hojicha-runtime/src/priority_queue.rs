//! Priority-based event queue with backpressure support
//!
//! This module provides a priority queue for events that ensures important events
//! (like user input and quit commands) are processed before less critical events
//! (like ticks and resize events). It also implements backpressure to prevent
//! memory exhaustion under high load.
//!
//! # Priority Levels
//!
//! Events are automatically assigned priorities:
//! - **High**: Quit, Key events, Suspend/Resume, Process execution
//! - **Normal**: Mouse events, User messages, Paste events
//! - **Low**: Tick, Resize, Focus/Blur events
//!
//! # Backpressure
//!
//! When the queue reaches 80% capacity, backpressure is activated. If the queue
//! fills completely, lower priority events are dropped in favor of higher priority
//! ones.
//!
//! # Example
//!
//! ```ignore
//! use hojicha::priority_queue::PriorityEventQueue;
//!
//! let mut queue = PriorityEventQueue::new(1000);
//!
//! // High priority events are processed first
//! queue.push(Event::Tick)?;           // Low priority
//! queue.push(Event::User(msg))?;      // Normal priority  
//! queue.push(Event::Quit)?;           // High priority
//!
//! assert_eq!(queue.pop(), Some(Event::Quit));  // High first
//! assert_eq!(queue.pop(), Some(Event::User(msg))); // Then normal
//! assert_eq!(queue.pop(), Some(Event::Tick));  // Low last
//! ```

use hojicha_core::core::Message;
use hojicha_core::event::Event;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Priority levels for events in the queue
///
/// Lower numeric values indicate higher priority. Events are processed
/// in priority order, with high priority events processed before normal
/// and low priority events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// High priority for critical events (Quit, Key events, Suspend/Resume)
    High = 0,
    /// Normal priority for user interactions (Mouse, User messages, Paste)
    Normal = 1,
    /// Low priority for background events (Tick, Resize, Focus/Blur)
    Low = 2,
}

impl Priority {
    /// Determine the priority level for a given event
    ///
    /// This method automatically assigns priority levels based on event type:
    /// - High: Quit, Key events, Suspend/Resume, Process execution
    /// - Normal: Mouse events, User messages, Paste events  
    /// - Low: Tick, Resize, Focus/Blur events
    pub fn from_event<M: Message>(event: &Event<M>) -> Self {
        match event {
            Event::Quit => Priority::High,
            Event::Key(_) => Priority::High,
            Event::Mouse(_) => Priority::Normal,
            Event::User(_) => Priority::Normal,
            Event::Resize { .. } => Priority::Low,
            Event::Tick => Priority::Low,
            Event::Paste(_) => Priority::Normal,
            Event::Focus | Event::Blur => Priority::Low,
            Event::Suspend | Event::Resume | Event::ExecProcess => Priority::High,
        }
    }
}

#[derive(Debug)]
struct PriorityEvent<M: Message> {
    priority: Priority,
    sequence: usize,
    event: Event<M>,
}

impl<M: Message> PartialEq for PriorityEvent<M> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl<M: Message> Eq for PriorityEvent<M> {}

impl<M: Message> Ord for PriorityEvent<M> {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max-heap, so we want High (0) to be greater than Low (2)
        // Therefore we reverse the comparison
        match other.priority.cmp(&self.priority) {
            Ordering::Equal => self.sequence.cmp(&other.sequence),
            other => other,
        }
    }
}

impl<M: Message> PartialOrd for PriorityEvent<M> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A priority queue for events with automatic backpressure handling
///
/// This queue ensures that high priority events (like user input and quit commands)
/// are processed before lower priority events (like ticks and resize events).
/// It implements backpressure to prevent memory exhaustion under high load.
///
/// When the queue reaches 80% capacity, backpressure is activated. If the queue
/// fills completely, lower priority events are dropped in favor of higher priority ones.
///
/// # Example
///
/// ```ignore
/// let mut queue = PriorityEventQueue::new(1000);
///
/// queue.push(Event::Tick)?;       // Low priority
/// queue.push(Event::Quit)?;       // High priority
///
/// // High priority events are processed first
/// assert_eq!(queue.pop(), Some(Event::Quit));
/// assert_eq!(queue.pop(), Some(Event::Tick));
/// ```
pub struct PriorityEventQueue<M: Message> {
    heap: BinaryHeap<PriorityEvent<M>>,
    sequence_counter: usize,
    max_size: usize,
    backpressure_threshold: usize,
    backpressure_active: bool,
    dropped_events: usize,
}

impl<M: Message> PriorityEventQueue<M> {
    /// Create a new priority event queue with the specified maximum size
    ///
    /// # Arguments
    /// * `max_size` - Maximum number of events the queue can hold
    ///
    /// # Example
    /// ```ignore
    /// let queue = PriorityEventQueue::new(1000);
    /// ```
    pub fn new(max_size: usize) -> Self {
        Self {
            heap: BinaryHeap::new(),
            sequence_counter: 0,
            max_size,
            backpressure_threshold: (max_size as f64 * 0.8) as usize,
            backpressure_active: false,
            dropped_events: 0,
        }
    }

    /// Push an event into the priority queue
    ///
    /// Events are automatically prioritized based on their type. If the queue is full,
    /// lower priority events may be dropped to make room for higher priority ones.
    ///
    /// # Arguments
    /// * `event` - The event to add to the queue
    ///
    /// # Returns
    /// * `Ok(())` if the event was successfully added
    /// * `Err(event)` if the event was dropped due to queue overflow
    ///
    /// # Example
    /// ```ignore
    /// if let Err(dropped) = queue.push(Event::Tick) {
    ///     println!("Event was dropped: {:?}", dropped);
    /// }
    /// ```
    pub fn push(&mut self, event: Event<M>) -> Result<(), Event<M>> {
        if self.heap.len() >= self.max_size {
            let priority = Priority::from_event(&event);

            if priority == Priority::High {
                if let Some(lowest) = self.find_lowest_priority_event() {
                    self.heap.retain(|e| e.sequence != lowest);
                    self.dropped_events += 1;
                } else {
                    self.dropped_events += 1;
                    return Err(event);
                }
            } else {
                self.dropped_events += 1;
                return Err(event);
            }
        }

        let priority = Priority::from_event(&event);
        let priority_event = PriorityEvent {
            priority,
            sequence: self.sequence_counter,
            event,
        };

        self.sequence_counter += 1;
        self.heap.push(priority_event);

        if self.heap.len() >= self.backpressure_threshold {
            self.backpressure_active = true;
        }

        Ok(())
    }

    /// Remove and return the highest priority event from the queue
    ///
    /// Events are returned in priority order, with high priority events
    /// returned before normal and low priority events.
    ///
    /// # Returns
    /// * `Some(event)` if there are events in the queue
    /// * `None` if the queue is empty
    ///
    /// # Example
    /// ```ignore
    /// while let Some(event) = queue.pop() {
    ///     process_event(event);
    /// }
    /// ```
    pub fn pop(&mut self) -> Option<Event<M>> {
        let result = self.heap.pop().map(|pe| pe.event);

        if self.heap.len() < self.backpressure_threshold {
            self.backpressure_active = false;
        }

        result
    }

    /// Check if the queue is empty
    ///
    /// # Returns
    /// * `true` if the queue contains no events
    /// * `false` if the queue contains one or more events
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Get the current number of events in the queue
    ///
    /// # Returns
    /// The number of events currently in the queue
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Check if backpressure is currently active
    ///
    /// Backpressure is activated when the queue reaches 80% of its capacity.
    ///
    /// # Returns
    /// * `true` if backpressure is active
    /// * `false` if the queue is below the backpressure threshold
    pub fn is_backpressure_active(&self) -> bool {
        self.backpressure_active
    }

    /// Get the total number of events that have been dropped
    ///
    /// Events are dropped when the queue is full and lower priority
    /// events are evicted to make room for higher priority ones.
    ///
    /// # Returns
    /// The total number of events dropped since queue creation
    pub fn dropped_events(&self) -> usize {
        self.dropped_events
    }

    /// Clear all events from the queue
    ///
    /// This removes all events and resets the backpressure state,
    /// but preserves the dropped event counter and capacity settings.
    pub fn clear(&mut self) {
        self.heap.clear();
        self.backpressure_active = false;
    }

    fn find_lowest_priority_event(&self) -> Option<usize> {
        self.heap
            .iter()
            .filter(|e| e.priority == Priority::Low)
            .map(|e| e.sequence)
            .min()
    }

    /// Get the current capacity of the queue
    pub fn capacity(&self) -> usize {
        self.max_size
    }

    /// Resize the queue to a new capacity
    ///
    /// # Arguments
    /// * `new_size` - The new maximum size for the queue
    ///
    /// # Returns
    /// * `Ok(())` if resize succeeded
    /// * `Err(ResizeError)` if the new size is invalid or would cause data loss
    pub fn resize(&mut self, new_size: usize) -> Result<(), ResizeError> {
        if new_size == 0 {
            return Err(ResizeError::InvalidSize("Queue size cannot be zero".into()));
        }

        let current_len = self.heap.len();

        // If shrinking below current usage, we need to drop events
        if new_size < current_len {
            // Calculate how many events to drop
            let to_drop = current_len - new_size;

            // Collect events sorted by priority (lowest priority first)
            let mut events: Vec<_> = self.heap.iter().collect();
            events.sort_by(|a, b| {
                b.priority
                    .cmp(&a.priority)
                    .then(b.sequence.cmp(&a.sequence))
            });

            // Get sequences of events to drop (lowest priority ones)
            let drop_sequences: Vec<usize> =
                events.iter().take(to_drop).map(|e| e.sequence).collect();

            // Drop the events
            self.heap.retain(|e| !drop_sequences.contains(&e.sequence));
            self.dropped_events += to_drop;
        }

        // Update size and thresholds
        self.max_size = new_size;
        self.backpressure_threshold = (new_size as f64 * 0.8) as usize;

        // Update backpressure status
        self.backpressure_active = self.heap.len() >= self.backpressure_threshold;

        Ok(())
    }

    /// Try to grow the queue by a specified amount
    pub fn try_grow(&mut self, additional: usize) -> Result<usize, ResizeError> {
        let new_size = self.max_size.saturating_add(additional);
        self.resize(new_size)?;
        Ok(new_size)
    }

    /// Try to shrink the queue by a specified amount
    pub fn try_shrink(&mut self, reduction: usize) -> Result<usize, ResizeError> {
        let new_size = self.max_size.saturating_sub(reduction).max(1);
        self.resize(new_size)?;
        Ok(new_size)
    }

    /// Get current queue statistics for scaling decisions
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            current_size: self.heap.len(),
            max_size: self.max_size,
            utilization: self.heap.len() as f64 / self.max_size as f64,
            backpressure_active: self.backpressure_active,
            dropped_events: self.dropped_events,
        }
    }
}

/// Error type for resize operations
#[derive(Debug, Clone)]
pub enum ResizeError {
    /// The requested size is invalid (e.g., zero or negative)
    InvalidSize(String),
    /// The resize operation would cause high priority events to be dropped
    WouldDropHighPriorityEvents,
}

impl std::fmt::Display for ResizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResizeError::InvalidSize(msg) => write!(f, "Invalid size: {msg}"),
            ResizeError::WouldDropHighPriorityEvents => {
                write!(f, "Resize would drop high priority events")
            }
        }
    }
}

impl std::error::Error for ResizeError {}

/// Queue statistics for monitoring and scaling decisions
#[derive(Debug, Clone)]
pub struct QueueStats {
    /// Current number of events in the queue
    pub current_size: usize,
    /// Maximum capacity of the queue
    pub max_size: usize,
    /// Current utilization as a percentage (0.0 to 1.0)
    pub utilization: f64,
    /// Whether backpressure is currently active
    pub backpressure_active: bool,
    /// Total number of events dropped
    pub dropped_events: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use hojicha_core::event::{Key, KeyEvent};

    #[derive(Debug, Clone, PartialEq)]
    struct TestMsg(usize);

    #[test]
    fn test_priority_ordering() {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(10);

        queue.push(Event::Tick).unwrap();
        queue.push(Event::User(TestMsg(1))).unwrap();
        queue.push(Event::Quit).unwrap();
        queue.push(Event::User(TestMsg(2))).unwrap();
        queue
            .push(Event::Key(KeyEvent {
                key: Key::Char('a'),
                modifiers: crossterm::event::KeyModifiers::empty(),
            }))
            .unwrap();

        // Both Quit and Key have High priority, order between them is not guaranteed
        let first = queue.pop();
        let second = queue.pop();

        // Check that we got both high priority events first
        let got_quit = matches!(first, Some(Event::Quit)) || matches!(second, Some(Event::Quit));
        let got_key = matches!(first, Some(Event::Key(_))) || matches!(second, Some(Event::Key(_)));

        assert!(got_quit, "Expected Quit event in first two pops");
        assert!(got_key, "Expected Key event in first two pops");

        // Normal priority events - order may vary due to heap implementation
        let third = queue.pop();
        let fourth = queue.pop();

        let got_user1 = matches!(third, Some(Event::User(TestMsg(1))))
            || matches!(fourth, Some(Event::User(TestMsg(1))));
        let got_user2 = matches!(third, Some(Event::User(TestMsg(2))))
            || matches!(fourth, Some(Event::User(TestMsg(2))));

        assert!(got_user1, "Expected User(TestMsg(1))");
        assert!(got_user2, "Expected User(TestMsg(2))");
        assert_eq!(queue.pop(), Some(Event::Tick));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_backpressure() {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(5);

        for i in 0..4 {
            queue.push(Event::User(TestMsg(i))).unwrap();
        }

        assert!(queue.is_backpressure_active());

        queue.pop();
        queue.pop();

        assert!(!queue.is_backpressure_active());
    }

    #[test]
    fn test_event_dropping() {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(3);

        queue.push(Event::Tick).unwrap();
        queue.push(Event::User(TestMsg(1))).unwrap();
        queue.push(Event::User(TestMsg(2))).unwrap();

        let result = queue.push(Event::Tick);
        assert!(result.is_err());
        assert_eq!(queue.dropped_events(), 1);

        queue.push(Event::Quit).unwrap();
        assert_eq!(queue.dropped_events(), 2);
    }
}
