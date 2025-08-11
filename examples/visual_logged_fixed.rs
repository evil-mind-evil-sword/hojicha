//! Visual example with better error handling and logging
//!
//! This version includes logging in the input thread to catch errors

use hojicha_core::{
    commands::{self, tick},
    core::{Cmd, Model},
    event::{Event, Key},
    logging,
    Result,
};
use hojicha_runtime::program::{Program, ProgramOptions};
use hojicha_pearls::{
    components::*,
    style::*,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
struct VisualShowcase {
    current_page: usize,
    theme: Theme,
    color_profile: ColorProfile,
    spinner: Spinner,
    tick_count: usize,
    progress: f64,
    sparkline_data: Vec<u64>,
    update_count: usize,
}

impl VisualShowcase {
    fn new() -> Self {
        Self {
            current_page: 0,
            theme: Theme::default(),
            color_profile: ColorProfile::default(),
            spinner: Spinner::with_style(SpinnerStyle::Dots),
            tick_count: 0,
            progress: 0.0,
            sparkline_data: vec![5, 10, 8, 15, 12, 20, 18],
            update_count: 0,
        }
    }

    fn next_page(&mut self) {
        self.current_page = (self.current_page + 1) % 4;
        logging::info(&format!("Switched to next page: {}", self.current_page));
    }

    fn prev_page(&mut self) {
        if self.current_page == 0 {
            self.current_page = 3;
        } else {
            self.current_page -= 1;
        }
        logging::info(&format!("Switched to previous page: {}", self.current_page));
    }
}

impl Model for VisualShowcase {
    type Message = String;

    fn init(&mut self) -> Cmd<Self::Message> {
        logging::info("VisualShowcase::init() called");
        tick(Duration::from_millis(100), || "tick".to_string())
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        self.update_count += 1;
        
        // Log every event received with update count
        logging::info(&format!("update() #{} received event: {:?}", self.update_count, event));
        
        let result = match event {
            Event::User(msg) if msg == "tick" => {
                self.tick_count += 1;
                self.spinner.tick();
                self.progress = (self.tick_count as f64 * 0.02) % 1.0;

                // Update sparkline
                let new_value = ((self.tick_count as f64 * 0.1).sin() * 10.0 + 10.0) as u64;
                self.sparkline_data.push(new_value);
                if self.sparkline_data.len() > 20 {
                    self.sparkline_data.remove(0);
                }

                // Log tick (only every 10th to avoid spam)
                if self.tick_count % 10 == 0 {
                    logging::debug(&format!("Tick #{}", self.tick_count));
                }

                tick(Duration::from_millis(100), || "tick".to_string())
            }
            Event::Key(event) => {
                logging::warn(&format!(
                    "KEY EVENT RECEIVED - key: {:?}, modifiers: {:?}",
                    event.key, event.modifiers
                ));
                
                match event.key {
                    Key::Char('q') | Key::Esc => {
                        logging::error("QUIT key pressed - returning quit command");
                        commands::quit()
                    }
                    Key::Tab if event.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) => {
                        logging::info("Shift+Tab pressed");
                        self.prev_page();
                        Cmd::none()
                    }
                    Key::Tab => {
                        logging::info("Tab pressed");
                        self.next_page();
                        Cmd::none()
                    }
                    _ => {
                        logging::warn(&format!("Unhandled key: {:?} - returning Cmd::none()", event.key));
                        Cmd::none()
                    }
                }
            }
            Event::Quit => {
                logging::error("Event::Quit received!");
                commands::quit()
            }
            _ => {
                logging::debug(&format!("Other event: {:?}", event));
                Cmd::none()
            }
        };
        
        logging::info(&format!("update() #{} returning - is_quit: {}", 
            self.update_count, result.is_quit()));
        result
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Main layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Render header
        let header = Paragraph::new(format!(
            "Visual Debug | Updates: {} | Ticks: {} | Log: /tmp/visual_debug.log", 
            self.update_count, self.tick_count
        ))
        .alignment(Alignment::Center)
        .style(ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD));
        frame.render_widget(header, main_chunks[0]);

        // Simple content for debugging
        let content = Paragraph::new(format!(
            "Page: {} | Progress: {:.0}%\n\n\
            Press any key to test\n\
            Tab/Shift+Tab: Navigate\n\
            'q' or Esc: Quit\n\n\
            If program exits on keypress, check the log!",
            self.current_page, self.progress * 100.0
        ))
        .alignment(Alignment::Center);
        frame.render_widget(content, main_chunks[1]);

        // Render footer
        let footer = Paragraph::new("Watch /tmp/visual_debug.log for events")
            .alignment(Alignment::Center);
        frame.render_widget(footer, main_chunks[2]);
    }
}

fn main() -> Result<()> {
    // Initialize file logger
    logging::init_file_logger("/tmp/visual_debug.log")
        .expect("Failed to initialize logger");
    
    logging::info("========================================");
    logging::info("Starting Visual Debug (with better error handling)");
    logging::info("========================================");
    
    // Create a custom input handler that logs errors
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);
    
    // Spawn a thread to monitor the program
    thread::spawn(move || {
        while running_clone.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(1));
            logging::debug("Monitor: Program still running");
        }
        logging::error("Monitor: Program stopped!");
    });
    
    logging::info("Creating program...");
    let program = Program::new(VisualShowcase::new())?;
    logging::info("Program created successfully");
    
    logging::info("Starting main loop...");
    let result = program.run();
    
    running.store(false, Ordering::SeqCst);
    
    match &result {
        Ok(()) => logging::info("Program ended normally"),
        Err(e) => logging::error(&format!("Program ended with error: {:?}", e)),
    }
    
    logging::info(&format!("Final result: {:?}", result));
    logging::info("========================================");
    
    result
}