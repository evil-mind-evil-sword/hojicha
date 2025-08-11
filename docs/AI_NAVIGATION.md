# AI Agent Navigation Guide

## 🤖 Quick Start for AI Agents

This document is specifically designed for AI agents (Claude, GPT, etc.) to efficiently navigate and understand the Hojicha codebase.

### Essential Context Files
1. **Read FIRST**: [`../CLAUDE.md`](../CLAUDE.md) - Project-specific rules and context
2. **Documentation Hub**: [`./README.md`](./README.md) - Central documentation index
3. **Architecture**: [`./ARCHITECTURE.md`](./ARCHITECTURE.md) - System design overview

## 📁 File Locations by Task

### Core Framework Modifications
```
src/core.rs          → Model trait, core abstractions
src/commands.rs      → Command builders (tick, spawn, batch, etc.)
src/event.rs         → Event types and handling
src/error.rs         → Error handling patterns
```

### Runtime and Async
```
runtime/src/program.rs         → Main event loop
runtime/src/async_handle.rs    → Async event injection
runtime/src/subscription.rs    → Stream subscriptions
runtime/src/priority_queue.rs  → Event prioritization
```

### UI Components
```
pearls/src/components/         → All UI components
pearls/src/style/             → Theming and styling
pearls/src/components/mod.rs  → Component exports
```

### Testing
```
tests/behavioral/             → Integration tests
tests/property/              → Property-based tests
tests/stress/                → Performance tests
```

### Examples
```
examples/src/tutorial.rs      → Basic getting started
examples/src/showcase.rs      → Component gallery
examples/src/async_examples.rs → Async patterns
examples/src/system.rs        → System integration
```

## 🔍 Common Search Patterns

### Finding Implementations
- Model implementations: `grep -r "impl Model for"`
- Command usage: `grep -r "commands::"` 
- Component usage: `grep -r "components::"`
- Error handling: `grep -r "Result<"`

### Key Type Definitions
- `src/core.rs:Model` - Core trait
- `src/core.rs:Cmd` - Command type
- `src/event.rs:Event` - Event enum
- `runtime/src/program.rs:Program` - Runtime

## 🛠 Task-Specific Guides

### Adding a New Command
1. Edit `src/commands.rs`
2. Add builder function
3. Update `src/lib.rs` exports
4. Add tests in `tests/behavioral/`

### Creating a Component
1. Create file in `pearls/src/components/`
2. Add to `pearls/src/components/mod.rs`
3. Follow existing component patterns
4. Add example usage in `examples/src/showcase.rs`

### Modifying Event Loop
1. Primary file: `runtime/src/program.rs`
2. Event processing: `runtime/src/program/event_processor.rs`
3. Priority handling: `runtime/src/program/priority_event_processor.rs`

### Adding Async Features
1. Async bridge: `runtime/src/async_handle.rs`
2. Subscriptions: `runtime/src/subscription.rs`
3. Example patterns: `examples/src/async_examples.rs`

## 📊 Code Metrics & Patterns

### File Size Guidelines
- Keep files under 500 lines
- Split large modules into submodules
- Use mod.rs for exports

### Testing Requirements
- Unit tests in same file (`#[cfg(test)]`)
- Integration tests in `tests/behavioral/`
- Property tests in `tests/property/`

### Documentation Standards
- Doc comments for all public APIs
- Examples in doc comments when helpful
- Update docs/ for architectural changes

## 🚀 Quick Commands Reference

```bash
# Verify changes
cargo fmt --all --check     # Check formatting
cargo clippy --all-targets  # Lint
cargo test --all-features   # Test

# Build examples
cargo build --bins -p hojicha-examples

# Run specific example
cargo run --bin showcase

# Test GitHub Actions locally
act -W .github/workflows/examples.yml -j test-examples \
    --matrix os:ubuntu-latest \
    --container-architecture linux/amd64 \
    --container-daemon-socket -
```

## 🎯 Priority File List

When understanding the codebase, read in this order:

1. `src/core.rs` - Core abstractions
2. `src/event.rs` - Event system
3. `runtime/src/program.rs` - Runtime
4. `src/commands.rs` - Side effects
5. `examples/src/tutorial.rs` - Usage example

## ⚠️ Important Constraints

- **NEVER** create docs unless explicitly asked
- **ALWAYS** run `cargo fmt --all` before committing
- **PREFER** editing existing files over creating new ones
- **TEST** examples with `cargo build --bins -p hojicha-examples`
- **CHECK** CLAUDE.md for project-specific rules

## 🔗 Cross-References

### Crate Dependencies
```
hojicha-core (base)
    ↓
hojicha-runtime (uses core)
    ↓
hojicha-pearls (uses core)
    ↓
hojicha-examples (uses all)
```

### Import Patterns
```rust
// Core imports
use hojicha_core::{Model, Cmd, Event};

// Runtime imports  
use hojicha_runtime::program::Program;

// Component imports
use hojicha_pearls::components::*;
```

---

*This guide is optimized for AI agent consumption. For human-readable docs, see [README.md](./README.md)*