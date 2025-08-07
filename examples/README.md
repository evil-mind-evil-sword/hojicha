# Boba Examples

This directory contains examples demonstrating various features of the Boba framework.

## Running Examples

All examples must be run in a real terminal (TTY). They will not work when piped or run in non-interactive environments.

```bash
# Run an example
cargo run --example counter

# Run with release optimizations
cargo run --release --example counter
```

## Available Examples

### Getting Started

- **getting_started** - Your first boba application
  ```bash
  cargo run --example getting_started
  ```
  - Learn the Elm Architecture (Model-Update-View)
  - Simple counter with timer
  - Interactive message display
  - Perfect starting point for beginners

### Core Examples

- **counter** - Simple counter with keyboard controls
  ```bash
  cargo run --example counter
  ```
  - ↑/k: increment
  - ↓/j: decrement
  - r: reset
  - q: quit

- **layouts** - Demonstrates flexible layout patterns
  ```bash
  cargo run --example layouts
  ```
  - Tab/→: next layout
  - ←: previous layout
  - q: quit
  - Shows Dashboard, Split, Grid, and Adaptive layouts

### Component Examples

- **components_gallery** - Interactive showcase of all boba components
  ```bash
  cargo run --example components_gallery
  ```
  - Tab: switch between components
  - Live demos of List, Table, TextArea, Viewport, Spinner
  - Keyboard shortcuts reference
  - Mouse interaction support

- **showcase** - Original component demonstration
  ```bash
  cargo run --example showcase
  ```
  - Tab: switch between components
  - Shows List, Table, TextArea, Viewport, and Spinner

- **interactive** - Comprehensive input handling demo
  ```bash
  cargo run --example interactive
  ```
  - Tests keyboard, mouse, paste, focus, and resize events

### System Integration

- **system** - External command execution and system features
  ```bash
  cargo run --example system
  ```
  - e: execute command
  - s: suspend (Ctrl+Z)
  - c: clear output
  - q: quit

- **debug_features** - Debug output and window title manipulation
  ```bash
  cargo run --example debug_features
  ```
  - d: trigger debug print
  - Shows debug output to stderr
  - Updates terminal window title

### Special Examples

- **headless** - Runs without terminal rendering (for testing)
  ```bash
  cargo run --example headless
  ```
  - Useful for testing TUI logic without rendering

- **styled** - Basic styled text output
  ```bash
  cargo run --example styled
  ```
  - Demonstrates style module integration with Ratatui

### Style Module Examples

- **style_showcase** - Comprehensive styling capabilities
  ```bash
  cargo run --example style_showcase
  ```
  - Tab: switch between demos
  - Space: cycle variations
  - RGB, Hex, and named colors
  - Gradient presets and animations
  - Text modifiers (bold, italic, underline, etc.)
  - Live animated effects

- **style_simple** - Simple interactive demo with gradients
  ```bash
  cargo run --example style_simple
  ```
  - Space: Change gradient
  - Shows basic styling, colors, and gradients with proper Ratatui integration

## Common Controls

Most examples follow these conventions:
- `q` or `Esc`: quit the application
- `Tab`: navigate between sections
- Arrow keys or vim keys (h,j,k,l): navigation
- Mouse: click and scroll in supported examples

## Troubleshooting

If an example exits immediately or shows "Device not configured":
- Make sure you're running in a real terminal (not piped or redirected)
- Try running directly: `cargo run --example counter`
- On some systems, you may need to run with `script -c "cargo run --example counter"`