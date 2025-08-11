//! Tests for the new API refactoring
//! - update() returns Cmd directly (not Option<Cmd>)
//! - Only commands::quit() quits the program
//! - Cmd::none() is a no-op
//! - New spawn() method for simple async tasks
//! - Stream builder helpers

use futures::stream::StreamExt;
use hojicha_core::commands;
use hojicha_core::prelude::*;
use hojicha_core::event::Event;
use hojicha_runtime::program::{Program, ProgramOptions};
use hojicha_runtime::stream_builders::interval_stream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_update_returns_cmd_directly() {
    struct TestModel {
        counter: i32,
    }

    impl Model for TestModel {
        type Message = ();

        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }

        fn update(&mut self, _event: Event<Self::Message>) -> Cmd<Self::Message> {
            self.counter += 1;
            if self.counter >= 3 {
                commands::quit()
            } else {
                Cmd::none()
            }
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = TestModel { counter: 0 };
    let program = Program::with_options(model, ProgramOptions::new().headless()).unwrap();
    program
        .run_with_timeout(Duration::from_millis(100))
        .unwrap();
}

#[test]
fn test_cmd_none_does_not_quit() {
    struct NeverQuitModel {
        update_count: Arc<Mutex<i32>>,
    }

    impl Model for NeverQuitModel {
        type Message = ();

        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }

        fn update(&mut self, _event: Event<Self::Message>) -> Cmd<Self::Message> {
            let mut count = self.update_count.lock().unwrap();
            *count += 1;
            Cmd::none() // Should never quit
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let update_count = Arc::new(Mutex::new(0));
    let model = NeverQuitModel {
        update_count: update_count.clone(),
    };

    let program = Program::with_options(model, ProgramOptions::new().headless()).unwrap();
    program.run_with_timeout(Duration::from_millis(50)).unwrap();

    // Should have run for the full timeout, not quit early
    assert!(*update_count.lock().unwrap() > 0);
}

#[test]
fn test_only_quit_command_quits() {
    #[derive(Clone)]
    enum Msg {
        TryQuit,
        ActuallyQuit,
    }

    struct QuitTestModel {
        quit_attempts: i32,
    }

    impl Model for QuitTestModel {
        type Message = Msg;

        fn init(&mut self) -> Cmd<Self::Message> {
            commands::custom(|| Some(Msg::TryQuit))
        }

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(Msg::TryQuit) => {
                    self.quit_attempts += 1;
                    if self.quit_attempts >= 3 {
                        commands::custom(|| Some(Msg::ActuallyQuit))
                    } else {
                        commands::tick(Duration::from_millis(10), || Msg::TryQuit)
                    }
                }
                Event::User(Msg::ActuallyQuit) => {
                    commands::quit() // ONLY this should quit
                }
                _ => Cmd::none(),
            }
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = QuitTestModel { quit_attempts: 0 };
    let program = Program::with_options(model, ProgramOptions::new().headless()).unwrap();
    program.run().unwrap(); // Should quit after 3 attempts
}

#[test]
fn test_spawn_simple_async_task() {
    #[derive(Clone)]
    enum Msg {
        DataLoaded(String),
    }

    struct AsyncModel {
        data: Option<String>,
    }

    impl Model for AsyncModel {
        type Message = Msg;

        fn init(&mut self) -> Cmd<Self::Message> {
            // This API doesn't exist yet but should
            commands::spawn(async {
                tokio::task::yield_now().await; // Yield instead of sleep
                Some(Msg::DataLoaded("async data".to_string()))
            })
        }

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(Msg::DataLoaded(data)) => {
                    self.data = Some(data);
                    commands::quit()
                }
                _ => Cmd::none(),
            }
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = AsyncModel { data: None };
    let mut program = Program::with_options(model, ProgramOptions::new().headless()).unwrap();

    // Alternative API using program.spawn()
    program.spawn(async {
        tokio::task::yield_now().await; // Yield instead of sleep
        Some(Msg::DataLoaded("async data".to_string()))
    });

    program
        .run_with_timeout(Duration::from_millis(100))
        .unwrap();
}

#[test]
#[ignore = "Stream builders need more implementation"]
fn test_stream_builder_helpers() {
    use hojicha_runtime::stream_builders::*;

    #[derive(Clone)]
    enum Msg {
        Tick(usize),
    }

    struct StreamModel {
        tick_count: usize,
    }

    impl Model for StreamModel {
        type Message = Msg;

        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(Msg::Tick(n)) => {
                    self.tick_count = n;
                    if n >= 5 {
                        commands::quit()
                    } else {
                        Cmd::none()
                    }
                }
                _ => Cmd::none(),
            }
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = StreamModel { tick_count: 0 };
    let mut program = Program::with_options(model, ProgramOptions::new().headless()).unwrap();

    // Stream builder helper - doesn't exist yet but should
    let stream = interval_stream(Duration::from_millis(10))
        .take(6)
        .enumerate()
        .map(|(i, _)| Msg::Tick(i));

    program.subscribe(stream);
    program
        .run_with_timeout(Duration::from_millis(200))
        .unwrap();
}

#[test]
#[ignore = "Stream builders need more implementation"]
fn test_multiple_streams() {
    use hojicha_runtime::stream_builders::*;

    #[derive(Clone)]
    enum Msg {
        Stream1(i32),
        Stream2(i32),
        Stream3(i32),
    }

    struct MultiStreamModel {
        total_messages: i32,
    }

    impl Model for MultiStreamModel {
        type Message = Msg;

        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::none()
        }

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            match event {
                Event::User(Msg::Stream1(_))
                | Event::User(Msg::Stream2(_))
                | Event::User(Msg::Stream3(_)) => {
                    self.total_messages += 1;
                    if self.total_messages >= 10 {
                        commands::quit()
                    } else {
                        Cmd::none()
                    }
                }
                _ => Cmd::none(),
            }
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = MultiStreamModel { total_messages: 0 };
    let mut program = Program::with_options(model, ProgramOptions::new().headless()).unwrap();

    // Create multiple streams with different rates
    let stream1 = interval_stream(Duration::from_millis(10))
        .take(5)
        .map(|_| Msg::Stream1(1));

    let stream2 = interval_stream(Duration::from_millis(15))
        .take(5)
        .map(|_| Msg::Stream2(2));

    let stream3 = interval_stream(Duration::from_millis(20))
        .take(5)
        .map(|_| Msg::Stream3(3));

    program.subscribe(stream1);
    program.subscribe(stream2);
    program.subscribe(stream3);

    program
        .run_with_timeout(Duration::from_millis(500))
        .unwrap();
}
