# Hojicha Framework - Comprehensive Development TODO

## Overview
This document contains detailed implementation tasks to achieve feature parity with Bubbletea and add Lipgloss-style styling capabilities. Each task includes implementation details, code examples, and required tests.

**Estimated Total Effort**: 8-12 weeks  
**Priority Levels**: 🔴 Critical | 🟡 Important | 🟢 Nice-to-have

---

## Phase 1: Quick Wins (Week 1)
These can be implemented immediately with minimal architectural changes.

### 1.1 🔴 Add Printf/Println Methods for Debug Output
**File**: `src/program.rs`  
**Effort**: 2 hours

**Implementation Details**:
```rust
impl<M: Model> Program<M> {
    /// Print formatted text to stderr (appears above the TUI)
    /// This is useful for debugging without disrupting the UI
    pub fn printf(&self, args: fmt::Arguments) {
        // Must temporarily release terminal, print, then restore
        let _ = self.release_terminal();
        eprint!("{}", args);
        let _ = self.restore_terminal();
    }
    
    /// Print a line to stderr (appears above the TUI)
    pub fn println(&self, text: &str) {
        let _ = self.release_terminal();
        eprintln!("{}", text);
        let _ = self.restore_terminal();
    }
}
```

**Required Tests**:
```rust
#[test]
fn test_printf_output() {
    // Capture stderr and verify output appears
    // Ensure terminal is properly restored after print
}

#[test]
fn test_println_during_render() {
    // Verify println doesn't corrupt the UI
    // Check that cursor position is restored
}

#[test]
fn test_printf_with_format_args() {
    // Test with various format strings
    // Verify proper escaping and formatting
}
```

**Acceptance Criteria**:
- [ ] Methods available on Program struct
- [ ] Output goes to stderr, not stdout
- [ ] Terminal state preserved after output
- [ ] No UI corruption when called during render

---

### 1.2 🔴 Implement SetWindowTitle Command
**File**: `src/commands.rs`  
**Effort**: 1 hour

**Implementation Details**:
```rust
/// Set the terminal window title
/// 
/// # Example
/// ```
/// Some(commands::set_window_title("My App - Loading..."))
/// ```
pub fn set_window_title<M: Message>(title: impl Into<String>) -> Cmd<M> {
    let title = title.into();
    Cmd::new(move || {
        use crossterm::{execute, terminal::SetTitle};
        use std::io::stdout;
        
        // Some terminals don't support this, so ignore errors
        let _ = execute!(stdout(), SetTitle(title.as_str()));
        None
    })
}
```

**Required Tests**:
```rust
#[test]
fn test_set_window_title_command() {
    // Verify command creation doesn't panic
    // Mock stdout and verify escape sequence sent
}

#[test]
fn test_set_window_title_empty_string() {
    // Test with empty title
    // Should clear title or set to default
}

#[test]
fn test_set_window_title_unicode() {
    // Test with unicode characters (emoji, CJK, etc.)
    // Verify proper encoding
}
```

**Acceptance Criteria**:
- [ ] Command successfully sets terminal title
- [ ] Works with Unicode strings
- [ ] Gracefully handles unsupported terminals
- [ ] Example in documentation works

---

### 1.3 🔴 Add FPS Control to ProgramOptions
**Files**: `src/program.rs`, `src/lib.rs`  
**Effort**: 3 hours

**Implementation Details**:
```rust
// In ProgramOptions
pub struct ProgramOptions {
    // ... existing fields
    pub target_fps: u16, // Default: 60, Max: 120
}

impl ProgramOptions {
    pub fn with_fps(mut self, fps: u16) -> Self {
        self.target_fps = fps.clamp(1, 120);
        self
    }
}

// In Program::run() - modify the render loop
let frame_duration = Duration::from_millis(1000 / options.target_fps as u64);
let mut last_frame = Instant::now();

loop {
    // ... event processing
    
    let now = Instant::now();
    if now.duration_since(last_frame) >= frame_duration {
        // Render frame
        self.render()?;
        last_frame = now;
    }
    
    // Sleep to prevent CPU spinning
    thread::sleep(Duration::from_millis(1));
}
```

**Required Tests**:
```rust
#[test]
fn test_fps_control_default() {
    // Verify default is 60 FPS
    // Measure actual frame timing
}

#[test]
fn test_fps_control_custom() {
    // Test with 30, 60, 120 FPS
    // Verify frame timing matches target
}

#[test]
fn test_fps_control_clamping() {
    // Test with 0, 1000 FPS
    // Verify clamped to valid range (1-120)
}

#[test]
fn test_fps_cpu_usage() {
    // Verify CPU usage is reasonable
    // Should not spin at 100% CPU
}
```

**Acceptance Criteria**:
- [ ] FPS can be configured via ProgramOptions
- [ ] Default is 60 FPS
- [ ] Maximum capped at 120 FPS
- [ ] CPU usage remains low
- [ ] Smooth rendering at target FPS

---

### 1.4 🔴 Implement Message Filtering (WithFilter)
**File**: `src/program.rs`  
**Effort**: 2 hours

**Implementation Details**:
```rust
pub struct Program<M: Model> {
    // ... existing fields
    filter: Option<Box<dyn Fn(&M, Event<M::Message>) -> Option<Event<M::Message>>>>,
}

impl<M: Model> Program<M> {
    /// Add a filter function to transform or block events
    /// 
    /// # Example
    /// ```
    /// program.with_filter(|model, event| {
    ///     // Block all mouse events when menu is closed
    ///     match event {
    ///         Event::Mouse(_) if !model.menu_open => None,
    ///         _ => Some(event),
    ///     }
    /// })
    /// ```
    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&M, Event<M::Message>) -> Option<Event<M::Message>> + 'static,
    {
        self.filter = Some(Box::new(filter));
        self
    }
}

// In event processing loop
let event = if let Some(ref filter) = self.filter {
    filter(&self.model, event)?
} else {
    event
};
```

**Required Tests**:
```rust
#[test]
fn test_filter_blocks_events() {
    // Filter that blocks all mouse events
    // Verify mouse events don't reach model
}

#[test]
fn test_filter_transforms_events() {
    // Filter that transforms key events
    // Verify transformation applied correctly
}

#[test]
fn test_filter_with_model_state() {
    // Filter that uses model state to decide
    // Verify correct behavior based on state
}

#[test]
fn test_multiple_filters() {
    // Chain multiple filters
    // Verify composition works correctly
}
```

**Acceptance Criteria**:
- [ ] Filter function receives model reference and event
- [ ] Can block events by returning None
- [ ] Can transform events by returning modified event
- [ ] Filter is called before model.update()
- [ ] No performance regression

---

## Phase 2: Core Enhancements (Week 2-3)

### 2.1 🔴 Add Context Support for Cancellation
**Files**: `src/program.rs`, `Cargo.toml`  
**Effort**: 4 hours

**Implementation Details**:
```rust
// Add to Cargo.toml
[dependencies]
tokio = { version = "1", features = ["sync", "time"] }

// In Program struct
pub struct Program<M: Model> {
    // ... existing fields
    cancel_token: Option<tokio_util::sync::CancellationToken>,
}

impl<M: Model> Program<M> {
    /// Run with cancellation support
    pub fn run_with_cancel(mut self, token: CancellationToken) -> Result<()> {
        // In event loop
        loop {
            if token.is_cancelled() {
                break;
            }
            
            // Use tokio::select! for cancellable operations
            tokio::select! {
                _ = token.cancelled() => break,
                event = self.next_event() => {
                    // Process event
                }
            }
        }
        Ok(())
    }
}
```

**Required Tests**:
```rust
#[test]
fn test_cancellation_stops_program() {
    // Create program with cancellation token
    // Cancel after 100ms
    // Verify program stops cleanly
}

#[test]
fn test_cancellation_cleanup() {
    // Verify terminal restored on cancellation
    // Check all resources freed
}

#[test]
fn test_cancellation_during_command() {
    // Cancel while command is executing
    // Verify command interrupted properly
}

#[test]
fn test_timeout_cancellation() {
    // Use timeout as cancellation
    // Verify program stops at timeout
}
```

**Acceptance Criteria**:
- [ ] Program can be cancelled via token
- [ ] Cancellation is checked in event loop
- [ ] Long-running commands are interruptible
- [ ] Terminal properly restored on cancel
- [ ] Compatible with async/await

---

### 2.2 🔴 Comprehensive Signal Handling
**Files**: `src/program.rs`, `Cargo.toml`  
**Effort**: 5 hours

**Implementation Details**:
```rust
// Add to Cargo.toml
[dependencies]
signal-hook = "0.3"

// Signal handler setup
use signal_hook::{consts::*, iterator::Signals};

impl<M: Model> Program<M> {
    fn setup_signal_handlers(&self) -> Result<()> {
        let mut signals = Signals::new(&[
            SIGINT,   // Ctrl+C
            SIGTERM,  // Termination request
            SIGWINCH, // Window resize
            SIGTSTP,  // Ctrl+Z (suspend)
            SIGCONT,  // Continue after suspend
            SIGHUP,   // Terminal hangup
        ])?;
        
        let event_tx = self.event_tx.clone();
        
        thread::spawn(move || {
            for sig in signals.forever() {
                let event = match sig {
                    SIGINT => Event::Interrupt,
                    SIGTERM => Event::Terminate,
                    SIGWINCH => {
                        let (width, height) = terminal::size().unwrap_or((80, 24));
                        Event::Resize { width, height }
                    }
                    SIGTSTP => Event::Suspend,
                    SIGCONT => Event::Resume,
                    SIGHUP => Event::Hangup,
                    _ => continue,
                };
                let _ = event_tx.send(event);
            }
        });
        
        Ok(())
    }
}

// Add new event variants
pub enum Event<M> {
    // ... existing variants
    Interrupt,    // SIGINT (Ctrl+C)
    Terminate,    // SIGTERM
    Hangup,       // SIGHUP
}
```

**Required Tests**:
```rust
#[test]
fn test_sigint_handling() {
    // Send SIGINT to process
    // Verify Interrupt event generated
    // Check graceful shutdown
}

#[test]
fn test_sigterm_handling() {
    // Send SIGTERM
    // Verify clean termination
    // Check cleanup performed
}

#[test]
fn test_sigwinch_resize() {
    // Send SIGWINCH
    // Verify resize event with correct dimensions
}

#[test]
fn test_sigtstp_suspend() {
    // Send SIGTSTP
    // Verify program suspends
    // Send SIGCONT
    // Verify program resumes
}

#[test]
fn test_signal_during_render() {
    // Send signal while rendering
    // Verify no corruption
}
```

**Acceptance Criteria**:
- [ ] All common signals handled
- [ ] Proper cleanup on termination signals
- [ ] Resize events generated correctly
- [ ] Suspend/resume cycle works
- [ ] No terminal corruption

---

### 2.3 🔴 Implement Headless/Nil Renderer
**Files**: `src/renderer.rs` (new), `src/program.rs`  
**Effort**: 4 hours

**Implementation Details**:
```rust
// src/renderer.rs
pub trait Renderer: Send {
    fn render(&mut self, frame: &Frame) -> Result<()>;
    fn clear(&mut self) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
    fn hide_cursor(&mut self) -> Result<()>;
    fn show_cursor(&mut self) -> Result<()>;
}

/// Standard renderer that outputs to terminal
pub struct TerminalRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

/// Nil renderer for headless operation
pub struct NilRenderer {
    output: Option<Vec<String>>, // Optional buffer for testing
}

impl Renderer for NilRenderer {
    fn render(&mut self, frame: &Frame) -> Result<()> {
        if let Some(ref mut output) = self.output {
            // Capture render for testing
            output.push(frame.to_string());
        }
        Ok(())
    }
    
    fn clear(&mut self) -> Result<()> { Ok(()) }
    fn flush(&mut self) -> Result<()> { Ok(()) }
    fn hide_cursor(&mut self) -> Result<()> { Ok(()) }
    fn show_cursor(&mut self) -> Result<()> { Ok(()) }
}

// In ProgramOptions
impl ProgramOptions {
    pub fn with_renderer(mut self, renderer: Box<dyn Renderer>) -> Self {
        self.renderer = Some(renderer);
        self
    }
    
    pub fn headless(self) -> Self {
        self.with_renderer(Box::new(NilRenderer::default()))
    }
}
```

**Required Tests**:
```rust
#[test]
fn test_nil_renderer_no_output() {
    // Run program with nil renderer
    // Verify no terminal output
}

#[test]
fn test_nil_renderer_capture() {
    // Use nil renderer with capture
    // Verify renders are captured
}

#[test]
fn test_headless_mode() {
    // Run program in headless mode
    // Verify events still processed
    // Check model updates occur
}

#[test]
fn test_renderer_switching() {
    // Start with nil, switch to terminal
    // Verify smooth transition
}
```

**Acceptance Criteria**:
- [ ] Renderer trait abstracts rendering
- [ ] NilRenderer produces no output
- [ ] Headless mode works for testing
- [ ] Can capture renders for verification
- [ ] No performance impact

---

### 2.4 🟡 Enhanced Keyboard Support
**File**: `src/event.rs`  
**Effort**: 6 hours

**Implementation Details**:
```rust
// Extend Key enum with missing keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    // ... existing variants
    
    // Lock keys
    CapsLock,
    ScrollLock,
    NumLock,
    
    // Media keys
    MediaPlay,
    MediaPause,
    MediaPlayPause,
    MediaStop,
    MediaNext,
    MediaPrevious,
    MediaRewind,
    MediaFastForward,
    VolumeUp,
    VolumeDown,
    VolumeMute,
    
    // Browser keys
    BrowserBack,
    BrowserForward,
    BrowserRefresh,
    BrowserStop,
    BrowserSearch,
    BrowserFavorites,
    BrowserHome,
    
    // Additional function keys
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
    
    // Keypad
    KeypadEnter,
    KeypadEquals,
    KeypadMultiply,
    KeypadPlus,
    KeypadMinus,
    KeypadDivide,
    KeypadDecimal,
    Keypad0, Keypad1, Keypad2, Keypad3, Keypad4,
    Keypad5, Keypad6, Keypad7, Keypad8, Keypad9,
    
    // System keys
    PrintScreen,
    Pause,
    Menu,
    Help,
    Select,
    Execute,
    Find,
    Again,
    Undo,
    Cut,
    Copy,
    Paste,
}

// Parse extended escape sequences
impl KeyEvent {
    pub fn from_escape_sequence(seq: &[u8]) -> Option<Self> {
        match seq {
            // CSI sequences for special keys
            b"\x1b[1;5A" => Some(KeyEvent::new(Key::Up, KeyModifiers::CONTROL)),
            b"\x1b[1;2B" => Some(KeyEvent::new(Key::Down, KeyModifiers::SHIFT)),
            // ... map all sequences
            _ => None,
        }
    }
}
```

**Required Tests**:
```rust
#[test]
fn test_media_keys() {
    // Test all media key variants
    // Verify correct parsing from sequences
}

#[test]
fn test_keypad_keys() {
    // Test keypad with NumLock on/off
    // Verify correct interpretation
}

#[test]
fn test_function_keys_extended() {
    // Test F13-F24
    // Verify terminal support detection
}

#[test]
fn test_escape_sequence_parsing() {
    // Test various escape sequences
    // Including malformed sequences
}

#[test]
fn test_modifier_combinations() {
    // Test Ctrl+Shift+Key combinations
    // Verify all modifiers work
}
```

**Acceptance Criteria**:
- [ ] All common special keys supported
- [ ] Media keys work where available
- [ ] Keypad properly distinguished
- [ ] Extended function keys (F13-F24)
- [ ] Robust escape sequence parsing

---

## Phase 3: Style System (Week 4-6)

### 3.1 🔴 Create hojicha-style Crate Structure
**Files**: New crate `hojicha-style/`  
**Effort**: 2 hours

**Implementation Details**:
```toml
# hojicha-style/Cargo.toml
[package]
name = "hojicha-style"
version = "0.1.0"
edition = "2021"
authors = ["Your Name"]
description = "Lipgloss-inspired styling for Hojicha TUI framework"
license = "MIT"

[dependencies]
unicode-width = "0.1"
unicode-segmentation = "1.10"
termcolor = "1.2"
rgb = "0.8"
csscolorparser = "0.6"

[dev-dependencies]
pretty_assertions = "1.3"
proptest = "1.0"
```

**Directory Structure**:
```
hojicha-style/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── style.rs      # Core Style struct
│   ├── color.rs      # Color system
│   ├── border.rs     # Border definitions
│   ├── padding.rs    # Padding/margin
│   ├── align.rs      # Alignment utilities
│   ├── layout.rs     # Layout functions
│   ├── render.rs     # Rendering engine
│   └── utils.rs      # Helper functions
├── tests/
│   └── integration.rs
└── examples/
    ├── basic.rs
    └── advanced.rs
```

**Required Tests**:
```rust
#[test]
fn test_crate_compilation() {
    // Verify crate compiles
    // Check all modules accessible
}

#[test]
fn test_public_api() {
    // Verify public API exports
    // Check documentation builds
}
```

---

### 3.2 🔴 Implement Core Style Struct with Fluent API
**File**: `hojicha-style/src/style.rs`  
**Effort**: 8 hours

**Implementation Details**:
```rust
use crate::{Color, Border, Padding, Margin, Alignment};

#[derive(Clone, Debug, Default)]
pub struct Style {
    // Colors
    foreground: Option<Color>,
    background: Option<Color>,
    
    // Text attributes
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strikethrough: Option<bool>,
    dim: Option<bool>,
    blink: Option<bool>,
    reverse: Option<bool>,
    hidden: Option<bool>,
    
    // Spacing
    padding: Padding,
    margin: Margin,
    
    // Border
    border: Option<Border>,
    border_style: Option<Box<Style>>, // Style for border itself
    border_top: bool,
    border_right: bool,
    border_bottom: bool,
    border_left: bool,
    
    // Dimensions
    width: Option<u16>,
    height: Option<u16>,
    max_width: Option<u16>,
    max_height: Option<u16>,
    
    // Alignment
    align_horizontal: Option<Alignment>,
    align_vertical: Option<Alignment>,
    
    // Advanced
    inline: bool,
    inherit_from: Option<Box<Style>>,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }
    
    // Fluent API - Colors
    pub fn fg(mut self, color: impl Into<Color>) -> Self {
        self.foreground = Some(color.into());
        self
    }
    
    pub fn bg(mut self, color: impl Into<Color>) -> Self {
        self.background = Some(color.into());
        self
    }
    
    // Fluent API - Text attributes
    pub fn bold(mut self, v: bool) -> Self {
        self.bold = Some(v);
        self
    }
    
    pub fn italic(mut self, v: bool) -> Self {
        self.italic = Some(v);
        self
    }
    
    // Fluent API - Spacing
    pub fn padding(mut self, top: u16, right: u16, bottom: u16, left: u16) -> Self {
        self.padding = Padding::new(top, right, bottom, left);
        self
    }
    
    pub fn margin(mut self, top: u16, right: u16, bottom: u16, left: u16) -> Self {
        self.margin = Margin::new(top, right, bottom, left);
        self
    }
    
    // Fluent API - Borders
    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self.border_top = true;
        self.border_right = true;
        self.border_bottom = true;
        self.border_left = true;
        self
    }
    
    pub fn border_top(mut self, v: bool) -> Self {
        self.border_top = v;
        self
    }
    
    // Rendering
    pub fn render(&self, content: &str) -> String {
        let mut result = String::new();
        
        // Apply padding
        let content = self.apply_padding(content);
        
        // Apply border
        let content = self.apply_border(content);
        
        // Apply margin
        let content = self.apply_margin(content);
        
        // Apply text attributes and colors
        self.apply_ansi_codes(&mut result);
        result.push_str(&content);
        self.apply_reset_codes(&mut result);
        
        result
    }
    
    // Inheritance
    pub fn inherit(mut self, parent: Style) -> Self {
        self.inherit_from = Some(Box::new(parent));
        self
    }
    
    // Get computed value (with inheritance)
    fn get_foreground(&self) -> Option<Color> {
        self.foreground.or_else(|| {
            self.inherit_from.as_ref().and_then(|p| p.get_foreground())
        })
    }
}
```

**Required Tests**:
```rust
#[test]
fn test_style_fluent_api() {
    let style = Style::new()
        .fg(Color::Red)
        .bg(Color::Blue)
        .bold(true)
        .padding(1, 2, 1, 2);
    
    // Verify all properties set correctly
}

#[test]
fn test_style_rendering() {
    let style = Style::new().fg(Color::Red).bold(true);
    let output = style.render("Hello");
    
    // Verify ANSI codes present
    // Check text is wrapped correctly
}

#[test]
fn test_style_inheritance() {
    let parent = Style::new().fg(Color::Red);
    let child = Style::new().bg(Color::Blue).inherit(parent);
    
    // Child should have red foreground from parent
    // And blue background from itself
}

#[test]
fn test_style_border_rendering() {
    let style = Style::new().border(Border::rounded());
    let output = style.render("Test");
    
    // Verify border characters present
    // Check corners are rounded
}

#[test]
fn test_style_dimension_constraints() {
    let style = Style::new().width(10).height(3);
    let output = style.render("Very long text that should wrap");
    
    // Verify text wrapped to 10 chars
    // Verify exactly 3 lines
}
```

**Acceptance Criteria**:
- [ ] Fluent API for all properties
- [ ] Immutable operations (returns new Style)
- [ ] Style inheritance works
- [ ] Efficient rendering
- [ ] ANSI code generation correct

---

### 3.3 🔴 Implement Advanced Color System
**File**: `hojicha-style/src/color.rs`  
**Effort**: 6 hours

**Implementation Details**:
```rust
use csscolorparser::Color as CssColor;

#[derive(Clone, Debug)]
pub enum Color {
    /// RGB color (0-255 for each component)
    RGB(u8, u8, u8),
    
    /// Hex color string (#RRGGBB or #RGB)
    Hex(String),
    
    /// ANSI 256 color (0-255)
    ANSI256(u8),
    
    /// Basic ANSI color (0-15)
    ANSI(u8),
    
    /// Named color (e.g., "red", "blue")
    Named(String),
    
    /// Adaptive color that changes based on terminal background
    Adaptive {
        light: Box<Color>,
        dark: Box<Color>,
    },
    
    /// Complete color with fallbacks for different terminal capabilities
    Complete {
        true_color: Option<(u8, u8, u8)>,
        ansi256: Option<u8>,
        ansi: Option<u8>,
    },
}

impl Color {
    /// Parse color from string (hex, rgb(), name, etc.)
    pub fn parse(s: &str) -> Result<Self, ColorError> {
        // Try hex format
        if s.starts_with('#') {
            return Ok(Color::Hex(s.to_string()));
        }
        
        // Try rgb() format
        if s.starts_with("rgb") {
            let css_color = CssColor::from_html(s)?;
            return Ok(Color::RGB(
                (css_color.r * 255.0) as u8,
                (css_color.g * 255.0) as u8,
                (css_color.b * 255.0) as u8,
            ));
        }
        
        // Try named color
        if let Ok(css_color) = CssColor::from_html(s) {
            return Ok(Color::RGB(
                (css_color.r * 255.0) as u8,
                (css_color.g * 255.0) as u8,
                (css_color.b * 255.0) as u8,
            ));
        }
        
        // Try ANSI number
        if let Ok(n) = s.parse::<u8>() {
            return Ok(Color::ANSI256(n));
        }
        
        Err(ColorError::InvalidColor(s.to_string()))
    }
    
    /// Convert to RGB values
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::RGB(r, g, b) => (*r, *g, *b),
            Color::Hex(hex) => Self::hex_to_rgb(hex),
            Color::ANSI256(n) => Self::ansi256_to_rgb(*n),
            Color::ANSI(n) => Self::ansi_to_rgb(*n),
            Color::Named(name) => Self::named_to_rgb(name),
            Color::Adaptive { dark, .. } => dark.to_rgb(), // Default to dark
            Color::Complete { true_color, .. } => {
                true_color.unwrap_or((0, 0, 0))
            }
        }
    }
    
    /// Generate ANSI escape sequence for foreground
    pub fn fg_sequence(&self, profile: ColorProfile) -> String {
        match profile {
            ColorProfile::TrueColor => {
                let (r, g, b) = self.to_rgb();
                format!("\x1b[38;2;{};{};{}m", r, g, b)
            }
            ColorProfile::ANSI256 => {
                let n = self.to_ansi256();
                format!("\x1b[38;5;{}m", n)
            }
            ColorProfile::ANSI => {
                let n = self.to_ansi();
                if n < 8 {
                    format!("\x1b[3{}m", n)
                } else {
                    format!("\x1b[9{}m", n - 8)
                }
            }
        }
    }
    
    /// Detect terminal color profile
    pub fn detect_profile() -> ColorProfile {
        if std::env::var("COLORTERM").unwrap_or_default().contains("truecolor") {
            ColorProfile::TrueColor
        } else if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") {
                ColorProfile::ANSI256
            } else {
                ColorProfile::ANSI
            }
        } else {
            ColorProfile::ANSI
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColorProfile {
    TrueColor,  // 24-bit color
    ANSI256,    // 256 colors
    ANSI,       // 16 colors
}

// Convenience functions
pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::RGB(r, g, b)
}

pub fn hex(s: &str) -> Color {
    Color::Hex(s.to_string())
}

pub fn adaptive(light: Color, dark: Color) -> Color {
    Color::Adaptive {
        light: Box::new(light),
        dark: Box::new(dark),
    }
}
```

**Required Tests**:
```rust
#[test]
fn test_color_parsing() {
    assert_eq!(Color::parse("#FF0000").unwrap(), Color::Hex("#FF0000".into()));
    assert_eq!(Color::parse("rgb(255, 0, 0)").unwrap(), Color::RGB(255, 0, 0));
    assert_eq!(Color::parse("red").unwrap(), Color::Named("red".into()));
    assert_eq!(Color::parse("196").unwrap(), Color::ANSI256(196));
}

#[test]
fn test_color_conversion() {
    let red = Color::RGB(255, 0, 0);
    assert_eq!(red.to_rgb(), (255, 0, 0));
    assert_eq!(red.to_ansi256(), 196);
    assert_eq!(red.to_ansi(), 9); // Bright red
}

#[test]
fn test_adaptive_color() {
    let color = adaptive(
        Color::RGB(0, 0, 0),    // Black on light background
        Color::RGB(255, 255, 255), // White on dark background
    );
    
    // Should pick based on terminal background
}

#[test]
fn test_color_profile_detection() {
    // Test with various TERM values
    std::env::set_var("TERM", "xterm-256color");
    assert_eq!(Color::detect_profile(), ColorProfile::ANSI256);
}

#[test]
fn test_ansi_sequence_generation() {
    let red = Color::RGB(255, 0, 0);
    assert_eq!(red.fg_sequence(ColorProfile::TrueColor), "\x1b[38;2;255;0;0m");
    assert_eq!(red.fg_sequence(ColorProfile::ANSI256), "\x1b[38;5;196m");
}
```

---

### 3.4 🔴 Implement Border System
**File**: `hojicha-style/src/border.rs`  
**Effort**: 4 hours

**Implementation Details**:
```rust
#[derive(Clone, Debug)]
pub struct Border {
    pub top: char,
    pub bottom: char,
    pub left: char,
    pub right: char,
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    
    // For tables and grids
    pub middle_left: Option<char>,
    pub middle_right: Option<char>,
    pub middle_top: Option<char>,
    pub middle_bottom: Option<char>,
    pub middle: Option<char>,
}

impl Border {
    /// Standard box-drawing border
    /// ```
    /// ┌─────┐
    /// │     │
    /// └─────┘
    /// ```
    pub fn normal() -> Self {
        Self {
            top: '─',
            bottom: '─',
            left: '│',
            right: '│',
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            middle_left: Some('├'),
            middle_right: Some('┤'),
            middle_top: Some('┬'),
            middle_bottom: Some('┴'),
            middle: Some('┼'),
        }
    }
    
    /// Rounded corners
    /// ```
    /// ╭─────╮
    /// │     │
    /// ╰─────╯
    /// ```
    pub fn rounded() -> Self {
        Self {
            top: '─',
            bottom: '─',
            left: '│',
            right: '│',
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            middle_left: Some('├'),
            middle_right: Some('┤'),
            middle_top: Some('┬'),
            middle_bottom: Some('┴'),
            middle: Some('┼'),
        }
    }
    
    /// Thick border
    /// ```
    /// ┏━━━━━┓
    /// ┃     ┃
    /// ┗━━━━━┛
    /// ```
    pub fn thick() -> Self {
        Self {
            top: '━',
            bottom: '━',
            left: '┃',
            right: '┃',
            top_left: '┏',
            top_right: '┓',
            bottom_left: '┗',
            bottom_right: '┛',
            middle_left: Some('┣'),
            middle_right: Some('┫'),
            middle_top: Some('┳'),
            middle_bottom: Some('┻'),
            middle: Some('╋'),
        }
    }
    
    /// Double border
    /// ```
    /// ╔═════╗
    /// ║     ║
    /// ╚═════╝
    /// ```
    pub fn double() -> Self {
        Self {
            top: '═',
            bottom: '═',
            left: '║',
            right: '║',
            top_left: '╔',
            top_right: '╗',
            bottom_left: '╚',
            bottom_right: '╝',
            middle_left: Some('╠'),
            middle_right: Some('╣'),
            middle_top: Some('╦'),
            middle_bottom: Some('╩'),
            middle: Some('╬'),
        }
    }
    
    /// ASCII-only border for compatibility
    /// ```
    /// +-----+
    /// |     |
    /// +-----+
    /// ```
    pub fn ascii() -> Self {
        Self {
            top: '-',
            bottom: '-',
            left: '|',
            right: '|',
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            middle_left: Some('+'),
            middle_right: Some('+'),
            middle_top: Some('+'),
            middle_bottom: Some('+'),
            middle: Some('+'),
        }
    }
    
    /// Hidden border (maintains spacing but invisible)
    pub fn hidden() -> Self {
        Self {
            top: ' ',
            bottom: ' ',
            left: ' ',
            right: ' ',
            top_left: ' ',
            top_right: ' ',
            bottom_left: ' ',
            bottom_right: ' ',
            middle_left: Some(' '),
            middle_right: Some(' '),
            middle_top: Some(' '),
            middle_bottom: Some(' '),
            middle: Some(' '),
        }
    }
    
    /// Custom border from string specification
    pub fn custom(spec: &str) -> Result<Self, BorderError> {
        // Parse format like "─│┌┐└┘" or with spaces "─ │ ┌ ┐ └ ┘"
        let chars: Vec<char> = spec.chars().filter(|c| !c.is_whitespace()).collect();
        
        if chars.len() < 6 {
            return Err(BorderError::InvalidSpec);
        }
        
        Ok(Self {
            top: chars[0],
            bottom: chars[0],
            left: chars[1],
            right: chars[1],
            top_left: chars[2],
            top_right: chars[3],
            bottom_left: chars[4],
            bottom_right: chars[5],
            middle_left: chars.get(6).copied(),
            middle_right: chars.get(7).copied(),
            middle_top: chars.get(8).copied(),
            middle_bottom: chars.get(9).copied(),
            middle: chars.get(10).copied(),
        })
    }
}
```

**Required Tests**:
```rust
#[test]
fn test_border_styles() {
    let normal = Border::normal();
    assert_eq!(normal.top_left, '┌');
    
    let rounded = Border::rounded();
    assert_eq!(rounded.top_left, '╭');
    
    let thick = Border::thick();
    assert_eq!(thick.top, '━');
}

#[test]
fn test_custom_border() {
    let border = Border::custom("─│┌┐└┘").unwrap();
    assert_eq!(border.top, '─');
    assert_eq!(border.left, '│');
}

#[test]
fn test_hidden_border() {
    let hidden = Border::hidden();
    assert!(hidden.top.is_whitespace());
}

#[test]
fn test_border_rendering() {
    // Test that borders render correctly around content
    let border = Border::normal();
    // Apply to "Hello" should produce:
    // ┌─────┐
    // │Hello│
    // └─────┘
}
```

---

### 3.5 🟡 Implement Layout Utilities
**File**: `hojicha-style/src/layout.rs`  
**Effort**: 6 hours

**Implementation Details**:
```rust
use unicode_width::UnicodeWidthStr;

/// Join strings horizontally with alignment
pub fn join_horizontal(align: Alignment, items: Vec<&str>) -> String {
    if items.is_empty() {
        return String::new();
    }
    
    // Calculate max height
    let max_height = items.iter()
        .map(|s| s.lines().count())
        .max()
        .unwrap_or(0);
    
    // Split each item into lines and pad to max height
    let mut item_lines: Vec<Vec<String>> = Vec::new();
    for item in items {
        let mut lines: Vec<String> = item.lines().map(String::from).collect();
        
        // Pad to max height based on alignment
        while lines.len() < max_height {
            match align {
                Alignment::Top => lines.push(String::new()),
                Alignment::Bottom => lines.insert(0, String::new()),
                Alignment::Center => {
                    if lines.len() % 2 == 0 {
                        lines.push(String::new());
                    } else {
                        lines.insert(0, String::new());
                    }
                }
                _ => lines.push(String::new()),
            }
        }
        
        item_lines.push(lines);
    }
    
    // Join lines horizontally
    let mut result = Vec::new();
    for line_idx in 0..max_height {
        let mut line = String::new();
        for item_idx in 0..item_lines.len() {
            line.push_str(&item_lines[item_idx][line_idx]);
        }
        result.push(line);
    }
    
    result.join("\n")
}

/// Join strings vertically with alignment
pub fn join_vertical(align: Alignment, items: Vec<&str>) -> String {
    if items.is_empty() {
        return String::new();
    }
    
    // Calculate max width
    let max_width = items.iter()
        .flat_map(|s| s.lines())
        .map(|line| UnicodeWidthStr::width(line))
        .max()
        .unwrap_or(0);
    
    let mut result = Vec::new();
    
    for item in items {
        for line in item.lines() {
            let width = UnicodeWidthStr::width(line);
            let padding = max_width.saturating_sub(width);
            
            let aligned_line = match align {
                Alignment::Left => format!("{}{}", line, " ".repeat(padding)),
                Alignment::Right => format!("{}{}", " ".repeat(padding), line),
                Alignment::Center => {
                    let left_pad = padding / 2;
                    let right_pad = padding - left_pad;
                    format!("{}{}{}", " ".repeat(left_pad), line, " ".repeat(right_pad))
                }
                _ => line.to_string(),
            };
            
            result.push(aligned_line);
        }
    }
    
    result.join("\n")
}

/// Place content at specific position in area
pub fn place(
    width: u16,
    height: u16,
    h_pos: Position,
    v_pos: Position,
    content: &str,
) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let content_height = lines.len() as u16;
    let content_width = lines.iter()
        .map(|l| UnicodeWidthStr::width(*l))
        .max()
        .unwrap_or(0) as u16;
    
    // Calculate offsets based on position
    let x_offset = h_pos.calculate(width, content_width);
    let y_offset = v_pos.calculate(height, content_height);
    
    let mut result = Vec::new();
    
    // Top padding
    for _ in 0..y_offset {
        result.push(" ".repeat(width as usize));
    }
    
    // Content lines
    for line in lines {
        let line_width = UnicodeWidthStr::width(line);
        let mut padded_line = " ".repeat(x_offset as usize);
        padded_line.push_str(line);
        
        // Pad to full width
        let remaining = width.saturating_sub(x_offset + line_width as u16);
        padded_line.push_str(&" ".repeat(remaining as usize));
        
        result.push(padded_line);
    }
    
    // Bottom padding
    let remaining_height = height.saturating_sub(y_offset + content_height);
    for _ in 0..remaining_height {
        result.push(" ".repeat(width as usize));
    }
    
    result.join("\n")
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Absolute(u16),     // Exact position
    Relative(f32),     // 0.0 to 1.0
    Start,             // 0
    Center,            // 0.5
    End,               // 1.0
}

impl Position {
    fn calculate(&self, available: u16, needed: u16) -> u16 {
        match self {
            Position::Absolute(n) => *n.min(&available.saturating_sub(needed)),
            Position::Relative(f) => {
                let pos = (available as f32 * f) as u16;
                pos.min(available.saturating_sub(needed))
            }
            Position::Start => 0,
            Position::Center => available.saturating_sub(needed) / 2,
            Position::End => available.saturating_sub(needed),
        }
    }
}

/// Center content in area
pub fn center(width: u16, height: u16, content: &str) -> String {
    place(width, height, Position::Center, Position::Center, content)
}
```

**Required Tests**:
```rust
#[test]
fn test_join_horizontal() {
    let items = vec!["A\nB\nC", "1\n2\n3"];
    let joined = join_horizontal(Alignment::Top, items);
    assert_eq!(joined, "A1\nB2\nC3");
}

#[test]
fn test_join_vertical() {
    let items = vec!["AAA", "B", "CC"];
    let joined = join_vertical(Alignment::Center, items);
    // B should be centered, CC should be left-padded
}

#[test]
fn test_place_center() {
    let content = "Hello";
    let placed = place(10, 3, Position::Center, Position::Center, content);
    // Should be centered in 10x3 area
}

#[test]
fn test_place_relative() {
    let content = "X";
    let placed = place(10, 10, Position::Relative(0.75), Position::Relative(0.25), content);
    // Should be at 75% horizontal, 25% vertical
}
```

---

## Phase 4: Advanced Components (Week 7-8)

### 4.1 🟡 Enhanced Table Component with Styling
**File**: `src/components/table_advanced.rs`  
**Effort**: 8 hours

**Implementation Details**:
```rust
use hojicha_style::{Style, Border, Color, Alignment};

pub struct StyledTable {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    widths: Vec<Option<u16>>,  // None = auto-size
    
    // Styling
    header_style: Style,
    row_style: Style,
    selected_style: Style,
    border: Border,
    border_style: Style,
    
    // Behavior
    selected_row: Option<usize>,
    show_headers: bool,
    show_row_separator: bool,
    show_column_separator: bool,
    
    // Scrolling
    offset: usize,
    visible_rows: Option<usize>,
}

impl StyledTable {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            widths: Vec::new(),
            header_style: Style::new().bold(true).fg(Color::parse("blue").unwrap()),
            row_style: Style::new(),
            selected_style: Style::new().bg(Color::parse("gray").unwrap()),
            border: Border::normal(),
            border_style: Style::new(),
            selected_row: None,
            show_headers: true,
            show_row_separator: false,
            show_column_separator: true,
            offset: 0,
            visible_rows: None,
        }
    }
    
    // Builder methods
    pub fn headers(mut self, headers: Vec<impl Into<String>>) -> Self {
        self.headers = headers.into_iter().map(Into::into).collect();
        self
    }
    
    pub fn add_row(mut self, row: Vec<impl Into<String>>) -> Self {
        self.rows.push(row.into_iter().map(Into::into).collect());
        self
    }
    
    pub fn column_widths(mut self, widths: Vec<Option<u16>>) -> Self {
        self.widths = widths;
        self
    }
    
    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }
    
    pub fn selected(mut self, row: Option<usize>) -> Self {
        self.selected_row = row;
        self
    }
    
    pub fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }
    
    pub fn visible_rows(mut self, n: usize) -> Self {
        self.visible_rows = Some(n);
        self
    }
    
    // Rendering
    pub fn render(&self) -> String {
        let mut lines = Vec::new();
        
        // Calculate column widths
        let widths = self.calculate_widths();
        
        // Top border
        lines.push(self.render_border_line(&widths, BorderPosition::Top));
        
        // Headers
        if self.show_headers && !self.headers.is_empty() {
            lines.push(self.render_row(&self.headers, &widths, &self.header_style));
            if self.show_row_separator {
                lines.push(self.render_border_line(&widths, BorderPosition::Middle));
            }
        }
        
        // Rows
        let visible_rows = self.get_visible_rows();
        for (idx, row) in visible_rows.iter().enumerate() {
            let style = if Some(self.offset + idx) == self.selected_row {
                &self.selected_style
            } else {
                &self.row_style
            };
            
            lines.push(self.render_row(row, &widths, style));
            
            if self.show_row_separator && idx < visible_rows.len() - 1 {
                lines.push(self.render_border_line(&widths, BorderPosition::Middle));
            }
        }
        
        // Bottom border
        lines.push(self.render_border_line(&widths, BorderPosition::Bottom));
        
        lines.join("\n")
    }
    
    fn calculate_widths(&self) -> Vec<u16> {
        let mut widths = Vec::new();
        
        for (col_idx, specified_width) in self.widths.iter().enumerate() {
            if let Some(w) = specified_width {
                widths.push(*w);
            } else {
                // Auto-size based on content
                let mut max_width = 0u16;
                
                // Check header
                if col_idx < self.headers.len() {
                    max_width = max_width.max(self.headers[col_idx].len() as u16);
                }
                
                // Check rows
                for row in &self.rows {
                    if col_idx < row.len() {
                        max_width = max_width.max(row[col_idx].len() as u16);
                    }
                }
                
                widths.push(max_width + 2); // Add padding
            }
        }
        
        widths
    }
    
    fn get_visible_rows(&self) -> Vec<Vec<String>> {
        let end = if let Some(visible) = self.visible_rows {
            (self.offset + visible).min(self.rows.len())
        } else {
            self.rows.len()
        };
        
        self.rows[self.offset..end].to_vec()
    }
}

enum BorderPosition {
    Top,
    Middle,
    Bottom,
}
```

**Required Tests**:
```rust
#[test]
fn test_table_basic_rendering() {
    let table = StyledTable::new()
        .headers(vec!["Name", "Age", "City"])
        .add_row(vec!["Alice", "30", "NYC"])
        .add_row(vec!["Bob", "25", "LA"]);
    
    let output = table.render();
    assert!(output.contains("Name"));
    assert!(output.contains("Alice"));
}

#[test]
fn test_table_selection() {
    let table = StyledTable::new()
        .add_row(vec!["A"])
        .add_row(vec!["B"])
        .selected(Some(1));
    
    // Selected row should have different style
}

#[test]
fn test_table_scrolling() {
    let table = StyledTable::new()
        .add_row(vec!["1"])
        .add_row(vec!["2"])
        .add_row(vec!["3"])
        .visible_rows(2)
        .offset(1);
    
    let output = table.render();
    assert!(!output.contains("1")); // Should be scrolled out
    assert!(output.contains("2"));
    assert!(output.contains("3"));
}

#[test]
fn test_table_auto_width() {
    let table = StyledTable::new()
        .headers(vec!["Short", "Very Long Header"])
        .add_row(vec!["A", "B"]);
    
    // Second column should be wider
}

#[test]
fn test_table_borders() {
    let table = StyledTable::new()
        .border(Border::rounded())
        .add_row(vec!["Test"]);
    
    let output = table.render();
    assert!(output.contains('╭')); // Rounded corner
}
```

---

### 4.2 🟡 Tree Component Implementation
**File**: `src/components/tree.rs`  
**Effort**: 6 hours

**Implementation Details**:
```rust
use hojicha_style::{Style, Color};

#[derive(Clone, Debug)]
pub struct Tree {
    root: TreeNode,
    selected_path: Vec<usize>,
    expanded_paths: HashSet<Vec<usize>>,
    
    // Styling
    node_style: Style,
    selected_style: Style,
    branch_style: Style,
    
    // Symbols
    expanded_symbol: String,
    collapsed_symbol: String,
    leaf_symbol: String,
    branch_symbols: TreeSymbols,
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    value: String,
    children: Vec<TreeNode>,
    is_leaf: bool,
}

#[derive(Clone, Debug)]
pub struct TreeSymbols {
    vertical: char,      // │
    horizontal: char,    // ─
    branch: char,        // ├
    last_branch: char,   // └
}

impl Tree {
    pub fn new(root_value: impl Into<String>) -> Self {
        Self {
            root: TreeNode {
                value: root_value.into(),
                children: Vec::new(),
                is_leaf: false,
            },
            selected_path: vec![],
            expanded_paths: HashSet::new(),
            node_style: Style::new(),
            selected_style: Style::new().bg(Color::parse("blue").unwrap()),
            branch_style: Style::new().fg(Color::parse("gray").unwrap()),
            expanded_symbol: "▼ ".to_string(),
            collapsed_symbol: "▶ ".to_string(),
            leaf_symbol: "  ".to_string(),
            branch_symbols: TreeSymbols::default(),
        }
    }
    
    pub fn add_node(&mut self, path: &[usize], value: impl Into<String>) {
        let node = self.navigate_to_node_mut(path);
        node.children.push(TreeNode {
            value: value.into(),
            children: Vec::new(),
            is_leaf: true,
        });
    }
    
    pub fn toggle_expanded(&mut self, path: &[usize]) {
        let path = path.to_vec();
        if self.expanded_paths.contains(&path) {
            self.expanded_paths.remove(&path);
        } else {
            self.expanded_paths.insert(path);
        }
    }
    
    pub fn select(&mut self, path: Vec<usize>) {
        self.selected_path = path;
    }
    
    pub fn render(&self) -> String {
        let mut lines = Vec::new();
        self.render_node(&self.root, &[], &mut lines, "");
        lines.join("\n")
    }
    
    fn render_node(
        &self,
        node: &TreeNode,
        path: &[usize],
        lines: &mut Vec<String>,
        prefix: &str,
    ) {
        // Determine if selected
        let is_selected = path == self.selected_path;
        
        // Determine symbol
        let symbol = if node.children.is_empty() {
            &self.leaf_symbol
        } else if self.expanded_paths.contains(path) {
            &self.expanded_symbol
        } else {
            &self.collapsed_symbol
        };
        
        // Render line
        let mut line = String::new();
        line.push_str(prefix);
        line.push_str(symbol);
        line.push_str(&node.value);
        
        // Apply style
        let styled_line = if is_selected {
            self.selected_style.render(&line)
        } else {
            self.node_style.render(&line)
        };
        
        lines.push(styled_line);
        
        // Render children if expanded
        if self.expanded_paths.contains(path) {
            for (idx, child) in node.children.iter().enumerate() {
                let mut child_path = path.to_vec();
                child_path.push(idx);
                
                let is_last = idx == node.children.len() - 1;
                let child_prefix = format!(
                    "{}{}  ",
                    prefix,
                    if is_last {
                        self.branch_symbols.last_branch
                    } else {
                        self.branch_symbols.branch
                    }
                );
                
                self.render_node(child, &child_path, lines, &child_prefix);
            }
        }
    }
}

impl Default for TreeSymbols {
    fn default() -> Self {
        Self {
            vertical: '│',
            horizontal: '─',
            branch: '├',
            last_branch: '└',
        }
    }
}
```

**Required Tests**:
```rust
#[test]
fn test_tree_basic_rendering() {
    let mut tree = Tree::new("Root");
    tree.add_node(&[], "Child 1");
    tree.add_node(&[], "Child 2");
    tree.add_node(&[0], "Grandchild");
    
    let output = tree.render();
    assert!(output.contains("Root"));
    assert!(output.contains("Child 1"));
}

#[test]
fn test_tree_expansion() {
    let mut tree = Tree::new("Root");
    tree.add_node(&[], "Child");
    tree.add_node(&[0], "Grandchild");
    
    // Initially collapsed
    let output = tree.render();
    assert!(!output.contains("Grandchild"));
    
    // Expand first child
    tree.toggle_expanded(&[0]);
    let output = tree.render();
    assert!(output.contains("Grandchild"));
}

#[test]
fn test_tree_selection() {
    let mut tree = Tree::new("Root");
    tree.add_node(&[], "Child");
    tree.select(vec![0]);
    
    // Selected node should have different style
}

#[test]
fn test_tree_symbols() {
    let tree = Tree::new("Root")
        .with_symbols(TreeSymbols {
            branch: '+',
            last_branch: '\\',
            ..Default::default()
        });
    
    // Custom symbols should be used
}
```

---

## Phase 5: Testing Infrastructure (Week 9)

### 5.1 🟡 Mock Terminal for Testing
**File**: `src/testing/mock.rs`  
**Effort**: 4 hours

**Implementation Details**:
```rust
use std::collections::VecDeque;

pub struct MockTerminal {
    // Input simulation
    input_events: VecDeque<Event>,
    
    // Output capture
    rendered_frames: Vec<String>,
    
    // Terminal state
    size: (u16, u16),
    cursor_visible: bool,
    mouse_enabled: bool,
    alt_screen: bool,
    
    // Behavior control
    should_fail: bool,
    fail_after: Option<usize>,
}

impl MockTerminal {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            input_events: VecDeque::new(),
            rendered_frames: Vec::new(),
            size: (width, height),
            cursor_visible: true,
            mouse_enabled: false,
            alt_screen: false,
            should_fail: false,
            fail_after: None,
        }
    }
    
    // Input simulation
    pub fn push_key(&mut self, key: Key) {
        self.input_events.push_back(Event::Key(KeyEvent::new(key, KeyModifiers::empty())));
    }
    
    pub fn push_text(&mut self, text: &str) {
        for ch in text.chars() {
            self.push_key(Key::Char(ch));
        }
    }
    
    pub fn push_mouse(&mut self, x: u16, y: u16, kind: MouseEventKind) {
        self.input_events.push_back(Event::Mouse(MouseEvent {
            column: x,
            row: y,
            kind,
            modifiers: KeyModifiers::empty(),
        }));
    }
    
    pub fn push_resize(&mut self, width: u16, height: u16) {
        self.size = (width, height);
        self.input_events.push_back(Event::Resize { width, height });
    }
    
    // Output capture
    pub fn last_frame(&self) -> Option<&String> {
        self.rendered_frames.last()
    }
    
    pub fn frame_count(&self) -> usize {
        self.rendered_frames.len()
    }
    
    pub fn all_frames(&self) -> &[String] {
        &self.rendered_frames
    }
    
    // Terminal state queries
    pub fn is_cursor_visible(&self) -> bool {
        self.cursor_visible
    }
    
    pub fn is_alt_screen(&self) -> bool {
        self.alt_screen
    }
    
    // Assertions
    pub fn assert_frame_contains(&self, text: &str) {
        assert!(
            self.last_frame().map_or(false, |f| f.contains(text)),
            "Expected frame to contain '{}', but it didn't.\nFrame: {:?}",
            text,
            self.last_frame()
        );
    }
    
    pub fn assert_frame_matches(&self, pattern: &str) {
        // Pattern matching with wildcards
        // * matches any sequence of characters
        // ? matches any single character
    }
}

// Implement as backend for testing
impl Backend for MockTerminal {
    fn draw<'a, I>(&mut self, content: I) -> Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        // Capture the rendered content
        let mut frame = vec![vec![' '; self.size.0 as usize]; self.size.1 as usize];
        
        for (x, y, cell) in content {
            if x < self.size.0 && y < self.size.1 {
                frame[y as usize][x as usize] = cell.symbol.chars().next().unwrap_or(' ');
            }
        }
        
        let frame_str = frame.iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        
        self.rendered_frames.push(frame_str);
        
        Ok(())
    }
    
    fn hide_cursor(&mut self) -> Result<()> {
        self.cursor_visible = false;
        Ok(())
    }
    
    fn show_cursor(&mut self) -> Result<()> {
        self.cursor_visible = true;
        Ok(())
    }
    
    fn size(&self) -> Result<Rect> {
        Ok(Rect::new(0, 0, self.size.0, self.size.1))
    }
}
```

**Required Tests**:
```rust
#[test]
fn test_mock_terminal_input() {
    let mut term = MockTerminal::new(80, 24);
    term.push_key(Key::Char('a'));
    term.push_text("hello");
    
    // Should have 6 events
    assert_eq!(term.input_events.len(), 6);
}

#[test]
fn test_mock_terminal_rendering() {
    let mut term = MockTerminal::new(80, 24);
    // Render something
    term.draw(...);
    
    assert_eq!(term.frame_count(), 1);
    term.assert_frame_contains("Hello");
}

#[test]
fn test_mock_terminal_state() {
    let mut term = MockTerminal::new(80, 24);
    term.hide_cursor().unwrap();
    assert!(!term.is_cursor_visible());
}
```

---

### 5.2 🟡 Test Utilities and Helpers
**File**: `src/testing/utils.rs`  
**Effort**: 3 hours

**Implementation Details**:
```rust
/// Run a model through a series of events and capture output
pub fn test_model<M: Model>(
    mut model: M,
    events: Vec<Event<M::Message>>,
) -> TestResult<M> {
    let mut terminal = MockTerminal::new(80, 24);
    let mut commands_executed = Vec::new();
    let mut frames = Vec::new();
    
    // Run init
    if let Some(cmd) = model.init() {
        commands_executed.push(format!("Init: {:?}", cmd));
    }
    
    // Process events
    for event in events {
        if let Some(cmd) = model.update(event.clone()) {
            commands_executed.push(format!("Update: {:?}", cmd));
        }
        
        // Render
        let mut frame = Frame::new(terminal.size().unwrap());
        model.view(&mut frame, frame.size());
        terminal.draw(frame.buffer.content).unwrap();
        frames.push(terminal.last_frame().unwrap().clone());
    }
    
    TestResult {
        final_model: model,
        frames,
        commands_executed,
        terminal,
    }
}

pub struct TestResult<M> {
    pub final_model: M,
    pub frames: Vec<String>,
    pub commands_executed: Vec<String>,
    pub terminal: MockTerminal,
}

/// Assert that a rendered view contains expected text (ANSI-aware)
pub fn assert_contains(rendered: &str, expected: &str) {
    let stripped = strip_ansi_codes(rendered);
    assert!(
        stripped.contains(expected),
        "Expected to find '{}' in rendered output:\n{}",
        expected,
        stripped
    );
}

/// Strip ANSI codes from string
pub fn strip_ansi_codes(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[^m]*m").unwrap();
    re.replace_all(s, "").to_string()
}

/// Generate a sequence of key events from a string
pub fn keys(s: &str) -> Vec<Event<()>> {
    s.chars()
        .map(|ch| Event::Key(KeyEvent::new(Key::Char(ch), KeyModifiers::empty())))
        .collect()
}

/// Create a key event with modifiers
pub fn key_with_mods(key: Key, mods: KeyModifiers) -> Event<()> {
    Event::Key(KeyEvent::new(key, mods))
}

/// Simulate typing with realistic delays
pub async fn type_text(text: &str) -> Vec<(Event<()>, Duration)> {
    let mut events = Vec::new();
    
    for ch in text.chars() {
        events.push((
            Event::Key(KeyEvent::new(Key::Char(ch), KeyModifiers::empty())),
            Duration::from_millis(50 + rand::random::<u64>() % 100),
        ));
    }
    
    events
}
```

**Required Tests**:
```rust
#[test]
fn test_model_testing() {
    struct TestModel { count: i32 }
    
    impl Model for TestModel {
        type Message = ();
        fn update(&mut self, _: Event<()>) -> Option<Cmd<()>> {
            self.count += 1;
            None
        }
        fn view(&self, frame: &mut Frame, area: Rect) {
            // Render count
        }
    }
    
    let model = TestModel { count: 0 };
    let events = vec![Event::Key(...), Event::Key(...)];
    let result = test_model(model, events);
    
    assert_eq!(result.final_model.count, 2);
    assert_eq!(result.frames.len(), 2);
}

#[test]
fn test_ansi_stripping() {
    let colored = "\x1b[31mRed Text\x1b[0m";
    assert_eq!(strip_ansi_codes(colored), "Red Text");
}

#[test]
fn test_key_generation() {
    let events = keys("hello");
    assert_eq!(events.len(), 5);
}
```

---

## Phase 6: Integration and Polish (Week 10)

### 6.1 🟢 Comprehensive Examples
**Files**: `examples/styled_*.rs`  
**Effort**: 4 hours

Create examples demonstrating:
- Styled counter with borders and colors
- File browser with tree component
- Data table with sorting and filtering
- Dashboard with multiple styled panels
- Text editor with syntax highlighting

### 6.2 🟢 Performance Benchmarks
**File**: `benches/rendering.rs`  
**Effort**: 3 hours

Benchmark:
- Style rendering performance
- Table with 1000 rows
- Tree with deep nesting
- Complex layouts

### 6.3 🟢 Documentation
**Files**: `README.md`, docs/  
**Effort**: 4 hours

Document:
- Migration guide from plain Ratatui
- Style system tutorial
- Component gallery
- Performance tips

---

## Success Metrics

Each task is complete when:
1. ✅ Implementation matches specification
2. ✅ All required tests pass
3. ✅ No performance regression
4. ✅ Documentation updated
5. ✅ Example code works

## Validation Tests

### Integration Test Suite
```rust
#[test]
fn test_full_app_with_all_features() {
    // Create app using all new features
    // Verify everything works together
}

#[test]
fn test_style_system_integration() {
    // Use hojicha-style with main framework
    // Verify seamless integration
}

#[test]
fn test_backwards_compatibility() {
    // Existing apps should still work
    // No breaking changes to core API
}
```

### Performance Tests
```rust
#[bench]
fn bench_styled_rendering(b: &mut Bencher) {
    // Measure overhead of style system
}

#[bench]
fn bench_large_table(b: &mut Bencher) {
    // Table with 1000 rows
}
```

---

## Total Effort Estimate

- **Phase 1 (Quick Wins)**: 8 hours
- **Phase 2 (Core)**: 22 hours  
- **Phase 3 (Style)**: 30 hours
- **Phase 4 (Components)**: 24 hours
- **Phase 5 (Testing)**: 14 hours
- **Phase 6 (Polish)**: 11 hours

**Total**: ~109 hours (2-3 weeks full-time)

---

_Last Updated: 2024-01-XX_  
_Version: 1.0.0_