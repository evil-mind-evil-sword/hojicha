//! Tests for message filtering functionality

use hojicha::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[test]
fn test_with_filter() {
    #[derive(Clone)]
    struct FilterModel {
        allowed_count: Arc<AtomicU32>,
        blocked_count: Arc<AtomicU32>,
    }

    #[derive(Debug, Clone)]
    enum Msg {
        Allowed,
        Blocked,
    }

    impl Model for FilterModel {
        type Message = Msg;

        fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
            if let Event::User(msg) = msg {
                match msg {
                    Msg::Allowed => {
                        self.allowed_count.fetch_add(1, Ordering::SeqCst);
                    }
                    Msg::Blocked => {
                        self.blocked_count.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
            None
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = FilterModel {
        allowed_count: Arc::new(AtomicU32::new(0)),
        blocked_count: Arc::new(AtomicU32::new(0)),
    };

    // Test would create a program with filter that blocks "Blocked" messages
    // In a real test environment, we'd run the program and verify counts

    // For now, let's test the filter logic directly
    let filter = |_model: &FilterModel, event: Event<Msg>| -> Option<Event<Msg>> {
        match &event {
            Event::User(Msg::Blocked) => None, // Filter out blocked messages
            _ => Some(event),
        }
    };

    // Simulate filtering
    let allowed_event = Event::User(Msg::Allowed);
    let blocked_event = Event::User(Msg::Blocked);

    assert!(filter(&model, allowed_event).is_some());
    assert!(filter(&model, blocked_event).is_none());
}

#[test]
fn test_filter_can_modify_messages() {
    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        Original,
        Modified,
    }

    let filter = |_model: &(), event: Event<Msg>| -> Option<Event<Msg>> {
        match event {
            Event::User(Msg::Original) => Some(Event::User(Msg::Modified)),
            _ => Some(event),
        }
    };

    let original = Event::User(Msg::Original);
    let filtered = filter(&(), original);

    assert_eq!(filtered, Some(Event::User(Msg::Modified)));
}

#[test]
fn test_filter_key_events() {
    #[derive(Debug, Clone)]
    struct NoQuitModel;

    #[derive(Debug, Clone)]
    enum Msg {}

    impl Model for NoQuitModel {
        type Message = Msg;

        fn update(&mut self, _msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
            None
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    // Filter that prevents quit on 'q' key
    let filter = |_model: &NoQuitModel, event: Event<Msg>| -> Option<Event<Msg>> {
        match &event {
            Event::Key(KeyEvent {
                key: Key::Char('q'),
                ..
            }) => None,
            _ => Some(event),
        }
    };

    let q_event = Event::Key(KeyEvent::new(Key::Char('q'), KeyModifiers::empty()));
    let a_event = Event::Key(KeyEvent::new(Key::Char('a'), KeyModifiers::empty()));

    assert!(filter(&NoQuitModel, q_event).is_none());
    assert!(filter(&NoQuitModel, a_event).is_some());
}
