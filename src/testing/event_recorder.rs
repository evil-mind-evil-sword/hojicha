//! Event recording and playback for testing

use crate::core::Message;
use crate::event::Event;
use std::time::Duration;

/// Records events with timing information
pub struct EventRecorder<M: Message> {
    events: Vec<RecordedEvent<M>>,
    start_time: std::time::Instant,
}

/// An event with timing information
#[derive(Debug)]
pub struct RecordedEvent<M: Message> {
    pub event: Event<M>,
    pub timestamp: Duration,
}

impl<M: Message + Clone> Clone for RecordedEvent<M> {
    fn clone(&self) -> Self {
        Self {
            event: self.event.clone(),
            timestamp: self.timestamp,
        }
    }
}

impl<M: Message> EventRecorder<M> {
    /// Create a new event recorder
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Record an event
    pub fn record(&mut self, event: Event<M>) {
        self.events.push(RecordedEvent {
            event,
            timestamp: self.start_time.elapsed(),
        });
    }

    /// Get all recorded events
    pub fn get_events(&self) -> &[RecordedEvent<M>] {
        &self.events
    }

    /// Create a playback sequence from recorded events
    pub fn to_playback(&self) -> EventPlayback<M>
    where
        M: Clone,
    {
        EventPlayback::new(self.events.clone())
    }

    /// Save events to a test fixture (could be JSON in real implementation)
    pub fn save_fixture(&self, name: &str) -> String {
        format!("Fixture '{}' with {} events", name, self.events.len())
    }
}

/// Plays back recorded events for testing
pub struct EventPlayback<M: Message> {
    events: Vec<RecordedEvent<M>>,
    current_index: usize,
}

impl<M: Message> EventPlayback<M> {
    /// Create a new playback from recorded events
    pub fn new(events: Vec<RecordedEvent<M>>) -> Self {
        Self {
            events,
            current_index: 0,
        }
    }

    /// Get the next event in the sequence
    pub fn next(&mut self) -> Option<Event<M>>
    where
        M: Clone,
    {
        if self.current_index < self.events.len() {
            let event = self.events[self.current_index].event.clone();
            self.current_index += 1;
            Some(event)
        } else {
            None
        }
    }

    /// Reset playback to the beginning
    pub fn reset(&mut self) {
        self.current_index = 0;
    }

    /// Check if there are more events
    pub fn has_next(&self) -> bool {
        self.current_index < self.events.len()
    }
}

impl<M: Message> Default for EventRecorder<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Key, KeyEvent, KeyModifiers};

    #[derive(Debug, Clone, PartialEq)]
    struct TestMsg(String);

    // No need to implement Message - it has a blanket impl for Send + 'static types

    #[test]
    fn test_event_recording() {
        let mut recorder = EventRecorder::new();

        recorder.record(Event::User(TestMsg("test".to_string())));
        recorder.record(Event::Key(KeyEvent::new(
            Key::Char('a'),
            KeyModifiers::empty(),
        )));

        assert_eq!(recorder.get_events().len(), 2);
    }

    #[test]
    fn test_event_playback() {
        let mut recorder = EventRecorder::new();
        recorder.record(Event::User(TestMsg("first".to_string())));
        recorder.record(Event::User(TestMsg("second".to_string())));

        let mut playback = recorder.to_playback();

        assert!(playback.has_next());
        assert_eq!(
            playback.next(),
            Some(Event::User(TestMsg("first".to_string())))
        );
        assert_eq!(
            playback.next(),
            Some(Event::User(TestMsg("second".to_string())))
        );
        assert!(!playback.has_next());
    }
}
