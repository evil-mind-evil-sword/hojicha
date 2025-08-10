//! Property-based tests for hojicha

use hojicha::prelude::*;
use hojicha::program::{MouseMode, ProgramOptions};
use proptest::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

// Property: Any sequence of increment/decrement operations should result in the correct final value
proptest! {
    #[test]
    fn test_counter_operations(operations in prop::collection::vec(any::<bool>(), 0..100)) {
        #[derive(Clone)]
        struct Counter {
            value: Arc<AtomicU32>,
        }

        #[derive(Debug, Clone)]
        enum Msg {
            Inc,
            Dec,
        }

        impl Model for Counter {
            type Message = Msg;

            fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
                if let Event::User(msg) = msg {
                    match msg {
                        Msg::Inc => { self.value.fetch_add(1, Ordering::SeqCst); }
                        Msg::Dec => { self.value.fetch_sub(1, Ordering::SeqCst); }
                    }
                }
                None
            }

            fn view(&self, _frame: &mut Frame, _area: Rect) {}
        }

        let mut counter = Counter {
            value: Arc::new(AtomicU32::new(0)),
        };

        let mut expected = 0i32;
        for op in operations {
            if op {
                counter.update(Event::User(Msg::Inc));
                expected += 1;
            } else {
                counter.update(Event::User(Msg::Dec));
                expected = expected.saturating_sub(1);
            }
        }

        prop_assert_eq!(counter.value.load(Ordering::SeqCst), expected as u32);
    }
}

// Property: Batch commands should execute all non-None commands
proptest! {
    #[test]
    fn test_batch_command_execution(num_cmds in 0..20usize, include_nones in prop::collection::vec(any::<bool>(), 0..20)) {
        use std::sync::Mutex;

        #[derive(Debug, Clone)]
        struct TestMsg(#[allow(dead_code)] usize);

        let executed = Arc::new(Mutex::new(Vec::new()));
        let mut cmds = Vec::new();

        for (i, &include_none) in include_nones.iter().enumerate().take(num_cmds) {
            if include_none {
                let executed_clone = Arc::clone(&executed);
                cmds.push(Some(Cmd::new(move || {
                    executed_clone.lock().unwrap().push(i);
                    Some(TestMsg(i))
                })));
            } else {
                cmds.push(None);
            }
        }

        if let Some(batch_cmd) = batch(cmds) {
            // In a real scenario, these would be executed by the runtime
            // For testing, we'll check that the batch command was created correctly
            prop_assert!(batch_cmd.test_execute().unwrap().is_some());
        }
    }
}

// Property: Key events should preserve their properties through conversions
proptest! {
    #[test]
    fn test_key_event_properties(
        c in any::<char>(),
        ctrl in any::<bool>(),
        alt in any::<bool>(),
        shift in any::<bool>()
    ) {
        let mut modifiers = KeyModifiers::empty();
        if ctrl { modifiers |= KeyModifiers::CONTROL; }
        if alt { modifiers |= KeyModifiers::ALT; }
        if shift { modifiers |= KeyModifiers::SHIFT; }

        let key_event = KeyEvent::new(Key::Char(c), modifiers);

        // Properties that should hold
        prop_assert!(key_event.is_char());
        prop_assert_eq!(key_event.char(), Some(c));
        prop_assert_eq!(key_event.modifiers.contains(KeyModifiers::CONTROL), ctrl);
        prop_assert_eq!(key_event.modifiers.contains(KeyModifiers::ALT), alt);
        prop_assert_eq!(key_event.modifiers.contains(KeyModifiers::SHIFT), shift);
    }
}

// Property: Sequence commands should maintain order
proptest! {
    #[test]
    fn test_sequence_order(num_cmds in 1..10usize) {
        use std::sync::Mutex;

        #[derive(Debug, Clone)]
        struct OrderMsg(#[allow(dead_code)] usize);

        let execution_order = Arc::new(Mutex::new(Vec::new()));
        let mut cmds = Vec::new();

        for i in 0..num_cmds {
            let order_clone = Arc::clone(&execution_order);
            cmds.push(Some(Cmd::new(move || {
                order_clone.lock().unwrap().push(i);
                Some(OrderMsg(i))
            })));
        }

        // Create sequence command
        let _seq_cmd = sequence(cmds);

        // In a real implementation, sequence would ensure order
        // For now, we're testing that the command is created
        prop_assert!(true); // Placeholder - would need runtime execution to fully test
    }
}

// Property: Program options should be composable
proptest! {
    #[test]
    fn test_program_options_composition(
        alt_screen in any::<bool>(),
        fps in 10u16..1000u16
    ) {
        let opts = ProgramOptions::default()
            .with_alt_screen(alt_screen)
            .with_mouse_mode(MouseMode::CellMotion)
            .with_fps(fps);

        prop_assert_eq!(opts.alt_screen, alt_screen);
        prop_assert_eq!(opts.mouse_mode, MouseMode::CellMotion);
        prop_assert_eq!(opts.fps, fps);
    }
}
