//! Hojicha Tutorial - Learn the Basics
//!
//! This interactive tutorial teaches you the fundamentals of Hojicha:
//! 1. Basic counter application
//! 2. User input handling
//! 3. Component composition
//! 4. Styling basics
//!
//! Press Tab to switch between examples, or follow the on-screen instructions.

use hojicha_core::{
    commands,
    core::{Cmd, Model},
    event::{Event, Key},
    Result,
};
use hojicha_runtime::program::Program;
use hojicha_pearls::{
    components::{Help, TextInput},
    style::{ColorProfile, Theme},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Stylize,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Tutorial sections
#[derive(Debug, Clone, Copy, PartialEq)]
enum Section {
    Counter,
    Input,
    Styling,
    Complete,
}

impl Section {
    fn next(self) -> Self {
        match self {
            Self::Counter => Self::Input,
            Self::Input => Self::Styling,
            Self::Styling => Self::Complete,
            Self::Complete => Self::Complete,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Counter => "1. Counter Example",
            Self::Input => "2. User Input",
            Self::Styling => "3. Styling",
            Self::Complete => "4. Complete!",
        }
    }
}

struct Tutorial {
    section: Section,

    // Counter state
    counter: i32,

    // Input state
    text_input: TextInput,
    submitted_text: Option<String>,

    // Help component
    help: Help,

    // Theme
    theme: Theme,
    color_profile: ColorProfile,
}

impl Tutorial {
    fn new() -> Self {
        let mut help = Help::new();
        help.add("Tab", "Next section")
            .add("↑/↓", "Increment/Decrement counter")
            .add("Enter", "Submit input")
            .add("q", "Quit");

        let mut text_input = TextInput::new().placeholder("Type something and press Enter...");
        text_input.focus();

        Self {
            section: Section::Counter,
            counter: 0,
            text_input,
            submitted_text: None,
            help,
            theme: Theme::default(),
            color_profile: ColorProfile::default(),
        }
    }

    fn render_counter(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .split(area);

        // Instructions
        let instructions = Paragraph::new(
            "Welcome to Hojicha! Let's start with a simple counter.\n\
             Press ↑ to increment, ↓ to decrement.",
        )
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));
        frame.render_widget(instructions, chunks[0]);

        // Counter display
        let counter_text = format!("Counter: {}", self.counter);
        let counter = Paragraph::new(counter_text)
            .style(
                ratatui::style::Style::default()
                    .fg(if self.counter > 0 {
                        ratatui::style::Color::Green
                    } else if self.counter < 0 {
                        ratatui::style::Color::Red
                    } else {
                        ratatui::style::Color::White
                    })
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Counter "));
        frame.render_widget(counter, chunks[1]);

        // Code snippet
        let code = Paragraph::new(
            "// In your update function:\n\
             Event::Key(key) => match key.key {\n\
                 Key::Up => self.counter += 1,\n\
                 Key::Down => self.counter -= 1,\n\
                 _ => {}\n\
             }",
        )
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Code "));
        frame.render_widget(code, chunks[2]);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        // Instructions
        let instructions = Paragraph::new(
            "Now let's handle text input!\n\
             Type something and press Enter to submit.",
        )
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));
        frame.render_widget(instructions, chunks[0]);

        // Text input
        self.text_input
            .render(frame, chunks[1], &self.color_profile);

        // Submitted text display
        let submitted = if let Some(ref text) = self.submitted_text {
            format!("You submitted: {}", text)
        } else {
            "Nothing submitted yet".to_string()
        };
        let submitted_widget = Paragraph::new(submitted)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Green))
            .block(Block::default().borders(Borders::ALL).title(" Submitted "));
        frame.render_widget(submitted_widget, chunks[2]);
    }

    fn render_styling(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Instructions
        let instructions = Paragraph::new(
            "Hojicha supports rich styling with themes and colors.\n\
             Here are some examples:",
        )
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));
        frame.render_widget(instructions, chunks[0]);

        // Style examples
        let style_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(chunks[1]);

        let styles = [
            ("Bold", ratatui::style::Style::default().bold()),
            ("Italic", ratatui::style::Style::default().italic()),
            (
                "Colored",
                ratatui::style::Style::default().fg(ratatui::style::Color::Magenta),
            ),
            ("Reversed", ratatui::style::Style::default().reversed()),
        ];

        for (i, chunk) in style_chunks.iter().enumerate() {
            if let Some((name, style)) = styles.get(i) {
                let demo = Paragraph::new(*name)
                    .style(*style)
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!(" {} ", name)),
                    );
                frame.render_widget(demo, *chunk);
            }
        }
    }

    fn render_complete(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            "",
            "🎉 Congratulations! 🎉",
            "",
            "You've completed the Hojicha tutorial!",
            "",
            "You've learned:",
            "✓ How to handle events and update state",
            "✓ How to work with user input",
            "✓ How to apply styles and themes",
            "",
            "Next steps:",
            "• Check out the 'showcase' example for more components",
            "• Try the 'async_examples' for async operations",
            "• Build your own TUI application!",
            "",
            "Press 'q' to quit",
        ];

        let complete = Paragraph::new(text.join("\n"))
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Green))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Tutorial Complete ")
                    .border_style(
                        ratatui::style::Style::default().fg(ratatui::style::Color::Green),
                    ),
            );
        frame.render_widget(complete, area);
    }
}

impl Model for Tutorial {
    type Message = ();

    fn init(&mut self) -> Cmd<Self::Message> {
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::Key(key) => match key.key {
                Key::Char('q') | Key::Esc => commands::quit(),
                Key::Tab => {
                    self.section = self.section.next();
                    Cmd::none()
                }
                Key::Up if self.section == Section::Counter => {
                    self.counter += 1;
                    Cmd::none()
                }
                Key::Down if self.section == Section::Counter => {
                    self.counter -= 1;
                    Cmd::none()
                }
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Header
        let header = Paragraph::new(self.section.title())
            .style(
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Yellow)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(header, chunks[0]);

        // Main content
        match self.section {
            Section::Counter => self.render_counter(frame, chunks[1]),
            Section::Input => self.render_input(frame, chunks[1]),
            Section::Styling => self.render_styling(frame, chunks[1]),
            Section::Complete => self.render_complete(frame, chunks[1]),
        }

        // Help footer
        self.help.render(frame, chunks[2], &self.color_profile);
    }
}

fn main() -> Result<()> {
    println!("Welcome to the Hojicha Tutorial!");
    println!("This interactive tutorial will teach you the basics.");
    println!("Press any key to start...");

    let program = Program::new(Tutorial::new())?;
    program.run()
}
