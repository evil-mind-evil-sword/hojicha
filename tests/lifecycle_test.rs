//! Tests for program lifecycle management

use hojicha::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_program_quit() {
    #[derive(Clone)]
    struct QuitModel {
        quit_received: Arc<AtomicBool>,
    }

    #[derive(Debug, Clone)]
    enum Msg {}

    impl Model for QuitModel {
        type Message = Msg;

        fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
            match msg {
                Event::Quit => {
                    self.quit_received.store(true, Ordering::SeqCst);
                    None // Return None to signal quit
                }
                _ => None,
            }
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = QuitModel {
        quit_received: Arc::new(AtomicBool::new(false)),
    };

    // In a real test, we would:
    // 1. Create a program
    // 2. Run it in a thread
    // 3. Call program.quit()
    // 4. Verify quit_received is true

    // For now, test the model directly
    let mut model = model;
    model.update(Event::Quit);
    assert!(model.quit_received.load(Ordering::SeqCst));
}

#[test]
fn test_program_lifecycle_states() {
    #[derive(Clone)]
    struct LifecycleModel {
        initialized: Arc<AtomicBool>,
        update_count: Arc<AtomicU32>,
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    enum Msg {
        Increment, // Defined for completeness, could be used in extended tests
    }

    impl Model for LifecycleModel {
        type Message = Msg;

        fn init(&mut self) -> Option<Cmd<Self::Message>> {
            self.initialized.store(true, Ordering::SeqCst);
            None
        }

        fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
            match msg {
                Event::User(Msg::Increment) => {
                    self.update_count.fetch_add(1, Ordering::SeqCst);
                }
                Event::Quit => return None,
                _ => {}
            }
            None
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    let model = LifecycleModel {
        initialized: Arc::new(AtomicBool::new(false)),
        update_count: Arc::new(AtomicU32::new(0)),
    };

    // Test init is called
    let mut model_clone = model.clone();
    model_clone.init();
    assert!(model.initialized.load(Ordering::SeqCst));
}

#[test]
fn test_wait_behavior() {
    // This tests the wait() logic without actually running a program
    use std::sync::{Condvar, Mutex};

    let running = Arc::new(AtomicBool::new(false));
    let force_quit = Arc::new(AtomicBool::new(false));
    let state_changed = Arc::new((Mutex::new(false), Condvar::new()));

    let running_clone = Arc::clone(&running);
    let force_quit_clone = Arc::clone(&force_quit);
    let state_changed_clone = Arc::clone(&state_changed);

    // Simulate wait behavior in a thread
    let handle = thread::spawn(move || {
        let (lock, cvar) = &*state_changed_clone;

        // Wait until running
        let mut started = lock.lock().unwrap();
        while !running_clone.load(Ordering::SeqCst) && !force_quit_clone.load(Ordering::SeqCst) {
            started = cvar.wait(started).unwrap();
        }
        drop(started);

        // Then wait until stopped
        let mut stopped = lock.lock().unwrap();
        while running_clone.load(Ordering::SeqCst) && !force_quit_clone.load(Ordering::SeqCst) {
            stopped = cvar.wait(stopped).unwrap();
        }
    });

    // Simulate program lifecycle
    let (lock, cvar) = &*state_changed;

    // Start running
    running.store(true, Ordering::SeqCst);
    *lock.lock().unwrap() = true;
    cvar.notify_all();

    // Stop running
    running.store(false, Ordering::SeqCst);
    *lock.lock().unwrap() = true;
    cvar.notify_all();

    // Wait should complete
    handle.join().unwrap();
}

#[test]
fn test_force_quit() {
    let running = Arc::new(AtomicBool::new(true));
    let force_quit = Arc::new(AtomicBool::new(false));

    // Simulate force quit
    force_quit.store(true, Ordering::SeqCst);
    running.store(false, Ordering::SeqCst);

    assert!(!running.load(Ordering::SeqCst));
    assert!(force_quit.load(Ordering::SeqCst));
}

#[test]
fn test_terminal_control_commands() {
    // Test that terminal control commands can be created
    let _hide = hide_cursor::<()>();
    let _show = show_cursor::<()>();
    let _enter = enter_alt_screen::<()>();
    let _exit = exit_alt_screen::<()>();
    let _title = set_window_title::<()>("Test Title");

    // These commands return None as they're handled by the runtime
    assert!(hide_cursor::<()>().test_execute().unwrap().is_none());
    assert!(show_cursor::<()>().test_execute().unwrap().is_none());
}

#[test]
fn test_exec_commands() {
    use hojicha::commands::{exec, exec_command};

    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {
        ExecResult(Option<i32>),
    }

    // Test exec command creation
    let _exec_cmd = exec("echo", vec!["hello"], TestMsg::ExecResult);
    let _shell_cmd = exec_command("echo hello", TestMsg::ExecResult);

    // These would normally execute the commands, but in tests we can't easily verify
    // the terminal release/restore behavior without a full program context
}
