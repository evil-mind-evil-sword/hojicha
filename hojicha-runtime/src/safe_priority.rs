//! Safe priority detection without unsafe transmutes
//!
//! This module provides a safe way to detect event priority without
//! using unsafe memory transmutation.

use crate::priority_queue::Priority;
use hojicha_core::core::Message;
use hojicha_core::event::Event;

/// Detect the priority of an event without unsafe operations
pub fn detect_priority<M: Message>(event: &Event<M>) -> Priority {
    // Use helper methods for cleaner priority detection
    if event.is_quit() || event.is_key() || event.is_suspend() || event.is_resume() {
        Priority::High
    } else if event.is_resize() || event.is_tick() {
        Priority::Low  
    } else {
        Priority::Normal
    }
}

/// Type-erased event for priority detection
///
/// This allows us to work with events without knowing their message type,
/// avoiding the need for unsafe transmutes.
#[derive(Debug, Clone)]
pub enum EventKind {
    /// Quit event - terminates the program
    Quit,
    /// Keyboard input event
    Key,
    /// Mouse input event
    Mouse,
    /// User-defined message event
    User,
    /// Terminal resize event
    Resize,
    /// Timer tick event
    Tick,
    /// Text paste event
    Paste,
    /// Terminal focus gained event
    Focus,
    /// Terminal focus lost event
    Blur,
    /// Process suspension event
    Suspend,
    /// Process resumption event
    Resume,
    /// Process execution event
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
        if event.is_quit() { EventKind::Quit }
        else if event.is_key() { EventKind::Key }
        else if event.is_mouse() { EventKind::Mouse }
        else if event.is_user() { EventKind::User }
        else if event.is_resize() { EventKind::Resize }
        else if event.is_tick() { EventKind::Tick }
        else if event.is_paste() { EventKind::Paste }
        else if event.is_focus() { EventKind::Focus }
        else if event.is_blur() { EventKind::Blur }
        else if event.is_suspend() { EventKind::Suspend }
        else if event.is_resume() { EventKind::Resume }
        else { EventKind::ExecProcess }
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
        let mapper = CustomPriorityMapper::new(|event| {
            if event.is_user() {
                Some(Priority::High)
            } else {
                None
            }
        });

        let user_event = Event::User("important".to_string());
        assert_eq!(mapper.map_priority(&user_event), Priority::High);

        let tick_event: Event<String> = Event::Tick;
        assert_eq!(mapper.map_priority(&tick_event), Priority::Low);
    }
}
