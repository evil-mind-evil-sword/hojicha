//! Tests for focus reporting functionality

use hojicha::commands;
use hojicha::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

#[test]
fn test_focus_blur_events() {
    #[derive(Clone)]
    struct FocusModel {
        has_focus: Arc<AtomicBool>,
        focus_count: Arc<AtomicU32>,
        blur_count: Arc<AtomicU32>,
    }

    #[derive(Debug, Clone)]
    enum Msg {}

    impl Model for FocusModel {
        type Message = Msg;

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::Focus => {
                    self.has_focus.store(true, Ordering::SeqCst);
                    self.focus_count.fetch_add(1, Ordering::SeqCst);
                }
                Event::Blur => {
                    self.has_focus.store(false, Ordering::SeqCst);
                    self.blur_count.fetch_add(1, Ordering::SeqCst);
                }
                Event::Quit => return commands::quit(),
                _ => {}
            }
            Cmd::none()
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = FocusModel {
        has_focus: Arc::new(AtomicBool::new(false)),
        focus_count: Arc::new(AtomicU32::new(0)),
        blur_count: Arc::new(AtomicU32::new(0)),
    };

    // Test focus event
    let mut test_model = model.clone();
    test_model.update(Event::Focus);
    assert!(model.has_focus.load(Ordering::SeqCst));
    assert_eq!(model.focus_count.load(Ordering::SeqCst), 1);
    assert_eq!(model.blur_count.load(Ordering::SeqCst), 0);

    // Test blur event
    test_model.update(Event::Blur);
    assert!(!model.has_focus.load(Ordering::SeqCst));
    assert_eq!(model.focus_count.load(Ordering::SeqCst), 1);
    assert_eq!(model.blur_count.load(Ordering::SeqCst), 1);

    // Test multiple focus/blur cycles
    test_model.update(Event::Focus);
    test_model.update(Event::Blur);
    assert_eq!(model.focus_count.load(Ordering::SeqCst), 2);
    assert_eq!(model.blur_count.load(Ordering::SeqCst), 2);
}

#[test]
fn test_focus_commands() {
    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {}

    // The commands should compile and be creatable
    let _enable = enable_focus_change::<TestMsg>();
    let _disable = disable_focus_change::<TestMsg>();

    // They should return None when executed (handled by runtime)
    let enable_cmd = enable_focus_change::<TestMsg>();
    assert!(enable_cmd.test_execute().unwrap().is_none());

    let disable_cmd = disable_focus_change::<TestMsg>();
    assert!(disable_cmd.test_execute().unwrap().is_none());
}

#[test]
fn test_program_options_focus_reporting() {
    let opts = ProgramOptions::default().with_focus_reporting(true);
    assert!(opts.focus_reporting);

    let default_opts = ProgramOptions::default();
    assert!(!default_opts.focus_reporting);
}
