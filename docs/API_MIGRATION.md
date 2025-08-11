# API Migration Guide

This guide helps you migrate from the original command API to the improved `commands_v2` API.

## Why the Change?

The original API had several inconsistencies:
- `batch()` had surprising optimization behavior
- Similar functions had different naming patterns (`batch` vs `batch_strict`)
- Timer function names (`tick`/`every`) didn't clearly indicate behavior
- Multiple ways to create async commands (`spawn` vs `custom_async`)

The new API provides:
- ✅ Consistent naming patterns
- ✅ Predictable behavior
- ✅ Clear intent from function names
- ✅ Reduced cognitive load

## Migration Table

| Old API | New API | Notes |
|---------|---------|-------|
| `batch(cmds)` | `combine_optimized(cmds)` | Keeps optimization behavior |
| `batch_strict(cmds)` | `combine(cmds)` | Always returns batch, no optimization |
| `sequence(cmds)` | `chain(cmds)` | Clearer name for sequential execution |
| `tick(duration, f)` | `after(duration, f)` or `delay(duration, f)` | Clearer one-shot semantics |
| `every(duration, f)` | `interval(duration, f)` or `repeat(duration, f)` | Clearer recurring semantics |
| `spawn(future)` | `async_cmd(future)` | Unified async interface |
| `custom_async(f)` | `async_with(f)` | Clearer closure-based async |
| `custom(f)` | `message(f)` | More descriptive name |
| `none()` | `none()` | Unchanged |
| `quit()` | `quit()` | Unchanged |

## Quick Examples

### Batch Commands

**Before:**
```rust
// Surprising: might return single command instead of batch
let cmd = batch(vec![my_cmd]);

// Predictable but verbose name
let cmd = batch_strict(vec![my_cmd]);
```

**After:**
```rust
// Clear optimization intent
let cmd = combine_optimized(vec![my_cmd]);

// Clear predictable behavior
let cmd = combine(vec![my_cmd]);
```

### Timer Commands

**Before:**
```rust
// Unclear if one-shot or recurring
let cmd = tick(Duration::from_secs(1), || Msg::Timer);
let cmd = every(Duration::from_secs(1), |_| Msg::Tick);
```

**After:**
```rust
// Clear one-shot timer
let cmd = after(Duration::from_secs(1), || Msg::Timer);
let cmd = delay(Duration::from_secs(1), || Msg::Timer);

// Clear recurring timer
let cmd = interval(Duration::from_secs(1), |_| Msg::Tick);
let cmd = repeat(Duration::from_secs(1), |_| Msg::Tick);
```

### Async Commands

**Before:**
```rust
// Two ways to do the same thing
let cmd = spawn(async { Some(msg) });
let cmd = custom_async(|| async { Some(msg) });
```

**After:**
```rust
// One clear way for each use case
let cmd = async_cmd(async { Some(msg) });
let cmd = async_with(|| async { Some(msg) });
```

## Using Both APIs During Migration

The old API functions are still available but deprecated. You can use both during migration:

```rust
// Use old API (will show deprecation warnings)
use hojicha_core::commands::{batch, tick, spawn};

// Use new API
use hojicha_core::commands_v2::{combine, after, async_cmd};
```

## Gradual Migration Strategy

1. **Start with new code**: Use `commands_v2` for all new code
2. **Update imports**: Replace `use hojicha_core::commands` with `use hojicha_core::commands_v2`
3. **Fix compilation errors**: The migration table above shows the replacements
4. **Test thoroughly**: The behavior is mostly the same, but `combine` vs `combine_optimized` have different semantics
5. **Remove deprecated usage**: Once migrated, remove any remaining deprecated function usage

## Behavioral Differences

### `combine()` vs `batch()`

The main behavioral difference is in how single-element and empty vectors are handled:

```rust
// Old batch() - optimizes
batch(vec![cmd])  // Returns cmd directly
batch(vec![])     // Returns Cmd::none()

// New combine() - predictable
combine(vec![cmd])  // Returns Batch([cmd])
combine(vec![])     // Returns Batch([])

// New combine_optimized() - same as old batch()
combine_optimized(vec![cmd])  // Returns cmd directly
combine_optimized(vec![])     // Returns Cmd::none()
```

Choose based on your needs:
- Use `combine()` when you need consistent batch behavior (e.g., for testing or when the command type matters)
- Use `combine_optimized()` when you want performance optimization and don't care about the exact command type

## Common Patterns

### Creating Multiple Timers

```rust
// Old
use hojicha_core::commands::{batch, tick, every};

let cmd = batch(vec![
    tick(Duration::from_secs(1), || Msg::OneShot),
    every(Duration::from_secs(2), |_| Msg::Recurring),
]);
```

```rust
// New
use hojicha_core::commands_v2::{combine, after, interval};

let cmd = combine(vec![
    after(Duration::from_secs(1), || Msg::OneShot),
    interval(Duration::from_secs(2), |_| Msg::Recurring),
]);
```

### Async Operations

```rust
// Old
use hojicha_core::commands::spawn;

let cmd = spawn(async {
    let data = fetch_data().await;
    Some(Msg::DataLoaded(data))
});
```

```rust
// New
use hojicha_core::commands_v2::async_cmd;

let cmd = async_cmd(async {
    let data = fetch_data().await;
    Some(Msg::DataLoaded(data))
});
```

## Need Help?

If you encounter issues during migration:
1. Check the deprecation warnings - they include the suggested replacement
2. Refer to the migration table above
3. Look at the `commands_v2` module documentation
4. Open an issue if you find any problems