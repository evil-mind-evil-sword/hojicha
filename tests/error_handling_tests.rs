//! Tests for error handling functionality

use hojicha::{
    commands,
    core::{Cmd, Model},
    error::{Error, Result},
    event::Event,
    fallible::{FallibleModel, FallibleModelExt},
    prelude::*,
};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
enum TestMsg {
    DoWork,
    CauseError,
    CausePanic,
    ErrorConverted(String),
    WorkCompleted,
}

struct TestModel {
    work_count: usize,
    error_count: usize,
    panic_count: usize,
    messages_received: Arc<Mutex<Vec<String>>>,
}

impl Model for TestModel {
    type Message = TestMsg;

    fn update(&mut self, event: Event<TestMsg>) -> Cmd<TestMsg> {
        self.update_with_error_handling(event)
    }

    fn view(&self, _: &mut Frame, _: Rect) {}
}

impl FallibleModel for TestModel {
    fn try_update(&mut self, event: Event<TestMsg>) -> Result<Cmd<TestMsg>> {
        match event {
            Event::User(TestMsg::DoWork) => {
                self.work_count += 1;
                Ok(commands::custom(|| Some(TestMsg::WorkCompleted)))
            }
            Event::User(TestMsg::CauseError) => {
                Err(Error::Model("Intentional test error".to_string()))
            }
            Event::User(TestMsg::CausePanic) => {
                panic!("Intentional test panic!");
            }
            Event::User(TestMsg::ErrorConverted(msg)) => {
                self.messages_received.lock().unwrap().push(msg);
                Ok(Cmd::none())
            }
            Event::User(TestMsg::WorkCompleted) => {
                self.messages_received
                    .lock()
                    .unwrap()
                    .push("Work completed".to_string());
                Ok(Cmd::none())
            }
            _ => Ok(Cmd::none()),
        }
    }

    fn handle_error(&mut self, error: Error) -> Cmd<TestMsg> {
        self.error_count += 1;
        commands::custom(move || Some(TestMsg::ErrorConverted(error.to_string())))
    }

    fn handle_panic(&mut self, panic_info: String) -> Cmd<TestMsg> {
        self.panic_count += 1;
        commands::custom(move || Some(TestMsg::ErrorConverted(format!("Panic: {}", panic_info))))
    }
}

#[test]
fn test_fallible_model_normal_operation() {
    let mut model = TestModel {
        work_count: 0,
        error_count: 0,
        panic_count: 0,
        messages_received: Arc::new(Mutex::new(Vec::new())),
    };

    let _cmd = model.update(Event::User(TestMsg::DoWork));
    // Command should be created (work completed message)
    assert_eq!(model.work_count, 1);
    assert_eq!(model.error_count, 0);
}

#[test]
fn test_fallible_model_error_handling() {
    let mut model = TestModel {
        work_count: 0,
        error_count: 0,
        panic_count: 0,
        messages_received: Arc::new(Mutex::new(Vec::new())),
    };

    let _cmd = model.update(Event::User(TestMsg::CauseError));
    // Should handle error and increment counter
    assert_eq!(model.error_count, 1);
    assert_eq!(model.work_count, 0);
}

#[test]
fn test_fallible_model_panic_recovery() {
    let mut model = TestModel {
        work_count: 0,
        error_count: 0,
        panic_count: 0,
        messages_received: Arc::new(Mutex::new(Vec::new())),
    };

    // Use the panic-catching version
    let _cmd = model.update_with_panic_catching(Event::User(TestMsg::CausePanic));
    // Should catch panic and increment counter
    assert_eq!(model.panic_count, 1);
}

#[test]
fn test_fallible_with_error_command() {
    // Test that the command compiles correctly
    #[derive(Clone)]
    enum Msg {
        Success(String),
        Error(String),
    }

    // Test successful operation - just verify it compiles
    let _cmd: Cmd<Msg> = commands::fallible_with_error(
        || Ok(Some(Msg::Success("data".to_string()))),
        |err| Msg::Error(err.to_string()),
    );

    // Test failed operation - just verify it compiles
    let _cmd: Cmd<Msg> = commands::fallible_with_error(
        || Err(Error::Io(std::io::Error::other("test error"))),
        |err| Msg::Error(err.to_string()),
    );
}

#[test]
fn test_error_context_in_fallible_model() {
    use hojicha::error::ErrorContext;

    struct ContextModel {
        last_error: Option<String>,
    }

    impl Model for ContextModel {
        type Message = ();
        fn update(&mut self, event: Event<()>) -> Cmd<()> {
            self.update_with_error_handling(event)
        }
        fn view(&self, _: &mut Frame, _: Rect) {}
    }

    impl FallibleModel for ContextModel {
        fn try_update(&mut self, _: Event<()>) -> Result<Cmd<()>> {
            let result: Result<()> = Err(Error::Model("base error".to_string()));
            result.context("while processing update")?;
            Ok(Cmd::none())
        }

        fn handle_error(&mut self, error: Error) -> Cmd<()> {
            self.last_error = Some(error.to_string());
            Cmd::none()
        }
    }

    let mut model = ContextModel { last_error: None };
    model.update(Event::Tick);

    assert!(model.last_error.is_some());
    let error = model.last_error.unwrap();
    assert!(error.contains("while processing update"));
    assert!(error.contains("base error"));
}

#[test]
fn test_multiple_error_types() {
    use std::io;

    struct MultiErrorModel {
        errors: Vec<String>,
    }

    impl Model for MultiErrorModel {
        type Message = TestMsg;
        fn update(&mut self, event: Event<TestMsg>) -> Cmd<TestMsg> {
            self.update_with_error_handling(event)
        }
        fn view(&self, _: &mut Frame, _: Rect) {}
    }

    impl FallibleModel for MultiErrorModel {
        fn try_update(&mut self, event: Event<TestMsg>) -> Result<Cmd<TestMsg>> {
            match event {
                Event::User(TestMsg::DoWork) => {
                    // IO error
                    Err(Error::Io(io::Error::other("io error")))
                }
                Event::User(TestMsg::CauseError) => {
                    // Terminal error
                    Err(Error::Terminal("terminal error".to_string()))
                }
                _ => Ok(Cmd::none()),
            }
        }

        fn handle_error(&mut self, error: Error) -> Cmd<TestMsg> {
            self.errors.push(error.to_string());
            Cmd::none()
        }
    }

    let mut model = MultiErrorModel { errors: Vec::new() };

    model.update(Event::User(TestMsg::DoWork));
    assert_eq!(model.errors.len(), 1);
    assert!(model.errors[0].contains("io error"));

    model.update(Event::User(TestMsg::CauseError));
    assert_eq!(model.errors.len(), 2);
    assert!(model.errors[1].contains("terminal error"));
}

#[test]
fn test_default_fallible_implementation() {
    // Test that a model can use the default FallibleModel implementation
    struct SimpleModel;

    impl Model for SimpleModel {
        type Message = ();
        fn update(&mut self, _: Event<()>) -> Cmd<()> {
            Cmd::none()
        }
        fn view(&self, _: &mut Frame, _: Rect) {}
    }

    impl FallibleModel for SimpleModel {}

    let mut model = SimpleModel;

    // Default try_update should delegate to update
    let _cmd = model.try_update(Event::Tick).unwrap();
    // Default implementation returns Cmd::none()

    // Default handle_error should log and return none
    let _cmd = model.handle_error(Error::Model("test".to_string()));
    // Default implementation returns Cmd::none()
}
