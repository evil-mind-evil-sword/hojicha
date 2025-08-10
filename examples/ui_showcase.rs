//! Comprehensive UI Showcase
//!
//! This example demonstrates the complete Hojicha UI system including:
//! - Core components (buttons, modals, tables, lists, inputs)
//! - Advanced layout (grid, floating elements, positioning)
//! - Style system (themes, gradients, text alignment)
//! - Time components (timer, stopwatch)
//! - Navigation aids (paginator, help, status bar)
//!
//! Controls:
//! - Tab/Shift+Tab: Navigate sections
//! - Arrow keys: Navigate within components
//! - Space/Enter: Interact with components
//! - F1-F5: Switch themes
//! - g: Toggle gradient background
//! - t: Show tooltip demo
//! - d: Show dropdown demo
//! - q/Esc: Quit

use hojicha::{
    commands::{self, tick},
    components::*,
    core::{Cmd, Model},
    event::{Event, Key, KeyEvent, KeyModifiers},
    program::{Program, ProgramOptions},
    style::*,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

struct UIShowcase {
    // Current section
    current_section: Section,
    
    // Components
    button: Button,
    modal: Modal,
    progress_bar: ProgressBar,
    timer: Timer,
    stopwatch: Stopwatch,
    paginator: Paginator,
    help: Help,
    status_bar: StatusBar,
    styled_list: StyledList<String>,
    text_input: TextInput,
    
    // Layout demos
    grid: Grid,
    tooltip: Option<Tooltip>,
    dropdown: Option<Dropdown>,
    show_gradient: bool,
    current_gradient: usize,
    
    // Theme and styling
    theme: Theme,
    theme_index: usize,
    color_profile: ColorProfile,
    
    // State
    last_tick: Instant,
    tick_count: u32,
}

#[derive(Clone, PartialEq)]
enum Section {
    Overview,
    Components,
    Layout,
    Styling,
    Advanced,
}

impl UIShowcase {
    fn new() -> Self {
        // Initialize components
        let button = Button::new("Click Me")
            .with_variant(ButtonVariant::Primary)
            .with_size(ButtonSize::Medium);
            
        let mut modal = Modal::new();
        modal.set_title("Demo Modal");
        modal.set_size(ModalSize::Medium);
            
        let progress_bar = ProgressBar::new()
            .with_value(0.0);
            
        let timer = Timer::from_seconds(60)
            .with_format(TimerFormat::MinutesSeconds)
            .with_warning_threshold(Duration::from_secs(10));
            
        let stopwatch = Stopwatch::new()
            .with_format(StopwatchFormat::MinutesSeconds)
            .with_milliseconds(true);
            
        let paginator = Paginator::new(10)
            .with_style(PaginatorStyle::Dots)
            .with_arrows(true);
            
        let help = HelpBuilder::new()
            .with_navigation()
            .with_common()
            .build();
            
        let mut status_bar = StatusBar::new()
            .with_position(StatusBarPosition::Bottom);
        status_bar.add_segment(
            StatusSegment::new("Ready")
                .with_constraint(Constraint::Length(20))
                .with_alignment(TextAlign::Left),
        );
        status_bar.add_segment(
            StatusSegment::new("UI Showcase")
                .with_constraint(Constraint::Min(0))
                .with_alignment(TextAlign::Center),
        );
        status_bar.add_segment(
            StatusSegment::new("F1-F5: Themes")
                .with_constraint(Constraint::Length(20))
                .with_alignment(TextAlign::Right),
        );
        
        let items = vec![
            "🎨 Style System".to_string(),
            "📦 Components".to_string(),
            "🔲 Grid Layout".to_string(),
            "🎯 Positioning".to_string(),
            "🌈 Gradients".to_string(),
            "💫 Animations".to_string(),
        ];
        let styled_list = StyledList::new(items)
            .with_title("Features");
            
        let text_input = TextInput::new()
            .placeholder("Enter text here...")
            .required();
            
        // Initialize grid
        let mut grid = GridBuilder::new()
            .rows(vec![
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .columns(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .gap(1)
            .build();
            
        grid.add_cell(
            GridCell::new(0, 1)
                .with_style(Style::new().bg(Color::rgb(40, 40, 40))),
        );
        
        Self {
            current_section: Section::Overview,
            button,
            modal,
            progress_bar,
            timer,
            stopwatch,
            paginator,
            help,
            status_bar,
            styled_list,
            text_input,
            grid,
            tooltip: None,
            dropdown: None,
            show_gradient: false,
            current_gradient: 0,
            theme: Theme::nord(),
            theme_index: 0,
            color_profile: ColorProfile::detect(),
            last_tick: Instant::now(),
            tick_count: 0,
        }
    }
    
    fn switch_theme(&mut self, index: usize) {
        self.theme_index = index;
        self.theme = match index {
            0 => Theme::nord(),
            1 => Theme::dracula(),
            2 => Theme::solarized_dark(),
            3 => Theme::solarized_light(),
            4 => Theme::tokyo_night(),
            _ => Theme::nord(),
        };
        self.apply_theme();
    }
    
    fn apply_theme(&mut self) {
        self.button.apply_theme(&self.theme);
        self.modal.apply_theme(&self.theme);
        self.timer.apply_theme(&self.theme);
        self.stopwatch.apply_theme(&self.theme);
        self.paginator.apply_theme(&self.theme);
        self.help.apply_theme(&self.theme);
        self.status_bar.apply_theme(&self.theme);
        self.styled_list.apply_theme(&self.theme);
        self.text_input.apply_theme(&self.theme);
    }
    
    fn next_section(&mut self) {
        self.current_section = match self.current_section {
            Section::Overview => Section::Components,
            Section::Components => Section::Layout,
            Section::Layout => Section::Styling,
            Section::Styling => Section::Advanced,
            Section::Advanced => Section::Overview,
        };
    }
    
    fn previous_section(&mut self) {
        self.current_section = match self.current_section {
            Section::Overview => Section::Advanced,
            Section::Components => Section::Overview,
            Section::Layout => Section::Components,
            Section::Styling => Section::Layout,
            Section::Advanced => Section::Styling,
        };
    }
    
    fn render_overview(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Min(0),
                Constraint::Length(5),
            ])
            .split(area);
            
        // Title with gradient background if enabled
        if self.show_gradient {
            let gradient = match self.current_gradient {
                0 => Gradient::sunset(),
                1 => Gradient::ocean(),
                2 => Gradient::forest(),
                3 => Gradient::fire(),
                4 => Gradient::night_sky(),
                _ => Gradient::rainbow(),
            };
            render_gradient_background(frame, chunks[0], &gradient, &self.color_profile);
        }
        
        let title = Paragraph::new(
            "🎨 Hojicha UI Showcase\n\n\
             A comprehensive demonstration of the Hojicha TUI framework\n\
             featuring advanced components, layouts, and styling capabilities.\n\n\
             Press Tab to navigate through different sections."
        )
        .block(Block::default().borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center)
        .style(
            Style::new()
                .fg(self.theme.colors.primary.clone())
                .bold()
                .to_ratatui(&self.color_profile),
        );
        frame.render_widget(title, chunks[0]);
        
        // Feature list
        // Note: We need mutable access to render, so we'll skip this for now
        // self.styled_list.render(frame, chunks[1], &self.color_profile);
        
        // Progress indicator
        let progress = (self.tick_count as f32 % 100.0) / 100.0;
        let mut bar = ProgressBar::new().with_value(progress);
        bar.render(frame, chunks[2], &self.theme, &self.color_profile);
    }
    
    fn render_components(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);
            
        // Title
        let title = Paragraph::new("Component Gallery")
            .block(Block::default().borders(Borders::ALL))
            .style(
                Style::new()
                    .fg(self.theme.colors.primary.clone())
                    .to_ratatui(&self.color_profile),
            );
        frame.render_widget(title, chunks[0]);
        
        // Timer and Stopwatch
        let time_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);
            
        self.timer.render(frame, time_chunks[0], &self.color_profile);
        self.stopwatch.render(frame, time_chunks[1], &self.color_profile);
        
        // Text input
        self.text_input.render(frame, chunks[2], &self.color_profile);
        
        // Paginator
        self.paginator.render(frame, chunks[3], &self.color_profile);
    }
    
    fn render_layout(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);
            
        // Title
        let title = Paragraph::new("Grid Layout Demo")
            .block(Block::default().borders(Borders::ALL))
            .style(
                Style::new()
                    .fg(self.theme.colors.primary.clone())
                    .to_ratatui(&self.color_profile),
            );
        frame.render_widget(title, chunks[0]);
        
        // Render grid
        self.grid.render(frame, chunks[1], &self.color_profile);
        
        // Render floating elements if active
        if let Some(ref tooltip) = self.tooltip {
            let tooltip_area = tooltip.calculate_area(
                Rect { x: area.width / 2, y: area.height / 2, width: 10, height: 3 },
                area,
            );
            tooltip.render(frame, tooltip_area, &self.color_profile);
        }
        
        if let Some(ref dropdown) = self.dropdown {
            let dropdown_area = dropdown.calculate_area(
                Rect { x: area.width / 2, y: 5, width: 20, height: 1 },
                area,
            );
            dropdown.render(frame, dropdown_area, &self.color_profile);
        }
    }
    
    fn render_styling(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .split(area);
            
        // Title
        let title = Paragraph::new("Styling & Themes")
            .block(Block::default().borders(Borders::ALL))
            .style(
                Style::new()
                    .fg(self.theme.colors.primary.clone())
                    .to_ratatui(&self.color_profile),
            );
        frame.render_widget(title, chunks[0]);
        
        // Text alignment demo
        let alignments = [
            ("Left Aligned", TextAlign::Left),
            ("Center Aligned", TextAlign::Center),
            ("Right Aligned", TextAlign::Right),
        ];
        
        let align_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 3); 3])
            .split(chunks[1]);
            
        for (i, (text, align)) in alignments.iter().enumerate() {
            let aligned_text = place_horizontal(
                text,
                align_chunks[i].width,
                match align {
                    TextAlign::Left => HAlign::Left,
                    TextAlign::Center => HAlign::Center,
                    TextAlign::Right => HAlign::Right,
                },
            );
            
            let paragraph = Paragraph::new(aligned_text)
                .block(Block::default().borders(Borders::ALL))
                .style(
                    Style::new()
                        .fg(self.theme.colors.text.clone())
                        .to_ratatui(&self.color_profile),
                );
            frame.render_widget(paragraph, align_chunks[i]);
        }
        
        // Gradient samples
        if self.show_gradient {
            let gradient_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Ratio(1, 5); 5])
                .split(chunks[2]);
                
            let gradients = [
                Gradient::sunset(),
                Gradient::ocean(),
                Gradient::forest(),
                Gradient::fire(),
                Gradient::rainbow(),
            ];
            
            for (i, gradient) in gradients.iter().enumerate() {
                if i < gradient_chunks.len() {
                    render_gradient_background(
                        frame,
                        gradient_chunks[i],
                        gradient,
                        &self.color_profile,
                    );
                }
            }
        }
    }
}

#[derive(Clone)]
enum Msg {
    Tick,
}

impl Model for UIShowcase {
    type Message = Msg;
    
    fn init(&mut self) -> Cmd<Self::Message> {
        tick(Duration::from_millis(100), || Msg::Tick)
    }
    
    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::Key(KeyEvent { key, modifiers, .. }) => match key {
                Key::Char('q') | Key::Esc => return commands::quit(),
                Key::Tab => {
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        self.previous_section();
                    } else {
                        self.next_section();
                    }
                }
                Key::F(1) => self.switch_theme(0),
                Key::F(2) => self.switch_theme(1),
                Key::F(3) => self.switch_theme(2),
                Key::F(4) => self.switch_theme(3),
                Key::F(5) => self.switch_theme(4),
                Key::Char('g') => {
                    self.show_gradient = !self.show_gradient;
                    if self.show_gradient {
                        self.current_gradient = (self.current_gradient + 1) % 6;
                    }
                }
                Key::Char('t') => {
                    self.tooltip = if self.tooltip.is_none() {
                        Some(Tooltip::new("This is a tooltip!\nIt can have multiple lines."))
                    } else {
                        None
                    };
                }
                Key::Char('d') => {
                    self.dropdown = if self.dropdown.is_none() {
                        Some(Dropdown::new(vec![
                            "Option 1".to_string(),
                            "Option 2".to_string(),
                            "Option 3".to_string(),
                            "Option 4".to_string(),
                        ]))
                    } else {
                        None
                    };
                }
                Key::Char(' ') => {
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
                Key::Left => self.paginator.previous_page(),
                Key::Right => self.paginator.next_page(),
                Key::Up => self.styled_list.select_previous(),
                Key::Down => self.styled_list.select_next(),
                _ => {}
            },
            Event::User(Msg::Tick) => {
                let now = Instant::now();
                let elapsed = now.duration_since(self.last_tick);
                self.last_tick = now;
                self.tick_count = self.tick_count.wrapping_add(1);
                
                self.timer.tick(elapsed);
                self.stopwatch.tick(elapsed);
                
                // Update status bar
                let status = format!(
                    "Section: {:?}",
                    self.current_section
                );
                self.status_bar.update_segment(0, status);
            }
            _ => {}
        }
        Cmd::none()
    }
    
    fn view(&self, frame: &mut Frame, area: Rect) {
        let (status_area, main_area) = self.status_bar.layout(area);
        
        // Render main content based on section
        match self.current_section {
            Section::Overview => self.render_overview(frame, main_area),
            Section::Components => {
                let mut showcase = self.clone();
                showcase.render_components(frame, main_area);
            }
            Section::Layout => self.render_layout(frame, main_area),
            Section::Styling => self.render_styling(frame, main_area),
            Section::Advanced => {
                // Render help
                self.help.render(frame, main_area, &self.color_profile);
            }
        }
        
        // Render status bar
        self.status_bar.render(frame, status_area, &self.color_profile);
    }
}

fn main() -> hojicha::Result<()> {
    let model = UIShowcase::new();
    let options = ProgramOptions::default();
    let program = Program::with_options(model, options)?;
    program.run()?;
    Ok(())
}