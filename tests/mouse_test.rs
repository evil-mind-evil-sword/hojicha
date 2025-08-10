//! Tests for mouse support

use hojicha::prelude::*;
use hojicha::program::{MouseMode, ProgramOptions};

#[test]
fn test_mouse_modes() {
    // Test default
    let opts = ProgramOptions::default();
    assert_eq!(opts.mouse_mode, MouseMode::None);

    // Test with_mouse_mode for CellMotion
    let opts = ProgramOptions::default().with_mouse_mode(MouseMode::CellMotion);
    assert_eq!(opts.mouse_mode, MouseMode::CellMotion);

    // Test with_mouse_mode for AllMotion
    let opts = ProgramOptions::default().with_mouse_mode(MouseMode::AllMotion);
    assert_eq!(opts.mouse_mode, MouseMode::AllMotion);
}

#[test]
fn test_mouse_commands() {
    // Test that mouse commands can be created
    let _enable_cell = enable_mouse_cell_motion::<()>();
    let _enable_all = enable_mouse_all_motion::<()>();
    let _disable = disable_mouse::<()>();

    // These commands return None as they're handled by the runtime
    assert!(enable_mouse_cell_motion::<()>()
        .test_execute()
        .unwrap()
        .is_none());
    assert!(enable_mouse_all_motion::<()>()
        .test_execute()
        .unwrap()
        .is_none());
    assert!(disable_mouse::<()>().test_execute().unwrap().is_none());
}

#[test]
fn test_mouse_event_handling() {
    #[derive(Clone)]
    struct MouseModel {
        last_pos: Option<(u16, u16)>,
        click_count: u32,
    }

    #[derive(Debug, Clone)]
    enum Msg {}

    impl Model for MouseModel {
        type Message = Msg;

        fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
            if let Event::Mouse(mouse) = event {
                self.last_pos = Some((mouse.column, mouse.row));
                if let crossterm::event::MouseEventKind::Down(_) = mouse.kind {
                    self.click_count += 1;
                }
            }
            Cmd::none()
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    // Test the model handles mouse events
    let mut model = MouseModel {
        last_pos: None,
        click_count: 0,
    };

    // Simulate a mouse event
    let mouse_event = MouseEvent {
        column: 10,
        row: 20,
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        modifiers: KeyModifiers::empty(),
    };

    model.update(Event::Mouse(mouse_event));
    assert_eq!(model.last_pos, Some((10, 20)));
    assert_eq!(model.click_count, 1);
}
