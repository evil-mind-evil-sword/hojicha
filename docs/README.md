# Hojicha Documentation Hub

> **For AI Agents**: This is the central documentation index for the Hojicha TUI framework. All paths are relative to the project root. This document provides structured navigation and context for efficient code understanding and modification.

## 🎯 Quick Navigation

### Core Documentation
- [Architecture Overview](./ARCHITECTURE.md) - System design, component relationships, and data flow
- [Development Guide](./DEVELOPMENT.md) - Setup, building, and contribution guidelines  
- [Async Design](./ASYNC_DESIGN.md) - Asynchronous programming patterns and implementation
- [Testing Best Practices](./TESTING_BEST_PRACTICES.md) - Testing strategies and guidelines
- [Roadmap](./ROADMAP.md) - Future plans and feature roadmap

### AI Agent Context Files
- [CLAUDE.md](../CLAUDE.md) - AI-specific context, commands, and project knowledge
- [Project README](../README.md) - User-facing documentation and examples

### Code Organization

```
hojicha/
├── src/                    # Core library (hojicha-core)
│   ├── core.rs            # Model trait and Cmd type (heart of TEA)
│   ├── commands.rs        # Side effect commands
│   ├── event.rs           # Event types (Key, Mouse, User messages)
│   ├── error.rs           # Error handling
│   ├── fallible.rs        # Fallible model support
│   └── testing/           # Testing utilities
│
├── runtime/               # Runtime library (hojicha-runtime)
│   └── src/
│       ├── program.rs     # Main event loop and async integration
│       ├── async_handle.rs # External event injection
│       ├── subscription.rs # Stream subscriptions
│       └── priority_queue.rs # Priority event handling
│
├── pearls/                # UI Components library (hojicha-pearls)
│   └── src/
│       ├── components/    # Pre-built UI components
│       └── style/         # Styling and theming system
│
├── examples/              # Example applications
│   └── src/
│       ├── tutorial.rs    # Getting started example
│       ├── showcase.rs    # Component gallery
│       ├── async_examples.rs # Async patterns
│       └── system.rs      # System integration
│
└── tests/                 # Test suites
    ├── behavioral/        # Integration tests
    ├── property/          # Property-based tests
    └── stress/           # Stress tests
```

## 🏗 Architecture Summary

Hojicha implements **The Elm Architecture (TEA)** for terminal UIs in Rust:

1. **Model** - Application state
2. **View** - Render function (Model → Frame)
3. **Update** - State transitions (Model × Event → Model × Cmd)
4. **Commands** - Side effects and async operations

### Key Concepts

- **Model Trait**: Core abstraction for application state
- **Cmd Type**: Represents side effects (async tasks, timers, etc.)
- **Event System**: Keyboard, mouse, and custom events with priority
- **Program**: Runtime that manages the event loop

## 📦 Crate Structure

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| `hojicha-core` | Core TEA abstractions | ratatui, crossterm |
| `hojicha-runtime` | Event loop & async runtime | hojicha-core, tokio |
| `hojicha-pearls` | UI components & styling | hojicha-core, ratatui |
| `hojicha-examples` | Example applications | All above |

## 🔧 Common Tasks

### For AI Agents

When working on this codebase:

1. **Before making changes**: Read [CLAUDE.md](../CLAUDE.md) for project-specific guidelines
2. **Finding code**: Use the organization map above to locate relevant files
3. **Running tests**: See quick commands in CLAUDE.md
4. **Testing workflows locally**: Use act with Colima (see CLAUDE.md)

### Quick Commands

```bash
# Development
cargo fmt --all           # Format code
cargo clippy --all-targets # Lint
cargo test --all-features  # Run tests

# Examples (note: these are binaries, not cargo examples)
cargo run --bin tutorial   # Basic example
cargo run --bin showcase   # Component gallery

# GitHub Actions (local testing with act + Colima)
act -W .github/workflows/examples.yml -j test-examples \
    --matrix os:ubuntu-latest \
    --container-architecture linux/amd64 \
    --container-daemon-socket -
```

## 🎨 Design Principles

1. **Simplicity**: Clean, minimal API surface
2. **Type Safety**: Leverage Rust's type system
3. **Testability**: Pure functions, controlled side effects
4. **Performance**: Efficient rendering and event handling
5. **Composability**: Small, reusable components

## 🔍 Key Files Reference

### Core Framework
- `src/core.rs`: Model trait definition
- `src/commands.rs`: Command builders (tick, spawn, etc.)
- `runtime/src/program.rs`: Main event loop

### Components
- `pearls/src/components/mod.rs`: Component exports
- `pearls/src/style/theme.rs`: Theming system

### Testing
- `tests/behavioral/integration_tests.rs`: Main integration tests
- `tests/property/program_property_tests.rs`: Property tests

## 📚 Learning Path

1. Start with [tutorial example](../examples/src/tutorial.rs)
2. Read [Architecture](./ARCHITECTURE.md)
3. Explore [showcase example](../examples/src/showcase.rs)
4. Study [async examples](../examples/src/async_examples.rs)
5. Review [testing patterns](./TESTING_BEST_PRACTICES.md)

## 🤖 AI Agent Tips

- **Context**: Always check CLAUDE.md first for project-specific rules
- **Testing**: Run `cargo test --all-features` before committing
- **Examples**: Test examples with `cargo build --bins -p hojicha-examples`
- **Formatting**: Always run `cargo fmt --all` before submitting changes
- **Documentation**: Update relevant docs when changing public APIs

## 📖 Additional Resources

- [Test Curation Plan](../tests/TEST_CURATION_PLAN.md)
- [Curated Tests Summary](../tests/CURATED_TESTS_SUMMARY.md)
- [Examples README](../examples/README.md)

---

*This documentation is optimized for both human developers and AI agents. For AI agents: treat this as your primary navigation hub for understanding and modifying the Hojicha codebase.*