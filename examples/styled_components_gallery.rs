//! Styled Components Gallery - Showcases the new style system components
//!
//! This example demonstrates all styled components with theme support:
//! - Button: Various button styles and sizes
//! - Modal: Dialog windows with different sizes
//! - StyledTable: Sortable data tables with themes
//! - ProgressBar: Multiple progress indicator styles
//! - StyledList: Themed lists with filtering
//! - TextInput: Form inputs with validation
//!
//! Controls:
//! - Tab/Shift+Tab: Switch between tabs
//! - Arrow keys: Navigate within components
//! - Enter/Space: Activate buttons
//! - Number keys: Sort table columns or show modals
//! - Ctrl+T: Change theme
//! - ESC: Close modal or quit

use hojicha::{
    components::{
        Button, ButtonSize, ButtonVariant, Column, Modal, ModalSize, ProgressBar, ProgressStyle,
        StyledList, StyledTable, TextInput, ValidationResult,
    },
    prelude::*,
    style::{ColorProfile, Theme},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use std::time::Duration;

/// Messages for the gallery
#[derive(Debug, Clone)]
enum Msg {
    Tick,
}

/// Gallery tabs
#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Buttons,
    Modals,
    Tables,
    Progress,
    Lists,
    Forms,
}

impl Tab {
    fn all() -> Vec<Self> {
        vec![
            Self::Buttons,
            Self::Modals,
            Self::Tables,
            Self::Progress,
            Self::Lists,
            Self::Forms,
        ]
    }

    fn title(&self) -> &str {
        match self {
            Self::Buttons => "Buttons",
            Self::Modals => "Modals",
            Self::Tables => "Tables",
            Self::Progress => "Progress",
            Self::Lists => "Lists",
            Self::Forms => "Forms",
        }
    }
}

/// Gallery model
struct GalleryModel {
    current_tab: Tab,
    theme: Theme,
    profile: ColorProfile,
    theme_index: usize,

    // Button demo state
    button_variants: Vec<ButtonVariant>,
    focused_button: usize,

    // Modal demo state
    modal: Modal,

    // Table demo state
    table: StyledTable,

    // Progress demo state
    progress_bars: Vec<ProgressBar>,
    progress_timer: u32,

    // List demo state
    list: StyledList<String>,

    // Form demo state
    text_input: TextInput,
    form_submitted: bool,
}

impl GalleryModel {
    fn new() -> Self {
        // Create sample table
        let columns = vec![
            Column::new("ID", Constraint::Length(5)).sortable(),
            Column::new("Name", Constraint::Length(20)).sortable(),
            Column::new("Status", Constraint::Length(10)).sortable(),
            Column::new("Progress", Constraint::Length(10)),
        ];

        let mut table = StyledTable::new(columns)
            .with_title("Sample Data Table")
            .with_rows(vec![
                vec!["001".into(), "Alice".into(), "Active".into(), "95%".into()],
                vec!["002".into(), "Bob".into(), "Pending".into(), "45%".into()],
                vec![
                    "003".into(),
                    "Charlie".into(),
                    "Complete".into(),
                    "100%".into(),
                ],
                vec!["004".into(), "Diana".into(), "Active".into(), "78%".into()],
                vec!["005".into(), "Eve".into(), "Inactive".into(), "0%".into()],
            ]);
        table.focus();

        // Create sample list
        let mut list = StyledList::new(vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Cherry".to_string(),
            "Date".to_string(),
            "Elderberry".to_string(),
            "Fig".to_string(),
            "Grape".to_string(),
        ])
        .with_title("Fruit Selection");
        list.focus();

        // Create progress bars with different styles
        let progress_bars = vec![
            ProgressBar::new()
                .with_label("Standard Bar")
                .with_style_variant(ProgressStyle::Bar),
            ProgressBar::new()
                .with_label("Line Progress")
                .with_style_variant(ProgressStyle::Line),
            ProgressBar::new()
                .with_label("Sparkline")
                .with_style_variant(ProgressStyle::Spark),
            ProgressBar::new()
                .with_label("Custom Style")
                .with_style_variant(ProgressStyle::Custom {
                    filled: '█',
                    empty: '░',
                })
                .with_gradient(true),
        ];

        // Create text input
        let mut text_input = TextInput::new()
            .placeholder("Enter your name...")
            .with_validation(|s| {
                if s.is_empty() {
                    ValidationResult::Invalid("Name cannot be empty".to_string())
                } else if s.len() < 3 {
                    ValidationResult::Invalid("Name must be at least 3 characters".to_string())
                } else {
                    ValidationResult::Valid
                }
            });
        text_input.focus();

        let theme = Theme::nord();

        Self {
            current_tab: Tab::Buttons,
            theme: theme.clone(),
            profile: ColorProfile::default(),
            theme_index: 0,

            button_variants: vec![
                ButtonVariant::Primary,
                ButtonVariant::Secondary,
                ButtonVariant::Success,
                ButtonVariant::Warning,
                ButtonVariant::Danger,
                ButtonVariant::Ghost,
            ],
            focused_button: 0,

            modal: Modal::info("This is a sample modal dialog.\n\nYou can display important information, confirmations, or any other content here.\n\nPress ESC to close.")
                .with_title("Information"),

            table,
            progress_bars,
            progress_timer: 0,

            list,
            text_input,
            form_submitted: false,
        }
    }

    fn next_tab(&mut self) {
        let tabs = Tab::all();
        let current_index = tabs.iter().position(|t| *t == self.current_tab).unwrap();
        let next_index = (current_index + 1) % tabs.len();
        self.current_tab = tabs[next_index];
    }

    fn prev_tab(&mut self) {
        let tabs = Tab::all();
        let current_index = tabs.iter().position(|t| *t == self.current_tab).unwrap();
        let prev_index = if current_index == 0 {
            tabs.len() - 1
        } else {
            current_index - 1
        };
        self.current_tab = tabs[prev_index];
    }

    fn next_theme(&mut self) {
        let themes = vec![
            ("Nord", Theme::nord()),
            ("Dracula", Theme::dracula()),
            ("Solarized Dark", Theme::solarized_dark()),
            ("Solarized Light", Theme::solarized_light()),
            ("Tokyo Night", Theme::tokyo_night()),
        ];

        self.theme_index = (self.theme_index + 1) % themes.len();
        self.theme = themes[self.theme_index].1.clone();

        // Apply theme to components
        self.table.apply_theme(&self.theme);
        self.list.apply_theme(&self.theme);
        self.modal.apply_theme(&self.theme);
        self.text_input.apply_theme(&self.theme);
        for pb in &mut self.progress_bars {
            pb.apply_theme(&self.theme);
        }
    }

    fn get_theme_name(&self) -> &str {
        match self.theme_index {
            0 => "Nord",
            1 => "Dracula",
            2 => "Solarized Dark",
            3 => "Solarized Light",
            4 => "Tokyo Night",
            _ => "Unknown",
        }
    }
}

impl Model for GalleryModel {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        // Start a tick timer for progress bar animation
        tick(Duration::from_millis(100), || Msg::Tick)
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(Msg::Tick) => {
                // Update progress bars
                self.progress_timer += 1;
                let progress = ((self.progress_timer as f64 * 2.0) % 100.0) / 100.0;

                for (i, pb) in self.progress_bars.iter_mut().enumerate() {
                    // Stagger the progress for visual variety
                    let offset = (i as f64 * 0.15).min(0.9);
                    let adjusted_progress = ((progress + offset) % 1.0).min(1.0);
                    pb.set_progress(adjusted_progress);
                }

                tick(Duration::from_millis(100), || Msg::Tick)
            }

            Event::Key(KeyEvent { key, modifiers }) => {
                // Global keys
                if modifiers.contains(KeyModifiers::CONTROL) {
                    match key {
                        Key::Char('c') | Key::Char('q') => return quit(),
                        Key::Char('t') => {
                            self.next_theme();
                            return Cmd::none();
                        }
                        _ => {}
                    }
                }

                match key {
                    Key::Esc => {
                        if self.modal.is_open() {
                            self.modal.close();
                        } else {
                            return quit();
                        }
                    }
                    Key::Tab => {
                        if modifiers.contains(KeyModifiers::SHIFT) {
                            self.prev_tab();
                        } else {
                            self.next_tab();
                        }
                    }
                    _ => {
                        // Handle tab-specific keys
                        match self.current_tab {
                            Tab::Buttons => match key {
                                Key::Left | Key::Char('h') => {
                                    if self.focused_button > 0 {
                                        self.focused_button -= 1;
                                    }
                                }
                                Key::Right | Key::Char('l') => {
                                    if self.focused_button < self.button_variants.len() - 1 {
                                        self.focused_button += 1;
                                    }
                                }
                                _ => {}
                            },

                            Tab::Modals => {
                                if !self.modal.is_open() {
                                    match key {
                                        Key::Char('1') => {
                                            self.modal = Modal::info("Information modal");
                                            self.modal.open();
                                        }
                                        Key::Char('2') => {
                                            self.modal = Modal::warning(
                                                "Warning! This action may have consequences.",
                                            );
                                            self.modal.open();
                                        }
                                        Key::Char('3') => {
                                            self.modal =
                                                Modal::error("Error: Something went wrong!");
                                            self.modal.open();
                                        }
                                        Key::Char('4') => {
                                            self.modal =
                                                Modal::confirm("Are you sure you want to proceed?");
                                            self.modal.open();
                                        }
                                        Key::Char('5') => {
                                            self.modal = Modal::new("This is a large modal with lots of content.\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\n\nUt enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\n\nDuis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.")
                                                .with_title("Large Modal")
                                                .with_size(ModalSize::Large);
                                            self.modal.open();
                                        }
                                        _ => {}
                                    }
                                }
                                self.modal.handle_event(Event::Key(KeyEvent { key, modifiers }));
                            }

                            Tab::Tables => {
                                self.table.handle_event(Event::Key(KeyEvent { key, modifiers }));
                            }

                            Tab::Lists => {
                                self.list.handle_event(Event::Key(KeyEvent { key, modifiers }));
                            }

                            Tab::Forms => match key {
                                Key::Enter if !self.text_input.value().is_empty() => {
                                    self.form_submitted = true;
                                }
                                Key::Char(c) => {
                                    self.text_input.insert_char(c);
                                }
                                Key::Backspace => {
                                    self.text_input.delete_char();
                                }
                                Key::Delete => {
                                    self.text_input.delete_char_forward();
                                }
                                Key::Left => {
                                    self.text_input.move_cursor_left();
                                }
                                Key::Right => {
                                    self.text_input.move_cursor_right();
                                }
                                Key::Home => {
                                    self.text_input.move_cursor_start();
                                }
                                Key::End => {
                                    self.text_input.move_cursor_end();
                                }
                                _ => {}
                            }

                            _ => {}
                        }
                    }
                }

                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let size = area;

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(size);

        // Render header with tabs
        self.render_header(frame, chunks[0]);

        // Render content based on current tab
        let content_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.current_tab.title()));
        let content_inner = content_block.inner(chunks[1]);
        frame.render_widget(content_block, chunks[1]);

        match self.current_tab {
            Tab::Buttons => self.render_buttons(frame, content_inner),
            Tab::Modals => self.render_modals(frame, content_inner),
            Tab::Tables => self.render_tables(frame, content_inner),
            Tab::Progress => self.render_progress(frame, content_inner),
            Tab::Lists => self.render_lists(frame, content_inner),
            Tab::Forms => self.render_forms(frame, content_inner),
        }

        // Render footer with help
        self.render_footer(frame, chunks[2]);

        // Render modal overlay if open
        self.modal.render(frame, size, &self.theme, &self.profile);
    }
}

impl GalleryModel {
    fn render_header(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let tabs = Tab::all();
        let tab_titles: Vec<&str> = tabs.iter().map(|t| t.title()).collect();
        let current_tab_index = Tab::all()
            .iter()
            .position(|t| *t == self.current_tab)
            .unwrap();

        let tabs = Tabs::new(tab_titles)
            .select(current_tab_index)
            .block(Block::default().borders(Borders::ALL).title(format!(
                " Styled Components Gallery - Theme: {} ",
                self.get_theme_name()
            )))
            .highlight_style(
                ratatui::style::Style::default()
                    .fg(self.theme.colors.primary.to_ratatui(&self.profile))
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(tabs, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let help_text = vec![
            Span::raw("Tab: "),
            Span::styled(
                "Switch tabs",
                ratatui::style::Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::raw("Ctrl+T: "),
            Span::styled(
                "Change theme",
                ratatui::style::Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::raw("ESC: "),
            Span::styled(
                "Close/Exit",
                ratatui::style::Style::default().add_modifier(Modifier::BOLD),
            ),
        ];

        let footer = Paragraph::new(Line::from(help_text))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        frame.render_widget(footer, area);
    }

    fn render_buttons(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .margin(1)
            .split(area);

        // Instructions
        let instructions = Paragraph::new("Use arrow keys to navigate between button variants")
            .alignment(Alignment::Center);
        frame.render_widget(instructions, chunks[0]);

        // Button variants
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 6); 6])
            .split(chunks[1]);

        for (i, (variant, chunk)) in self
            .button_variants
            .iter()
            .zip(button_chunks.iter())
            .enumerate()
        {
            let mut button = Button::new(format!("{:?}", variant))
                .with_variant(*variant)
                .with_size(ButtonSize::Medium);

            if i == self.focused_button {
                button.focus();
            }

            button.render(frame, *chunk, &self.theme, &self.profile);
        }

        // Button sizes
        let size_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 3); 3])
            .split(chunks[2]);

        let sizes = [ButtonSize::Small, ButtonSize::Medium, ButtonSize::Large];
        for (size, chunk) in sizes.iter().zip(size_chunks.iter()) {
            let button = Button::new(format!("{:?}", size))
                .with_variant(ButtonVariant::Secondary)
                .with_size(*size);
            button.render(frame, *chunk, &self.theme, &self.profile);
        }
    }

    fn render_modals(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let text = vec![
            Line::from("Press a number key to show different modal types:"),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "1",
                    ratatui::style::Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Information Modal"),
            ]),
            Line::from(vec![
                Span::styled(
                    "2",
                    ratatui::style::Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Warning Modal"),
            ]),
            Line::from(vec![
                Span::styled(
                    "3",
                    ratatui::style::Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Error Modal"),
            ]),
            Line::from(vec![
                Span::styled(
                    "4",
                    ratatui::style::Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Confirmation Modal"),
            ]),
            Line::from(vec![
                Span::styled(
                    "5",
                    ratatui::style::Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - Large Content Modal"),
            ]),
            Line::from(""),
            Line::from("Press ESC to close the modal"),
        ];

        let paragraph = Paragraph::new(text).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn render_tables(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Cast away const for rendering (safe since render doesn't mutate logical state)
        let table_ptr = &self.table as *const StyledTable as *mut StyledTable;
        unsafe {
            (*table_ptr).render(frame, area, &self.profile);
        }
    }

    fn render_progress(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3); 4])
            .margin(1)
            .split(area);

        for (pb, chunk) in self.progress_bars.iter().zip(chunks.iter()) {
            pb.render(frame, *chunk, &self.theme, &self.profile);
        }
    }

    fn render_lists(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Cast away const for rendering
        let list_ptr = &self.list as *const StyledList<String> as *mut StyledList<String>;
        unsafe {
            (*list_ptr).render(frame, area, &self.profile);
        }
    }

    fn render_forms(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .margin(1)
            .split(area);

        // Label
        let label = Paragraph::new("Name:").alignment(Alignment::Left);
        frame.render_widget(label, chunks[0]);

        // Text input
        self.text_input.render(frame, chunks[1], &self.profile);

        // Show submission status
        if self.form_submitted {
            let status =
                Paragraph::new(format!("Form submitted with: {}", self.text_input.value())).style(
                    ratatui::style::Style::default()
                        .fg(self.theme.colors.success.to_ratatui(&self.profile))
                        .add_modifier(Modifier::BOLD),
                );
            frame.render_widget(status, chunks[2]);
        }
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let model = GalleryModel::new();
    let program = Program::new(model)?;
    program.run()?;
    Ok(())
}
