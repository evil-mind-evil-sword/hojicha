//! Test with actual Tabs component

use hojicha::{
    commands,
    components::{TabPosition, TabStyle, Tabs, TabsBuilder},
    core::{Cmd, Model},
    event::{Event, Key},
    program::Program,
    style::ColorProfile,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Paragraph,
    Frame,
};

struct TabsActual {
    tabs: Tabs,
    color_profile: ColorProfile,
    update_count: usize,
}

impl Model for TabsActual {
    type Message = ();

    fn init(&mut self) -> Cmd<Self::Message> {
        eprintln!("Init: tabs.selected() = {}", self.tabs.selected());
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        self.update_count += 1;
        eprintln!("Update #{}: event = {:?}", self.update_count, event);

        match event {
            Event::Key(key) => {
                eprintln!("  Key: {:?}, modifiers: {:?}", key.key, key.modifiers);

                match key.key {
                    Key::Char('q') => {
                        eprintln!("  -> Quitting");
                        commands::quit()
                    }
                    Key::Tab => {
                        let before = self.tabs.selected();
                        self.tabs.select_next();
                        let after = self.tabs.selected();
                        eprintln!("  -> Tab: {} -> {}", before, after);
                        Cmd::none()
                    }
                    _ => {
                        eprintln!("  -> Unhandled key");
                        Cmd::none()
                    }
                }
            }
            _ => {
                eprintln!("  -> Non-key event");
                Cmd::none()
            }
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        eprintln!(
            "View: tabs.selected() = {}, area = {:?}",
            self.tabs.selected(),
            area
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        eprintln!("  chunks[0] = {:?}", chunks[0]);
        eprintln!("  chunks[1] = {:?}", chunks[1]);

        // Render tabs
        eprintln!("  Calling tabs.render()...");
        self.tabs.render(frame, chunks[0], &self.color_profile);
        eprintln!("  tabs.render() returned");

        // Render content
        let content = format!(
            "Tab {} is selected\n\
             Updates: {}\n\n\
             Press Tab to switch\n\
             Press q to quit",
            self.tabs.selected(),
            self.update_count
        );
        frame.render_widget(Paragraph::new(content), chunks[1]);

        eprintln!("  View complete");
    }
}

fn main() -> hojicha::Result<()> {
    eprintln!("\n=== Starting Tabs Actual Test ===\n");

    let tabs = TabsBuilder::new()
        .tab("First")
        .tab("Second")
        .tab("Third")
        .position(TabPosition::Top)
        .style(TabStyle::Rounded)
        .build();

    eprintln!("Created tabs with {} tabs", 3);

    let model = TabsActual {
        tabs,
        color_profile: ColorProfile::default(),
        update_count: 0,
    };

    eprintln!("Starting program...\n");
    let result = Program::new(model)?.run();

    eprintln!("\n=== Program exited ===");
    match &result {
        Ok(()) => eprintln!("Exit: OK"),
        Err(e) => eprintln!("Exit: Error: {}", e),
    }

    result
}
