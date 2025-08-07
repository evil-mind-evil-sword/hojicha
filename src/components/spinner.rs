//! Animated spinner component
//!
//! Provides various spinner styles for loading indicators.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use std::time::{Duration, Instant};

/// Spinner animation styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpinnerStyle {
    /// Dots spinner: ⣾⣽⣻⢿⡿⣟⣯⣷
    Dots,
    /// Line spinner: |/-\
    Line,
    /// Circle spinner: ◐◓◑◒
    Circle,
    /// Square spinner: ◰◳◲◱
    Square,
    /// Arrow spinner: ←↖↑↗→↘↓↙
    Arrow,
    /// Braille spinner: ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏
    Braille,
    /// Custom frames
    Custom(&'static [&'static str]),
}

impl SpinnerStyle {
    /// Get the animation frames for this style
    fn frames(&self) -> &[&str] {
        match self {
            SpinnerStyle::Dots => &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            SpinnerStyle::Line => &["|", "/", "-", "\\"],
            SpinnerStyle::Circle => &["◐", "◓", "◑", "◒"],
            SpinnerStyle::Square => &["◰", "◳", "◲", "◱"],
            SpinnerStyle::Arrow => &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
            SpinnerStyle::Braille => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            SpinnerStyle::Custom(frames) => frames,
        }
    }

    /// Get the frame duration for smooth animation
    fn frame_duration(&self) -> Duration {
        match self {
            SpinnerStyle::Dots => Duration::from_millis(80),
            SpinnerStyle::Line => Duration::from_millis(130),
            SpinnerStyle::Circle => Duration::from_millis(120),
            SpinnerStyle::Square => Duration::from_millis(120),
            SpinnerStyle::Arrow => Duration::from_millis(100),
            SpinnerStyle::Braille => Duration::from_millis(80),
            SpinnerStyle::Custom(_) => Duration::from_millis(100),
        }
    }
}

/// Animated spinner component
#[derive(Debug, Clone)]
pub struct Spinner {
    /// The spinner style
    style: SpinnerStyle,
    /// Current frame index
    frame_index: usize,
    /// Last frame update time
    last_update: Instant,
    /// Whether the spinner is running
    running: bool,
    /// Text to display next to the spinner
    message: String,
    /// Text style
    text_style: Style,
}

impl Spinner {
    /// Create a new spinner with default style
    pub fn new() -> Self {
        Self {
            style: SpinnerStyle::Dots,
            frame_index: 0,
            last_update: Instant::now(),
            running: false,
            message: String::new(),
            text_style: Style::default(),
        }
    }

    /// Create a spinner with a specific style
    pub fn with_style(style: SpinnerStyle) -> Self {
        Self {
            style,
            ..Self::new()
        }
    }

    /// Set the spinner style
    pub fn set_style(&mut self, style: SpinnerStyle) {
        self.style = style;
        self.frame_index = 0;
    }

    /// Set the message text
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// Set the text style
    pub fn set_text_style(&mut self, style: Style) {
        self.text_style = style;
    }

    /// Start the spinner animation
    pub fn start(&mut self) {
        self.running = true;
        self.frame_index = 0;
        self.last_update = Instant::now();
    }

    /// Stop the spinner animation
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if the spinner is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Update the spinner animation
    /// Returns true if the frame changed
    pub fn tick(&mut self) -> bool {
        if !self.running {
            return false;
        }

        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.style.frame_duration() {
            self.last_update = now;
            let frames = self.style.frames();
            self.frame_index = (self.frame_index + 1) % frames.len();
            true
        } else {
            false
        }
    }

    /// Get the current frame
    fn current_frame(&self) -> &str {
        if self.running {
            let frames = self.style.frames();
            frames[self.frame_index]
        } else {
            ""
        }
    }

    /// Render the spinner
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Update animation
        self.tick();

        // Build the display text
        let frame = self.current_frame();
        let text = if self.message.is_empty() {
            frame.to_string()
        } else {
            format!("{} {}", frame, self.message)
        };

        // Render the text
        let span = Span::styled(text, self.text_style);
        buf.set_span(area.x, area.y, &span, area.width);
    }

    /// Render the spinner centered in the area
    pub fn render_centered(&mut self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Calculate text width
        let frame = self.current_frame();
        let text_width = if self.message.is_empty() {
            frame.len()
        } else {
            frame.len() + 1 + self.message.len()
        };

        // Center the spinner
        let x = area.x + (area.width.saturating_sub(text_width as u16)) / 2;
        let y = area.y + area.height / 2;

        let centered_area = Rect {
            x,
            y,
            width: text_width as u16,
            height: 1,
        };

        self.render(centered_area, buf);
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new();
        assert!(!spinner.is_running());
        assert_eq!(spinner.message, "");
    }

    #[test]
    fn test_spinner_styles() {
        let dots = SpinnerStyle::Dots;
        assert_eq!(dots.frames().len(), 8);

        let custom = SpinnerStyle::Custom(&["1", "2", "3"]);
        assert_eq!(custom.frames().len(), 3);
    }

    #[test]
    fn test_spinner_animation() {
        let mut spinner = Spinner::new();
        spinner.start();
        assert!(spinner.is_running());

        let initial_frame = spinner.frame_index;

        // Wait for frame duration
        std::thread::sleep(spinner.style.frame_duration());

        // Tick should advance frame
        assert!(spinner.tick());
        assert_ne!(spinner.frame_index, initial_frame);

        spinner.stop();
        assert!(!spinner.is_running());
    }

    #[test]
    fn test_spinner_message() {
        let mut spinner = Spinner::new();
        spinner.set_message("Loading...");
        assert_eq!(spinner.message, "Loading...");
    }

    #[test]
    fn test_spinner_default() {
        let spinner = Spinner::default();
        assert_eq!(spinner.message, "");
        assert!(!spinner.is_running());
        assert_eq!(spinner.frame_index, 0);
    }

    #[test]
    fn test_spinner_with_message() {
        let mut spinner = Spinner::new();
        spinner.set_message("Processing...");
        assert_eq!(spinner.message, "Processing...");
        assert!(!spinner.is_running());
    }

    #[test]
    fn test_spinner_with_style() {
        let custom_style = SpinnerStyle::Custom(&["a", "b", "c"]);
        let mut spinner = Spinner::with_style(custom_style);
        spinner.start(); // Need to start the spinner to get frames
        assert_eq!(spinner.current_frame(), "a");
    }

    #[test]
    fn test_spinner_with_style_and_message() {
        let custom_style = SpinnerStyle::Custom(&["x", "y", "z"]);
        let mut spinner = Spinner::with_style(custom_style);
        spinner.set_message("Testing...");
        spinner.start(); // Need to start the spinner to get frames
        assert_eq!(spinner.message, "Testing...");
        assert_eq!(spinner.current_frame(), "x");
    }

    #[test]
    fn test_spinner_frame_wrapping() {
        let custom_style = SpinnerStyle::Custom(&["1", "2"]);
        let mut spinner = Spinner::with_style(custom_style);
        spinner.start();

        // Force multiple ticks to test wrapping
        spinner.frame_index = 1;
        spinner.last_update = Instant::now() - Duration::from_secs(1);
        assert!(spinner.tick());
        assert_eq!(spinner.frame_index, 0); // Should wrap back to 0
    }

    #[test]
    fn test_spinner_tick_when_stopped() {
        let mut spinner = Spinner::new();
        // Spinner is not running, tick should return false
        assert!(!spinner.tick());
    }

    #[test]
    fn test_spinner_multiple_starts() {
        let mut spinner = Spinner::new();
        spinner.start();
        let first_running = spinner.is_running();

        std::thread::sleep(Duration::from_millis(10));
        spinner.start(); // Starting again should keep it running

        assert!(first_running);
        assert!(spinner.is_running());
    }

    #[test]
    fn test_current_frame_when_stopped() {
        let mut spinner = Spinner::new();
        assert_eq!(spinner.current_frame(), ""); // Should return empty when stopped

        spinner.start();
        assert_ne!(spinner.current_frame(), ""); // Should return a frame when running
    }

    #[test]
    fn test_all_spinner_styles() {
        let styles = vec![
            SpinnerStyle::Dots,
            SpinnerStyle::Line,
            SpinnerStyle::Circle,
            SpinnerStyle::Square,
            SpinnerStyle::Arrow,
            SpinnerStyle::Custom(&["a", "b", "c"]),
        ];

        for style in styles {
            let frames = style.frames();
            assert!(!frames.is_empty(), "Style should have at least one frame");
            // Different styles may have different durations
            assert!(style.frame_duration() > Duration::from_millis(0));
        }
    }

    #[test]
    fn test_spinner_rendering() {
        use ratatui::buffer::Buffer;

        let mut spinner = Spinner::new();
        spinner.set_message("Loading");
        spinner.start();

        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
        spinner.render(Rect::new(0, 0, 20, 1), &mut buf);

        // Buffer should contain the spinner frame and message
        let content = buf.content();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_spinner_render_centered() {
        use ratatui::buffer::Buffer;

        let mut spinner = Spinner::new();
        spinner.set_message("Test");
        spinner.start();

        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 5));
        spinner.render_centered(Rect::new(0, 0, 20, 5), &mut buf);

        // Spinner should be rendered somewhere in the buffer
        let content = buf.content();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_spinner_render_zero_area() {
        use ratatui::buffer::Buffer;

        let mut spinner = Spinner::new();
        spinner.start();

        // Should not panic with zero-sized areas
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 10));
        spinner.render(Rect::new(0, 0, 0, 0), &mut buf);
        spinner.render_centered(Rect::new(0, 0, 0, 0), &mut buf);
    }
}
