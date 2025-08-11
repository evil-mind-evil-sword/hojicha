//! Priority-aware event processing with automatic event prioritization
//!
//! This module implements the default event processing with built-in prioritization
//! to ensure UI responsiveness even under heavy event load.

use hojicha_core::core::Message;
use hojicha_core::event::Event;
use crate::metrics::{AdvancedEventStats, MetricsCollector, MetricsConfig};
use crate::priority_queue::{Priority, PriorityEventQueue, ResizeError};
use crate::queue_scaling::{AutoScaleConfig, QueueAutoScaler, ScalingDecision};
use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use log::{debug, info, trace, warn};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Type alias for custom priority mapping function
type PriorityMapper = Arc<dyn Fn(&Event<()>) -> Priority + Send + Sync>;

/// Statistics for monitoring event processing behavior
#[derive(Debug, Clone, Default)]
pub struct EventStats {
    /// Total number of events processed
    pub total_events: usize,
    /// Number of high priority events processed
    pub high_priority_events: usize,
    /// Number of normal priority events processed
    pub normal_priority_events: usize,
    /// Number of low priority events processed
    pub low_priority_events: usize,
    /// Number of events dropped due to queue overflow
    pub dropped_events: usize,
    /// Total processing time in milliseconds
    pub processing_time_ms: u128,
    /// Maximum queue size reached during processing
    pub queue_size_max: usize,
}

/// Configuration for priority event processing
#[derive(Clone)]
pub struct PriorityConfig {
    /// Maximum queue size (default: 1000)
    pub max_queue_size: usize,
    /// Whether to log dropped events (default: true)
    pub log_drops: bool,
    /// Custom priority mapper (default: None, uses automatic detection)
    pub priority_mapper: Option<PriorityMapper>,
}

impl std::fmt::Debug for PriorityConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PriorityConfig")
            .field("max_queue_size", &self.max_queue_size)
            .field("log_drops", &self.log_drops)
            .field("priority_mapper", &self.priority_mapper.is_some())
            .finish()
    }
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            log_drops: true,
            priority_mapper: None,
        }
    }
}

/// Priority-aware event processor
pub struct PriorityEventProcessor<M: Message> {
    queue: Arc<Mutex<PriorityEventQueue<M>>>,
    stats: Arc<Mutex<EventStats>>,
    config: PriorityConfig,
    metrics: Arc<MetricsCollector>,
    auto_scaler: Option<Arc<Mutex<QueueAutoScaler>>>,
}

impl<M: Message> PriorityEventProcessor<M> {
    /// Create a new priority event processor with default configuration
    pub fn new() -> Self {
        Self::with_config(PriorityConfig::default())
    }

    /// Create a new priority event processor with custom configuration
    pub fn with_config(config: PriorityConfig) -> Self {
        debug!(
            "Initializing PriorityEventProcessor with max_queue_size: {}",
            config.max_queue_size
        );

        let metrics_config = MetricsConfig {
            track_percentiles: true,
            track_by_type: true,
            sampling_rate: 1.0,
            max_histogram_size: 100_000,
            rate_window: Duration::from_secs(60),
        };

        Self {
            queue: Arc::new(Mutex::new(PriorityEventQueue::new(config.max_queue_size))),
            stats: Arc::new(Mutex::new(EventStats::default())),
            config,
            metrics: Arc::new(MetricsCollector::new(metrics_config)),
            auto_scaler: None,
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> EventStats {
        self.stats.lock().unwrap().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.lock().unwrap() = EventStats::default();
        self.metrics.reset();
    }

    /// Get advanced metrics snapshot
    pub fn advanced_metrics(&self) -> AdvancedEventStats {
        self.metrics.snapshot()
    }

    /// Get metrics collector for external monitoring
    pub fn metrics_collector(&self) -> Arc<MetricsCollector> {
        self.metrics.clone()
    }

    /// Enable auto-scaling with the given configuration
    pub fn enable_auto_scaling(&mut self, config: AutoScaleConfig) {
        info!("Enabling auto-scaling with config: {:?}", config);
        self.auto_scaler = Some(Arc::new(Mutex::new(QueueAutoScaler::new(config))));
    }

    /// Disable auto-scaling
    pub fn disable_auto_scaling(&mut self) {
        info!("Disabling auto-scaling");
        self.auto_scaler = None;
    }

    /// Manually resize the queue
    pub fn resize_queue(&self, new_size: usize) -> Result<(), ResizeError> {
        let mut queue = self.queue.lock().unwrap();
        let old_size = queue.capacity();
        queue.resize(new_size)?;
        info!("Queue resized from {} to {}", old_size, new_size);
        Ok(())
    }

    /// Get current queue capacity
    pub fn queue_capacity(&self) -> usize {
        self.queue.lock().unwrap().capacity()
    }

    /// Push an event to the queue
    pub fn push(&self, event: Event<M>) -> Result<(), Event<M>> {
        let priority = self.detect_priority(&event);

        trace!(
            "Pushing event with priority {:?}: {:?}",
            priority,
            std::mem::discriminant(&event)
        );

        let mut queue = self.queue.lock().unwrap();
        let result = queue.push(event);

        // Update statistics
        let mut stats = self.stats.lock().unwrap();
        stats.total_events += 1;

        match priority {
            Priority::High => stats.high_priority_events += 1,
            Priority::Normal => stats.normal_priority_events += 1,
            Priority::Low => stats.low_priority_events += 1,
        }

        let current_size = queue.len();
        let capacity = self.config.max_queue_size;
        if current_size > stats.queue_size_max {
            stats.queue_size_max = current_size;
        }

        // Update metrics
        self.metrics.update_queue_depth(current_size, capacity);

        if result.is_err() {
            stats.dropped_events += 1;
            self.metrics.record_dropped();
            if self.config.log_drops {
                warn!(
                    "Dropped event due to queue overflow (priority: {:?})",
                    priority
                );
            }
        }

        // Log if queue is getting full
        if queue.is_backpressure_active() {
            debug!("Queue backpressure active: {} events queued", current_size);
            self.metrics.record_backpressure();
        }

        result
    }

    /// Pop the highest priority event from the queue
    pub fn pop(&self) -> Option<Event<M>> {
        let start = Instant::now();
        let event = self.queue.lock().unwrap().pop();

        if let Some(ref e) = event {
            let elapsed = start.elapsed();
            let mut stats = self.stats.lock().unwrap();
            stats.processing_time_ms += elapsed.as_millis();

            // Record metrics for the event
            let priority = Priority::from_event(e);
            let event_type = match e {
                Event::Quit => Some("quit"),
                Event::Key(_) => Some("key"),
                Event::Mouse(_) => Some("mouse"),
                Event::User(_) => Some("user"),
                Event::Resize { .. } => Some("resize"),
                Event::Tick => Some("tick"),
                Event::Paste(_) => Some("paste"),
                Event::Focus => Some("focus"),
                Event::Blur => Some("blur"),
                Event::Suspend => Some("suspend"),
                Event::Resume => Some("resume"),
                Event::ExecProcess => Some("exec"),
            };

            self.metrics.record_event(priority, elapsed, event_type);

            // Check if auto-scaling should be triggered
            if let Some(ref auto_scaler) = self.auto_scaler {
                let mut scaler = auto_scaler.lock().unwrap();
                let mut queue = self.queue.lock().unwrap();

                if let Some(decision) = scaler.on_event_processed(&mut queue) {
                    match decision {
                        ScalingDecision::Grow(amount) => {
                            debug!("Auto-scaling: Growing queue by {}", amount);
                        }
                        ScalingDecision::Shrink(amount) => {
                            debug!("Auto-scaling: Shrinking queue by {}", amount);
                        }
                        ScalingDecision::NoChange => {}
                    }
                }
            }
        }

        event
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    /// Get the current queue size
    pub fn queue_size(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    /// Detect the priority of an event
    fn detect_priority(&self, event: &Event<M>) -> Priority {
        // Use custom mapper if provided
        if let Some(ref mapper) = self.config.priority_mapper {
            // We need to transmute to Event<()> for the mapper
            // This is safe because we only look at the discriminant, not the data
            let event_ref = unsafe { std::mem::transmute::<&Event<M>, &Event<()>>(event) };
            return mapper(event_ref);
        }

        // Default automatic priority detection
        Priority::from_event(event)
    }

    /// Process events from multiple sources with priority handling
    pub fn process_events(
        &self,
        message_rx: &mpsc::Receiver<Event<M>>,
        crossterm_rx: &mpsc::Receiver<CrosstermEvent>,
        tick_rate: Duration,
    ) -> Option<Event<M>> {
        trace!("Processing events, queue size: {}", self.queue_size());

        // First, drain all available events into the priority queue
        self.drain_channels(message_rx, crossterm_rx);

        // If we have events in the queue, return the highest priority one
        if let Some(event) = self.pop() {
            debug!(
                "Returning event from queue: {:?}",
                std::mem::discriminant(&event)
            );
            return Some(event);
        }

        // No events available, wait for new ones with timeout
        match crossterm_rx.recv_timeout(tick_rate) {
            Ok(ct_event) => self.handle_crossterm_event(ct_event, crossterm_rx),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Check for messages one more time
                if let Ok(msg) = message_rx.try_recv() {
                    Some(msg)
                } else {
                    trace!("Generating tick event");
                    Some(Event::Tick)
                }
            }
            Err(_) => None,
        }
    }

    /// Process events in headless mode (no terminal events)
    pub fn process_events_headless(
        &self,
        message_rx: &mpsc::Receiver<Event<M>>,
        tick_rate: Duration,
    ) -> Option<Event<M>> {
        // Drain all available messages into the priority queue
        while let Ok(msg) = message_rx.try_recv() {
            let _ = self.push(msg);
        }

        // Return highest priority event if available
        if let Some(event) = self.pop() {
            return Some(event);
        }

        // Wait for new messages with timeout
        match message_rx.recv_timeout(tick_rate) {
            Ok(msg) => Some(msg),
            Err(mpsc::RecvTimeoutError::Timeout) => Some(Event::Tick),
            Err(_) => None,
        }
    }

    /// Drain all available events from channels into the priority queue
    fn drain_channels(
        &self,
        message_rx: &mpsc::Receiver<Event<M>>,
        crossterm_rx: &mpsc::Receiver<CrosstermEvent>,
    ) {
        // Drain all available user messages
        while let Ok(msg) = message_rx.try_recv() {
            if self.push(msg).is_err() {
                break; // Queue is full
            }
        }

        // Drain all available terminal events
        while let Ok(ct_event) = crossterm_rx.try_recv() {
            if let Some(event) = Self::convert_crossterm_event(ct_event) {
                // Unsafe transmute to add the message type
                // This is safe because Event<()> and Event<M> have the same layout
                let typed_event = unsafe { std::mem::transmute_copy(&event) };
                if self.push(typed_event).is_err() {
                    break; // Queue is full
                }
            }
        }
    }

    /// Handle a crossterm event, with special handling for resize events
    fn handle_crossterm_event(
        &self,
        ct_event: CrosstermEvent,
        crossterm_rx: &mpsc::Receiver<CrosstermEvent>,
    ) -> Option<Event<M>> {
        match ct_event {
            CrosstermEvent::Resize(width, height) => {
                // Coalesce multiple resize events
                let (final_width, final_height) =
                    Self::coalesce_resize_events(width, height, crossterm_rx);
                Some(Event::Resize {
                    width: final_width,
                    height: final_height,
                })
            }
            _ => Self::convert_crossterm_event(ct_event)
                .map(|e| unsafe { std::mem::transmute_copy(&e) }),
        }
    }

    /// Convert a crossterm event to a hojicha event
    fn convert_crossterm_event(event: CrosstermEvent) -> Option<Event<()>> {
        match event {
            CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => {
                Some(Event::Key(key.into()))
            }
            CrosstermEvent::Mouse(mouse) => Some(Event::Mouse(mouse.into())),
            CrosstermEvent::Resize(width, height) => Some(Event::Resize { width, height }),
            CrosstermEvent::Paste(data) => Some(Event::Paste(data)),
            CrosstermEvent::FocusGained => Some(Event::Focus),
            CrosstermEvent::FocusLost => Some(Event::Blur),
            _ => None,
        }
    }

    /// Coalesce multiple resize events into one
    fn coalesce_resize_events(
        initial_width: u16,
        initial_height: u16,
        rx: &mpsc::Receiver<CrosstermEvent>,
    ) -> (u16, u16) {
        let mut width = initial_width;
        let mut height = initial_height;

        // Drain any additional resize events
        while let Ok(CrosstermEvent::Resize(w, h)) = rx.try_recv() {
            width = w;
            height = h;
        }

        debug!("Coalesced resize events to {}x{}", width, height);
        (width, height)
    }
}

impl<M: Message> Default for PriorityEventProcessor<M> {
    fn default() -> Self {
        Self::new()
    }
}

/// Public API for getting event processing statistics
pub fn get_event_stats<M: Message>(processor: &PriorityEventProcessor<M>) -> String {
    let stats = processor.stats();
    format!(
        "Events: {} total ({} high, {} normal, {} low), {} dropped, max queue: {}",
        stats.total_events,
        stats.high_priority_events,
        stats.normal_priority_events,
        stats.low_priority_events,
        stats.dropped_events,
        stats.queue_size_max
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestMsg(String);

    #[test]
    fn test_priority_processor_creation() {
        let processor: PriorityEventProcessor<TestMsg> = PriorityEventProcessor::new();
        assert_eq!(processor.queue_size(), 0);
        assert!(processor.is_empty());
    }

    #[test]
    fn test_event_prioritization() {
        let processor: PriorityEventProcessor<TestMsg> = PriorityEventProcessor::new();

        // Add events in reverse priority order
        processor.push(Event::Tick).unwrap();
        processor
            .push(Event::User(TestMsg("normal".to_string())))
            .unwrap();
        processor.push(Event::Quit).unwrap();

        // Should get them back in priority order
        assert_eq!(processor.pop(), Some(Event::Quit));
        assert_eq!(
            processor.pop(),
            Some(Event::User(TestMsg("normal".to_string())))
        );
        assert_eq!(processor.pop(), Some(Event::Tick));
    }

    #[test]
    fn test_statistics_tracking() {
        let processor: PriorityEventProcessor<TestMsg> = PriorityEventProcessor::new();

        processor.push(Event::Quit).unwrap();
        processor
            .push(Event::User(TestMsg("test".to_string())))
            .unwrap();
        processor.push(Event::Tick).unwrap();

        let stats = processor.stats();
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.high_priority_events, 1);
        assert_eq!(stats.normal_priority_events, 1);
        assert_eq!(stats.low_priority_events, 1);
    }

    #[test]
    fn test_custom_priority_mapper() {
        let config = PriorityConfig {
            max_queue_size: 100,
            log_drops: false,
            priority_mapper: Some(Arc::new(|_event| {
                // Make everything high priority for testing
                Priority::High
            })),
        };

        let processor: PriorityEventProcessor<TestMsg> =
            PriorityEventProcessor::with_config(config);

        processor.push(Event::Tick).unwrap();
        let stats = processor.stats();
        assert_eq!(stats.high_priority_events, 1);
        assert_eq!(stats.low_priority_events, 0);
    }
}
