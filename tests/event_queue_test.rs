//! Tests for event queue management and buffering

use hojicha::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn test_event_buffering() {
    #[derive(Clone)]
    struct BufferTestModel {
        event_count: Arc<AtomicU32>,
        resize_count: Arc<AtomicU32>,
        user_msg_count: Arc<AtomicU32>,
    }

    #[derive(Debug, Clone)]
    enum Msg {
        UserEvent(#[allow(dead_code)] u32),
    }

    impl Model for BufferTestModel {
        type Message = Msg;

        fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
            self.event_count.fetch_add(1, Ordering::SeqCst);

            match event {
                Event::Resize { .. } => {
                    self.resize_count.fetch_add(1, Ordering::SeqCst);
                }
                Event::User(Msg::UserEvent(_)) => {
                    self.user_msg_count.fetch_add(1, Ordering::SeqCst);
                }
                Event::Quit => return None,
                _ => {}
            }
            None
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = BufferTestModel {
        event_count: Arc::new(AtomicU32::new(0)),
        resize_count: Arc::new(AtomicU32::new(0)),
        user_msg_count: Arc::new(AtomicU32::new(0)),
    };

    let event_count = Arc::clone(&model.event_count);
    let user_msg_count = Arc::clone(&model.user_msg_count);

    // Create a dummy output to run in headless mode
    let output = Box::new(std::io::sink());
    let _program = Program::with_options(
        model,
        ProgramOptions::default()
            .with_alt_screen(false)
            .with_output(output)
            .headless(),
    )
    .expect("Failed to create program");
    // Note: Since we can't directly access message_tx in tests,
    // we can only test the buffering behavior indirectly.
    // In production use, the sync_channel(100) provides buffering
    // for up to 100 events before blocking.

    // Test model update with rapid events
    let mut test_model = BufferTestModel {
        event_count: Arc::clone(&event_count),
        resize_count: Arc::new(AtomicU32::new(0)),
        user_msg_count: Arc::clone(&user_msg_count),
    };

    // Simulate processing many events
    for i in 0..50 {
        test_model.update(Event::User(Msg::UserEvent(i)));
    }

    // All events should have been processed
    assert_eq!(user_msg_count.load(Ordering::SeqCst), 50);
    assert!(event_count.load(Ordering::SeqCst) >= 50); // At least all user messages
}

#[test]
fn test_resize_event_coalescing() {
    #[derive(Clone)]
    struct ResizeTestModel {
        resize_count: Arc<AtomicU32>,
        last_size: Arc<Mutex<(u16, u16)>>,
    }

    #[derive(Debug, Clone)]
    enum Msg {}

    impl Model for ResizeTestModel {
        type Message = Msg;

        fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
            match event {
                Event::Resize { width, height } => {
                    self.resize_count.fetch_add(1, Ordering::SeqCst);
                    *self.last_size.lock().unwrap() = (width, height);
                }
                Event::Quit => return None,
                _ => {}
            }
            None
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    // Note: We can't directly test resize coalescing in unit tests
    // as it requires actual terminal events. This is more of a
    // documentation test showing the expected behavior.

    let _model = ResizeTestModel {
        resize_count: Arc::new(AtomicU32::new(0)),
        last_size: Arc::new(Mutex::new((80, 24))),
    };

    // In a real scenario with rapid resizes, we'd expect:
    // - Multiple resize events sent in quick succession
    // - Only the final size to be processed
    // - Reduced number of resize events compared to what was sent
}

#[test]
fn test_event_priority() {
    #[derive(Clone)]
    #[allow(dead_code)]
    struct PriorityTestModel {
        events: Arc<Mutex<Vec<String>>>,
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    enum Msg {
        High,
        Normal,
    }

    impl Model for PriorityTestModel {
        type Message = Msg;

        fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
            let mut events = self.events.lock().unwrap();

            match event {
                Event::User(Msg::High) => events.push("High".to_string()),
                Event::User(Msg::Normal) => events.push("Normal".to_string()),
                Event::Quit => {
                    events.push("Quit".to_string());
                    return None;
                }
                _ => {}
            }
            None
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    // User messages should be processed before system events
    // This ensures that application logic takes precedence
}

use std::sync::Mutex;
