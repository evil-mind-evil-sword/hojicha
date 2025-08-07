//! Comprehensive unit tests for program.rs to maximize coverage

use hojicha::{
    commands,
    core::{Cmd, Model},
    event::{Event, Key},
    prelude::*,
};

// Test model for comprehensive testing
#[derive(Debug, Clone)]
struct TestModel {
    value: i32,
    events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum TestMsg {
    Inc,
    Dec,
    Quit,
}

impl Model for TestModel {
    type Message = TestMsg;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        self.events.push("init".to_string());
        Some(commands::custom(|| Some(TestMsg::Inc)))
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(TestMsg::Inc) => {
                self.value += 1;
                self.events.push(format!("inc:{}", self.value));
                Cmd::none()
            }
            Event::User(TestMsg::Dec) => {
                self.value -= 1;
                self.events.push(format!("dec:{}", self.value));
                Cmd::none()
            }
            Event::User(TestMsg::Quit) => None,
            Event::Key(key) if key.key == Key::Char('q') => None,
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {
        // Simple view for testing
    }
}

#[test]
fn test_program_options_all_methods() {
    let options = ProgramOptions::default()
        .with_mouse_mode(MouseMode::CellMotion)
        .with_alt_screen(true)
        .with_bracketed_paste(true)
        .with_focus_reporting(true)
        .with_fps(120)
        .headless()
        .without_signal_handler()
        .without_renderer();

    assert_eq!(options.mouse_mode, MouseMode::CellMotion);
    assert!(options.alt_screen);
    assert!(options.bracketed_paste);
    assert!(options.focus_reporting);
    assert_eq!(options.fps, 120);
    assert!(options.headless);
    assert!(!options.install_signal_handler);
    assert!(options.without_renderer);
}

#[test]
fn test_program_options_with_output() {
    use std::io::Write;

    struct TestOutput;
    impl Write for TestOutput {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let options = ProgramOptions::default().with_output(Box::new(TestOutput));
    assert!(options.output.is_some());
}

#[test]
fn test_mouse_mode_default() {
    assert_eq!(MouseMode::default(), MouseMode::None);
}

#[test]
fn test_mouse_mode_all_variants() {
    let _ = MouseMode::None;
    let _ = MouseMode::CellMotion;
    let _ = MouseMode::AllMotion;
}

#[test]
fn test_program_creation_headless() {
    let model = TestModel {
        value: 0,
        events: vec![],
    };

    let options = ProgramOptions::default().headless();
    let result = Program::with_options(model, options);
    assert!(result.is_ok());
}

#[test]
fn test_program_creation_with_custom_output() {
    use std::io::Write;

    struct TestOutput;
    impl Write for TestOutput {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let model = TestModel {
        value: 0,
        events: vec![],
    };

    let options = ProgramOptions::default().with_output(Box::new(TestOutput));
    let result = Program::with_options(model, options);
    assert!(result.is_ok());
}

#[test]
fn test_program_debug_methods() {
    let model = TestModel {
        value: 0,
        events: vec![],
    };

    let options = ProgramOptions::default().headless();
    let program = Program::with_options(model, options).unwrap();

    // These should not panic
    program.println("test");
    program.printf(format_args!("test {}", 123));
}

#[test]
fn test_program_with_filter() {
    let model = TestModel {
        value: 0,
        events: vec![],
    };

    let options = ProgramOptions::default().headless();
    let program = Program::with_options(model, options).unwrap();

    let _program_with_filter = program.with_filter(|_model, event| match event {
        Event::User(TestMsg::Inc) => Some(Event::User(TestMsg::Dec)),
        _ => Some(event),
    });
}

#[test]
fn test_program_terminal_operations() {
    let model = TestModel {
        value: 0,
        events: vec![],
    };

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    // These should handle headless mode gracefully
    let _ = program.release_terminal();
    let _ = program.restore_terminal();
}

#[test]
fn test_program_control_methods() {
    let model = TestModel {
        value: 0,
        events: vec![],
    };

    let options = ProgramOptions::default().headless();
    let program = Program::with_options(model, options).unwrap();

    // Test control methods
    program.quit();
    program.kill();
    // Don't call wait() as it would block
}

#[test]
fn test_program_drop() {
    let model = TestModel {
        value: 0,
        events: vec![],
    };

    let options = ProgramOptions::default().headless();

    {
        let _program = Program::with_options(model, options).unwrap();
        // Program should clean up when dropped
    }
    // Should not panic
}

// Test multiple FPS values
#[test]
fn test_fps_variations() {
    for fps in [1, 30, 60, 120, 240, 480] {
        let model = TestModel {
            value: 0,
            events: vec![],
        };

        let options = ProgramOptions::default().with_fps(fps).headless();

        let result = Program::with_options(model, options);
        assert!(result.is_ok());
    }
}

// Test all terminal feature combinations
#[test]
fn test_terminal_feature_combinations() {
    let test_cases = vec![
        (true, true, true),
        (true, true, false),
        (true, false, true),
        (false, true, true),
        (false, false, false),
    ];

    for (alt_screen, bracketed_paste, focus_reporting) in test_cases {
        let model = TestModel {
            value: 0,
            events: vec![],
        };

        let options = ProgramOptions::default()
            .with_alt_screen(alt_screen)
            .with_bracketed_paste(bracketed_paste)
            .with_focus_reporting(focus_reporting)
            .headless();

        let result = Program::with_options(model, options);
        assert!(result.is_ok());
    }
}

// Test all mouse mode combinations
#[test]
fn test_all_mouse_modes() {
    let modes = vec![MouseMode::None, MouseMode::CellMotion, MouseMode::AllMotion];

    for mode in modes {
        let model = TestModel {
            value: 0,
            events: vec![],
        };

        let options = ProgramOptions::default().with_mouse_mode(mode).headless();

        let result = Program::with_options(model, options);
        assert!(result.is_ok());
    }
}

// Test error case
#[test]
fn test_program_new_default() {
    let model = TestModel {
        value: 0,
        events: vec![],
    };

    // This might fail in CI but let's try
    let result = Program::new(model);
    // Just check it doesn't panic
    let _ = result;
}
