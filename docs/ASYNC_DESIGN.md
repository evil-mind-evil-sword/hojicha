# Async Event Bridge Design Document

## Overview

This document outlines the design and implementation plan for adding async event bridge support to Hojicha, enabling integration with external async event sources while maintaining backward compatibility and the simplicity of the Elm Architecture.

## Implementation Status

✅ **Phase 1 Completed**: Basic async event bridge is now implemented and working!

### What's Been Implemented

1. **Public API for External Event Injection**
   - `init_async_bridge()` - Initialize message channel and get sender before `run()`
   - `sender()` - Get sender if channel is initialized  
   - `send_message()` - Convenience method to send user messages

2. **Working Examples**
   - `examples/async_timer.rs` - Demonstrates timer threads and external message injection

3. **Comprehensive Tests**
   - Property-based tests for message ordering
   - Integration tests for concurrent senders
   - Tests for channel capacity and backpressure

### Current API

```rust
// Initialize async bridge before running
let mut program = Program::new(model)?;
let sender = program.init_async_bridge();

// Use sender in external threads
thread::spawn(move || {
    loop {
        thread::sleep(Duration::from_secs(1));
        let _ = sender.send(Event::User(Msg::Tick));
    }
});

// Run the program
program.run()?;
```

## Motivation

Modern TUI applications need to handle events from multiple async sources:
- Language Server Protocol (LSP) servers
- AI/LLM streaming responses  
- WebSocket/SSE connections
- File system watchers
- Background computations

Current Hojicha limitations:
- No way to inject events from external sources
- Creates new Tokio runtime for each async command (inefficient)
- No support for continuous event streams
- No cancellation mechanism for async operations

## Design Principles

1. **Backward Compatibility**: Existing code must continue to work unchanged
2. **Opt-in Complexity**: Async features should be optional
3. **Type Safety**: Maintain Rust's type safety guarantees
4. **Performance**: Minimize overhead for non-async use cases
5. **Simplicity**: Keep the simple cases simple

## Proposed Architecture

### 1. Public Event Injection API

#### Current State
```rust
pub struct Program<M> {
    event_rx: Receiver<Event<M>>,  // Private, no external access
    // ...
}
```

#### Proposed Change
```rust
pub struct Program<M> {
    event_rx: Receiver<Event<M>>,
    message_tx: Sender<Event<M>>,  // Make sender accessible
    // ...
}

impl<M: Message> Program<M> {
    /// Get a sender for injecting events from external sources
    pub fn event_sender(&self) -> Sender<Event<M>> {
        self.message_tx.clone()
    }
    
    /// Convenience method to send user messages
    pub fn send_message(&self, msg: M) -> Result<(), SendError<Event<M>>> {
        self.message_tx.send(Event::User(msg))
    }
}
```

#### Example Usage
```rust
// Simple message sending example
fn main() -> Result<()> {
    let mut program = Program::new(MyModel::default())?;
    let sender = program.event_sender();
    
    // Spawn async task that sends events
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let _ = sender.send(Event::User(Msg::Tick));
        }
    });
    
    program.run()?;
    Ok(())
}
```

### 2. Shared Tokio Runtime

#### Current State
```rust
// In commands.rs
pub fn custom_async<M, F, Fut>(f: F) -> Cmd<M> {
    Cmd::new(move || {
        let runtime = tokio::runtime::Runtime::new().ok()?;  // NEW runtime each time!
        runtime.block_on(f())
    })
}
```

#### Proposed Change
```rust
pub struct Program<M> {
    runtime: Option<Arc<tokio::runtime::Runtime>>,  // Shared runtime
    // ...
}

pub struct ProgramOptions {
    pub use_shared_runtime: bool,  // Opt-in to shared runtime
    // ...
}

impl<M: Message> Program<M> {
    pub fn with_shared_runtime(mut self) -> Self {
        self.runtime = Some(Arc::new(
            tokio::runtime::Runtime::new().expect("Failed to create runtime")
        ));
        self
    }
    
    /// Spawn a future on the shared runtime
    pub fn spawn<F>(&self, future: F) -> Result<JoinHandle<()>, Error>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        match &self.runtime {
            Some(rt) => Ok(rt.spawn(future)),
            None => Err(Error::NoRuntime),
        }
    }
}

// Update command execution to use shared runtime if available
impl<M: Message> Program<M> {
    fn execute_async_command(&self, f: impl Future<Output = Option<M>>) {
        if let Some(runtime) = &self.runtime {
            // Use shared runtime
            runtime.spawn(async move {
                if let Some(msg) = f.await {
                    let _ = self.message_tx.send(Event::User(msg));
                }
            });
        } else {
            // Fall back to creating new runtime (backward compatible)
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().ok()?;
                runtime.block_on(f())
            });
        }
    }
}
```

#### Example Usage
```rust
// Example: Shared runtime for efficient async operations
#[tokio::main]
async fn main() -> Result<()> {
    let program = Program::new(MyModel::default())?
        .with_shared_runtime();  // Enable shared runtime
    
    let sender = program.event_sender();
    
    // Spawn multiple async tasks efficiently
    program.spawn(async move {
        // This runs on the shared runtime
        let data = fetch_data().await;
        sender.send(Event::User(Msg::DataLoaded(data))).unwrap();
    })?;
    
    program.run()?;
    Ok(())
}
```

### 3. Stream Integration Support

#### Proposed API
```rust
use tokio_stream::Stream;

pub struct StreamHandle {
    cancel_token: CancellationToken,
    handle: JoinHandle<()>,
}

impl<M: Message> Program<M> {
    /// Subscribe to a stream of events
    pub fn subscribe<S>(&self, stream: S) -> Result<StreamHandle, Error>
    where
        S: Stream<Item = M> + Send + 'static,
    {
        let sender = self.message_tx.clone();
        let cancel_token = CancellationToken::new();
        let token_clone = cancel_token.clone();
        
        let handle = self.spawn(async move {
            tokio::pin!(stream);
            loop {
                tokio::select! {
                    _ = token_clone.cancelled() => break,
                    Some(msg) = stream.next() => {
                        if sender.send(Event::User(msg)).is_err() {
                            break;  // Program has shut down
                        }
                    }
                    else => break,  // Stream ended
                }
            }
        })?;
        
        Ok(StreamHandle { cancel_token, handle })
    }
}

impl StreamHandle {
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
}
```

#### Example Usage
```rust
// Example: AI token streaming
use tokio_stream::StreamExt;

struct AiModel {
    response: String,
    stream_handle: Option<StreamHandle>,
}

enum Msg {
    StartGeneration(String),
    TokenReceived(String),
    GenerationComplete,
}

impl Model for AiModel {
    type Message = Msg;
    
    fn update(&mut self, event: Event<Msg>) -> Option<Cmd<Msg>> {
        match event {
            Event::User(Msg::StartGeneration(prompt)) => {
                // Cancel any existing generation
                if let Some(handle) = &self.stream_handle {
                    handle.cancel();
                }
                
                // Start new generation
                let stream = ai_client::generate_stream(prompt)
                    .map(|token| Msg::TokenReceived(token));
                
                self.stream_handle = Some(
                    self.program.subscribe(stream).unwrap()
                );
            }
            Event::User(Msg::TokenReceived(token)) => {
                self.response.push_str(&token);
            }
            Event::User(Msg::GenerationComplete) => {
                self.stream_handle = None;
            }
            _ => {}
        }
        None
    }
}
```

### 4. Cancellable Async Operations

#### Proposed API
```rust
pub struct AsyncTask {
    cancel_token: CancellationToken,
    handle: JoinHandle<()>,
}

impl AsyncTask {
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
    
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
}

impl<M: Message> Program<M> {
    /// Spawn a cancellable async task
    pub fn spawn_cancellable<F, Fut>(&self, f: F) -> Result<AsyncTask, Error>
    where
        F: FnOnce(CancellationToken) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let cancel_token = CancellationToken::new();
        let handle = self.spawn(f(cancel_token.clone()))?;
        Ok(AsyncTask { cancel_token, handle })
    }
}
```

#### Example Usage
```rust
// Example: Cancellable file search
struct FileSearchModel {
    search_task: Option<AsyncTask>,
    results: Vec<PathBuf>,
}

enum Msg {
    StartSearch(String),
    CancelSearch,
    SearchResult(PathBuf),
    SearchComplete,
}

impl Model for FileSearchModel {
    type Message = Msg;
    
    fn update(&mut self, event: Event<Msg>) -> Option<Cmd<Msg>> {
        match event {
            Event::User(Msg::StartSearch(query)) => {
                // Cancel any existing search
                if let Some(task) = &self.search_task {
                    task.cancel();
                }
                
                let sender = self.program.event_sender();
                self.search_task = Some(
                    self.program.spawn_cancellable(|token| async move {
                        let mut walker = WalkDir::new(".");
                        for entry in walker {
                            // Check for cancellation
                            if token.is_cancelled() {
                                break;
                            }
                            
                            if let Ok(entry) = entry {
                                if entry.path().to_string_lossy().contains(&query) {
                                    sender.send(Event::User(Msg::SearchResult(entry.path()))).unwrap();
                                }
                            }
                        }
                        sender.send(Event::User(Msg::SearchComplete)).unwrap();
                    }).unwrap()
                );
            }
            Event::User(Msg::CancelSearch) => {
                if let Some(task) = &self.search_task {
                    task.cancel();
                    self.search_task = None;
                }
            }
            Event::User(Msg::SearchResult(path)) => {
                self.results.push(path);
            }
            _ => {}
        }
        None
    }
}
```

### 5. Event Priority and Backpressure

#### Proposed API
```rust
pub enum Priority {
    High,    // User input, cancellations
    Normal,  // Regular updates
    Low,     // Background tasks, progress updates
}

pub struct PriorityEvent<M> {
    event: Event<M>,
    priority: Priority,
}

pub struct ProgramOptions {
    pub channel_capacity: usize,  // Default: unbounded
    pub overflow_strategy: OverflowStrategy,
}

pub enum OverflowStrategy {
    DropOldest,   // Drop oldest events when full
    DropNewest,   // Drop new events when full
    Block,        // Block sender until space available
}

impl<M: Message> Program<M> {
    pub fn send_with_priority(&self, msg: M, priority: Priority) -> Result<(), SendError<M>> {
        // Implementation would use priority queue internally
        match priority {
            Priority::High => {
                // High priority events bypass queue
                self.high_priority_tx.send(Event::User(msg))
            }
            Priority::Normal => {
                self.message_tx.send(Event::User(msg))
            }
            Priority::Low => {
                // Low priority events can be dropped if queue is full
                self.message_tx.try_send(Event::User(msg)).ok();
            }
        }
    }
}
```

#### Example Usage
```rust
// Example: File watcher with backpressure handling
struct FileWatcherModel {
    files_changed: Vec<PathBuf>,
}

enum Msg {
    FileChanged(PathBuf),
    RefreshUI,
}

fn setup_file_watcher(program: &Program<Msg>) {
    let sender = program.event_sender();
    
    std::thread::spawn(move || {
        let mut watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(event) => {
                    // File changes are low priority - OK to drop if overwhelmed
                    let _ = program.send_with_priority(
                        Msg::FileChanged(event.path),
                        Priority::Low
                    );
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        }).unwrap();
        
        watcher.watch(Path::new("."), RecursiveMode::Recursive).unwrap();
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    });
}
```

## Migration Strategy

### Phase 1: Event Injection (Non-breaking)
1. Add `event_sender()` method to Program
2. No changes to existing code required
3. New code can start using external event injection

### Phase 2: Shared Runtime (Opt-in)
1. Add `with_shared_runtime()` builder method
2. Existing async commands continue to work
3. New code can opt into shared runtime for efficiency

### Phase 3: Streams and Cancellation (Additive)
1. Add new APIs for streams and cancellable tasks
2. Existing command system unchanged
3. Gradually migrate intensive operations to new APIs

### Phase 4: Priority/Backpressure (Opt-in)
1. Add configuration options for channel capacity
2. Default to unbounded (current behavior)
3. Applications can opt into bounded channels with overflow handling

## Complete Example: Chat Application

Here's how all features come together in a real application:

```rust
use hojicha::prelude::*;
use tokio_stream::StreamExt;

struct ChatModel {
    messages: Vec<Message>,
    input: String,
    connection: Option<StreamHandle>,
}

#[derive(Clone)]
enum Msg {
    Connect,
    Disconnect,
    SendMessage(String),
    MessageReceived(ChatMessage),
    InputChanged(String),
    ConnectionLost,
}

impl Model for ChatModel {
    type Message = Msg;
    
    fn init(&mut self) -> Option<Cmd<Msg>> {
        Some(commands::custom(|| Some(Msg::Connect)))
    }
    
    fn update(&mut self, event: Event<Msg>) -> Option<Cmd<Msg>> {
        match event {
            Event::User(Msg::Connect) => {
                // Subscribe to WebSocket stream
                let stream = websocket::connect("wss://chat.example.com")
                    .await?
                    .map(|msg| Msg::MessageReceived(msg));
                
                self.connection = Some(
                    self.program.subscribe(stream).unwrap()
                );
            }
            Event::User(Msg::Disconnect) => {
                if let Some(conn) = &self.connection {
                    conn.cancel();
                    self.connection = None;
                }
            }
            Event::User(Msg::SendMessage(text)) => {
                // High priority - user action
                websocket::send(text).await?;
                self.messages.push(Message::Sent(text));
            }
            Event::User(Msg::MessageReceived(msg)) => {
                // Normal priority - incoming messages
                self.messages.push(Message::Received(msg));
            }
            Event::Key(key) => match key.key {
                Key::Enter => {
                    return Some(commands::custom({
                        let msg = self.input.clone();
                        move || Some(Msg::SendMessage(msg))
                    }));
                }
                Key::Char(c) => {
                    self.input.push(c);
                }
                Key::Backspace => {
                    self.input.pop();
                }
                Key::Esc => return None,  // Quit
                _ => {}
            }
            _ => {}
        }
        None
    }
    
    fn view(&self, frame: &mut Frame, area: Rect) {
        // Render chat UI
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let program = Program::new(ChatModel::default())?
        .with_shared_runtime()
        .with_channel_capacity(1000);
    
    program.run()?;
    Ok(())
}
```

## Benefits

1. **Performance**: Shared runtime eliminates overhead of creating runtimes
2. **Flexibility**: Support for any async event source
3. **Control**: Cancellation and priority handling
4. **Scalability**: Backpressure prevents overwhelming the UI
5. **Compatibility**: Existing code continues to work unchanged

## Open Questions

1. Should we use `tokio::sync::mpsc` or `crossbeam-channel` for message passing?
2. How should we handle runtime shutdown and cleanup?
3. Should priority be part of Event or a separate channel?
4. What's the best API for configuring backpressure strategies?

## Next Steps

1. Prototype the event injection API (simplest change)
2. Benchmark shared vs. per-command runtime performance
3. Test stream integration with real services (LSP, WebSocket)
4. Gather feedback on API ergonomics
5. Implement in phases to maintain stability