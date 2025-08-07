//! Tests for suspend/resume functionality

use hojicha::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[test]
fn test_suspend_resume_events() {
    #[derive(Clone)]
    struct SuspendModel {
        suspend_count: Arc<AtomicU32>,
        resume_count: Arc<AtomicU32>,
        suspend_command_sent: Arc<AtomicBool>,
    }

    #[derive(Debug, Clone)]
    enum Msg {}

    impl Model for SuspendModel {
        type Message = Msg;

        fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
            match event {
                Event::Key(key)
                    if key.key == Key::Char('z')
                        && key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.suspend_command_sent.store(true, Ordering::SeqCst);
                    return Some(suspend());
                }
                Event::Suspend => {
                    self.suspend_count.fetch_add(1, Ordering::SeqCst);
                }
                Event::Resume => {
                    self.resume_count.fetch_add(1, Ordering::SeqCst);
                }
                Event::Quit => return None,
                _ => {}
            }
            None
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = SuspendModel {
        suspend_count: Arc::new(AtomicU32::new(0)),
        resume_count: Arc::new(AtomicU32::new(0)),
        suspend_command_sent: Arc::new(AtomicBool::new(false)),
    };

    // Test direct suspend event
    let mut test_model = model.clone();
    test_model.update(Event::Suspend);
    assert_eq!(model.suspend_count.load(Ordering::SeqCst), 1);

    // Test resume event
    test_model.update(Event::Resume);
    assert_eq!(model.resume_count.load(Ordering::SeqCst), 1);

    // Test Ctrl+Z triggers suspend command
    let key_event = KeyEvent::new(Key::Char('z'), KeyModifiers::CONTROL);
    let result = test_model.update(Event::Key(key_event));
    assert!(result.is_some());
    assert!(model.suspend_command_sent.load(Ordering::SeqCst));
}

#[test]
fn test_suspend_command_creation() {
    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {}

    // The suspend command should compile and be creatable
    let _cmd: Cmd<TestMsg> = suspend();

    // It should return None when executed (handled by runtime)
    let cmd = suspend::<TestMsg>();
    assert!(cmd.test_execute().unwrap().is_none());
}
