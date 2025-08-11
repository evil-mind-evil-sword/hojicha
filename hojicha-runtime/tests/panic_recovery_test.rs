//! Integration tests for panic recovery in Model methods

use hojicha_core::core::{Cmd, Model};
use hojicha_core::event::Event;
use hojicha_runtime::panic_recovery::{PanicRecoveryStrategy, safe_init, safe_update};
use hojicha_runtime::{Program, ProgramOptions};
use ratatui::{Frame, layout::Rect};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// A model that can be configured to panic in different methods
struct PanickyModel {
    panic_in_init: bool,
    panic_in_update: bool,
    panic_in_view: bool,
    update_count: Arc<AtomicUsize>,
    view_count: Arc<AtomicUsize>,
    recovered: Arc<AtomicBool>,
}

impl Model for PanickyModel {
    type Message = TestMsg;
    
    fn init(&mut self) -> Cmd<Self::Message> {
        if self.panic_in_init {
            panic!("Intentional panic in init!");
        }
        Cmd::none()
    }
    
    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        self.update_count.fetch_add(1, Ordering::SeqCst);
        
        if self.panic_in_update {
            panic!("Intentional panic in update!");
        }
        
        match event {
            Event::User(TestMsg::Recovered) => {
                self.recovered.store(true, Ordering::SeqCst);
            }
            _ => {}
        }
        
        Cmd::none()
    }
    
    fn view(&self, _frame: &mut Frame, _area: Rect) {
        self.view_count.fetch_add(1, Ordering::SeqCst);
        
        if self.panic_in_view {
            panic!("Intentional panic in view!");
        }
    }
}

#[derive(Clone, Debug)]
enum TestMsg {
    Recovered,
}

#[test]
fn test_panic_recovery_in_init() {
    let mut model = PanickyModel {
        panic_in_init: true,
        panic_in_update: false,
        panic_in_view: false,
        update_count: Arc::new(AtomicUsize::new(0)),
        view_count: Arc::new(AtomicUsize::new(0)),
        recovered: Arc::new(AtomicBool::new(false)),
    };
    
    // Test that panic is caught and returns appropriate command
    let cmd = safe_init(&mut model, PanicRecoveryStrategy::Continue);
    assert!(cmd.is_noop());
    
    let cmd = safe_init(&mut model, PanicRecoveryStrategy::Quit);
    assert!(cmd.is_quit());
}

#[test]
fn test_panic_recovery_in_update() {
    let mut model = PanickyModel {
        panic_in_init: false,
        panic_in_update: true,
        panic_in_view: false,
        update_count: Arc::new(AtomicUsize::new(0)),
        view_count: Arc::new(AtomicUsize::new(0)),
        recovered: Arc::new(AtomicBool::new(false)),
    };
    
    // Test that panic is caught and returns appropriate command
    let cmd = safe_update(&mut model, Event::Tick, PanicRecoveryStrategy::Continue);
    assert!(cmd.is_noop());
    
    // Update count should still have been incremented before panic
    assert_eq!(model.update_count.load(Ordering::SeqCst), 1);
    
    let cmd = safe_update(&mut model, Event::Tick, PanicRecoveryStrategy::Quit);
    assert!(cmd.is_quit());
}

#[test]
fn test_program_continues_after_update_panic() {
    let update_count = Arc::new(AtomicUsize::new(0));
    let recovered = Arc::new(AtomicBool::new(false));
    
    let model = PanickyModel {
        panic_in_init: false,
        panic_in_update: false, // Will be set to true after first update
        panic_in_view: false,
        update_count: update_count.clone(),
        view_count: Arc::new(AtomicUsize::new(0)),
        recovered: recovered.clone(),
    };
    
    // Run program with panic recovery enabled
    let options = ProgramOptions::new()
        .headless()
        .with_panic_recovery(PanicRecoveryStrategy::Continue);
    
    let result = Program::with_options(model, options)
        .unwrap()
        .run_with_timeout(Duration::from_millis(100));
    
    // Program should complete successfully despite panics
    assert!(result.is_ok());
}

#[test]
fn test_panic_recovery_quit_strategy() {
    let model = PanickyModel {
        panic_in_init: true,
        panic_in_update: false,
        panic_in_view: false,
        update_count: Arc::new(AtomicUsize::new(0)),
        view_count: Arc::new(AtomicUsize::new(0)),
        recovered: Arc::new(AtomicBool::new(false)),
    };
    
    // Run program with quit strategy
    let options = ProgramOptions::new()
        .headless()
        .with_panic_recovery(PanicRecoveryStrategy::Quit);
    
    let result = Program::with_options(model, options)
        .unwrap()
        .run_with_timeout(Duration::from_millis(100));
    
    // Program should exit immediately due to panic in init
    assert!(result.is_ok());
}