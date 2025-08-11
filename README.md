# Hojicha

> The Elm Architecture for Terminal UIs in Rust

[![Crates.io](https://img.shields.io/crates/v/hojicha.svg)](https://crates.io/crates/hojicha)
[![Documentation](https://docs.rs/hojicha/badge.svg)](https://docs.rs/hojicha)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)

Hojicha implements [The Elm Architecture](https://guide.elm-lang.org/architecture/) for terminal applications. Built on [Ratatui](https://github.com/ratatui-org/ratatui), inspired by [Bubbletea](https://github.com/charmbracelet/bubbletea).

## Features

- **Simple Architecture** - Model-View-Update pattern
- **High Performance** - Priority event processing with adaptive queues
- **Async Native** - First-class async/await, cancellation, streams
- **Component Library** - Pre-built UI components
- **Testing Utilities** - Headless mode, deterministic testing
- **Metrics** - Built-in performance monitoring

## Installation

```toml
[dependencies]
hojicha-core = "0.1"
hojicha-runtime = "0.1"
```

## Quick Start

```rust
use hojicha_core::prelude::*;
use hojicha_runtime::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

struct Counter {
    value: i32,
}

impl Model for Counter {
    type Message = ();

    fn update(&mut self, event: Event<()>) -> Cmd<()> {
        match event {
            Event::Key(key) => match key.key {
                Key::Up => self.value += 1,
                Key::Down => self.value -= 1,
                Key::Char('q') => return commands::quit(),
                _ => {}
            },
            _ => {}
        }
        Cmd::none()
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let text = format!("Counter: {}\n\nUp/Down: change | q: quit", self.value);
        let widget = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Counter"));
        frame.render_widget(widget, area);
    }
}

fn main() -> Result<()> {
    Program::new(Counter { value: 0 })?.run()
}
```

## Architecture

The Elm Architecture consists of:

| Component | Purpose |
|-----------|---------|
| **Model** | Application state |
| **Message** | Events that trigger state changes |
| **Update** | Handle events and update state |
| **View** | Render UI from state |
| **Command** | Side effects (async operations, I/O) |

## Common Patterns

### Async Operations

```rust
fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match event {
        Event::User(Msg::FetchData) => {
            commands::spawn(async {
                let data = fetch_api().await.ok()?;
                Some(Msg::DataLoaded(data))
            })
        }
        _ => Cmd::none()
    }
}
```

### Timers

```rust
fn init(&mut self) -> Cmd<Msg> {
    commands::every(Duration::from_secs(1), |_| Msg::Tick)
}
```

### Stream Subscriptions

```rust
let stream = websocket.messages().map(Msg::WsMessage);
let subscription = program.subscribe(stream);
```

## Components

Available via `hojicha-pearls`:

**Input**: TextInput, TextArea, Button  
**Display**: List, Table, Tabs, Modal, ProgressBar, Spinner  
**Layout**: Grid, FloatingElement, StatusBar, Viewport

## Testing

```rust
#[test]
fn test_counter() {
    TestHarness::new(Counter { value: 0 })
        .send_event(Event::Key(KeyEvent::new(Key::Up)))
        .run()
        .assert_model(|m| m.value == 1);
}
```

## Performance Metrics

```rust
let program = Program::new(model)?
    .with_priority_config(PriorityConfig {
        enable_metrics: true,
        ..Default::default()
    });

// Export metrics
program.metrics_json();
program.metrics_prometheus();
```

## Advanced Features

### Priority Event Processing

1. **Critical**: Quit, suspend
2. **High**: User input (keyboard, mouse)
3. **Normal**: User messages, timers
4. **Low**: Resize, background tasks

### Cancellable Operations

```rust
let handle = program.spawn_cancellable(|token| async move {
    while !token.is_cancelled() {
        process_batch().await;
    }
});
```

## Documentation

- [Common Patterns](./docs/COMMON_PATTERNS.md)
- [Architecture Guide](./docs/ARCHITECTURE.md)
- [Development Guide](./docs/DEVELOPMENT.md)
- [Testing Guide](./docs/TESTING_BEST_PRACTICES.md)
- [API Reference](https://docs.rs/hojicha)

## Examples

```bash
cargo run --example counter
cargo run --example todo_list
cargo run --example text_editor
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

GPL-3.0