//! Phase 2 Components Showcase
//!
//! Demonstrates the new components from Phase 2 implementation:
//! - Timer component for countdown
//! - Stopwatch component for counting up
//! - StatusBar component with multiple segments
//!
//! Controls:
//! - Tab: Switch between demos
//! - Space: Start/pause timer and stopwatch
//! - r: Reset timer/stopwatch
//! - l: Record lap (stopwatch)
//! - q: Quit

use hojicha::{
    commands::{self, tick},
    components::{
        StatusBar, StatusBarBuilder, StatusBarPosition, StatusSegment, Stopwatch,
        StopwatchFormat, Timer, TimerFormat,
    },
    core::{Cmd, Model},
    event::{Event, Key, KeyEvent},
    program::{Program, ProgramOptions},
    style::{Color, ColorProfile, Style, TextAlign, Theme},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

struct Phase2Demo {
    /// Timer component
    timer: Timer,
    /// Stopwatch component
    stopwatch: Stopwatch,
    /// Status bar
    status_bar: StatusBar,
    /// Currently selected demo
    current_demo: usize,
    /// Theme
    theme: Theme,
    /// Color profile
    color_profile: ColorProfile,
    /// Last tick time for accurate timing
    last_tick: Instant,
}

impl Phase2Demo {
    fn new() -> Self {
        // Setup timer (30 seconds countdown)
        let timer = Timer::from_seconds(30)
            .with_title("Timer Demo")
            .with_format(TimerFormat::MinutesSeconds)
            .with_warning_threshold(Duration::from_secs(10))
            .with_critical_threshold(Duration::from_secs(5))
            .with_finished_message("Time's up! ⏰");

        // Setup stopwatch
        let stopwatch = Stopwatch::new()
            .with_title("Stopwatch Demo")
            .with_format(StopwatchFormat::MinutesSeconds)
            .with_milliseconds(true);

        // Setup status bar
        let mut status_bar = StatusBar::new()
            .with_position(StatusBarPosition::Bottom)
            .with_separator(" | ")
            .with_height(1);

        // Add segments
        status_bar.add_segment(
            StatusSegment::new("Timer: Ready")
                .with_constraint(Constraint::Length(20))
                .with_alignment(TextAlign::Left),
        );
        status_bar.add_segment(
            StatusSegment::new("Phase 2 Demo")
                .with_constraint(Constraint::Min(0))
                .with_alignment(TextAlign::Center)
                .with_style(Style::new().bold()),
        );
        status_bar.add_segment(
            StatusSegment::new("Space: Start/Pause | r: Reset | l: Lap")
                .with_constraint(Constraint::Length(40))
                .with_alignment(TextAlign::Right),
        );

        let theme = Theme::nord();
        Self {
            timer,
            stopwatch,
            status_bar,
            current_demo: 0,
            theme,
            color_profile: ColorProfile::detect(),
            last_tick: Instant::now(),
        }
    }

    fn update_status_bar(&mut self) {
        // Update timer status
        let timer_status = format!(
            "Timer: {} ({})",
            match self.timer.state() {
                hojicha::components::TimerState::Ready => "Ready",
                hojicha::components::TimerState::Running => "Running",
                hojicha::components::TimerState::Paused => "Paused",
                hojicha::components::TimerState::Finished => "Finished",
            },
            format_duration(self.timer.remaining())
        );
        self.status_bar.update_segment(0, timer_status);

        // Update center segment with current demo
        let demo_name = match self.current_demo {
            0 => "Timer & Stopwatch",
            1 => "Timer Focus",
            2 => "Stopwatch Focus",
            _ => "Unknown",
        };
        self.status_bar.update_segment(1, format!("Demo: {}", demo_name));
    }

    fn render_timer_demo(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(5),  // Timer display
                Constraint::Min(0),     // Instructions
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Timer Component")
            .block(Block::default().borders(Borders::ALL))
            .style(
                Style::new()
                    .fg(self.theme.colors.primary.clone())
                    .bold()
                    .to_ratatui(&self.color_profile),
            );
        frame.render_widget(title, chunks[0]);

        // Timer
        self.timer.render(frame, chunks[1], &self.color_profile);

        // Instructions
        let instructions = Paragraph::new(
            "Press Space to start/pause the timer\n\
             Press 'r' to reset the timer\n\
             Watch the color change as time runs out!",
        )
        .style(
            Style::new()
                .fg(self.theme.colors.text_secondary.clone())
                .to_ratatui(&self.color_profile),
        );
        frame.render_widget(instructions, chunks[2]);
    }

    fn render_stopwatch_demo(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(5),  // Stopwatch display
                Constraint::Min(0),     // Lap times
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Stopwatch Component")
            .block(Block::default().borders(Borders::ALL))
            .style(
                Style::new()
                    .fg(self.theme.colors.primary.clone())
                    .bold()
                    .to_ratatui(&self.color_profile),
            );
        frame.render_widget(title, chunks[0]);

        // Stopwatch
        self.stopwatch.render(frame, chunks[1], &self.color_profile);

        // Lap times
        if !self.stopwatch.laps().is_empty() {
            self.stopwatch.render_laps(frame, chunks[2], &self.color_profile);
        } else {
            let instructions = Paragraph::new(
                "Press Space to start/pause\n\
                 Press 'l' to record a lap\n\
                 Press 'r' to reset",
            )
            .style(
                Style::new()
                    .fg(self.theme.colors.text_secondary.clone())
                    .to_ratatui(&self.color_profile),
            );
            frame.render_widget(instructions, chunks[2]);
        }
    }

    fn render_combined_demo(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.render_timer_demo(frame, chunks[0]);
        self.render_stopwatch_demo(frame, chunks[1]);
    }
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

#[derive(Clone)]
enum Msg {
    Tick,
}

impl Model for Phase2Demo {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        tick(Duration::from_millis(50), || Msg::Tick)
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::Key(KeyEvent { key, .. }) => match key {
                Key::Char('q') | Key::Esc => return commands::quit(),
                Key::Tab => {
                    self.current_demo = (self.current_demo + 1) % 3;
                }
                Key::Char(' ') => {
                    // Toggle timer/stopwatch based on current demo
                    if self.current_demo == 0 || self.current_demo == 1 {
                        if self.timer.is_running() {
                            self.timer.pause();
                        } else {
                            self.timer.start();
                        }
                    }
                    if self.current_demo == 0 || self.current_demo == 2 {
                        if self.stopwatch.is_running() {
                            self.stopwatch.pause();
                        } else {
                            self.stopwatch.start();
                        }
                    }
                }
                Key::Char('r') => {
                    self.timer.reset();
                    self.stopwatch.reset();
                }
                Key::Char('l') => {
                    self.stopwatch.lap();
                }
                _ => {}
            },
            Event::User(Msg::Tick) => {
                let now = Instant::now();
                let elapsed = now.duration_since(self.last_tick);
                self.last_tick = now;

                // Update timer and stopwatch
                self.timer.tick(elapsed);
                self.stopwatch.tick(elapsed);

                // Update status bar
                self.update_status_bar();
            }
            _ => {}
        }
        Cmd::none()
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Layout with status bar
        let (status_area, main_area) = self.status_bar.layout(area);

        // Render main content
        match self.current_demo {
            0 => self.render_combined_demo(frame, main_area),
            1 => self.render_timer_demo(frame, main_area),
            2 => self.render_stopwatch_demo(frame, main_area),
            _ => {}
        }

        // Render status bar
        self.status_bar.render(frame, status_area, &self.color_profile);
    }
}

fn main() -> hojicha::Result<()> {
    let model = Phase2Demo::new();
    let options = ProgramOptions::default();
    let program = Program::with_options(model, options)?;
    program.run()?;
    Ok(())
}