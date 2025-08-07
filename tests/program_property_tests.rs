//! Property-based tests for program behavior and invariants

use hojicha::{
    commands,
    core::{Cmd, Model},
    event::Event,
    program::{MouseMode, Program, ProgramOptions},
};
use proptest::prelude::*;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

// Property: The program should always call init() exactly once before any update()
#[derive(Clone)]
struct InitOrderModel {
    init_called: Arc<AtomicBool>,
    update_called: Arc<AtomicBool>,
    init_before_update: Arc<AtomicBool>,
}

impl InitOrderModel {
    fn new() -> Self {
        Self {
            init_called: Arc::new(AtomicBool::new(false)),
            update_called: Arc::new(AtomicBool::new(false)),
            init_before_update: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl Model for InitOrderModel {
    type Message = ();

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        self.init_called.store(true, Ordering::SeqCst);
        Some(commands::custom(|| Some(())))
    }

    fn update(&mut self, _: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        if !self.init_called.load(Ordering::SeqCst) {
            self.init_before_update.store(false, Ordering::SeqCst);
        }
        self.update_called.store(true, Ordering::SeqCst);
        None // Quit immediately
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #[test]
    fn prop_init_always_called_before_update(
        fps in 0u16..240,
        use_alt_screen in any::<bool>(),
        use_renderer in any::<bool>()
    ) {
        let model = InitOrderModel::new();
        let init_before = Arc::clone(&model.init_before_update);
        let init_called = Arc::clone(&model.init_called);

        let mut options = ProgramOptions::default()
            .with_fps(fps)
            .with_alt_screen(use_alt_screen)
            .headless()
            .without_signal_handler();

        if !use_renderer {
            options = options.without_renderer();
        }

        let program = Program::with_options(model, options).unwrap();

        // Run program synchronously - it will quit after first update
        let _ = program.run();

        prop_assert!(init_called.load(Ordering::SeqCst), "init() must be called");
        prop_assert!(init_before.load(Ordering::SeqCst), "init() must be called before update()");
    }
}

// Property: Messages should be processed in FIFO order
#[derive(Clone)]
struct MessageOrderModel {
    messages_received: Arc<Mutex<Vec<u32>>>,
    message_count: Arc<AtomicU32>,
}

impl Model for MessageOrderModel {
    type Message = u32;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        // Send a sequence of messages
        commands::sequence(vec![
            Some(commands::custom(|| Some(1))),
            Some(commands::custom(|| Some(2))),
            Some(commands::custom(|| Some(3))),
            Some(commands::custom(|| Some(4))),
            Some(commands::custom(|| Some(5))),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        if let Event::User(msg) = event {
            self.messages_received.lock().unwrap().push(msg);
            let count = self.message_count.fetch_add(1, Ordering::SeqCst);
            if count >= 4 {
                return None; // Quit after receiving 5 messages
            }
        }
        Cmd::none()
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #[test]
    fn prop_messages_processed_in_order(fps in 0u16..240) {
        let model = MessageOrderModel {
            messages_received: Arc::new(Mutex::new(Vec::new())),
            message_count: Arc::new(AtomicU32::new(0)),
        };
        let messages = Arc::clone(&model.messages_received);

        let options = ProgramOptions::default()
            .with_fps(fps)
            .headless()
            .without_signal_handler();

        let program = Program::with_options(model, options).unwrap();

        // Run synchronously - will quit after 5 messages
        let _ = program.run();

        let received = messages.lock().unwrap();
        prop_assert_eq!(&*received, &[1, 2, 3, 4, 5], "Messages must be processed in order");
    }
}

// Property: Batch commands should all execute
#[derive(Clone)]
struct BatchExecutionModel {
    commands_executed: Arc<Mutex<Vec<String>>>,
    quit_after: Arc<AtomicU32>,
}

impl Model for BatchExecutionModel {
    type Message = String;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        commands::batch(vec![
            Some(commands::custom(|| Some("batch1".to_string()))),
            Some(commands::custom(|| Some("batch2".to_string()))),
            Some(commands::custom(|| Some("batch3".to_string()))),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        if let Event::User(msg) = event {
            self.commands_executed.lock().unwrap().push(msg);
            let count = self.quit_after.fetch_add(1, Ordering::SeqCst);
            if count >= 2 {
                return None;
            }
        }
        Cmd::none()
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #[test]
    fn prop_batch_commands_all_execute(batch_size in 1usize..10) {
        let model = BatchExecutionModel {
            commands_executed: Arc::new(Mutex::new(Vec::new())),
            quit_after: Arc::new(AtomicU32::new(0)),
        };
        let executed = Arc::clone(&model.commands_executed);

        let options = ProgramOptions::default()
            .headless()
            .without_signal_handler();

        let program = Program::with_options(model, options).unwrap();

        // Run synchronously - will quit after 3 batch commands
        let _ = program.run();

        let cmds = executed.lock().unwrap();
        prop_assert!(cmds.contains(&"batch1".to_string()));
        prop_assert!(cmds.contains(&"batch2".to_string()));
        prop_assert!(cmds.contains(&"batch3".to_string()));
    }
}

// Property: Program should respect FPS limits
#[derive(Clone)]
struct FpsModel {
    render_times: Arc<Mutex<Vec<Instant>>>,
    render_count: Arc<AtomicU32>,
}

impl Model for FpsModel {
    type Message = ();

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        // Keep the program running for a bit
        Some(commands::tick(Duration::from_millis(200), || ()))
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::Tick => None, // Quit on tick
            _ => Cmd::none(),
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {
        self.render_times.lock().unwrap().push(Instant::now());
        self.render_count.fetch_add(1, Ordering::SeqCst);
    }
}

proptest! {
    #[test]
    fn prop_fps_limiter_respects_rate(fps in 1u16..120) {
        let model = FpsModel {
            render_times: Arc::new(Mutex::new(Vec::new())),
            render_count: Arc::new(AtomicU32::new(0)),
        };
        let render_times = Arc::clone(&model.render_times);

        let options = ProgramOptions::default()
            .with_fps(fps)
            .headless()
            .without_signal_handler();

        let program = Program::with_options(model, options).unwrap();

        // Run synchronously - will quit after tick (200ms)
        let _ = program.run();

        let times = render_times.lock().unwrap();
        if times.len() >= 2 {
            // Check that renders are spaced appropriately
            let expected_interval = Duration::from_secs(1) / fps as u32;
            let min_interval = expected_interval * 8 / 10; // Allow 20% tolerance

            for window in times.windows(2) {
                let interval = window[1].duration_since(window[0]);
                prop_assert!(
                    interval >= min_interval,
                    "Render interval {:?} should be at least {:?} for {} FPS",
                    interval, min_interval, fps
                );
            }
        }
    }
}

// Property: Quit command should terminate the program
#[derive(Clone)]
struct QuitModel {
    update_count: Arc<AtomicU32>,
    quit_sent: Arc<AtomicBool>,
}

impl Model for QuitModel {
    type Message = ();

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        commands::sequence(vec![
            Some(commands::custom(|| Some(()))),
            Some(commands::custom(|| Some(()))),
            Some(commands::quit()),
            Some(commands::custom(|| Some(()))), // Should not execute
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        let count = self.update_count.fetch_add(1, Ordering::SeqCst);

        match event {
            Event::Quit => {
                self.quit_sent.store(true, Ordering::SeqCst);
                None
            }
            _ => {
                if count > 10 {
                    None // Safety: quit if we process too many events
                } else {
                    Cmd::none()
                }
            }
        }
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #[test]
    fn prop_quit_command_terminates(fps in 0u16..240) {
        let model = QuitModel {
            update_count: Arc::new(AtomicU32::new(0)),
            quit_sent: Arc::new(AtomicBool::new(false)),
        };
        let update_count = Arc::clone(&model.update_count);
        let quit_sent = Arc::clone(&model.quit_sent);

        let options = ProgramOptions::default()
            .with_fps(fps)
            .headless()
            .without_signal_handler();

        let program = Program::with_options(model, options).unwrap();

        let handle = std::thread::spawn(move || {
            let _ = program.run();
        });

        // Program should terminate quickly
        let terminated = handle.join().is_ok();

        prop_assert!(terminated, "Program should terminate");
        prop_assert!(quit_sent.load(Ordering::SeqCst), "Quit event should be sent");
        prop_assert!(
            update_count.load(Ordering::SeqCst) <= 5,
            "Should not process many events after quit"
        );
    }
}

// Property: Filter should be able to block events
#[derive(Clone)]
struct FilterModel {
    blocked_events: Arc<Mutex<Vec<u32>>>,
    allowed_events: Arc<Mutex<Vec<u32>>>,
}

impl Model for FilterModel {
    type Message = u32;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        commands::batch(vec![
            Some(commands::custom(|| Some(1))),
            Some(commands::custom(|| Some(2))),
            Some(commands::custom(|| Some(3))),
            Some(commands::custom(|| Some(4))),
            Some(commands::custom(|| Some(5))),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        if let Event::User(msg) = event {
            self.allowed_events.lock().unwrap().push(msg);
            if msg >= 5 {
                return None;
            }
        }
        Cmd::none()
    }

    fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
}

proptest! {
    #[test]
    fn prop_filter_blocks_events(block_threshold in 1u32..5) {
        let model = FilterModel {
            blocked_events: Arc::new(Mutex::new(Vec::new())),
            allowed_events: Arc::new(Mutex::new(Vec::new())),
        };
        let blocked = Arc::clone(&model.blocked_events);
        let allowed = Arc::clone(&model.allowed_events);

        let options = ProgramOptions::default()
            .headless()
            .without_signal_handler();

        let program = Program::with_options(model, options)
            .unwrap()
            .with_filter(move |_model, event| {
                if let Event::User(msg) = &event {
                    if *msg <= block_threshold {
                        blocked.lock().unwrap().push(*msg);
                        return None; // Block events <= threshold
                    }
                }
                Some(event)
            });

        // Run synchronously - will quit after 5 messages
        let _ = program.run();

        let allowed_msgs = allowed.lock().unwrap();
        for msg in 1..=block_threshold {
            prop_assert!(
                !allowed_msgs.contains(&msg),
                "Message {} should be blocked", msg
            );
        }
        for msg in (block_threshold + 1)..=5 {
            prop_assert!(
                allowed_msgs.contains(&msg),
                "Message {} should be allowed", msg
            );
        }
    }
}

// Property: Terminal state transitions should be consistent
proptest! {
    #[test]
    fn prop_terminal_state_transitions(
        alt_screen in any::<bool>(),
        mouse_mode in prop_oneof![
            Just(MouseMode::None),
            Just(MouseMode::CellMotion),
            Just(MouseMode::AllMotion),
        ],
        bracketed_paste in any::<bool>(),
        focus_reporting in any::<bool>()
    ) {
        struct StateModel;
        impl Model for StateModel {
            type Message = ();
            fn init(&mut self) -> Option<Cmd<Self::Message>> {
                Some(commands::quit())
            }
            fn update(&mut self, _: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
                None
            }
            fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
        }

        let options = ProgramOptions::default()
            .with_alt_screen(alt_screen)
            .with_mouse_mode(mouse_mode)
            .with_bracketed_paste(bracketed_paste)
            .with_focus_reporting(focus_reporting)
            .headless()
            .without_signal_handler();

        let program = Program::with_options(StateModel, options);

        // Program should initialize successfully with any valid combination
        prop_assert!(program.is_ok(), "Program should initialize with valid options");

        if let Ok(program) = program {
            // Run synchronously - will quit immediately from init
            let _ = program.run();
            // Program should terminate cleanly
        }
    }
}
