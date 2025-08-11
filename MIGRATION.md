# Migration to Multi-Crate Architecture

## Overview
The hojicha project has been reorganized from a monolithic crate into three focused subcrates:

1. **hojicha-core**: Core TEA abstractions (Model, Cmd, Event, Commands)
2. **hojicha-runtime**: Event handling and async runtime (Program, subscriptions, async)
3. **hojicha-pearls**: Components and styling (UI components, themes, styles)

## For Library Users

### Update Your Dependencies
Replace your single hojicha dependency with the specific crates you need:

```toml
# Old
hojicha = "0.1"

# New
hojicha-core = "0.1"
hojicha-runtime = "0.1"  # If you use Program
hojicha-pearls = "0.1"   # If you use components/styles
```

### Update Your Imports

```rust
// Old
use hojicha::{
    core::{Cmd, Model},
    program::Program,
    components::Help,
    style::Theme,
};

// New
use hojicha_core::{
    core::{Cmd, Model},
};
use hojicha_runtime::program::Program;
use hojicha_pearls::{
    components::Help,
    style::Theme,
};
```

## Common Migration Patterns

### Basic Application
```rust
// Core abstractions always from hojicha-core
use hojicha_core::{
    commands,
    core::{Cmd, Model},
    event::{Event, Key},
    Result,
};

// Program from runtime
use hojicha_runtime::program::Program;

// Components and styles from pearls
use hojicha_pearls::{
    components::{TextInput, Help},
    style::{ColorProfile, Theme},
};
```

### Testing
```rust
// Testing utilities are split:
// - Basic test harness in hojicha-core
use hojicha_core::testing::{TestHarness, MockTerminal};

// - Test runner with Program in hojicha-runtime
use hojicha_runtime::testing::TestRunner;
```

## Running Examples

Examples are now in a separate workspace member:
```bash
cd hojicha-examples
cargo run --example counter
cargo run --example tutorial
```

## What Stays the Same

- All public APIs remain unchanged
- The Elm Architecture pattern is the same
- Component interfaces are identical
- Event types and commands work as before

## Benefits of the New Structure

1. **Smaller Dependencies**: Only include what you need
2. **Better Compilation Times**: Parallel compilation of subcrates
3. **Clearer Architecture**: Separation of concerns
4. **Easier Testing**: Test each layer independently

## Troubleshooting

### Compilation Errors
If you see "unresolved import" errors, check that you're importing from the correct crate:
- Event types → hojicha-core
- Program → hojicha-runtime
- Components → hojicha-pearls

### Missing Features
Each crate has its own features. Check the specific crate's Cargo.toml for available features.

### Version Mismatches
All three crates should use the same version. They're designed to work together.

## Questions?

File issues at: https://github.com/evil-mind-evil-sword/hojicha/issues