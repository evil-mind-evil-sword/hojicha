//! Stopwatch component for counting up time
//!
//! A flexible component for measuring elapsed time with customizable display formats.

use crate::style::{Color, ColorProfile, Style, Theme};
use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Duration;

/// Display format for the stopwatch
#[derive(Clone, Debug)]
pub enum StopwatchFormat {
    /// HH:MM:SS format
    HoursMinutesSeconds,
    /// MM:SS format
    MinutesSeconds,
    /// SS.mmm format (seconds with milliseconds)
    SecondsMilliseconds,
    /// Custom format function
    Custom(fn(Duration) -> String),
}

impl PartialEq for StopwatchFormat {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::HoursMinutesSeconds, Self::HoursMinutesSeconds) => true,
            (Self::MinutesSeconds, Self::MinutesSeconds) => true,
            (Self::SecondsMilliseconds, Self::SecondsMilliseconds) => true,
            (Self::Custom(_), Self::Custom(_)) => false, // Function pointers can't be meaningfully compared
            _ => false,
        }
    }
}

/// Stopwatch state
#[derive(Clone, Debug, PartialEq)]
pub enum StopwatchState {
    /// Stopwatch is ready to start
    Ready,
    /// Stopwatch is currently running
    Running,
    /// Stopwatch is paused
    Paused,
    /// Stopwatch is stopped (but not reset)
    Stopped,
}

/// A lap time entry
#[derive(Clone, Debug)]
pub struct Lap {
    /// Lap number
    pub number: usize,
    /// Duration of this specific lap
    pub lap_time: Duration,
    /// Total elapsed time when lap was recorded
    pub total_time: Duration,
}

/// A stopwatch component for counting up
#[derive(Clone)]
pub struct Stopwatch {
    /// Elapsed duration
    elapsed: Duration,
    /// Current state
    state: StopwatchState,
    /// Display format
    format: StopwatchFormat,
    /// Recorded laps
    laps: Vec<Lap>,
    /// Style for normal display
    normal_style: Style,
    /// Style for running state
    running_style: Style,
    /// Style for paused state
    paused_style: Style,
    /// Container style
    container_style: Style,
    /// Whether to show milliseconds
    show_milliseconds: bool,
    /// Title for the stopwatch
    title: Option<String>,
    /// Maximum laps to store
    max_laps: usize,
}

impl Stopwatch {
    /// Create a new stopwatch
    pub fn new() -> Self {
        Self {
            elapsed: Duration::ZERO,
            state: StopwatchState::Ready,
            format: StopwatchFormat::MinutesSeconds,
            laps: Vec::new(),
            normal_style: Style::new().fg(Color::white()),
            running_style: Style::new().fg(Color::green()),
            paused_style: Style::new().fg(Color::yellow()),
            container_style: Style::new(),
            show_milliseconds: true,
            title: None,
            max_laps: 100,
        }
    }

    /// Start the stopwatch
    pub fn start(&mut self) {
        if self.state == StopwatchState::Ready || self.state == StopwatchState::Paused {
            self.state = StopwatchState::Running;
        }
    }

    /// Pause the stopwatch
    pub fn pause(&mut self) {
        if self.state == StopwatchState::Running {
            self.state = StopwatchState::Paused;
        }
    }

    /// Resume the stopwatch
    pub fn resume(&mut self) {
        if self.state == StopwatchState::Paused {
            self.state = StopwatchState::Running;
        }
    }

    /// Stop the stopwatch (doesn't reset)
    pub fn stop(&mut self) {
        if self.state != StopwatchState::Ready {
            self.state = StopwatchState::Stopped;
        }
    }

    /// Reset the stopwatch
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.state = StopwatchState::Ready;
        self.laps.clear();
    }

    /// Record a lap
    pub fn lap(&mut self) {
        if self.state == StopwatchState::Running {
            let lap_number = self.laps.len() + 1;
            let lap_time = if self.laps.is_empty() {
                self.elapsed
            } else {
                self.elapsed - self.laps.last().unwrap().total_time
            };

            let lap = Lap {
                number: lap_number,
                lap_time,
                total_time: self.elapsed,
            };

            if self.laps.len() < self.max_laps {
                self.laps.push(lap);
            }
        }
    }

    /// Update the stopwatch by the given duration (typically called on tick)
    pub fn tick(&mut self, elapsed: Duration) {
        if self.state == StopwatchState::Running {
            self.elapsed += elapsed;
        }
    }

    /// Get the elapsed duration
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Get the current state
    pub fn state(&self) -> &StopwatchState {
        &self.state
    }

    /// Get the recorded laps
    pub fn laps(&self) -> &[Lap] {
        &self.laps
    }

    /// Get the last lap time
    pub fn last_lap(&self) -> Option<&Lap> {
        self.laps.last()
    }

    /// Get the best (shortest) lap
    pub fn best_lap(&self) -> Option<&Lap> {
        self.laps.iter().min_by_key(|lap| lap.lap_time)
    }

    /// Get the average lap time
    pub fn average_lap_time(&self) -> Option<Duration> {
        if self.laps.is_empty() {
            return None;
        }

        let total: Duration = self.laps.iter().map(|lap| lap.lap_time).sum();
        Some(total / self.laps.len() as u32)
    }

    /// Check if the stopwatch is running
    pub fn is_running(&self) -> bool {
        self.state == StopwatchState::Running
    }

    /// Set the display format
    pub fn with_format(mut self, format: StopwatchFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set whether to show milliseconds
    pub fn with_milliseconds(mut self, show: bool) -> Self {
        self.show_milliseconds = show;
        self
    }

    /// Set maximum laps to store
    pub fn with_max_laps(mut self, max: usize) -> Self {
        self.max_laps = max;
        self
    }

    /// Set normal style
    pub fn with_normal_style(mut self, style: Style) -> Self {
        self.normal_style = style;
        self
    }

    /// Set running style
    pub fn with_running_style(mut self, style: Style) -> Self {
        self.running_style = style;
        self
    }

    /// Set paused style
    pub fn with_paused_style(mut self, style: Style) -> Self {
        self.paused_style = style;
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
        self.running_style = Style::new().fg(theme.colors.success.clone());
        self.paused_style = Style::new().fg(theme.colors.warning.clone());

        if let Some(style) = theme.get_style("stopwatch.container") {
            self.container_style = style.clone();
        }
    }

    /// Format the duration for display
    fn format_duration(&self, duration: Duration) -> String {
        match &self.format {
            StopwatchFormat::HoursMinutesSeconds => {
                let total_secs = duration.as_secs();
                let hours = total_secs / 3600;
                let minutes = (total_secs % 3600) / 60;
                let seconds = total_secs % 60;

                if self.show_milliseconds && hours == 0 {
                    let millis = duration.as_millis() % 1000;
                    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
                } else {
                    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                }
            }
            StopwatchFormat::MinutesSeconds => {
                let total_secs = duration.as_secs();
                let minutes = total_secs / 60;
                let seconds = total_secs % 60;

                if self.show_milliseconds {
                    let millis = duration.as_millis() % 1000;
                    format!("{:02}:{:02}.{:03}", minutes, seconds, millis)
                } else {
                    format!("{:02}:{:02}", minutes, seconds)
                }
            }
            StopwatchFormat::SecondsMilliseconds => {
                let total_millis = duration.as_millis();
                let seconds = total_millis / 1000;
                let millis = total_millis % 1000;
                format!("{}.{:03}", seconds, millis)
            }
            StopwatchFormat::Custom(formatter) => formatter(duration),
        }
    }

    /// Render the stopwatch
    pub fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        let time_text = self.format_duration(self.elapsed);

        // Determine style based on state
        let style = match self.state {
            StopwatchState::Running => self.running_style.clone(),
            StopwatchState::Paused => self.paused_style.clone(),
            _ => self.normal_style.clone(),
        };

        // Add state indicator
        let display_text = match self.state {
            StopwatchState::Ready => format!("⏱ {}", time_text),
            StopwatchState::Running => format!("▶ {}", time_text),
            StopwatchState::Paused => format!("⏸ {}", time_text),
            StopwatchState::Stopped => format!("⏹ {}", time_text),
        };

        // Add lap counter if there are laps
        let final_text = if !self.laps.is_empty() {
            format!("{} [Lap {}]", display_text, self.laps.len())
        } else {
            display_text
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
        let paragraph = Paragraph::new(final_text)
            .block(block)
            .style(style.to_ratatui(profile))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Render lap times
    pub fn render_laps(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        use ratatui::text::{Line, Span};

        if self.laps.is_empty() {
            return;
        }

        let mut lines = Vec::new();

        // Find best lap for highlighting
        let best_lap_num = self.best_lap().map(|l| l.number);

        for lap in self.laps.iter().rev().take(area.height as usize - 2) {
            let is_best = Some(lap.number) == best_lap_num;
            let style = if is_best {
                Style::new().fg(Color::green()).bold()
            } else {
                Style::new()
            };

            let lap_text = format!(
                "Lap {:3}: {} (Total: {})",
                lap.number,
                self.format_duration(lap.lap_time),
                self.format_duration(lap.total_time)
            );

            lines.push(Line::from(Span::styled(
                lap_text,
                style.to_ratatui(profile),
            )));
        }

        let block = Block::default().borders(Borders::ALL).title("Lap Times");

        let paragraph = Paragraph::new(lines)
            .block(block)
            .style(self.normal_style.to_ratatui(profile));

        frame.render_widget(paragraph, area);
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}
