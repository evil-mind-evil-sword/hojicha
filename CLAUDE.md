# Claude Code Context

This document provides essential context for Claude Code when working with the hojicha project.

## Project Overview

**hojicha** is a Rust TUI framework implementing the Elm Architecture, inspired by Charm's Bubbletea and built on Ratatui.

## Quick Commands

```bash
# Run tests
cargo test --all-features         # All tests (150+)
cargo test --test readme_examples  # Verify README examples work

# Run examples  
cargo run --example counter       # Simple counter
cargo run --example components_gallery  # All components showcase

# Code quality
cargo fmt --all                   # Format
cargo clippy --all-targets        # Lint

# Test GitHub Actions locally with act (requires Docker/Colima)
cargo act-examples                # Test examples workflow
cargo act-ci                      # Test CI workflow
cargo act-coverage                # Test coverage workflow
cargo act-security                # Test security workflow
```

## Key Files to Know

- `src/core.rs` - Model trait and Cmd type (heart of the framework)
- `runtime/src/program.rs` - Main event loop and async integration
- `src/event.rs` - Event types (Key, Mouse, User messages)
- `src/commands.rs` - Side effect commands
- `docs/README.md` - Documentation hub (START HERE for navigation)
- `docs/AI_NAVIGATION.md` - AI-specific navigation guide

## Current Focus Areas

### Async Event Handling (Recently Added)
- External event injection via `init_async_bridge()`
- Stream subscriptions with `subscribe()`  
- Cancellable operations with `spawn_cancellable()`
- Priority queue with backpressure

### Testing Philosophy
- All README examples must have tests
- Property-based testing for robustness
- Mock I/O for headless testing
- Target: 40%+ code coverage

## Common Tasks

### Adding New Features
1. Implement in appropriate module
2. Add tests (unit + integration)
3. Update README if user-facing
4. Run `cargo test --test readme_examples` to verify docs

### Before Committing
Always run:
```bash
cargo fmt --all
cargo clippy --all-targets -- -W clippy::all
cargo test --all-features
```

## Architecture Decisions

- **Cmd::none()** returns `Some(Cmd)` not `None` (None quits the program)
- Components own their state (cloning tradeoff for simplicity)
- Shared Tokio runtime for all async operations
- Event priority: Quit/Keys > User/Mouse > Tick/Resize

## Known Issues & TODOs

- Every command needs API redesign for true recurring execution
- Consider adding more built-in components
- Documentation could use more examples

## Testing Coverage

Current: ~29% (focus on critical paths)
- Commands: 78% ✅
- Core: 47% ✅  
- Program: Needs improvement
- Components: Basic coverage

## Git Workflow

```bash
# After changes
git add -A
git commit -m "type: Brief description

Detailed explanation if needed

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
git push origin master
```

## Important Reminders

- **NEVER** create docs unless asked
- **ALWAYS** test README examples  
- **PREFER** editing over creating files
- **RUN** lints before committing