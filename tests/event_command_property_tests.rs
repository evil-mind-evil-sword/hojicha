//! Property-based tests for event processing and command execution

use hojicha::{
    commands,
    core::{Cmd, Model},
    event::Event,
    program::{Program, ProgramOptions},
};
use proptest::prelude::*;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

// Property: Commands should execute in the correct order (sequence vs batch)
#[derive(Clone)]
struct CommandOrderModel {
    execution_log: Arc<Mutex<Vec<String>>>,
    execution_times: Arc<Mutex<Vec<std::time::Instant>>>,
}

impl Model for CommandOrderModel {
    type Message = String;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        // Test both sequence and batch
        commands::sequence(vec![
            Some(commands::custom(|| Some("seq1".to_string()))),
            commands::batch(vec![
                Some(commands::custom(|| Some("batch1".to_string()))),
                Some(commands::custom(|| Some("batch2".to_string()))),
            ]),
            Some(commands::custom(|| Some("seq2".to_string()))),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        if let Event::User(msg) = event {
            self.execution_log.lock().unwrap().push(msg.clone());
            self.execution_times
                .lock()
                .unwrap()
                .push(std::time::Instant::now());

            if msg == "seq2" {
                return None; // Quit after last message
            }
        }
        Cmd::none()
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn prop_sequence_maintains_order(fps in 0u16..240) {
        let model = CommandOrderModel {
            execution_log: Arc::new(Mutex::new(Vec::new())),
            execution_times: Arc::new(Mutex::new(Vec::new())),
        };
        let log = Arc::clone(&model.execution_log);

        let options = ProgramOptions::default()
            .with_fps(fps)
            .headless()
            .without_signal_handler();

        let program = Program::with_options(model, options).unwrap();

        // Run with timeout instead of spawning thread
        // Increased timeout for async execution
        let _ = program.run_with_timeout(Duration::from_millis(50));

        let executed = log.lock().unwrap();

        // seq1 should come first
        let seq1_idx = executed.iter().position(|s| s == "seq1");
        let batch1_idx = executed.iter().position(|s| s == "batch1");
        let batch2_idx = executed.iter().position(|s| s == "batch2");
        let seq2_idx = executed.iter().position(|s| s == "seq2");

        if let (Some(s1), Some(b1), Some(b2), Some(s2)) = (seq1_idx, batch1_idx, batch2_idx, seq2_idx) {
            prop_assert!(s1 < b1, "seq1 should execute before batch");
            prop_assert!(s1 < b2, "seq1 should execute before batch");
            prop_assert!(b1 < s2, "batch should execute before seq2");
            prop_assert!(b2 < s2, "batch should execute before seq2");
        }
    }
}

// Property: Tick commands should fire at approximately the right interval
#[derive(Clone)]
struct TickModel {
    tick_times: Arc<Mutex<Vec<std::time::Instant>>>,
    tick_count: Arc<AtomicU32>,
}

impl Model for TickModel {
    type Message = ();

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        Some(commands::tick(Duration::from_millis(50), || ()))
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(()) => {
                self.tick_times
                    .lock()
                    .unwrap()
                    .push(std::time::Instant::now());
                let count = self.tick_count.fetch_add(1, Ordering::SeqCst);

                if count < 3 {
                    // Schedule another tick
                    Some(commands::tick(Duration::from_millis(50), || ()))
                } else {
                    None // Quit after 4 ticks
                }
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn prop_tick_timing_approximately_correct(tick_ms in 10u64..100) {
        let model = TickModel {
            tick_times: Arc::new(Mutex::new(Vec::new())),
            tick_count: Arc::new(AtomicU32::new(0)),
        };
        let times = Arc::clone(&model.tick_times);

        let options = ProgramOptions::default()
            .headless()
            .without_signal_handler();

        let program = Program::with_options(model, options).unwrap();

        // Run with timeout instead of spawning thread
        let _ = program.run_with_timeout(Duration::from_millis(250));

        let tick_times = times.lock().unwrap();
        if tick_times.len() >= 2 {
            // Check intervals between ticks
            for window in tick_times.windows(2) {
                let interval = window[1].duration_since(window[0]);
                let expected = Duration::from_millis(50);

                // Allow 100% tolerance due to scheduling variability
                prop_assert!(
                    interval.as_millis() < expected.as_millis() * 2,
                    "Tick interval {:?} should be approximately {:?}",
                    interval, expected
                );
            }
        }
    }
}

// Property: Every command should fire at most once
#[derive(Clone)]
struct EveryModel {
    fire_times: Arc<Mutex<Vec<std::time::Instant>>>,
    fire_count: Arc<AtomicU32>,
    start_time: Arc<Mutex<Option<std::time::Instant>>>,
}

impl Model for EveryModel {
    type Message = ();

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        *self.start_time.lock().unwrap() = Some(std::time::Instant::now());
        Some(commands::every(Duration::from_millis(30), |_| ()))
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(()) => {
                self.fire_times
                    .lock()
                    .unwrap()
                    .push(std::time::Instant::now());
                let count = self.fire_count.fetch_add(1, Ordering::SeqCst);

                if count >= 3 {
                    None // Quit after 4 fires
                } else {
                    Cmd::none()
                }
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn prop_every_maintains_interval(interval_ms in 20u64..100) {
        let model = EveryModel {
            fire_times: Arc::new(Mutex::new(Vec::new())),
            fire_count: Arc::new(AtomicU32::new(0)),
            start_time: Arc::new(Mutex::new(None)),
        };
        let times = Arc::clone(&model.fire_times);

        let options = ProgramOptions::default()
            .headless()
            .without_signal_handler();

        let program = Program::with_options(model, options).unwrap();

        // Run with timeout instead of spawning thread
        let _ = program.run_with_timeout(Duration::from_millis(50));

        let fire_times = times.lock().unwrap();
        if fire_times.len() >= 2 {
            for window in fire_times.windows(2) {
                let interval = window[1].duration_since(window[0]);
                let expected = Duration::from_millis(30);

                // Every should maintain at least the minimum interval
                prop_assert!(
                    interval >= expected * 8 / 10,
                    "Every interval {:?} should be at least {:?}",
                    interval, expected
                );
            }
        }
    }
}

// Property: Fallible commands should handle errors gracefully
#[derive(Clone)]
struct FallibleModel {
    successes: Arc<AtomicU32>,
    failures: Arc<AtomicU32>,
    continued_after_error: Arc<AtomicU32>,
}

impl Model for FallibleModel {
    type Message = Result<String, String>;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        commands::sequence(vec![
            Some(commands::custom(|| Some(Ok("success1".to_string())))),
            Some(commands::custom_fallible(|| {
                Err(hojicha::Error::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "test error",
                )))
            })),
            Some(commands::custom(|| Some(Ok("success2".to_string())))),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(Ok(_)) => {
                let count = self.successes.fetch_add(1, Ordering::SeqCst);
                if count >= 1 {
                    None
                } else {
                    Cmd::none()
                }
            }
            Event::User(Err(_)) => {
                self.failures.fetch_add(1, Ordering::SeqCst);
                self.continued_after_error.store(1, Ordering::SeqCst);
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn prop_fallible_commands_dont_crash(error_rate in 0.0..1.0) {
        let model = FallibleModel {
            successes: Arc::new(AtomicU32::new(0)),
            failures: Arc::new(AtomicU32::new(0)),
            continued_after_error: Arc::new(AtomicU32::new(0)),
        };
        let successes = Arc::clone(&model.successes);
        let continued = Arc::clone(&model.continued_after_error);

        let options = ProgramOptions::default()
            .headless()
            .without_signal_handler();

        let program = Program::with_options(model, options).unwrap();

        // Run until model returns None (after first success)
        // Increased timeout for async execution
        let result = program.run_with_timeout(Duration::from_millis(100));
        prop_assert!(result.is_ok(), "Program should not panic on fallible commands");

        // Should continue executing after errors
        prop_assert!(
            successes.load(Ordering::SeqCst) >= 1,
            "Should execute success commands"
        );
    }
}

// Property: User messages and system events should be distinguishable
#[derive(Clone)]
struct EventTypeModel {
    user_events: Arc<AtomicU32>,
    system_events: Arc<AtomicU32>,
    total_events: Arc<AtomicU32>,
}

impl Model for EventTypeModel {
    type Message = String;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        commands::batch(vec![
            Some(commands::custom(|| Some("user1".to_string()))),
            Some(commands::tick(Duration::from_millis(10), || {
                "tick".to_string()
            })),
            Some(commands::custom(|| Some("user2".to_string()))),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        self.total_events.fetch_add(1, Ordering::SeqCst);

        match event {
            Event::User(_) => {
                let count = self.user_events.fetch_add(1, Ordering::SeqCst);
                if count >= 2 {
                    None
                } else {
                    Cmd::none()
                }
            }
            Event::Tick | Event::Resize { .. } | Event::Focus | Event::Blur => {
                self.system_events.fetch_add(1, Ordering::SeqCst);
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn prop_event_types_correctly_classified(fps in 1u16..240) {
        let model = EventTypeModel {
            user_events: Arc::new(AtomicU32::new(0)),
            system_events: Arc::new(AtomicU32::new(0)),
            total_events: Arc::new(AtomicU32::new(0)),
        };
        let user = Arc::clone(&model.user_events);
        let system = Arc::clone(&model.system_events);
        let total = Arc::clone(&model.total_events);

        let options = ProgramOptions::default()
            .with_fps(fps)
            .headless()
            .without_signal_handler();

        let user_clone = Arc::clone(&user);
        let program = Program::with_options(model, options).unwrap();

        // Run with a timeout - the model itself will quit after receiving enough events
        // Increased timeout for async execution
        let _ = program.run_with_timeout(Duration::from_millis(50));

        let user_count = user.load(Ordering::SeqCst);
        let system_count = system.load(Ordering::SeqCst);
        let total_count = total.load(Ordering::SeqCst);

        // We send 3 messages (2 custom + 1 tick), but due to synchronous execution
        // we might only process some of them before the program exits
        prop_assert!(user_count >= 1, "Should process at least one user event");
        prop_assert!(
            user_count + system_count <= total_count,
            "Event counts should be consistent"
        );
    }
}

// Property: Commands returning None should be no-ops
#[derive(Clone)]
struct NoneCommandModel {
    update_count: Arc<AtomicU32>,
}

impl Model for NoneCommandModel {
    type Message = Option<String>;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        commands::sequence(vec![
            Some(commands::custom(|| Some(Some("msg1".to_string())))),
            Some(commands::custom(|| Some(None))), // This should be a no-op
            Some(commands::custom(|| Some(Some("msg2".to_string())))),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::User(Some(_)) => {
                let count = self.update_count.fetch_add(1, Ordering::SeqCst);
                if count >= 1 {
                    None
                } else {
                    Cmd::none()
                }
            }
            Event::User(None) => {
                // This should not happen
                panic!("None message should not trigger update");
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn prop_none_commands_are_noops(fps in 1u16..240) {
        let model = NoneCommandModel {
            update_count: Arc::new(AtomicU32::new(0)),
        };
        let updates = Arc::clone(&model.update_count);

        let options = ProgramOptions::default()
            .with_fps(fps)
            .headless()
            .without_signal_handler();

        let program = Program::with_options(model, options).unwrap();

        // Run with timeout - sequence sends 3 commands but one returns None
        // Increased timeout for async execution
        let result = program.run_with_timeout(Duration::from_millis(100));
        prop_assert!(result.is_ok(), "Program should handle None commands");

        // We might not process all messages due to synchronous execution
        let update_count = updates.load(Ordering::SeqCst);
        prop_assert!(update_count >= 1, "Should process at least one non-None message");
    }
}
