//! Demonstration of the improved Event API with helper methods
//!
//! This example shows how to use the new helper methods for more ergonomic event handling.

use hojicha_core::{
    commands,
    core::{Cmd, Model},
    event::{Event, Key, KeyModifiers, MouseButton},
};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
enum Message {
    Quit,
}

struct EventApiDemo {
    last_event: String,
    key_count: usize,
    click_count: usize,
    last_click_pos: Option<(u16, u16)>,
    window_size: (u16, u16),
}

impl EventApiDemo {
    fn new() -> Self {
        Self {
            last_event: "Waiting for input...".to_string(),
            key_count: 0,
            click_count: 0,
            last_click_pos: None,
            window_size: (80, 24),
        }
    }
}

impl Model for EventApiDemo {
    type Message = Message;

    fn init(&mut self) -> Cmd<Self::Message> {
        commands::batch(vec![
            commands::enable_mouse_all_motion(),
            commands::enable_focus_change(),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        // NEW: Using improved Event API helper methods
        
        // Check for quit events using helper methods
        if event.is_quit() || event.is_key_press(Key::Esc) {
            return commands::quit();
        }
        
        // Check for Ctrl+Q using the new helper
        if event.is_key_with_modifiers(Key::Char('q'), KeyModifiers::CONTROL) {
            return commands::quit();
        }
        
        // Handle different event types with helper methods
        
        // Key events
        if let Some(key) = event.as_key() {
            self.key_count += 1;
            
            // Use KeyEvent helper methods
            if key.is_navigation() {
                self.last_event = format!("Navigation key: {:?}", key.key);
            } else if key.is_function_key() {
                self.last_event = format!("Function key: {:?}", key.key);
            } else if key.is_char() && key.no_modifiers() {
                self.last_event = format!("Character: '{}'", key.char().unwrap());
            } else if key.is_ctrl() {
                self.last_event = format!("Ctrl+{:?}", key.key);
            } else if key.is_alt() {
                self.last_event = format!("Alt+{:?}", key.key);
            } else {
                self.last_event = format!("Key: {:?} with modifiers", key.key);
            }
        }
        
        // Mouse events
        if let Some(mouse) = event.as_mouse() {
            // Use MouseEvent helper methods
            if mouse.is_left_click() {
                self.click_count += 1;
                self.last_click_pos = Some(mouse.position());
                self.last_event = format!("Left click at ({}, {})", mouse.column, mouse.row);
            } else if mouse.is_right_click() {
                self.last_event = format!("Right click at ({}, {})", mouse.column, mouse.row);
            } else if mouse.is_scroll_up() {
                self.last_event = "Scroll up".to_string();
            } else if mouse.is_scroll_down() {
                self.last_event = "Scroll down".to_string();
            } else if mouse.is_drag() {
                let button = mouse.button().map(|b| format!("{:?}", b)).unwrap_or_else(|| "Unknown".to_string());
                self.last_event = format!("Dragging {} to ({}, {})", button, mouse.column, mouse.row);
            } else if mouse.is_move() {
                self.last_event = format!("Mouse moved to ({}, {})", mouse.column, mouse.row);
            }
            
            // Check for modified mouse events
            if mouse.has_modifiers() {
                if mouse.is_ctrl() {
                    self.last_event.push_str(" with Ctrl");
                }
                if mouse.is_shift() {
                    self.last_event.push_str(" with Shift");
                }
            }
        }
        
        // Resize events - using the new helper
        if let Some((width, height)) = event.as_resize() {
            self.window_size = (width, height);
            self.last_event = format!("Window resized to {}x{}", width, height);
        }
        
        // Focus events
        if event.is_focus() {
            self.last_event = "Terminal gained focus".to_string();
        }
        
        if event.is_blur() {
            self.last_event = "Terminal lost focus".to_string();
        }
        
        // Paste events
        if let Some(text) = event.as_paste() {
            self.last_event = format!("Pasted {} characters", text.len());
        }
        
        Cmd::none()
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Event API Demo - Press ESC or Ctrl+Q to quit ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Last Event: ", Style::default().fg(Color::Yellow)),
                Span::raw(&self.last_event),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Statistics:", Style::default().fg(Color::Green)),
            ]),
            Line::from(format!("  Key presses: {}", self.key_count)),
            Line::from(format!("  Mouse clicks: {}", self.click_count)),
        ];
        
        if let Some((x, y)) = self.last_click_pos {
            lines.push(Line::from(format!("  Last click: ({}, {})", x, y)));
        }
        
        lines.push(Line::from(format!("  Window size: {}x{}", 
            self.window_size.0, self.window_size.1)));
        
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Instructions:", Style::default().fg(Color::Magenta)),
        ]));
        lines.push(Line::from("  • Type any key to see key events"));
        lines.push(Line::from("  • Use arrow keys to see navigation events"));
        lines.push(Line::from("  • Press function keys (F1-F12)"));
        lines.push(Line::from("  • Click, drag, or scroll with mouse"));
        lines.push(Line::from("  • Try keyboard shortcuts with Ctrl/Alt/Shift"));
        lines.push(Line::from("  • Resize the terminal window"));
        lines.push(Line::from("  • Focus/unfocus the terminal"));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }
}

fn main() -> hojicha_core::Result<()> {
    let model = EventApiDemo::new();
    hojicha_runtime::program::run(model)
}