//! Panic recovery utilities for Model trait methods
//!
//! This module provides safe wrappers for Model trait methods that catch
//! panics and allow the application to continue running.

use hojicha_core::core::{Cmd, Model};
use hojicha_core::event::Event;
use ratatui::{Frame, layout::Rect};
use std::panic::{self, AssertUnwindSafe};
use log::error;

/// Strategy for handling panics in Model methods
#[derive(Debug, Clone, Copy)]
pub enum PanicRecoveryStrategy {
    /// Continue with current state (default)
    Continue,
    /// Reset model to initial state
    Reset,
    /// Show error screen
    ShowError,
    /// Quit the application
    Quit,
}

impl Default for PanicRecoveryStrategy {
    fn default() -> Self {
        Self::Continue
    }
}

/// Safely call Model::init with panic recovery
pub fn safe_init<M: Model>(model: &mut M, strategy: PanicRecoveryStrategy) -> Cmd<M::Message> {
    let result = panic::catch_unwind(AssertUnwindSafe(|| model.init()));
    
    match result {
        Ok(cmd) => cmd,
        Err(panic_info) => {
            let msg = format_panic_message(&panic_info, "Model::init");
            error!("Panic in Model::init: {}", msg);
            
            match strategy {
                PanicRecoveryStrategy::Continue => Cmd::none(),
                PanicRecoveryStrategy::Reset => Cmd::none(),
                PanicRecoveryStrategy::ShowError => {
                    // Could return a command to show error screen
                    Cmd::none()
                }
                PanicRecoveryStrategy::Quit => Cmd::quit(),
            }
        }
    }
}

/// Safely call Model::update with panic recovery
pub fn safe_update<M: Model>(
    model: &mut M,
    event: Event<M::Message>,
    strategy: PanicRecoveryStrategy,
) -> Cmd<M::Message> {
    // We can't clone the event, so we need to move it into the closure
    // This means we can't log the specific event on panic
    let result = panic::catch_unwind(AssertUnwindSafe(move || model.update(event)));
    
    match result {
        Ok(cmd) => cmd,
        Err(panic_info) => {
            let msg = format_panic_message(&panic_info, "Model::update");
            error!("Panic in Model::update: {}", msg);
            
            match strategy {
                PanicRecoveryStrategy::Continue => Cmd::none(),
                PanicRecoveryStrategy::Reset => {
                    // Could trigger a reset command
                    Cmd::none()
                }
                PanicRecoveryStrategy::ShowError => {
                    // Could return a command to show error screen
                    Cmd::none()
                }
                PanicRecoveryStrategy::Quit => Cmd::quit(),
            }
        }
    }
}

/// Safely call Model::view with panic recovery
/// 
/// Returns true if the application should quit
pub fn safe_view<M: Model>(
    model: &M,
    frame: &mut Frame,
    area: Rect,
    strategy: PanicRecoveryStrategy,
) -> bool {
    let result = panic::catch_unwind(AssertUnwindSafe(|| model.view(frame, area)));
    
    match result {
        Ok(()) => false, // Success, don't quit
        Err(panic_info) => {
            let msg = format_panic_message(&panic_info, "Model::view");
            error!("Panic in Model::view: {}", msg);
            
            // Try to render an error message
            match strategy {
                PanicRecoveryStrategy::ShowError => {
                    render_panic_error(frame, area, &msg);
                }
                _ => {
                    // At minimum, try to render something
                    render_minimal_error(frame, area);
                }
            }
            
            // Return true if strategy says to quit
            matches!(strategy, PanicRecoveryStrategy::Quit)
        }
    }
}

/// Format panic information into a readable message
fn format_panic_message(panic_info: &dyn std::any::Any, context: &str) -> String {
    if let Some(s) = panic_info.downcast_ref::<String>() {
        format!("{} panicked: {}", context, s)
    } else if let Some(s) = panic_info.downcast_ref::<&str>() {
        format!("{} panicked: {}", context, s)
    } else {
        format!("{} panicked with unknown error", context)
    }
}

/// Render a detailed panic error screen
fn render_panic_error(frame: &mut Frame, area: Rect, message: &str) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::style::{Color, Style};
    
    let error_text = format!(
        "╔══════════════════════════════════════╗\n\
         ║           PANIC DETECTED             ║\n\
         ╚══════════════════════════════════════╝\n\n\
         {}\n\n\
         The application recovered from this error.\n\
         Press 'q' to quit or continue using the app.",
        message
    );
    
    let widget = Paragraph::new(error_text)
        .style(Style::default().fg(Color::Red))
        .block(Block::default().borders(Borders::ALL).title("Error"));
    
    frame.render_widget(widget, area);
}

/// Render a minimal error indicator
fn render_minimal_error(frame: &mut Frame, area: Rect) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::style::{Color, Style};
    
    let widget = Paragraph::new("An error occurred during rendering")
        .style(Style::default().fg(Color::Red))
        .block(Block::default().borders(Borders::ALL));
    
    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct PanickyModel {
        should_panic_init: bool,
        should_panic_update: bool,
        should_panic_view: bool,
    }
    
    impl Model for PanickyModel {
        type Message = ();
        
        fn init(&mut self) -> Cmd<Self::Message> {
            if self.should_panic_init {
                panic!("Init panic!");
            }
            Cmd::none()
        }
        
        fn update(&mut self, _event: Event<Self::Message>) -> Cmd<Self::Message> {
            if self.should_panic_update {
                panic!("Update panic!");
            }
            Cmd::none()
        }
        
        fn view(&self, _frame: &mut Frame, _area: Rect) {
            if self.should_panic_view {
                panic!("View panic!");
            }
        }
    }
    
    #[test]
    fn test_safe_init_catches_panic() {
        let mut model = PanickyModel {
            should_panic_init: true,
            should_panic_update: false,
            should_panic_view: false,
        };
        
        let cmd = safe_init(&mut model, PanicRecoveryStrategy::Continue);
        assert!(cmd.is_noop());
    }
    
    #[test]
    fn test_safe_update_catches_panic() {
        let mut model = PanickyModel {
            should_panic_init: false,
            should_panic_update: true,
            should_panic_view: false,
        };
        
        let cmd = safe_update(&mut model, Event::Tick, PanicRecoveryStrategy::Continue);
        assert!(cmd.is_noop());
    }
}