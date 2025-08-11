# Hojicha Architecture

## Crate Structure

Hojicha is now organized as a workspace with three distinct crates:

### 1. `hojicha-core` - TEA Abstractions
The core crate contains only the fundamental Elm Architecture abstractions:

- **Model trait** - Your application state
- **Message trait** - Events that trigger updates  
- **Cmd type** - Side effects and async operations
- **Event enum** - Terminal events (keyboard, mouse, etc.)
- **Error handling** - Result types and error context
- **Commands** - Basic command constructors

This crate has minimal dependencies and defines the contract between your application and the runtime.

### 2. `hojicha-runtime` - Event Loop & Async
The runtime crate provides the execution environment:

- **Program** - Main event loop and application runner
- **Event Processing** - Priority queue with backpressure
- **Async Support** - Tokio integration, subscriptions, streams
- **Terminal Management** - Screen handling and restoration  
- **Error Resilience** - Panic recovery, safe mutexes
- **Metrics** - Performance monitoring and statistics

This crate handles all the complex runtime concerns so your app can focus on logic.

### 3. `hojicha-pearls` - UI Components & Styling
The pearls crate provides beautiful, pre-built components:

**Components:**
- Button, Modal, Tabs
- List, Table, Paginator
- TextInput, TextArea
- Spinner, ProgressBar
- Timer, Stopwatch
- StatusBar, Viewport

**Styling:**
- Theme system with variants
- Color profiles and gradients
- Grid and layout utilities
- Floating elements
- Style builder API

## Usage Patterns

### Basic TEA Application
```rust
use hojicha_core::{Model, Cmd, Event};
use hojicha_runtime::{Program, ProgramOptions};

struct App { /* ... */ }

impl Model for App {
    type Message = Msg;
    
    fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
        // Handle events
    }
    
    fn view(&self, frame: &mut Frame, area: Rect) {
        // Render UI
    }
}

fn main() -> Result<()> {
    let program = Program::new(App::new())?;
    program.run()
}
```

### With Components
```rust
use hojicha_pearls::components::{Button, Spinner, TextInput};
use hojicha_pearls::style::{Theme, ColorProfile};

fn view(&self, frame: &mut Frame, area: Rect) {
    let button = Button::new("Click me")
        .theme(Theme::Ocean);
    
    let spinner = Spinner::with_style(SpinnerStyle::Dots);
    
    // Render components
}
```

### With Async Operations
```rust
use hojicha_runtime::stream_builders::interval_stream;
use std::time::Duration;

fn init(&mut self) -> Cmd<Msg> {
    // Subscribe to a timer
    let stream = interval_stream(Duration::from_secs(1))
        .map(|_| Msg::Tick);
    
    program.subscribe(stream);
    
    Cmd::none()
}
```

## Migration from Monolithic Structure

### Before (Single Crate)
```rust
use hojicha::{Model, Program, components::Button};
```

### After (Workspace)
```rust
use hojicha_core::{Model, Cmd};
use hojicha_runtime::{Program, ProgramOptions};
use hojicha_pearls::components::Button;
```

## Benefits of This Architecture

1. **Modularity** - Use only what you need
2. **Clear Separation** - Core logic vs runtime vs UI
3. **Testability** - Mock runtime for testing models
4. **Flexibility** - Swap components or runtime implementations
5. **Compilation Speed** - Parallel compilation of crates
6. **API Stability** - Core rarely changes, runtime/pearls can evolve

## Dependency Graph

```
Application
    ├── hojicha-core (TEA abstractions)
    ├── hojicha-runtime (Event loop, async)
    │   └── hojicha-core
    └── hojicha-pearls (Components, styling)
        └── hojicha-core
```

## Future Possibilities

With this structure, we can:

1. **Alternative Runtimes** - WebAssembly, native GUI, etc.
2. **Component Libraries** - Different themed component sets
3. **Testing Framework** - Mock runtime for property testing
4. **Plugin System** - Third-party components as separate crates
5. **Minimal Builds** - Core-only for embedded systems