# AI Agent Quick Reference Card

## 🚀 Instant Commands

```bash
# Fix formatting issues
cargo fmt --all

# Check for errors
cargo check --all

# Run tests
cargo test --all-features

# Build examples
cargo build --bins -p hojicha-examples

# Lint code
cargo clippy --all-targets
```

## 📍 Most Important Files

| Task | File |
|------|------|
| Core trait | `src/core.rs` |
| Commands | `src/commands.rs` |
| Event loop | `runtime/src/program.rs` |
| Components | `pearls/src/components/` |
| Examples | `examples/src/` |

## 🎯 Common Fixes

### Import Errors
```rust
// Core imports
use hojicha_core::{Model, Cmd, Event};
use hojicha_runtime::program::Program;
use hojicha_pearls::components::*;
```

### Doc Comment Errors
```rust
// Wrong (inner doc)
//! This is wrong

// Correct (regular comment)
// This is correct
```

### Example Binary Paths
```bash
# Wrong
cargo run --example tutorial

# Correct
cargo run --bin tutorial
```

## 🔗 Navigation

- **Docs Hub**: [`docs/README.md`](./README.md)
- **AI Guide**: [`docs/AI_NAVIGATION.md`](./AI_NAVIGATION.md)
- **Project Context**: [`CLAUDE.md`](../CLAUDE.md)

## ⚠️ Rules

1. **NEVER** create docs unless asked
2. **ALWAYS** run `cargo fmt --all`
3. **PREFER** editing over creating files
4. **CHECK** CLAUDE.md first

## 🧪 Testing Workflows Locally

```bash
# With act + Colima
act -W .github/workflows/examples.yml \
    -j test-examples \
    --matrix os:ubuntu-latest \
    --container-architecture linux/amd64 \
    --container-daemon-socket -
```

---
*Keep this reference handy when working with the Hojicha codebase*