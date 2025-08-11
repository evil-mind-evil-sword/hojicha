# Deprecation Tracking for Error Resilience Migration

This document tracks code that will be deprecated once we fully migrate to the new error-resilient implementations.

## Phase 1: Immediate Deprecations (Can be removed after migration)

### 1. Unsafe Priority Detection
**Files to deprecate:**
- `src/program/priority_event_processor.rs` - Lines 273-276 (unsafe transmute for priority detection)
- `src/program/event_processor.rs` - Line 98 (unsafe transmute_copy)

**Replacement:**
- Use `src/safe_priority.rs` module instead
- `SafePriorityMapper` trait and implementations

**Migration steps:**
```rust
// Old (DEPRECATED):
let event_ref = unsafe { std::mem::transmute::<&Event<M>, &Event<()>>(event) };

// New:
use crate::safe_priority::detect_priority;
let priority = detect_priority(event);
```

### 2. Panic-prone Mutex Operations
**Files to deprecate:**
- All `.lock().unwrap()` calls in:
  - `src/program/priority_event_processor.rs` (11 occurrences)
  - `src/program/command_executor.rs` (Default impl, line 160)

**Replacement:**
- Use `src/safe_mutex.rs` module
- `SafeMutex` and `SafeArcMutex` types

**Migration steps:**
```rust
// Old (DEPRECATED):
use std::sync::{Arc, Mutex};
let stats = Arc::new(Mutex::new(EventStats::default()));
let data = self.stats.lock().unwrap().clone();

// New:
use crate::safe_mutex::{SafeMutex, safe_arc_mutex};
let stats = safe_arc_mutex(EventStats::default());
let data = self.stats.lock().clone();  // Automatically recovers from poison
```

### 3. Non-resilient Input Thread
**Files to deprecate:**
- `src/program.rs` - Lines 799-810 (basic input thread without panic recovery)

**Replacement:**
- Use `src/resilient_input.rs` module
- `spawn_resilient_input_thread` function

**Migration steps:**
```rust
// Old (DEPRECATED):
let input_thread = thread::spawn(move || loop {
    if event::poll(Duration::from_millis(100)).unwrap_or(false) {
        if let Ok(event) = event::read() {
            let _ = crossterm_tx.send(event);
        }
    }
});

// New:
use crate::resilient_input::spawn_resilient_input_thread;
let input_thread = spawn_resilient_input_thread(running, force_quit, crossterm_tx);
```

## Phase 2: Gradual Deprecations (Keep during transition)

### 1. Channel .expect() Calls
**Files with issues:**
- `src/program.rs` - Lines 492, 559, 692, 727, 785

**Current workaround:**
- Keep for now but wrap in error handling
- Eventually replace with proper Result propagation

### 2. Runtime Creation Panics
**Files with issues:**
- `src/program/command_executor.rs` - Default implementation

**Current workaround:**
- Keep but add fallback behavior or better error messages

## Phase 3: Optional Enhancements

### 1. Global Panic Handler
**When to enable:**
- In `Program::new()` or `Program::with_options()`
- Call `panic_handler::install()` at program start

**Benefits:**
- Terminal always restored on panic
- Panic information logged properly
- Clean shutdown even on unexpected errors

### 2. Event Priority Customization
**New capability:**
- Users can now provide custom priority mappers
- Type-safe, no unsafe code required

## Migration Checklist

- [ ] Update `priority_event_processor.rs` to use `SafeMutex`
- [ ] Replace all unsafe transmutes with `safe_priority` functions
- [ ] Switch input thread to `spawn_resilient_input_thread`
- [ ] Add panic handler installation to Program initialization
- [ ] Update documentation with new error handling patterns
- [ ] Add tests for panic recovery scenarios
- [ ] Benchmark performance impact of safety changes

## Files to Delete After Full Migration

1. `/src/program_debug.rs` - Temporary debug version
2. Any remaining unsafe transmute operations
3. Old mutex patterns without recovery

## Performance Considerations

The new safe implementations may have slight performance impacts:
- `SafeMutex`: Minimal overhead (just error handling)
- `safe_priority`: No performance impact (same logic, no unsafe)
- `resilient_input`: Small overhead from panic catching
- `panic_handler`: One-time setup cost, no runtime overhead

## Testing Requirements

Before removing deprecated code:
1. Test panic recovery in input thread
2. Test mutex poison recovery
3. Test priority detection with various event types
4. Test terminal restoration on panic
5. Run all existing tests to ensure compatibility

## API Compatibility

All changes maintain backward compatibility:
- New modules are additions, not replacements
- Existing public APIs unchanged
- Migration can be gradual

## Timeline

1. **Immediate**: Add new resilient modules
2. **Next PR**: Update internal usage to new modules
3. **After testing**: Mark old code as deprecated
4. **Next major version**: Remove deprecated code