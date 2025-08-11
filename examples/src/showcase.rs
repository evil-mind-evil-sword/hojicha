// Complete UI Component Showcase
//
// This example demonstrates all UI components available in Hojicha.
// Navigate through different component categories to see them in action.
//
// Controls:
// - Tab/Shift+Tab: Navigate between categories
// - Arrow keys: Navigate within components
// - Space: Interact with components
// - q: Quit

use hojicha_core::commands;
use hojicha_core::event::{Event, Key, KeyModifiers};
use hojicha_core::prelude::*;
use hojicha_pearls::components::{List, Spinner, SpinnerStyle, Table, TextArea, Viewport};
use hojicha_pearls::style::{ColorProfile, Theme};
use hojicha_runtime::program::{MouseMode, Program, ProgramOptions};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Line,
};
use std::time::Duration;

#[derive(Debug, Clone)]
enum Tab {
    List,
    Table,
    TextArea,
    Viewport,
    Spinner,
}

struct App {
    current_tab: Tab,
    list: List<String>,
    table: Table<Vec<String>>,
    textarea: TextArea,
    viewport: Viewport,
    spinner: Spinner,
    counter: usize,
    status_message: String,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    #[allow(dead_code)]
    Quit,
}

impl App {
    fn new() -> Self {
        let list = List::new(vec![
            "🎨 First item".to_string(),
            "🚀 Second item".to_string(),
            "💡 Third item".to_string(),
            "🎯 Fourth item".to_string(),
            "⚡ Fifth item".to_string(),
            "🌟 Sixth item".to_string(),
        ]);

        let table = Table::new(vec![
            "Name".to_string(),
            "Type".to_string(),
            "Status".to_string(),
        ]);

        let textarea = TextArea::new();

        let viewport_content = r#"# Hojicha Framework

The Elm Architecture for Ratatui - A framework for building terminal user interfaces.

## Features

- **Model-Update-View pattern**: Clean separation of concerns
- **Type-safe message passing**: Leverage Rust's type system
- **Composable commands**: Build complex behaviors from simple parts

## Components

- **List**: Scrollable lists with keyboard and mouse support
- **Table**: Data tables with headers and selection
- **TextArea**: Multi-line text editor
- **Viewport**: Scrollable content viewer
- **Spinner**: Loading animations

## Getting Started

Check out the examples directory for usage patterns.

Happy coding with hojicha! 🍵
"#;

        let mut viewport = Viewport::new();
        viewport.set_content(viewport_content.to_string());

        let mut spinner = Spinner::new();
        spinner.set_style(SpinnerStyle::Dots);

        Self {
            current_tab: Tab::List,
            list,
            table,
            textarea,
            viewport,
            spinner,
            counter: 0,
            status_message: "Press Tab to switch components, q/Esc to quit".to_string(),
        }
    }

    fn handle_tab_input(&mut self, event: &Event<Message>) -> Cmd<Message> {
        match self.current_tab {
            Tab::List => {
                if let Event::Key(key) = event {
                    if self.list.handle_key(key) {
                        if let Some(selected) = self.list.selected_item() {
                            self.status_message = format!("Selected: {selected}");
                        }
                    }
                } else if let Event::Mouse(mouse) = event {
                    self.list.handle_mouse(mouse, Rect::new(0, 3, 80, 20));
                }
            }
            Tab::Table => {
                if let Event::Key(key) = event {
                    self.table.handle_key(key);
                    if let Some(row) = self.table.selected_row() {
                        self.status_message = format!("Selected row: {row:?}");
                    }
                } else if let Event::Mouse(mouse) = event {
                    self.table.handle_mouse(mouse, Rect::new(0, 3, 80, 20));
                }
            }
            Tab::TextArea => {
                if let Event::Key(key) = event {
                    if self.textarea.handle_event(key) {
                        self.status_message = "Text updated".to_string();
                    }
                } else if let Event::Mouse(_mouse) = event {
                    // TextArea doesn't have handle_mouse
                } else if let Event::Paste(text) = event {
                    self.textarea.insert_text(text);
                    self.status_message = "Pasted text from clipboard".to_string();
                }
            }
            Tab::Viewport => {
                if let Event::Key(key) = event {
                    self.viewport.handle_key(key);
                    self.status_message = "Viewport scrolled".to_string();
                } else if let Event::Mouse(mouse) = event {
                    self.viewport.handle_mouse(mouse);
                }
            }
            Tab::Spinner => {
                self.spinner.tick();
                self.status_message = format!("Spinner frame: {}", self.counter % 8);
            }
        }
        Cmd::none()
    }
}

impl Model for App {
    type Message = Message;

    fn init(&mut self) -> Cmd<Self::Message> {
        commands::tick(Duration::from_millis(100), || Message::Tick)
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(Message::Tick) => {
                self.counter += 1;
                if matches!(self.current_tab, Tab::Spinner) {
                    self.spinner.tick();
                }
                commands::tick(Duration::from_millis(100), || Message::Tick)
            }
            Event::User(Message::Quit) => commands::quit(),
            Event::Key(key) => {
                // Check for quit keys first - using improved API
                if key.is(Key::Char('q')) && key.no_modifiers() || key.is(Key::Esc) {
                    commands::quit()
                } else if key.is(Key::Tab) && key.no_modifiers() {
                    // Don't call update recursively - just handle the tab switch directly
                    self.current_tab = match self.current_tab {
                        Tab::List => Tab::Table,
                        Tab::Table => Tab::TextArea,
                        Tab::TextArea => Tab::Viewport,
                        Tab::Viewport => Tab::Spinner,
                        Tab::Spinner => Tab::List,
                    };
                    self.status_message = format!("Switched to {:?} tab", self.current_tab);
                    Cmd::none()
                } else if key.is_with_modifiers(Key::Tab, KeyModifiers::SHIFT) {
                    // Don't call update recursively - just handle the tab switch directly
                    self.current_tab = match self.current_tab {
                        Tab::List => Tab::Spinner,
                        Tab::Table => Tab::List,
                        Tab::TextArea => Tab::Table,
                        Tab::Viewport => Tab::TextArea,
                        Tab::Spinner => Tab::Viewport,
                    };
                    self.status_message = format!("Switched to {:?} tab", self.current_tab);
                    Cmd::none()
                } else {
                    self.handle_tab_input(&event)
                }
            }
            Event::Mouse(mouse) => self.handle_tab_input(&Event::Mouse(mouse)),
            Event::Paste(text) => self.handle_tab_input(&Event::Paste(text)),
            Event::Resize { width, height } => {
                self.status_message = format!("Terminal resized to {width}x{height}");
                Cmd::none()
            }
            Event::Focus => {
                self.status_message = "Terminal focused".to_string();
                Cmd::none()
            }
            Event::Blur => {
                self.status_message = "Terminal blurred".to_string();
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        // Tab bar
        let titles = vec!["List", "Table", "TextArea", "Viewport", "Spinner"];
        let index = match self.current_tab {
            Tab::List => 0,
            Tab::Table => 1,
            Tab::TextArea => 2,
            Tab::Viewport => 3,
            Tab::Spinner => 4,
        };

        let tabs = Tabs::new(titles.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Hojicha Showcase "),
            )
            .select(index)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(tabs, chunks[0]);

        // Content area
        let content_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", titles[index]))
            .style(Style::default().fg(Color::White));

        frame.render_widget(content_block.clone(), chunks[1]);

        let inner = Rect::new(
            chunks[1].x + 1,
            chunks[1].y + 1,
            chunks[1].width.saturating_sub(2),
            chunks[1].height.saturating_sub(2),
        );

        // Render the current component
        match self.current_tab {
            Tab::List => {
                self.list.clone().render(inner, frame.buffer_mut());
            }
            Tab::Table => {
                // For now, just show a placeholder since Table rendering is complex
                let placeholder = Paragraph::new(vec![
                    Line::from("Table component"),
                    Line::from(""),
                    Line::from("Headers: Name | Type | Status"),
                    Line::from(""),
                    Line::from("Use arrow keys to navigate"),
                ]);
                frame.render_widget(placeholder, inner);
            }
            Tab::TextArea => {
                // Render TextArea using frame's buffer
                self.textarea.render(inner, frame.buffer_mut());
            }
            Tab::Viewport => {
                // Render Viewport using frame's buffer
                self.viewport.render(inner, frame.buffer_mut());
            }
            Tab::Spinner => {
                // Render Spinner with some text
                let spinner_area = Rect::new(
                    inner.x + inner.width / 2 - 20,
                    inner.y + inner.height / 2,
                    40,
                    1,
                );
                self.spinner
                    .clone()
                    .render(spinner_area, frame.buffer_mut());

                let info = Paragraph::new(vec![
                    Line::from(""),
                    Line::from("The spinner animates automatically with each tick."),
                    Line::from(""),
                    Line::from("Available styles:"),
                    Line::from("• Dots, Line, Circle, Square"),
                    Line::from("• Triangle, Arrow, Bounce, Box"),
                ])
                .style(Style::default().fg(Color::Gray));
                frame.render_widget(info, inner);
            }
        }

        // Status bar
        let status =
            Paragraph::new(self.status_message.as_str()).style(Style::default().fg(Color::Cyan));
        frame.render_widget(status, chunks[2]);
    }
}

fn main() -> anyhow::Result<()> {
    let options = ProgramOptions::default()
        .with_mouse_mode(MouseMode::AllMotion)
        .with_bracketed_paste(true)
        .with_focus_reporting(true)
        .with_fps(100);

    let program = Program::with_options(App::new(), options)?;
    program.run()?;
    Ok(())
}
