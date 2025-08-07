# Context for Claude

This document contains key information to help you quickly understand and work with the hojicha project.

## Project Overview

**hojicha** is a Rust implementation of the Elm Architecture for terminal user interfaces (TUIs), inspired by Charm's Bubbletea and built on Ratatui. It provides a structured, type-safe framework for building terminal applications with predictable state management.


### Why This Exists
- Ratatui is great for rendering but lacks application architecture
- Developers coming from Bubbletea want similar patterns in Rust
- The Elm Architecture provides predictable state management for TUIs

## Architecture & Key Concepts

### Core Pattern: Model-Update-View
```rust
trait Model {
    type Message;
    fn init(&mut self) -> Option<Cmd<Self::Message>>;
    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>>;
    fn view(&self, frame: &mut Frame, area: Rect);
}
```

### Event System
- **Event<M>** enum wraps all possible events (Key, Mouse, Resize, Focus, User messages, Quit)
- Events flow: Terminal → Program → Model.update() → Commands → New Events
- Two ways to quit:
  - Return `None` from update (traditional pattern)
  - Return `Some(commands::quit())` (explicit command)

### Command System
- **Cmd<M>** represents side effects that produce messages
- Commands can be batched (`batch()`) or sequenced (`sequence()`)
- Special commands: `tick()`, `every()`, `exec()`, `window_size()`, `quit()`

## Important Code Locations

### Core Files
- `src/core.rs` - Model trait, Cmd type, core abstractions
- `src/program.rs` - Main Program struct that runs the event loop
- `src/event.rs` - Event types and key mapping
- `src/commands.rs` - All built-in commands

### Component System
- `src/components/textarea.rs` - Multi-line text editor with selection
- `src/components/viewport.rs` - Scrollable content viewer  
- `src/components/spinner.rs` - Loading animations
- `src/components/list.rs` - Scrollable lists with selection and mouse support
- `src/components/table.rs` - Data tables with headers, selection, and scrolling
- `src/components/keybinding.rs` - Structured keyboard shortcuts

### Key Implementation Details

1. **Program Lifecycle**
   - Uses crossterm for terminal handling
   - Separate input thread for non-blocking event reading
   - Proper cleanup in Drop trait

2. **Terminal State Management**
   - Alt screen support (optional) 
   - Mouse tracking modes (None, CellMotion, AllMotion)
   - Bracketed paste mode (fully implemented)
   - Focus reporting (fully implemented)

3. **External Process Execution**
   - `exec()` properly releases/restores terminal
   - Uses `std::process::Command` with proper stdio handling
   - Callback receives exit code when process completes

4. **Enhanced Terminal Features**
   - Bracketed paste support with proper terminal sequences
   - Focus reporting integration with terminal capabilities
   - Suspend/resume with SIGTSTP/SIGCONT signal handling
   - FPS control to prevent CPU spinning (default 60fps)

5. **Improved Error Handling**
   - Fallible commands with `Cmd::fallible()`
   - Better error propagation throughout the framework
   - Comprehensive error context with `ErrorContext` trait

## Testing Strategy

### Current Test Coverage: 28.73% (422/1469 lines)

### Unit Tests (42 tests)
- Each module has embedded tests with comprehensive coverage
- Commands module: 78% coverage (excellent)
- Core module: 47% coverage (good)
- Use `cargo test --lib`

### Integration Tests (100+ tests across 7 files)
- `comprehensive_property_tests.rs` - Property-based testing with proptest
- `program_comprehensive_tests.rs` - Program lifecycle and options
- `commands_comprehensive_tests.rs` - All command types and composition
- `components_comprehensive_tests.rs` - Component interactions and rendering
- `event_state_property_tests.rs` - Event handling and state transitions
- `terminal_integration_tests.rs` - Terminal I/O with mocking
- `error_panic_recovery_tests.rs` - Error handling and resilience

### Test Quality Features
- Property-based testing for robustness
- Mock I/O implementations
- Unicode and edge case handling
- Error recovery and panic testing
- Concurrent access testing

### Running Tests
```bash
cargo test --all-features         # Run all 150+ tests
cargo test --lib                 # Unit tests only (42 tests)
cargo test --test comprehensive* # Property-based tests
cargo tarpaulin --lib --all-features --out Html  # Coverage analysis
```

## Common Development Tasks

### Adding a New Command
1. Add function in `src/commands.rs`
2. Create appropriate CmdInner variant if needed
3. Handle in Program's command execution logic
4. Add tests

### Creating a New Component
1. Create module in `src/components/`
2. Implement standard methods (new, handle_event, render)
3. Add to `src/components/mod.rs`
4. Create example in `examples/`

### Debugging
- Use `Program.println()` or `Program.printf()` - goes to stderr
- Enable debug output with custom logger
- Check `examples/headless.rs` for I/O debugging

## Implementation Status

### Features Complete
- Elm Architecture (Model-Update-View)
- Terminal control (alt screen, mouse, cursor)
- Bracketed paste and focus reporting
- Suspend/resume with signal handling
- FPS control and error handling
- Process execution with terminal management

### Components Available
- TextArea, Viewport, Spinner, List, Table, KeyBinding

### Testing
- 28.73% code coverage, 150+ tests
- Property-based testing, error recovery

## Quick Command Reference

```bash
# Development
cargo run --example counter        # Run simplest example
cargo run --example components_showcase  # See all components
cargo run --example enhanced_list  # New enhanced list demo
cargo run --example list          # Basic list example
cargo run --example table         # Table component demo
cargo run --example exec          # External process example

# Quality
cargo fmt --all                   # Format code
cargo clippy --all-targets -- -W clippy::all  # Lint
cargo test --all-features         # Test everything

# Documentation  
cargo doc --open --no-deps        # View API docs
```

## Key Design Notes

- `Event<M>` wraps system events (keyboard, mouse) with user messages
- Separate input thread for non-blocking event reading
- Components own their state (cloning vs borrowing tradeoff)
- Uses Ratatui directly without abstraction layer
- `update()` returning `None` quits the program