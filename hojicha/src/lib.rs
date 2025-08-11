//! # Hojicha
//!
//! The Elm Architecture for Terminal UIs in Rust.
//!
//! This is a convenience facade that re-exports the core hojicha crates:
//! - `hojicha-core`: Core TEA abstractions (Model, Cmd, Event)
//! - `hojicha-runtime`: Event loop and async runtime (Program)
//! - `hojicha-pearls`: UI components and styling (optional)
//!
//! ## Quick Start
//!
//! ```no_run
//! use hojicha::prelude::*;
//! use ratatui::widgets::{Block, Borders, Paragraph};
//!
//! struct Counter {
//!     value: i32,
//! }
//!
//! impl Model for Counter {
//!     type Message = ();
//!
//!     fn update(&mut self, event: Event<()>) -> Cmd<()> {
//!         match event {
//!             Event::Key(key) => match key.key {
//!                 Key::Up => self.value += 1,
//!                 Key::Down => self.value -= 1,
//!                 Key::Char('q') => return quit(),
//!                 _ => {}
//!             },
//!             _ => {}
//!         }
//!         Cmd::none()
//!     }
//!
//!     fn view(&self, frame: &mut Frame, area: Rect) {
//!         let text = format!("Counter: {}\n\nUp/Down: change | q: quit", self.value);
//!         let widget = Paragraph::new(text)
//!             .block(Block::default().borders(Borders::ALL).title("Counter"));
//!         frame.render_widget(widget, area);
//!     }
//! }
//!
//! fn main() -> Result<()> {
//!     Program::new(Counter { value: 0 })?.run()
//! }
//! ```

#![warn(missing_docs)]

// Re-export core functionality
pub use hojicha_core::*;

// Re-export runtime
pub use hojicha_runtime::*;

// Re-export components (when feature enabled)
#[cfg(feature = "pearls")]
pub use hojicha_pearls as pearls;

/// Prelude module containing commonly used types and traits
pub mod prelude {
    // Core prelude
    pub use hojicha_core::prelude::*;
    
    // Runtime prelude
    pub use hojicha_runtime::prelude::*;
    
    // Pearls prelude (when feature enabled)
    #[cfg(feature = "pearls")]
    pub use hojicha_pearls::prelude::*;
}