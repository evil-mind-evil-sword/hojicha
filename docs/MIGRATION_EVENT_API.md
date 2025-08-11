# Event API Migration Guide

This guide helps you migrate to the improved Event API introduced in version 0.3.0, which provides more consistent and ergonomic event handling patterns.

## Overview

The Event API improvements add helper methods to make event handling more consistent and less verbose, while maintaining full backward compatibility.

## Key Improvements

### 1. Event Helper Methods

The `Event<M>` enum now provides direct helper methods for common checks:

#### Before:
```rust
match event {
    Event::Key(key) if key.key == Key::Enter => {
        // Handle enter key
    }
    Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) => {
        // Handle left click
    }
    Event::Resize { width, height } => {
        // Handle resize
    }
    _ => {}
}
```

#### After:
```rust
// Check for specific keys
if event.is_key_press(Key::Enter) {
    // Handle enter key
}

// Check for mouse clicks with position
if let Some((x, y)) = event.as_click() {
    // Handle click at position (x, y)
}

// Check for resize with dimensions
if let Some((width, height)) = event.as_resize() {
    // Handle resize
}
```

### 2. KeyEvent Helper Methods

KeyEvent now provides intuitive methods for checking keys and modifiers:

#### Before:
```rust
if key.key == Key::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
    // Handle Ctrl+S
}

if matches!(key.key, Key::Up | Key::Down | Key::Left | Key::Right) {
    // Handle arrow keys
}
```

#### After:
```rust
// Check for key combinations
if key.is_with_modifiers(Key::Char('s'), KeyModifiers::CONTROL) {
    // Handle Ctrl+S
}

// Check for navigation keys
if key.is_navigation() {
    // Handle arrow keys, Home, End, PageUp, PageDown
}

// Check modifiers directly
if key.is_ctrl() {
    // Key pressed with Ctrl
}
```

### 3. MouseEvent Helper Methods

MouseEvent provides rich methods for mouse interaction:

#### Before:
```rust
if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
    let x = mouse.column;
    let y = mouse.row;
    // Handle click
}

if matches!(mouse.kind, MouseEventKind::Drag(_)) {
    // Handle drag
}
```

#### After:
```rust
// Check specific mouse buttons
if mouse.is_left_click() {
    let (x, y) = mouse.position();
    // Handle click
}

// Check drag events
if mouse.is_drag() {
    if let Some(button) = mouse.button() {
        // Handle drag with specific button
    }
}

// Check if click is within area
if mouse.is_click() && mouse.is_within(10, 10, 20, 20) {
    // Click within rectangle
}
```

## Complete Method Reference

### Event<M> Methods

| Method | Description |
|--------|-------------|
| `is_key()` | Check if this is a key event |
| `is_key_press(key)` | Check for specific key press |
| `is_key_with_modifiers(key, mods)` | Check for key with modifiers |
| `as_key()` | Get KeyEvent reference |
| `is_mouse()` | Check if this is a mouse event |
| `as_mouse()` | Get MouseEvent reference |
| `is_click()` | Check if this is any mouse click |
| `as_click()` | Get click position |
| `is_resize()` | Check if this is a resize event |
| `as_resize()` | Get resize dimensions |
| `is_user()` | Check if this is a user message |
| `as_user()` | Get user message reference |
| `into_user()` | Take user message ownership |
| `is_quit()` | Check if this is a quit event |
| `is_tick()` | Check if this is a tick event |
| `is_paste()` | Check if this is a paste event |
| `as_paste()` | Get pasted text |
| `is_focus()` | Check if terminal gained focus |
| `is_blur()` | Check if terminal lost focus |

### KeyEvent Methods

| Method | Description |
|--------|-------------|
| `is(key)` | Check if matches specific key |
| `is_with_modifiers(key, mods)` | Check key with modifiers |
| `is_ctrl()` | Check if Ctrl is held |
| `is_alt()` | Check if Alt is held |
| `is_shift()` | Check if Shift is held |
| `is_super()` | Check if Super/Meta is held |
| `is_navigation()` | Check if navigation key |
| `is_function_key()` | Check if function key (F1-F24) |
| `is_media_key()` | Check if media control key |
| `has_modifiers()` | Check if any modifiers held |
| `no_modifiers()` | Check if no modifiers held |

### MouseEvent Methods

| Method | Description |
|--------|-------------|
| `is_left_click()` | Check for left button click |
| `is_right_click()` | Check for right button click |
| `is_middle_click()` | Check for middle button click |
| `is_click()` | Check for any button click |
| `is_release()` | Check for button release |
| `is_drag()` | Check for drag event |
| `is_left_drag()` | Check for left button drag |
| `is_scroll_up()` | Check for scroll up |
| `is_scroll_down()` | Check for scroll down |
| `is_scroll()` | Check for any scroll |
| `is_move()` | Check for mouse move |
| `button()` | Get button involved in event |
| `position()` | Get (column, row) tuple |
| `is_within(x, y, w, h)` | Check if within rectangle |
| `is_at(col, row)` | Check if at specific position |
| `is_ctrl()` | Check if Ctrl held during event |
| `has_modifiers()` | Check if any modifiers held |
| `no_modifiers()` | Check if no modifiers held |

## Migration Examples

### Example 1: Quit Handling

**Before:**
```rust
match event {
    Event::Key(key) => {
        if key.key == Key::Char('q') && key.modifiers.is_empty() {
            return commands::quit();
        }
        if key.key == Key::Esc {
            return commands::quit();
        }
    }
    Event::Quit => return commands::quit(),
    _ => {}
}
```

**After:**
```rust
// Much cleaner with helper methods
if event.is_quit() || 
   event.is_key_press(Key::Esc) || 
   event.is_key_press(Key::Char('q')) {
    return commands::quit();
}
```

### Example 2: Mouse Interaction

**Before:**
```rust
match event {
    Event::Mouse(mouse) => {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if mouse.column >= 10 && mouse.column < 30 &&
                   mouse.row >= 5 && mouse.row < 15 {
                    // Handle button click
                }
            }
            MouseEventKind::ScrollUp => {
                // Handle scroll
            }
            _ => {}
        }
    }
    _ => {}
}
```

**After:**
```rust
if let Some(mouse) = event.as_mouse() {
    if mouse.is_left_click() && mouse.is_within(10, 5, 20, 10) {
        // Handle button click
    }
    
    if mouse.is_scroll_up() {
        // Handle scroll
    }
}
```

### Example 3: Keyboard Shortcuts

**Before:**
```rust
match event {
    Event::Key(key) => {
        if key.key == Key::Char('s') && 
           key.modifiers == KeyModifiers::CONTROL {
            // Save
        }
        if key.key == Key::Char('z') && 
           key.modifiers == KeyModifiers::CONTROL {
            // Undo
        }
        if key.key == Key::Char('c') && 
           key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) {
            // Copy with formatting
        }
    }
    _ => {}
}
```

**After:**
```rust
if let Some(key) = event.as_key() {
    if key.is_with_modifiers(Key::Char('s'), KeyModifiers::CONTROL) {
        // Save
    }
    if key.is_with_modifiers(Key::Char('z'), KeyModifiers::CONTROL) {
        // Undo
    }
    if key.is_with_modifiers(Key::Char('c'), 
                             KeyModifiers::CONTROL | KeyModifiers::SHIFT) {
        // Copy with formatting
    }
}
```

## Best Practices

1. **Use specific helper methods** instead of pattern matching when checking for single conditions
2. **Combine helpers** for complex conditions: `if event.is_key_press(Key::Enter) || event.is_click()`
3. **Use `as_*` methods** when you need the inner value
4. **Prefer `no_modifiers()`** over `modifiers.is_empty()` for clarity
5. **Use category checks** like `is_navigation()` for groups of related keys

## Backward Compatibility

All existing code continues to work. The new helper methods are additions, not replacements. You can migrate gradually, updating event handling code as you work on different parts of your application.

## Performance

The helper methods are all inline functions that compile to the same machine code as manual pattern matching. There is no runtime overhead.