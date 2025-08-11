# Changelog

All notable changes to Hojicha will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-11

### Added
- **Async Bridge System**: Full async event handling with external event injection
  - `init_async_bridge()` for external event sources
  - `subscribe()` for stream subscriptions  
  - `spawn_cancellable()` for cancellable operations
  - Priority queue with backpressure control
- **Component System**: Reusable, composable UI components
  - TextInput with Unicode support
  - Button with click detection
  - List with selection
  - Spinner with animation states
  - ScrollableText for large content
  - Progress bars, tabs, and more
- **Async Helpers**: High-level async command utilities
  - HTTP operations (GET, POST with retry logic)
  - WebSocket connections
  - File I/O operations
  - Timer utilities (delay, interval, throttle, debounce)
- **Comprehensive Documentation**
  - Common patterns guide with 15+ examples
  - Validated doctests (71 passing, 0 failures)
  - Complete API documentation with examples
- **Testing Infrastructure**
  - Property-based testing with proptest
  - Mock I/O for headless testing
  - Test coverage tracking
  - All README examples validated

### Changed
- **API Improvements**
  - Marked internal APIs with `#[doc(hidden)]`
  - Reorganized prelude modules for better discoverability
  - Enhanced error messages and debugging support
  - Consistent naming conventions across crates
- **Command System**
  - `Cmd::none()` now returns `Some(Cmd)` for consistency
  - Batch/sequence optimizations for single elements
  - Better command chunking for large batches
- **Documentation**
  - Removed emojis for clarity and precision
  - All doctests converted from `ignore` to validated examples
  - Module-level documentation added throughout

### Fixed
- Fixed type inference issues with generic commands
- Fixed missing exports (MouseEventKind, MouseButton, HttpResponse)
- Fixed WebSocketEvent exhaustive matching
- Fixed all doctest compilation errors
- Fixed test failures in batch/sequence optimization

### Technical Details
- **Architecture**: Clean separation into three crates
  - `hojicha-core`: Core TEA implementation
  - `hojicha-runtime`: Event loop and async runtime
  - `hojicha-pearls`: UI components and styling
- **Dependencies**: Minimal and focused
  - Built on ratatui 0.29 for rendering
  - Tokio 1.47 for async runtime
  - Crossterm 0.29 for terminal interaction

## [0.1.0] - 2024-12-15

### Initial Release
- Basic Elm Architecture implementation
- Core Model trait and Cmd type
- Event handling (keyboard, mouse, resize)
- Basic command utilities
- Initial examples (counter, components gallery)