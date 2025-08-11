//! Safe priority detection without unsafe transmutes
//!
//! This module provides a safe way to detect event priority without
//! using unsafe memory transmutation.

use crate::priority_queue::Priority;
use hojicha_core::core::Message;
use hojicha_core::event::Event;

/// Detect the priority of an event without unsafe operations
pub fn detect_priority<M: Message>(event: &Event<M>) -> Priority {
    // Use pattern matching instead of transmutes
    match event {
        Event::Quit => Priority::High,
        Event::Key(_) => Priority::High,
        Event::Suspend => Priority::High,
        Event::Resume => Priority::High,

        Event::Mouse(_) => Priority::Normal,
        Event::User(_) => Priority::Normal,
        Event::Paste(_) => Priority::Normal,
        Event::Focus => Priority::Normal,
        Event::Blur => Priority::Normal,
        Event::ExecProcess => Priority::Normal,

        Event::Resize { .. } => Priority::Low,
        Event::Tick => Priority::Low,
    }
}

/// Type-erased event for priority detection
///
/// This allows us to work with events without knowing their message type,
/// avoiding the need for unsafe transmutes.
#[derive(Debug, Clone)]
pub enum EventKind {
    Quit,
    Key,
    Mouse,
    User,
    Resize,
    Tick,
    Paste,
    Focus,
    Blur,
    Suspend,
    Resume,
    ExecProcess,
}

impl EventKind {
    /// Get the priority for this event kind
    pub fn priority(&self) -> Priority {
        match self {
            EventKind::Quit => Priority::High,
            EventKind::Key => Priority::High,
            EventKind::Suspend => Priority::High,
            EventKind::Resume => Priority::High,

            EventKind::Mouse => Priority::Normal,
            EventKind::User => Priority::Normal,
            EventKind::Paste => Priority::Normal,
            EventKind::Focus => Priority::Normal,
            EventKind::Blur => Priority::Normal,
            EventKind::ExecProcess => Priority::Normal,

            EventKind::Resize => Priority::Low,
            EventKind::Tick => Priority::Low,
        }
    }

    /// Create from an event
    pub fn from_event<M: Message>(event: &Event<M>) -> Self {
        match event {
            Event::Quit => EventKind::Quit,
            Event::Key(_) => EventKind::Key,
            Event::Mouse(_) => EventKind::Mouse,
            Event::User(_) => EventKind::User,
            Event::Resize { .. } => EventKind::Resize,
            Event::Tick => EventKind::Tick,
            Event::Paste(_) => EventKind::Paste,
            Event::Focus => EventKind::Focus,
            Event::Blur => EventKind::Blur,
            Event::Suspend => EventKind::Suspend,
            Event::Resume => EventKind::Resume,
            Event::ExecProcess => EventKind::ExecProcess,
        }
    }
}

/// A priority mapper that doesn't require unsafe code
pub trait SafePriorityMapper<M: Message>: Send + Sync {
    /// Map an event to its priority
    fn map_priority(&self, event: &Event<M>) -> Priority;
}

/// Default priority mapper implementation
pub struct DefaultPriorityMapper;

impl<M: Message> SafePriorityMapper<M> for DefaultPriorityMapper {
    fn map_priority(&self, event: &Event<M>) -> Priority {
        detect_priority(event)
    }
}

/// Custom priority mapper that can override specific event priorities
pub struct CustomPriorityMapper<M: Message> {
    mapper: Box<dyn Fn(&Event<M>) -> Option<Priority> + Send + Sync>,
}

impl<M: Message> CustomPriorityMapper<M> {
    /// Create a new custom priority mapper
    pub fn new<F>(mapper: F) -> Self
    where
        F: Fn(&Event<M>) -> Option<Priority> + Send + Sync + 'static,
    {
        Self {
            mapper: Box::new(mapper),
        }
    }
}

impl<M: Message> SafePriorityMapper<M> for CustomPriorityMapper<M> {
    fn map_priority(&self, event: &Event<M>) -> Priority {
        // Try custom mapper first, fall back to default
        (self.mapper)(event).unwrap_or_else(|| detect_priority(event))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hojicha_core::event::{Event, Key, KeyEvent, KeyModifiers};

    #[test]
    fn test_priority_detection() {
        // High priority events
        assert_eq!(detect_priority::<()>(&Event::Quit), Priority::High);
        assert_eq!(
            detect_priority::<()>(&Event::Key(KeyEvent::new(
                Key::Char('a'),
                KeyModifiers::empty()
            ))),
            Priority::High
        );

        // Normal priority events
        assert_eq!(
            detect_priority::<String>(&Event::User("test".to_string())),
            Priority::Normal
        );

        // Low priority events
        assert_eq!(detect_priority::<()>(&Event::Tick), Priority::Low);
        assert_eq!(
            detect_priority::<()>(&Event::Resize {
                width: 80,
                height: 24
            }),
            Priority::Low
        );
    }

    #[test]
    fn test_event_kind() {
        let quit_event: Event<()> = Event::Quit;
        assert_eq!(
            EventKind::from_event(&quit_event).priority(),
            Priority::High
        );

        let tick_event: Event<()> = Event::Tick;
        assert_eq!(EventKind::from_event(&tick_event).priority(), Priority::Low);
    }

    #[test]
    fn test_custom_mapper() {
        // Create a custom mapper that makes all User events high priority
        let mapper = CustomPriorityMapper::new(|event| match event {
            Event::User(_) => Some(Priority::High),
            _ => None,
        });

        let user_event = Event::User("important".to_string());
        assert_eq!(mapper.map_priority(&user_event), Priority::High);

        let tick_event: Event<String> = Event::Tick;
        assert_eq!(mapper.map_priority(&tick_event), Priority::Low);
    }
}
