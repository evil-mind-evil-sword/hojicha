//! Timer component for countdown functionality
//!
//! A flexible component for counting down time with customizable display formats.

use crate::style::{Color, ColorProfile, Style, Theme};
use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Duration;

/// Display format for the timer
#[derive(Clone, Debug)]
pub enum TimerFormat {
    /// HH:MM:SS format
    HoursMinutesSeconds,
    /// MM:SS format
    MinutesSeconds,
    /// SS format (seconds only)
    SecondsOnly,
    /// Custom format function
    Custom(fn(Duration) -> String),
}

impl PartialEq for TimerFormat {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::HoursMinutesSeconds, Self::HoursMinutesSeconds) => true,
            (Self::MinutesSeconds, Self::MinutesSeconds) => true,
            (Self::SecondsOnly, Self::SecondsOnly) => true,
            (Self::Custom(_), Self::Custom(_)) => false, // Function pointers can't be meaningfully compared
            _ => false,
        }
    }
}

/// Timer state
#[derive(Clone, Debug, PartialEq)]
pub enum TimerState {
    /// Timer is ready to start
    Ready,
    /// Timer is currently running
    Running,
    /// Timer is paused
    Paused,
    /// Timer has finished (reached zero)
    Finished,
}

/// A countdown timer component
#[derive(Clone)]
pub struct Timer {
    /// Initial duration for the timer
    initial_duration: Duration,
    /// Remaining duration
    remaining: Duration,
    /// Current state
    state: TimerState,
    /// Display format
    format: TimerFormat,
    /// Style for normal display
    normal_style: Style,
    /// Style for warning state (e.g., < 10 seconds)
    warning_style: Style,
    /// Style for critical state (e.g., < 5 seconds)
    critical_style: Style,
    /// Style for finished state
    finished_style: Style,
    /// Container style
    container_style: Style,
    /// Warning threshold
    warning_threshold: Duration,
    /// Critical threshold
    critical_threshold: Duration,
    /// Whether to show milliseconds when under 1 second
    show_milliseconds: bool,
    /// Title for the timer
    title: Option<String>,
    /// Message to show when finished
    finished_message: Option<String>,
}

impl Timer {
    /// Create a new timer with the specified duration
    pub fn new(duration: Duration) -> Self {
        Self {
            initial_duration: duration,
            remaining: duration,
            state: TimerState::Ready,
            format: TimerFormat::MinutesSeconds,
            normal_style: Style::new().fg(Color::white()),
            warning_style: Style::new().fg(Color::yellow()).bold(),
            critical_style: Style::new().fg(Color::red()).bold(),
            finished_style: Style::new().fg(Color::green()).bold(),
            container_style: Style::new(),
            warning_threshold: Duration::from_secs(10),
            critical_threshold: Duration::from_secs(5),
            show_milliseconds: false,
            title: None,
            finished_message: Some("Time's up!".to_string()),
        }
    }

    /// Create a timer for a specific number of seconds
    pub fn from_seconds(seconds: u64) -> Self {
        Self::new(Duration::from_secs(seconds))
    }

    /// Create a timer for a specific number of minutes
    pub fn from_minutes(minutes: u64) -> Self {
        Self::new(Duration::from_secs(minutes * 60))
    }

    /// Start the timer
    pub fn start(&mut self) {
        if self.state == TimerState::Ready || self.state == TimerState::Paused {
            self.state = TimerState::Running;
        }
    }

    /// Pause the timer
    pub fn pause(&mut self) {
        if self.state == TimerState::Running {
            self.state = TimerState::Paused;
        }
    }

    /// Resume the timer
    pub fn resume(&mut self) {
        if self.state == TimerState::Paused {
            self.state = TimerState::Running;
        }
    }

    /// Reset the timer to initial duration
    pub fn reset(&mut self) {
        self.remaining = self.initial_duration;
        self.state = TimerState::Ready;
    }

    /// Stop the timer and reset
    pub fn stop(&mut self) {
        self.reset();
    }

    /// Update the timer by the given duration (typically called on tick)
    pub fn tick(&mut self, elapsed: Duration) {
        if self.state == TimerState::Running {
            if let Some(new_remaining) = self.remaining.checked_sub(elapsed) {
                self.remaining = new_remaining;
                if self.remaining == Duration::ZERO {
                    self.state = TimerState::Finished;
                }
            } else {
                self.remaining = Duration::ZERO;
                self.state = TimerState::Finished;
            }
        }
    }

    /// Get the remaining duration
    pub fn remaining(&self) -> Duration {
        self.remaining
    }

    /// Get the current state
    pub fn state(&self) -> &TimerState {
        &self.state
    }

    /// Check if the timer is finished
    pub fn is_finished(&self) -> bool {
        self.state == TimerState::Finished
    }

    /// Check if the timer is running
    pub fn is_running(&self) -> bool {
        self.state == TimerState::Running
    }

    /// Set the display format
    pub fn with_format(mut self, format: TimerFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the finished message
    pub fn with_finished_message(mut self, message: impl Into<String>) -> Self {
        self.finished_message = Some(message.into());
        self
    }

    /// Set warning threshold
    pub fn with_warning_threshold(mut self, threshold: Duration) -> Self {
        self.warning_threshold = threshold;
        self
    }

    /// Set critical threshold
    pub fn with_critical_threshold(mut self, threshold: Duration) -> Self {
        self.critical_threshold = threshold;
        self
    }

    /// Set whether to show milliseconds
    pub fn with_milliseconds(mut self, show: bool) -> Self {
        self.show_milliseconds = show;
        self
    }

    /// Set normal style
    pub fn with_normal_style(mut self, style: Style) -> Self {
        self.normal_style = style;
        self
    }

    /// Set warning style
    pub fn with_warning_style(mut self, style: Style) -> Self {
        self.warning_style = style;
        self
    }

    /// Set critical style
    pub fn with_critical_style(mut self, style: Style) -> Self {
        self.critical_style = style;
        self
    }

    /// Set finished style
    pub fn with_finished_style(mut self, style: Style) -> Self {
        self.finished_style = style;
        self
    }

    /// Set container style
    pub fn with_container_style(mut self, style: Style) -> Self {
        self.container_style = style;
        self
    }

    /// Apply a theme
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.normal_style = Style::new().fg(theme.colors.text.clone());
        self.warning_style = Style::new().fg(theme.colors.warning.clone()).bold();
        self.critical_style = Style::new().fg(theme.colors.error.clone()).bold();
        self.finished_style = Style::new().fg(theme.colors.success.clone()).bold();

        if let Some(style) = theme.get_style("timer.container") {
            self.container_style = style.clone();
        }
    }

    /// Format the duration for display
    fn format_duration(&self, duration: Duration) -> String {
        match &self.format {
            TimerFormat::HoursMinutesSeconds => {
                let total_secs = duration.as_secs();
                let hours = total_secs / 3600;
                let minutes = (total_secs % 3600) / 60;
                let seconds = total_secs % 60;

                if self.show_milliseconds && duration.as_secs() == 0 {
                    let millis = duration.as_millis() % 1000;
                    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
                } else {
                    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                }
            }
            TimerFormat::MinutesSeconds => {
                let total_secs = duration.as_secs();
                let minutes = total_secs / 60;
                let seconds = total_secs % 60;

                if self.show_milliseconds && duration.as_secs() == 0 {
                    let millis = duration.as_millis() % 1000;
                    format!("{:02}:{:02}.{:03}", minutes, seconds, millis)
                } else {
                    format!("{:02}:{:02}", minutes, seconds)
                }
            }
            TimerFormat::SecondsOnly => {
                let seconds = duration.as_secs();

                if self.show_milliseconds && seconds == 0 {
                    let millis = duration.as_millis();
                    format!("{}.{:03}", millis / 1000, millis % 1000)
                } else {
                    format!("{}", seconds)
                }
            }
            TimerFormat::Custom(formatter) => formatter(duration),
        }
    }

    /// Render the timer
    pub fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        // Determine the display text and style
        let (text, style) = if self.state == TimerState::Finished {
            (
                self.finished_message
                    .clone()
                    .unwrap_or_else(|| "00:00".to_string()),
                self.finished_style.clone(),
            )
        } else {
            let time_text = self.format_duration(self.remaining);
            let style = if self.remaining <= self.critical_threshold {
                self.critical_style.clone()
            } else if self.remaining <= self.warning_threshold {
                self.warning_style.clone()
            } else {
                self.normal_style.clone()
            };
            (time_text, style)
        };

        // Add state indicator
        let display_text = match self.state {
            TimerState::Ready => format!("⏱ {}", text),
            TimerState::Running => text,
            TimerState::Paused => format!("⏸ {}", text),
            TimerState::Finished => format!("✓ {}", text),
        };

        // Create block with optional title
        let mut block = Block::default();
        if let Some(ref title) = self.title {
            block = block.title(title.as_str());
        }

        if self.container_style.get_border() != &crate::style::BorderStyle::None {
            block = block
                .borders(Borders::ALL)
                .border_style(self.container_style.to_ratatui(profile));
        }

        // Create paragraph
        let paragraph = Paragraph::new(display_text)
            .block(block)
            .style(style.to_ratatui(profile))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Get progress as a percentage (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.initial_duration == Duration::ZERO {
            return 1.0;
        }

        let remaining_ms = self.remaining.as_millis() as f32;
        let initial_ms = self.initial_duration.as_millis() as f32;

        (initial_ms - remaining_ms) / initial_ms
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::from_seconds(60)
    }
}
