//! Snapshot tests for UI components using insta and ratatui's TestBackend

use hojicha::components::{List, Spinner, SpinnerStyle, Table, TextArea, Viewport};
use insta::assert_snapshot;
use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
    widgets::{Block, Borders},
};

/// Helper to render a component and capture its buffer as a string
fn render_component<F>(width: u16, height: u16, render_fn: F) -> String
where
    F: FnOnce(&mut ratatui::Frame),
{
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(render_fn).unwrap();

    // Get the buffer and convert to string
    let buffer = terminal.backend().buffer();
    buffer_to_string(buffer)
}

/// Convert a buffer to string for snapshot testing
fn buffer_to_string(buffer: &Buffer) -> String {
    let mut result = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = buffer.get(x, y);
            result.push_str(cell.symbol());
        }
        if y < buffer.area.height - 1 {
            result.push('\n');
        }
    }
    result
}

/// Helper to copy buffer contents
fn copy_buffer_to_frame(src: &Buffer, frame: &mut ratatui::Frame, offset_x: u16, offset_y: u16) {
    // Buffer content is stored as a flat array, indexed from the buffer's area
    let src_area = src.area;
    for y in 0..src_area.height {
        for x in 0..src_area.width {
            // src.get() expects absolute coordinates
            let cell = src.get(src_area.x + x, src_area.y + y);
            let dst_x = offset_x + x;
            let dst_y = offset_y + y;
            if dst_x < frame.size().width && dst_y < frame.size().height {
                let dst_idx = (dst_y * frame.size().width + dst_x) as usize;
                frame.buffer_mut().content[dst_idx] = cell.clone();
            }
        }
    }
}

#[test]
fn test_list_snapshot() {
    let output = render_component(30, 10, |f| {
        let items = vec!["First item", "Second item", "Third item", "Fourth item"];
        let mut list = List::new(items);
        list.select(1); // Select second item

        let area = f.size();
        let block = Block::default().title("List Widget").borders(Borders::ALL);

        // Calculate inner area for the list
        let inner = block.inner(area);
        f.render_widget(block, area);

        // Render list in a buffer first
        let mut list_buffer = Buffer::empty(inner);
        list.render(inner, &mut list_buffer);

        // Copy list buffer to frame
        copy_buffer_to_frame(&list_buffer, f, inner.x, inner.y);
    });

    assert_snapshot!(output);
}

#[test]
fn test_spinner_snapshot() {
    let output = render_component(20, 5, |f| {
        let mut spinner = Spinner::new();
        spinner.set_style(SpinnerStyle::Dots);
        spinner.set_message("Loading...");

        // Tick a few times to get a specific frame
        for _ in 0..3 {
            spinner.tick();
        }

        let area = f.size();
        let mut buffer = Buffer::empty(area);
        spinner.render(area, &mut buffer);

        // Copy to frame
        copy_buffer_to_frame(&buffer, f, 0, 0);
    });

    assert_snapshot!(output);
}

#[test]
fn test_table_snapshot() {
    // Define a simple row type that implements TableRow
    #[derive(Debug, Clone)]
    struct Row {
        id: String,
        name: String,
        status: String,
    }

    impl hojicha::components::table::TableRow for Row {
        fn to_row(&self) -> Vec<String> {
            vec![self.id.clone(), self.name.clone(), self.status.clone()]
        }
    }

    let output = render_component(40, 10, |f| {
        let headers = vec!["ID".to_string(), "Name".to_string(), "Status".to_string()];
        let rows = vec![
            Row {
                id: "1".to_string(),
                name: "Alice".to_string(),
                status: "Active".to_string(),
            },
            Row {
                id: "2".to_string(),
                name: "Bob".to_string(),
                status: "Inactive".to_string(),
            },
            Row {
                id: "3".to_string(),
                name: "Charlie".to_string(),
                status: "Active".to_string(),
            },
        ];

        let mut table = Table::new(headers).with_rows(rows);
        table.select(1); // Select Bob's row

        let area = f.size();
        let block = Block::default().title("Table Widget").borders(Borders::ALL);

        let inner = block.inner(area);
        f.render_widget(block, area);

        let mut table_buffer = Buffer::empty(inner);
        table.render(inner, &mut table_buffer);

        // Copy to frame
        copy_buffer_to_frame(&table_buffer, f, inner.x, inner.y);
    });

    assert_snapshot!(output);
}

#[test]
fn test_textarea_snapshot() {
    let output = render_component(30, 8, |f| {
        let mut textarea = TextArea::new();
        textarea.set_value("Hello, World!\nThis is line 2\nAnd line 3.");

        let area = f.size();
        let block = Block::default().title("TextArea").borders(Borders::ALL);

        let inner = block.inner(area);
        f.render_widget(block, area);

        let mut textarea_buffer = Buffer::empty(inner);
        textarea.render(inner, &mut textarea_buffer);

        // Copy to frame
        copy_buffer_to_frame(&textarea_buffer, f, inner.x, inner.y);
    });

    assert_snapshot!(output);
}

#[test]
fn test_viewport_snapshot() {
    let output = render_component(25, 10, |f| {
        let content = (0..15)
            .map(|i| format!("Line {i}"))
            .collect::<Vec<_>>()
            .join("\n");

        let mut viewport = Viewport::new();
        viewport.set_content(content);

        // Scroll down a bit
        viewport.scroll_down(3);

        let area = f.size();
        let block = Block::default().title("Viewport").borders(Borders::ALL);

        let inner = block.inner(area);
        f.render_widget(block, area);

        let mut viewport_buffer = Buffer::empty(inner);
        viewport.render(inner, &mut viewport_buffer);

        // Copy to frame
        copy_buffer_to_frame(&viewport_buffer, f, inner.x, inner.y);
    });

    assert_snapshot!(output);
}

#[test]
fn test_list_with_selection_snapshot() {
    // Test different selection states
    for selected in [None, Some(0), Some(2)] {
        let output = render_component(20, 5, |f| {
            let items = vec!["Item A", "Item B", "Item C"];
            let mut list = List::new(items);
            if let Some(idx) = selected {
                list.select(idx);
            }

            let area = f.size();
            let mut buffer = Buffer::empty(area);
            list.render(area, &mut buffer);

            // Copy to frame
            copy_buffer_to_frame(&buffer, f, 0, 0);
        });

        let snapshot_name = match selected {
            None => "list_no_selection",
            Some(0) => "list_first_selected",
            Some(2) => "list_third_selected",
            _ => "list_other",
        };

        assert_snapshot!(snapshot_name, output);
    }
}

#[test]
fn test_spinner_styles_snapshot() {
    let styles = [
        (SpinnerStyle::Dots, "dots"),
        (SpinnerStyle::Line, "line"),
        (SpinnerStyle::Circle, "circle"),
        (SpinnerStyle::Square, "square"),
        (SpinnerStyle::Arrow, "arrow"),
    ];

    for (style, name) in styles {
        let output = render_component(15, 3, |f| {
            let mut spinner = Spinner::new();
            spinner.set_style(style);

            // Tick to get a frame
            spinner.tick();

            let area = f.size();
            let mut buffer = Buffer::empty(area);
            spinner.render_centered(area, &mut buffer);

            // Copy to frame
            copy_buffer_to_frame(&buffer, f, 0, 0);
        });

        assert_snapshot!(format!("spinner_{}", name), output);
    }
}
