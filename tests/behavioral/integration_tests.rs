//! Comprehensive integration tests for hojicha

use hojicha_core::{
    commands,
    core::{Cmd, Model},
    event::{Event, Key, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
};
use hojicha_runtime::program::{MouseMode, ProgramOptions};
use std::time::Duration;

// Import test utilities
#[path = "../common/mod.rs"]
mod common;
use common::{SimpleTestModel, TestMsg};

#[test]
fn test_model_init() {
    let mut model = SimpleTestModel::default();

    let cmd = model.init();
    assert!(!cmd.is_quit());
}

#[test]
fn test_model_update_user_messages() {
    let mut model = SimpleTestModel::default();

    // Test increment
    let cmd = model.update(Event::User(TestMsg::Increment));
    assert!(!cmd.is_quit());
    assert_eq!(model.counter, 1);

    // Test decrement
    let cmd = model.update(Event::User(TestMsg::Decrement));
    assert!(!cmd.is_quit());
    assert_eq!(model.counter, 0);

    // Test add message
    let cmd = model.update(Event::User(TestMsg::AddMessage("test".to_string())));
    assert!(!cmd.is_quit());
    assert!(model.messages.contains(&"test".to_string()));

    // Test quit - should return quit command
    let cmd = model.update(Event::User(TestMsg::Quit));
    assert!(cmd.is_quit());
}

#[test]
fn test_model_keyboard_events() {
    // Create a custom test model for keyboard events
    #[derive(Debug, Clone)]
    struct KeyboardTestModel {
        counter: i32,
        quit_requested: bool,
    }

    impl Model for KeyboardTestModel {
        type Message = ();

        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            if let Event::Key(key_event) = event {
                match key_event.key {
                    Key::Char('+') => {
                        self.counter += 1;
                        Cmd::none()
                    }
                    Key::Char('-') => {
                        self.counter = self.counter.saturating_sub(1);
                        Cmd::none()
                    }
                    Key::Char('q') => {
                        self.quit_requested = true;
                        commands::quit()
                    }
                    _ => Cmd::none(),
                }
            } else {
                Cmd::none()
            }
        }

        fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
    }

    let mut model = KeyboardTestModel {
        counter: 0,
        quit_requested: false,
    };

    // Test '+' key
    let event = Event::Key(KeyEvent::new(Key::Char('+'), KeyModifiers::empty()));
    let cmd = model.update(event);
    assert!(!cmd.is_quit());
    assert_eq!(model.counter, 1);

    // Test '-' key
    let event = Event::Key(KeyEvent::new(Key::Char('-'), KeyModifiers::empty()));
    let cmd = model.update(event);
    assert!(!cmd.is_quit());
    assert_eq!(model.counter, 0);

    // Test 'q' key (quit)
    let event = Event::Key(KeyEvent::new(Key::Char('q'), KeyModifiers::empty()));
    let cmd = model.update(event);
    assert!(cmd.is_quit());
    assert!(model.quit_requested);
}

#[test]
fn test_model_mouse_events() {
    let mut model = SimpleTestModel::default();

    let event = Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 20,
        modifiers: KeyModifiers::empty(),
    });

    // SimpleTestModel doesn't handle mouse events, should return Cmd::none()
    let cmd = model.update(event);
    assert!(!cmd.is_quit());
}

#[test]
fn test_model_resize_event() {
    let mut model = SimpleTestModel::default();

    let event = Event::Resize {
        width: 80,
        height: 24,
    };
    // SimpleTestModel doesn't handle resize events, should return Cmd::none()
    let cmd = model.update(event);
    assert!(!cmd.is_quit());
}

#[test]
fn test_model_focus_event() {
    let mut model = SimpleTestModel::default();

    let event = Event::Focus;
    let cmd = model.update(event);
    assert!(!cmd.is_quit());

    let event = Event::Blur;
    let cmd = model.update(event);
    assert!(!cmd.is_quit());
}

#[test]
fn test_model_paste_event() {
    let mut model = SimpleTestModel::default();

    let event = Event::Paste("Hello, World!".to_string());
    // SimpleTestModel doesn't handle paste events
    let cmd = model.update(event);
    assert!(!cmd.is_quit());
}

#[test]
fn test_model_quit_event() {
    let mut model = SimpleTestModel::default();

    let event = Event::Quit;
    // SimpleTestModel doesn't handle Quit event directly
    let cmd = model.update(event);
    assert!(!cmd.is_quit());
}

#[test]
fn test_batch_commands() {
    let _batch = commands::batch::<TestMsg>(vec![
        commands::custom(|| Some(TestMsg::Increment)),
        commands::custom(|| Some(TestMsg::Decrement)),
        commands::custom(|| Some(TestMsg::AddMessage("batch".to_string()))),
    ]);

    // Batch command should exist
    // Commands are created, not None
}

#[test]
fn test_sequence_commands() {
    let _seq = commands::sequence::<TestMsg>(vec![
        commands::custom(|| Some(TestMsg::Increment)),
        commands::tick(Duration::from_millis(100), || TestMsg::Tick),
        commands::custom(|| Some(TestMsg::Decrement)),
    ]);

    // Sequence command should exist
    // Commands are created successfully
}

#[test]
fn test_tick_command() {
    let _tick = commands::tick(Duration::from_millis(100), || TestMsg::Tick);
    // Tick command created successfully
}

#[test]
fn test_every_command() {
    let _every = commands::every(Duration::from_millis(100), |_| TestMsg::Tick);
    // Every command created successfully
}

#[test]
fn test_send_command() {
    let _send = commands::custom(|| Some(TestMsg::Increment));
    // Send command created successfully
}

#[test]
fn test_window_size_command() {
    let _size = commands::window_size(|_| TestMsg::Tick);
    // Window size command created successfully
}

#[test]
fn test_set_window_title_command() {
    let _title = commands::set_window_title::<TestMsg>("Test App");
    // Set window title command created successfully
}

#[test]
fn test_custom_command() {
    let _custom = commands::custom(|| {
        // Custom logic here
        Some(TestMsg::Increment)
    });
    // Custom command created successfully
}

#[test]
fn test_custom_async_command() {
    let _custom = commands::custom_async(|| {
        Box::pin(async {
            // Async logic here
            Some(TestMsg::Increment)
        })
    });
    // Custom async command created successfully
}

#[test]
fn test_custom_fallible_command() {
    let _custom = commands::custom_fallible(|| {
        // Fallible logic here
        Ok(Some(TestMsg::Increment))
    });
    // Custom fallible command created successfully
}

#[test]
fn test_quit_command() {
    let _quit = commands::quit::<TestMsg>();
    // Quit command created successfully
}

#[test]
fn test_interrupt_command() {
    let _interrupt = commands::interrupt::<TestMsg>();
    // Interrupt command created successfully
}

// Test using our testing harness
// NOTE: Commented out as testing module is internal only
// #[test]
// fn test_with_harness() {
//     use hojicha_core::testing::TestHarness;
//
//     let model = TestModel {
//         counter: 0,
//         messages: Vec::new(),
//         should_quit: false,
//     };
//
//     let result = TestHarness::new(model)
//         .send_event(Event::User(TestMsg::Increment))
//         .send_event(Event::User(TestMsg::Increment))
//         .send_event(Event::User(TestMsg::Decrement))
//         .send_event(Event::User(TestMsg::AddMessage("test".to_string())))
//         .run();
//
//     assert_eq!(result.model.counter, 1);
//     assert!(result.model.messages.contains(&"test".to_string()));
// }

// Test program options
#[test]
fn test_program_options() {
    let options = ProgramOptions::default()
        .with_mouse_mode(MouseMode::CellMotion)
        .with_alt_screen(true)
        .with_bracketed_paste(true)
        .with_focus_reporting(true)
        .with_fps(60)
        .headless()
        .without_signal_handler();

    assert_eq!(options.mouse_mode, MouseMode::CellMotion);
    assert!(options.alt_screen);
    assert!(options.bracketed_paste);
    assert!(options.focus_reporting);
    assert_eq!(options.fps, 60);
    assert!(options.headless);
    assert!(!options.install_signal_handler);
}

#[test]
fn test_cmd_none() {
    let _cmd: Cmd<TestMsg> = Cmd::none();
    // None command created successfully
}

#[test]
fn test_cmd_fallible() {
    let _cmd: Cmd<TestMsg> = Cmd::fallible(|| Ok(Some(TestMsg::Increment)));
    // Fallible command created successfully
}

// Test event recorder
// NOTE: Commented out as testing module is internal only
// #[test]
// fn test_event_recorder() {
//     use hojicha_core::testing::EventRecorder;
//
//     let mut recorder = EventRecorder::<TestMsg>::new();
//     recorder.record(Event::User(TestMsg::Increment));
//     recorder.record(Event::User(TestMsg::Decrement));
//
//     let events = recorder.get_events();
//     assert_eq!(events.len(), 2);
//
//     let mut playback = recorder.to_playback();
//     assert!(playback.has_next());
//
//     let event1 = playback.next();
//     assert_eq!(event1, Some(Event::User(TestMsg::Increment)));
//
//     let event2 = playback.next();
//     assert_eq!(event2, Some(Event::User(TestMsg::Decrement)));
//
//     assert!(!playback.has_next());
// }

// Test mock terminal
// NOTE: Commented out as MockTerminal is internal only
// #[test]
// fn test_mock_terminal() {
//     use hojicha_core::testing::MockTerminal;
//     use ratatui::Terminal;
//
//     let backend = MockTerminal::new(80, 24);
//     let mut terminal = Terminal::new(backend).unwrap();
//
//     terminal
//         .draw(|f| {
//             // Drawing operations
//             let area = f.size();
//             assert_eq!(area.width, 80);
//             assert_eq!(area.height, 24);
//         })
//         .unwrap();
//
//     let backend = terminal.backend();
//     let operations = backend.get_operations();
//     assert!(operations.contains(&"draw".to_string()));
// }
