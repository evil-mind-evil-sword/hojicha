//! Components Gallery - Interactive showcase of all boba components
//!
//! This example demonstrates all built-in components:
//! - List: Scrollable lists with selection
//! - Table: Data tables with headers and scrolling  
//! - TextArea: Multi-line text editor
//! - Viewport: Scrollable content viewer
//! - Spinner: Loading animations
//! - KeyBinding: Keyboard shortcut display
//!
//! Controls:
//! - Tab: Switch between components
//! - ↑/↓: Navigate within components
//! - Enter: Select/activate
//! - e: Edit mode (in TextArea)
//! - q: Quit

use hojicha::event::Key;
use hojicha::prelude::*;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};

struct ComponentsGallery {
    selected_tab: usize,
    list: List<String>,
    text_area: TextArea,
    viewport: Viewport,
    spinner_index: usize,
    tick: u32,
}

impl Model for ComponentsGallery {
    type Message = Msg;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        Some(tick(std::time::Duration::from_millis(100), || Msg::Tick))
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::Key(key_event) => match key_event.key {
                Key::Char('q') | Key::Esc => return None,
                Key::Tab => {
                    self.selected_tab = (self.selected_tab + 1) % 5;
                }
                _ => {} // Components don't have handle_event - they're handled in render
            },
            Event::Mouse(_mouse) => {
                // Mouse events would be handled per component
            }
            Event::User(Msg::Tick) => {
                self.tick = self.tick.wrapping_add(1);
                if self.tick % 2 == 0 {
                    self.spinner_index = (self.spinner_index + 1) % 8;
                }
            }
            _ => {}
        }
        None
    }

    fn view(&self, frame: &mut Frame, _area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(frame.area());

        // Header
        self.render_header(frame, chunks[0]);

        // Tabs
        let tab_titles = vec!["List", "TextArea", "Viewport", "Spinner", "KeyBindings"];
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .select(self.selected_tab);
        frame.render_widget(tabs, chunks[1]);

        // Content area with padding
        let content_area = Layout::default().margin(1).split(chunks[2])[0];

        // Render selected component
        match self.selected_tab {
            0 => self.render_list_demo(frame, content_area),
            1 => self.render_textarea_demo(frame, content_area),
            2 => self.render_viewport_demo(frame, content_area),
            3 => self.render_spinner_demo(frame, content_area),
            4 => self.render_keybindings_demo(frame, content_area),
            _ => {}
        }

        // Footer
        self.render_footer(frame, chunks[3]);
    }
}

impl ComponentsGallery {
    fn new() -> Self {
        // Initialize List
        let items = vec![
            "🍎 Apple - A sweet red fruit".to_string(),
            "🍌 Banana - A yellow tropical fruit".to_string(),
            "🍇 Grapes - Small round fruits in clusters".to_string(),
            "🍊 Orange - A citrus fruit full of vitamin C".to_string(),
            "🍓 Strawberry - A red berry with seeds outside".to_string(),
            "🥝 Kiwi - A fuzzy brown fruit with green inside".to_string(),
            "🍑 Peach - A soft fruit with fuzzy skin".to_string(),
            "🍒 Cherry - Small red fruits, often in pairs".to_string(),
            "🥭 Mango - A tropical stone fruit".to_string(),
            "🍍 Pineapple - A tropical fruit with spiky skin".to_string(),
        ];

        let list = List::new(items).with_options(ListOptions {
            item_style: Style::default().fg(Color::White),
            selected_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            highlight_selection: true,
            wrap_around: true,
            page_size: 10,
        });

        // Initialize TextArea
        let mut text_area = TextArea::with_options(TextAreaOptions {
            show_line_numbers: true,
            line_number_style: Style::default().fg(Color::DarkGray),
            text_style: Style::default().fg(Color::White),
            cursor_style: Style::default().fg(Color::Black).bg(Color::Cyan),
            ..TextAreaOptions::default()
        });

        text_area.set_value(
            "Welcome to the TextArea component!\n\
             \n\
             This is a multi-line text editor with:\n\
             - Line numbers\n\
             - Cursor movement (arrows)\n\
             - Text selection\n\
             - Insert and delete\n\
             \n\
             Press 'e' to enter edit mode.\n\
             Press Esc to exit edit mode.\n\
             \n\
             Try editing this text!",
        );

        // Initialize Viewport
        let mut viewport = Viewport::with_options(ViewportOptions {
            show_scrollbar: true,
            scrollbar_style: Style::default().fg(Color::White),
            scrollbar_track_style: Style::default().fg(Color::DarkGray),
            text_style: Style::default().fg(Color::White),
            wrap_lines: true,
            scroll_amount: 3,
        });

        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                     Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
                     Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris \
                     nisi ut aliquip ex ea commodo consequat.\n\n\
                     Duis aute irure dolor in reprehenderit in voluptate velit esse \
                     cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat \
                     cupidatat non proident, sunt in culpa qui officia deserunt mollit \
                     anim id est laborum.\n\n\
                     The viewport component allows you to:\n\
                     • Scroll through long content\n\
                     • Use arrow keys or mouse wheel\n\
                     • Jump with Page Up/Down\n\
                     • Go to top with Home\n\
                     • Go to bottom with End\n\n";

        viewport.set_content(lorem.repeat(5));

        Self {
            selected_tab: 0,
            list,
            text_area,
            viewport,
            spinner_index: 0,
            tick: 0,
        }
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new("🎨 Boba Components Gallery")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(title, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let style = Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::ITALIC);

        let help_text = match self.selected_tab {
            0 => "↑/↓: Navigate | Enter: Select | Tab: Next component",
            1 => "e: Edit mode | Esc: Exit edit | Tab: Next component",
            2 => "↑/↓: Scroll | PgUp/PgDn: Page | Home/End: Jump | Tab: Next",
            3 => "Watch the spinner animate! | Tab: Next component",
            4 => "View keyboard shortcuts | Tab: Next component",
            _ => "Tab: Switch component | q: Quit",
        };

        let footer = Paragraph::new(help_text)
            .style(style)
            .block(Block::default().borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(footer, area);
    }

    fn render_list_demo(&self, frame: &mut Frame, area: Rect) {
        let mut list = self.list.clone().with_block(
            Block::default()
                .borders(Borders::ALL)
                .title("List Component - Fruit Selection"),
        );

        list.render(area, frame.buffer_mut());

        // Show selected item
        let selected = self.list.selected();
        if selected > 0 {
            let info_area = Rect {
                x: area.x + area.width - 35,
                y: area.y + 2,
                width: 30,
                height: 3,
            };

            let selected_text = format!("Selected: Item #{}", selected + 1);
            let info = Paragraph::new(selected_text)
                .block(Block::default().borders(Borders::ALL).title("Selection"))
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(info, info_area);
        }
    }

    fn render_textarea_demo(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(if self.text_area.is_focused() {
                "TextArea Component - EDIT MODE (Press Esc to exit)"
            } else {
                "TextArea Component - Press 'e' to edit"
            });

        let text_area = self.text_area.clone();
        // TextArea doesn't have set_block, render with block directly
        let inner = block.inner(area);
        frame.render_widget(block, area);
        text_area.render(inner, frame.buffer_mut());

        // Show cursor position
        let (line, col) = self.text_area.cursor();
        let info_area = Rect {
            x: area.x + area.width - 35,
            y: area.y + 2,
            width: 30,
            height: 3,
        };

        let cursor_text = format!("Line: {} Col: {}", line + 1, col + 1);
        let info = Paragraph::new(cursor_text)
            .block(Block::default().borders(Borders::ALL).title("Cursor"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(info, info_area);
    }

    fn render_viewport_demo(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Viewport Component - Scrollable Content");

        let viewport = self.viewport.clone();
        // Viewport doesn't have set_block, render with block directly
        let inner = block.inner(area);
        frame.render_widget(block, area);
        viewport.render(inner, frame.buffer_mut());

        // Show scroll position
        let (current, total) = self.viewport.scroll_position();
        let percentage = if total > 0 {
            (current as f32 / total as f32 * 100.0) as u16
        } else {
            0
        };

        let info_area = Rect {
            x: area.x + area.width - 35,
            y: area.y + 2,
            width: 30,
            height: 3,
        };

        let scroll_text = format!("Line {}/{} ({}%)", current + 1, total, percentage);
        let info = Paragraph::new(scroll_text)
            .block(Block::default().borders(Borders::ALL).title("Position"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(info, info_area);
    }

    fn render_spinner_demo(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .split(area);

        // Different spinner styles
        let spinners = [
            (SpinnerStyle::Dots, "Dots"),
            (SpinnerStyle::Line, "Line"),
            (SpinnerStyle::Circle, "Circle"),
            (SpinnerStyle::Square, "Square"),
            (SpinnerStyle::Arrow, "Arrow"),
        ];

        for (i, (style, name)) in spinners.iter().enumerate().take(chunks.len()) {
            // Get frames directly from the style
            let frames: &[&str] = match style {
                SpinnerStyle::Dots => &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
                SpinnerStyle::Line => &["|", "/", "-", "\\"],
                SpinnerStyle::Circle => &["◐", "◓", "◑", "◒"],
                SpinnerStyle::Square => &["◰", "◳", "◲", "◱"],
                SpinnerStyle::Arrow => &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
                _ => &[" "], // Default for other styles
            };
            let frame_char = frames[self.spinner_index % frames.len()];

            let text = format!("{} {} - Loading...", frame_char, name);
            let color = match style {
                SpinnerStyle::Dots => Color::Cyan,
                SpinnerStyle::Line => Color::Green,
                SpinnerStyle::Circle => Color::Yellow,
                SpinnerStyle::Square => Color::Magenta,
                SpinnerStyle::Arrow => Color::Blue,
                _ => Color::White, // Default for other styles
            };

            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title(*name))
                .style(Style::default().fg(color));

            frame.render_widget(paragraph, chunks[i]);
        }
    }

    fn render_keybindings_demo(&self, frame: &mut Frame, area: Rect) {
        // Create help text lines for keyboard shortcuts
        let shortcuts = vec![
            ("q/Esc", "Quit application"),
            ("Tab", "Switch between components"),
            ("↑/k", "Move up / Previous item"),
            ("↓/j", "Move down / Next item"),
            ("Enter", "Select / Activate"),
            ("e", "Enter edit mode (TextArea)"),
            ("PgUp", "Scroll up one page"),
            ("PgDn", "Scroll down one page"),
            ("←/→", "Navigate columns (Table)"),
            ("Space", "Toggle selection"),
            ("Home/End", "Jump to start/end"),
        ];

        let mut lines = vec![];
        for (key, desc) in shortcuts {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:12}", key),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(desc, Style::default().fg(Color::White)),
            ]));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Keyboard Shortcuts");

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area)
    }
}

#[derive(Clone)]
enum Msg {
    Tick,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let options = ProgramOptions::default()
        .with_alt_screen(true)
        .with_mouse_mode(MouseMode::CellMotion);

    let program = Program::with_options(ComponentsGallery::new(), options)?;
    program.run()?;
    Ok(())
}
