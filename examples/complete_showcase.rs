//! Complete UI System Showcase
//!
//! A comprehensive demonstration of all Hojicha UI components and features.
//!
//! Navigation:
//! - Tab: Switch between demo tabs
//! - Arrow keys: Navigate within components
//! - Enter/Space: Interact with components
//! - F1-F5: Switch themes
//! - q: Quit

use hojicha::{
    commands::{self, tick},
    components::*,
    core::{Cmd, Model},
    event::{Event, Key, KeyEvent},
    program::{Program, ProgramOptions},
    style::*,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

struct CompleteShowcase {
    // Navigation
    tabs: Tabs,
    
    // Components for each tab
    help: Help,
    paginator: Paginator,
    timer: Timer,
    stopwatch: Stopwatch,
    status_bar: StatusBar,
    
    // Layout demos
    show_grid: bool,
    show_gradient: bool,
    gradient_index: usize,
    
    // Theme
    theme: Theme,
    theme_index: usize,
    theme_names: Vec<String>,
    color_profile: ColorProfile,
    
    // State
    last_tick: Instant,
    focused_component: usize,
}

impl CompleteShowcase {
    fn new() -> Self {
        // Create tabs
        let tabs = TabsBuilder::new()
            .tab_with_icon("🏠", "Overview")
            .tab_with_icon("🧩", "Components")
            .tab_with_icon("🎨", "Styling")
            .tab_with_icon("📐", "Layout")
            .tab_with_icon("⏱", "Time & Status")
            .tab_with_icon("❓", "Help")
            .position(TabPosition::Top)
            .style(TabStyle::Line)
            .build();
        
        // Setup help
        let mut help = Help::new();
        help.add("Tab", "Switch sections")
            .add("←/→", "Navigate tabs")
            .add("↑/↓", "Navigate items")
            .add("Enter", "Select/Interact")
            .add("Space", "Toggle/Start/Stop")
            .add("F1-F5", "Switch themes")
            .add("g", "Toggle gradient")
            .add("q", "Quit");
        
        // Setup paginator
        let paginator = Paginator::new(10)
            .with_style(PaginatorStyle::Dots)
            .with_arrows(true)
            .with_shortcuts(true);
        
        // Setup timer
        let timer = Timer::from_seconds(30)
            .with_format(TimerFormat::MinutesSeconds)
            .with_title("Demo Timer");
        
        // Setup stopwatch
        let stopwatch = Stopwatch::new()
            .with_format(StopwatchFormat::MinutesSeconds)
            .with_title("Demo Stopwatch");
        
        // Setup status bar
        let mut status_bar = StatusBar::new()
            .with_position(StatusBarPosition::Bottom);
        
        status_bar.add_segment(
            StatusSegment::new("Ready")
                .with_constraint(Constraint::Length(20))
                .with_alignment(TextAlign::Left),
        );
        status_bar.add_segment(
            StatusSegment::new("Hojicha Complete Showcase")
                .with_constraint(Constraint::Min(0))
                .with_alignment(TextAlign::Center),
        );
        status_bar.add_segment(
            StatusSegment::new("Tab: Navigate")
                .with_constraint(Constraint::Length(20))
                .with_alignment(TextAlign::Right),
        );
        
        let theme_names = vec![
            "Nord".to_string(),
            "Dracula".to_string(),
            "Solarized Dark".to_string(),
            "Solarized Light".to_string(),
            "Tokyo Night".to_string(),
        ];
        
        let mut showcase = Self {
            tabs,
            help,
            paginator,
            timer,
            stopwatch,
            status_bar,
            show_grid: false,
            show_gradient: false,
            gradient_index: 0,
            theme: Theme::nord(),
            theme_index: 0,
            theme_names,
            color_profile: ColorProfile::detect(),
            last_tick: Instant::now(),
            focused_component: 0,
        };
        
        showcase.apply_theme();
        showcase
    }
    
    fn apply_theme(&mut self) {
        self.tabs.apply_theme(&self.theme);
        self.help.apply_theme(&self.theme);
        self.paginator.apply_theme(&self.theme);
        self.timer.apply_theme(&self.theme);
        self.stopwatch.apply_theme(&self.theme);
        self.status_bar.apply_theme(&self.theme);
    }
    
    fn switch_theme(&mut self, index: usize) {
        self.theme_index = index % self.theme_names.len();
        self.theme = match self.theme_index {
            0 => Theme::nord(),
            1 => Theme::dracula(),
            2 => Theme::solarized_dark(),
            3 => Theme::solarized_light(),
            4 => Theme::tokyo_night(),
            _ => Theme::nord(),
        };
        self.apply_theme();
        self.update_status();
    }
    
    fn update_status(&mut self) {
        let status = format!(
            "Tab: {} | Theme: {}",
            self.tabs.selected() + 1,
            self.theme_names[self.theme_index]
        );
        self.status_bar.update_segment(0, status);
    }
    
    fn render_overview(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12),
                Constraint::Min(0),
            ])
            .split(area);
        
        // Title and description
        let overview = Paragraph::new(
            "🎨 Hojicha UI Framework - Complete Showcase\n\n\
             A modern TUI framework for Rust with:\n\
             • 20+ Built-in Components\n\
             • Advanced Layout System (Grid, Floating, Position)\n\
             • Rich Style System (Themes, Gradients, Alignment)\n\
             • Comprehensive Event Handling\n\
             • Cross-platform Support\n\n\
             Navigate through tabs to explore different features!"
        )
        .block(Block::default().borders(Borders::ALL).title("Overview"))
        .style(
            Style::new()
                .fg(self.theme.colors.text.clone())
                .to_ratatui(&self.color_profile),
        )
        .alignment(ratatui::layout::Alignment::Center);
        
        frame.render_widget(overview, chunks[0]);
        
        // Show gradient demo if enabled
        if self.show_gradient {
            let gradient = match self.gradient_index {
                0 => Gradient::sunset(),
                1 => Gradient::ocean(),
                2 => Gradient::forest(),
                3 => Gradient::fire(),
                4 => Gradient::night_sky(),
                _ => Gradient::rainbow(),
            };
            render_gradient_background(frame, chunks[1], &gradient, &self.color_profile);
        }
    }
    
    fn render_components(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .split(area);
        
        // Button demo
        let button_demo = Paragraph::new(
            "Button Component:\n\
             [Primary] [Secondary] [Success] [Warning] [Danger]"
        )
        .block(Block::default().borders(Borders::ALL).title("Buttons"))
        .style(
            Style::new()
                .fg(self.theme.colors.text.clone())
                .to_ratatui(&self.color_profile),
        );
        frame.render_widget(button_demo, chunks[0]);
        
        // Paginator
        self.paginator.render(frame, chunks[1], &self.color_profile);
        
        // Progress bar demo
        let progress_demo = Paragraph::new(
            "Progress Bars:\n\
             Simple:   [████████░░░░░░░░░░░░] 40%\n\
             Gradient: [▓▓▓▓▓▓▓▓▒▒▒▒▒░░░░░░] 60%"
        )
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .style(
            Style::new()
                .fg(self.theme.colors.text.clone())
                .to_ratatui(&self.color_profile),
        );
        frame.render_widget(progress_demo, chunks[2]);
    }
    
    fn render_styling(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(6),
                Constraint::Min(0),
            ])
            .split(area);
        
        // Theme info
        let theme_info = format!(
            "Current Theme: {}\n\n\
             Colors:\n\
             Primary: ████  Secondary: ████  Success: ████\n\
             Warning: ████  Error: ████      Info: ████\n\n\
             Press F1-F5 to switch themes",
            self.theme_names[self.theme_index]
        );
        
        let theme_widget = Paragraph::new(theme_info)
            .block(Block::default().borders(Borders::ALL).title("Theme System"))
            .style(
                Style::new()
                    .fg(self.theme.colors.text.clone())
                    .to_ratatui(&self.color_profile),
            );
        frame.render_widget(theme_widget, chunks[0]);
        
        // Text alignment demo
        let align_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(chunks[1]);
        
        for (i, (text, align)) in [
            ("Left", TextAlign::Left),
            ("Center", TextAlign::Center),
            ("Right", TextAlign::Right),
        ].iter().enumerate() {
            let aligned = place_horizontal(
                text,
                align_chunks[i].width,
                match align {
                    TextAlign::Left => HAlign::Left,
                    TextAlign::Center => HAlign::Center,
                    TextAlign::Right => HAlign::Right,
                },
            );
            
            let widget = Paragraph::new(aligned)
                .block(Block::default().borders(Borders::ALL))
                .style(
                    Style::new()
                        .fg(self.theme.colors.primary.clone())
                        .to_ratatui(&self.color_profile),
                );
            frame.render_widget(widget, align_chunks[i]);
        }
    }
    
    fn render_layout(&self, frame: &mut Frame, area: Rect) {
        if self.show_grid {
            // Demo grid layout
            let grid = GridBuilder::new()
                .rows(vec![
                    Constraint::Length(5),
                    Constraint::Length(5),
                    Constraint::Min(0),
                ])
                .columns(vec![
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ])
                .gap(1)
                .build()
                .with_grid_lines(true);
            
            grid.render(frame, area, &self.color_profile);
        } else {
            let info = Paragraph::new(
                "Grid Layout System:\n\n\
                 • CSS Grid-like layout\n\
                 • Row and column constraints\n\
                 • Cell spanning\n\
                 • Gap configuration\n\
                 • Grid lines display\n\n\
                 Position Utilities:\n\
                 • place_in_area() - Absolute positioning\n\
                 • place_horizontal() - Text alignment\n\
                 • place_vertical() - Vertical positioning\n\n\
                 Floating Elements:\n\
                 • Tooltips\n\
                 • Dropdowns\n\
                 • Overlays\n\
                 • Layer management\n\n\
                 Press 'g' to toggle grid demo"
            )
            .block(Block::default().borders(Borders::ALL).title("Layout Features"))
            .style(
                Style::new()
                    .fg(self.theme.colors.text.clone())
                    .to_ratatui(&self.color_profile),
            );
            frame.render_widget(info, area);
        }
    }
    
    fn render_time_status(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        
        // Timer on left
        self.timer.render(frame, chunks[0], &self.color_profile);
        
        // Stopwatch on right
        self.stopwatch.render(frame, chunks[1], &self.color_profile);
    }
    
    fn render_help_tab(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Min(0),
            ])
            .split(area);
        
        // Help component
        self.help.render(frame, chunks[0], &self.color_profile);
        
        // Additional info
        let info = Paragraph::new(
            "Keyboard Shortcuts:\n\n\
             Navigation:\n\
             • Tab/Shift+Tab - Navigate between sections\n\
             • Arrow keys - Navigate within components\n\
             • Enter/Space - Interact with components\n\n\
             Themes:\n\
             • F1 - Nord\n\
             • F2 - Dracula\n\
             • F3 - Solarized Dark\n\
             • F4 - Solarized Light\n\
             • F5 - Tokyo Night\n\n\
             Other:\n\
             • g - Toggle gradient/grid\n\
             • q/Esc - Quit"
        )
        .block(Block::default().borders(Borders::ALL).title("Full Help"))
        .style(
            Style::new()
                .fg(self.theme.colors.text_secondary.clone())
                .to_ratatui(&self.color_profile),
        );
        frame.render_widget(info, chunks[1]);
    }
}

#[derive(Clone)]
enum Msg {
    Tick,
}

impl Model for CompleteShowcase {
    type Message = Msg;
    
    fn init(&mut self) -> Cmd<Self::Message> {
        tick(Duration::from_millis(100), || Msg::Tick)
    }
    
    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::Key(KeyEvent { key, .. }) => match key {
                Key::Char('q') | Key::Esc => return commands::quit(),
                Key::Tab => {
                    self.tabs.select_next();
                    self.update_status();
                }
                Key::Left => {
                    self.tabs.select_previous();
                    self.update_status();
                }
                Key::Right => {
                    self.tabs.select_next();
                    self.update_status();
                }
                Key::F(1) => self.switch_theme(0),
                Key::F(2) => self.switch_theme(1),
                Key::F(3) => self.switch_theme(2),
                Key::F(4) => self.switch_theme(3),
                Key::F(5) => self.switch_theme(4),
                Key::Char('g') => {
                    if self.tabs.selected() == 3 {
                        self.show_grid = !self.show_grid;
                    } else {
                        self.show_gradient = !self.show_gradient;
                        if self.show_gradient {
                            self.gradient_index = (self.gradient_index + 1) % 6;
                        }
                    }
                }
                Key::Char(' ') => {
                    // Start/stop timer and stopwatch
                    if self.timer.is_running() {
                        self.timer.pause();
                    } else {
                        self.timer.start();
                    }
                    if self.stopwatch.is_running() {
                        self.stopwatch.pause();
                    } else {
                        self.stopwatch.start();
                    }
                }
                Key::Char('r') => {
                    self.timer.reset();
                    self.stopwatch.reset();
                }
                Key::PageUp => self.paginator.previous_page(),
                Key::PageDown => self.paginator.next_page(),
                _ => {}
            },
            Event::User(Msg::Tick) => {
                let now = Instant::now();
                let elapsed = now.duration_since(self.last_tick);
                self.last_tick = now;
                
                self.timer.tick(elapsed);
                self.stopwatch.tick(elapsed);
            }
            _ => {}
        }
        Cmd::none()
    }
    
    fn view(&self, frame: &mut Frame, area: Rect) {
        // Layout with status bar
        let (status_area, main_area) = self.status_bar.layout(area);
        
        // Layout with tabs
        let (tabs_area, content_area) = self.tabs.layout(main_area);
        
        // Render tabs
        self.tabs.render(frame, tabs_area, &self.color_profile);
        
        // Render content based on selected tab
        match self.tabs.selected() {
            0 => self.render_overview(frame, content_area),
            1 => self.render_components(frame, content_area),
            2 => self.render_styling(frame, content_area),
            3 => self.render_layout(frame, content_area),
            4 => self.render_time_status(frame, content_area),
            5 => self.render_help_tab(frame, content_area),
            _ => {}
        }
        
        // Render status bar
        self.status_bar.render(frame, status_area, &self.color_profile);
    }
}

fn main() -> hojicha::Result<()> {
    let model = CompleteShowcase::new();
    let options = ProgramOptions::default();
    let program = Program::with_options(model, options)?;
    program.run()?;
    Ok(())
}