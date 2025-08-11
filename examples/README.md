# Hojicha Examples

This directory contains examples demonstrating various features of Hojicha.

## Core Examples

### 🎓 `tutorial`
Interactive tutorial that teaches the basics of Hojicha:
- Basic counter application
- User input handling  
- Component composition
- Styling basics

```bash
cargo run --example tutorial
```

### 🎨 `showcase`
Complete demonstration of all UI components:
- Input components (TextInput, TextArea)
- Display components (Lists, Tables, Progress bars)
- Navigation (Tabs, Paginator)
- Time components (Timer, Stopwatch)
- Feedback (Modals, Spinners)

```bash
cargo run --example showcase
```

### ✨ `visual`
Beautiful visual effects and styling showcase:
- Color palettes and gradients
- Border styles and layouts
- Animations and transitions
- Text effects and ASCII art

```bash
cargo run --example visual
```

### ⚡ `async_examples`
Async programming patterns:
- Async timers
- Stream subscriptions
- Cancellable tasks
- External event injection

```bash
cargo run --example async_examples
```

## Advanced Examples

### 🐛 `debug_features`
Debug and development features:
- Metrics collection
- Performance monitoring
- Debug overlays

```bash
cargo run --example debug_features
```

### ⚠️ `error_handling`
Error handling patterns:
- Graceful error recovery
- Error display
- Fallback UI

```bash
cargo run --example error_handling
```

### 🖥️ `headless`
Headless testing capabilities:
- Running without a terminal
- Automated testing
- CI/CD integration

```bash
cargo run --example headless
```

### 💻 `system`
System integration features:
- File operations
- Process management
- System information

```bash
cargo run --example system
```

## Running Examples

All examples can be run with:

```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example tutorial
cargo run --example showcase
```

## Navigation

Most examples support common navigation:
- `Tab` / `Shift+Tab` - Navigate between sections
- `↑` / `↓` - Navigate within sections
- `Enter` / `Space` - Select/interact
- `q` / `Esc` - Quit

## Learning Path

1. Start with **`tutorial`** to learn the basics
2. Explore **`showcase`** to see all components
3. Check **`visual`** for styling inspiration
4. Study **`async_examples`** for async patterns
5. Reference advanced examples as needed

## Troubleshooting

If an example exits immediately or shows "Device not configured":
- Make sure you're running in a real terminal (not piped or redirected)
- Try running directly: `cargo run --example <name>`
- On some systems, you may need to run with `script -c "cargo run --example <name>"`