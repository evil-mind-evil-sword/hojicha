# Hojicha

> The Elm Architecture for terminal UIs in Rust, built on [Ratatui](https://github.com/ratatui-org/ratatui)

[![Crates.io](https://img.shields.io/crates/v/hojicha.svg)](https://crates.io/crates/hojicha)
[![Documentation](https://docs.rs/hojicha/badge.svg)](https://docs.rs/hojicha)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)

Hojicha is a Rust TUI framework inspired by [Bubbletea](https://github.com/charmbracelet/bubbletea) (Go) and [The Elm Architecture](https://guide.elm-lang.org/architecture/). Build interactive terminal applications using a simple, declarative model-view-update pattern.

## Features

- **Simple Architecture** - Model, Message, Update, View - that's it!
- **High Performance** - Priority-based event processing with adaptive queue management
- **Async Native** - First-class async/await support with cancellable operations
- **Built-in Components** - TextArea, List, Table, Viewport, Spinner, and more
- **Advanced Metrics** - Performance monitoring with HDR histograms and export capabilities
- **Testable** - Headless mode and deterministic testing utilities
- **Flexible Rendering** - Full Ratatui compatibility for custom widgets

## Installation

```toml
[dependencies]
hojicha = "0.1.0"
ratatui = "0.29"
```

## Quick Start

```rust
use hojicha::prelude::*;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

#[derive(Default)]
struct Counter {
    value: i32,
}

impl Model for Counter {
    type Message = ();

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::Key(key) => match key.key {
                Key::Up => self.value += 1,
                Key::Down => self.value -= 1,
                Key::Char('q') => return None, // Quit
                _ => {}
            },
            _ => {}
        }
        Cmd::none()
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let text = format!("Counter: {}\n\n↑/↓: change | q: quit", self.value);
        let widget = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Counter"));
        frame.render_widget(widget, area);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Program::new(Counter::default())?.run()
}
```

## Core Concepts

Hojicha follows The Elm Architecture pattern:

| Concept | Description |
|---------|-------------|
| **Model** | Your application state |
| **Message** | Events that trigger state changes |
| **Update** | Pure function that handles messages and updates state |
| **View** | Pure function that renders the UI based on state |
| **Command** | Side effects (async operations, timers, etc.) |

## Advanced Features

### Async Integration

Hojicha provides multiple patterns for async operations:
```rust
// External event injection
let sender = program.init_async_bridge();
thread::spawn(move || {
    sender.send(Event::User(Msg::Tick)).ok();
});

// Stream subscriptions
let stream = IntervalStream::new(interval).map(|_| Msg::Tick);
let subscription = program.subscribe(stream);

// Cancellable operations
let handle = program.spawn_cancellable(|token| async move {
    // Long-running task with cancellation support
});
```

### Performance & Reliability

#### Priority Event Processing
Automatic event prioritization ensures UI responsiveness:
- **Critical**: Quit, suspend/resume signals
- **High**: Keyboard input
- **Normal**: Mouse, user messages, paste
- **Low**: Tick, resize, focus/blur

#### Dynamic Queue Management
```rust
// Adaptive queue resizing based on load
program.resize_queue(new_capacity)?;
let capacity = program.queue_capacity();

// Custom priority configuration
let config = PriorityConfig {
    max_queue_size: 10000,
    priority_mapper: Some(Box::new(custom_priority_fn)),
};
program.with_priority_config(config);
```

#### Advanced Metrics
```rust
// Real-time performance monitoring
let stats = program.advanced_stats();
println!("P99 latency: {:?}", stats.event_latency_p99());

// Export metrics in various formats
let json = stats.export_json();
let prometheus = stats.export_prometheus();
```

### Built-in Components

| Component | Description |
|-----------|-------------|
| `TextArea` | Multi-line text editor with vim-like keybindings |
| `List` | Scrollable lists with keyboard navigation |
| `Table` | Data tables with sortable headers |
| `Viewport` | Scrollable content area for large text |
| `Spinner` | Animated loading indicators |
| `KeyBinding` | Display keyboard shortcuts |

### Terminal Features
- Alt screen mode
- Mouse tracking (click, drag, scroll)
- Cursor control
- Bracketed paste
- Focus change detection
- External process execution with TTY management

## Examples

Explore the examples to learn different aspects of Hojicha:

```bash
# Basic counter application
cargo run --example counter

# All components showcase
cargo run --example components_gallery

# Async operations demo
cargo run --example async_timer

# Cancellable tasks
cargo run --example cancellable_demo

# Stream integration
cargo run --example stream_demo
```

## Testing

Hojicha provides optimized testing strategies for fast feedback:

```bash
# Run all tests (~1 second for unit tests)
cargo test --all-features

# Run specific test categories
cargo test --test readme_examples  # Verify README code
cargo test program                 # Core functionality
cargo test async                   # Async integration

# Run benchmarks
cargo bench
```

Tests use deterministic patterns without timing dependencies for reliability.

## Documentation

- **API Reference**: [docs.rs/hojicha](https://docs.rs/hojicha)
- **Development Guide**: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)
- **Async Design**: [docs/ASYNC_DESIGN.md](docs/ASYNC_DESIGN.md)
- **Testing Guide**: [docs/TESTING_BEST_PRACTICES.md](docs/TESTING_BEST_PRACTICES.md)

## Contributing

Contributions are welcome! Please read our [development guide](docs/DEVELOPMENT.md) for:
- Architecture overview
- Testing requirements
- Code style guidelines
- Performance considerations

## License

GPL v3.0 - See [LICENSE](LICENSE) for details.

## Acknowledgments

- [Bubbletea](https://github.com/charmbracelet/bubbletea) - Original inspiration from the Go ecosystem
- [The Elm Architecture](https://guide.elm-lang.org/architecture/) - Architectural pattern
- [Ratatui](https://github.com/ratatui-org/ratatui) - Outstanding TUI rendering library
