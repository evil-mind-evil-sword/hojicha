//! Global panic handler for graceful TUI recovery
//!
//! This module provides a panic handler that ensures the terminal is restored
//! to a usable state when a panic occurs, and optionally logs panic information.

use log::error;
use std::io::Write;
use std::panic::{self, PanicHookInfo};
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to track if we're in a TUI context
static TUI_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Cleanup function to be called on panic
static mut CLEANUP_FN: Option<Box<dyn Fn() + Send + Sync>> = None;

/// Install a panic handler that will restore the terminal on panic
///
/// This should be called at the start of your program, before entering the TUI.
///
/// # Example
/// ```no_run
/// use hojicha::panic_handler;
///
/// fn main() {
///     panic_handler::install();
///     // ... run your TUI application
/// }
/// ```
pub fn install() {
    panic::set_hook(Box::new(|panic_info| {
        handle_panic(panic_info);
    }));
}

/// Install a panic handler with a custom cleanup function
///
/// The cleanup function will be called before the terminal is restored.
/// This is useful for saving application state or performing other cleanup.
///
/// # Safety
/// The cleanup function must be thread-safe as it may be called from any thread.
///
/// # Example
/// ```no_run
/// use hojicha::panic_handler;
///
/// fn main() {
///     panic_handler::install_with_cleanup(|| {
///         // Save application state, close files, etc.
///         eprintln!("Saving application state before exit...");
///     });
///     // ... run your TUI application
/// }
/// ```
pub fn install_with_cleanup<F>(cleanup: F)
where
    F: Fn() + Send + Sync + 'static,
{
    unsafe {
        CLEANUP_FN = Some(Box::new(cleanup));
    }
    install();
}

/// Mark that the TUI is active
///
/// This should be called when entering TUI mode and ensures that
/// the panic handler knows to restore the terminal.
pub fn set_tui_active(active: bool) {
    TUI_ACTIVE.store(active, Ordering::SeqCst);
}

/// Create a guard that automatically sets TUI active/inactive
pub struct TuiGuard;

impl TuiGuard {
    /// Create a new TUI guard
    pub fn new() -> Self {
        set_tui_active(true);
        TuiGuard
    }
}

impl Drop for TuiGuard {
    fn drop(&mut self) {
        set_tui_active(false);
    }
}

/// The actual panic handler
fn handle_panic(panic_info: &PanicHookInfo) {
    // First, log the panic if logging is available
    error!("PANIC: {}", panic_info);

    // Run custom cleanup if provided
    unsafe {
        if let Some(ref cleanup) = CLEANUP_FN {
            cleanup();
        }
    }

    // If we're in TUI mode, restore the terminal
    if TUI_ACTIVE.load(Ordering::SeqCst) {
        restore_terminal();
    }

    // Print panic information to stderr
    eprintln!("\n\n==================== PANIC ====================");
    eprintln!("{}", panic_info);

    // Print location if available
    if let Some(location) = panic_info.location() {
        eprintln!(
            "\nLocation: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    // Print backtrace if available
    if let Ok(var) = std::env::var("RUST_BACKTRACE") {
        if var == "1" || var == "full" {
            eprintln!("\nBacktrace:");
            eprintln!("{:?}", std::backtrace::Backtrace::capture());
        }
    } else {
        eprintln!("\nNote: Set RUST_BACKTRACE=1 to see a backtrace");
    }

    eprintln!("================================================\n");
}

/// Attempt to restore the terminal to a usable state
fn restore_terminal() {
    use crossterm::{
        cursor,
        event::{DisableBracketedPaste, DisableFocusChange, DisableMouseCapture},
        execute,
        terminal::{self, LeaveAlternateScreen},
    };

    // Try to restore terminal state
    let _ = execute!(
        std::io::stderr(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste,
        DisableFocusChange,
        cursor::Show,
    );

    // Disable raw mode
    let _ = terminal::disable_raw_mode();

    // Flush stderr to ensure all output is visible
    let _ = std::io::stderr().flush();
}

/// A panic hook that can be used in tests to verify panic behavior
#[cfg(test)]
pub struct TestPanicHook {
    pub panicked: Arc<AtomicBool>,
    pub panic_message: Arc<std::sync::Mutex<Option<String>>>,
}

#[cfg(test)]
impl TestPanicHook {
    /// Create a new test panic hook
    pub fn new() -> Self {
        Self {
            panicked: Arc::new(AtomicBool::new(false)),
            panic_message: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Install this hook as the panic handler
    pub fn install(&self) {
        let panicked = Arc::clone(&self.panicked);
        let panic_message = Arc::clone(&self.panic_message);

        panic::set_hook(Box::new(move |panic_info| {
            panicked.store(true, Ordering::SeqCst);

            let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };

            *panic_message.lock().unwrap() = Some(msg);
        }));
    }

    /// Check if a panic occurred
    pub fn did_panic(&self) -> bool {
        self.panicked.load(Ordering::SeqCst)
    }

    /// Get the panic message if one occurred
    pub fn get_panic_message(&self) -> Option<String> {
        self.panic_message.lock().unwrap().clone()
    }

    /// Reset the panic state
    pub fn reset(&self) {
        self.panicked.store(false, Ordering::SeqCst);
        *self.panic_message.lock().unwrap() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    #[test]
    fn test_panic_hook_captures_panic() {
        let hook = TestPanicHook::new();
        hook.install();

        let result = panic::catch_unwind(|| {
            panic!("Test panic message");
        });

        assert!(result.is_err());
        assert!(hook.did_panic());
        assert_eq!(
            hook.get_panic_message(),
            Some("Test panic message".to_string())
        );

        // Restore default panic hook
        let _ = panic::take_hook();
    }
}
