# Common Patterns

This guide covers common patterns and best practices for building Hojicha applications.

## Table of Contents

- [Application Structure](#application-structure)
- [State Management](#state-management)
- [Event Handling](#event-handling)
- [Async Patterns](#async-patterns)
- [Component Composition](#component-composition)
- [Error Handling](#error-handling)
- [Testing Patterns](#testing-patterns)

## Application Structure

### Basic Application

```rust
use hojicha_core::prelude::*;
use hojicha_runtime::prelude::*;

// Model - Your application state
struct App {
    // State fields
}

// Messages - Events that change state
enum Msg {
    // Your message variants
}

// Implementation
impl Model for App {
    type Message = Msg;
    
    fn init(&mut self) -> Cmd<Msg> {
        // Initial setup
        Cmd::none()
    }
    
    fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
        // Handle events
        Cmd::none()
    }
    
    fn view(&self, frame: &mut Frame, area: Rect) {
        // Render UI
    }
}
```

### Multi-Component Application

```rust
// Split complex apps into sub-models
struct App {
    sidebar: Sidebar,
    main_view: MainView,
    status_bar: StatusBar,
}

enum Msg {
    Sidebar(SidebarMsg),
    MainView(MainViewMsg),
    StatusBar(StatusBarMsg),
}

impl Model for App {
    type Message = Msg;
    
    fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
        match event {
            Event::User(Msg::Sidebar(msg)) => {
                self.sidebar.update(msg).map(Msg::Sidebar)
            }
            Event::User(Msg::MainView(msg)) => {
                self.main_view.update(msg).map(Msg::MainView)
            }
            _ => Cmd::none()
        }
    }
    
    fn view(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(0)])
            .split(area);
            
        self.sidebar.render(frame, chunks[0]);
        self.main_view.render(frame, chunks[1]);
    }
}
```

## State Management

### Loading States

```rust
enum LoadingState<T> {
    NotStarted,
    Loading,
    Loaded(T),
    Failed(String),
}

struct App {
    data: LoadingState<Vec<Item>>,
}

impl Model for App {
    type Message = Msg;
    
    fn init(&mut self) -> Cmd<Msg> {
        self.data = LoadingState::Loading;
        commands::spawn(async {
            match fetch_data().await {
                Ok(data) => Some(Msg::DataLoaded(data)),
                Err(e) => Some(Msg::LoadFailed(e.to_string())),
            }
        })
    }
    
    fn view(&self, frame: &mut Frame, area: Rect) {
        match &self.data {
            LoadingState::Loading => {
                let spinner = Spinner::default();
                frame.render_widget(spinner, area);
            }
            LoadingState::Loaded(data) => {
                // Render data
            }
            LoadingState::Failed(error) => {
                // Render error
            }
            _ => {}
        }
    }
}
```

### Form State

```rust
struct FormState {
    username: String,
    email: String,
    errors: HashMap<String, String>,
    submitting: bool,
}

impl FormState {
    fn validate(&mut self) -> bool {
        self.errors.clear();
        
        if self.username.is_empty() {
            self.errors.insert("username".into(), "Required".into());
        }
        
        if !self.email.contains('@') {
            self.errors.insert("email".into(), "Invalid email".into());
        }
        
        self.errors.is_empty()
    }
}
```

## Event Handling

### Keyboard Shortcuts

```rust
fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match event {
        Event::Key(key) => {
            // Global shortcuts
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.key {
                    Key::Char('s') => return self.save(),
                    Key::Char('o') => return self.open(),
                    Key::Char('q') => return commands::quit(),
                    _ => {}
                }
            }
            
            // Mode-specific handling
            match self.mode {
                Mode::Normal => self.handle_normal_mode(key),
                Mode::Insert => self.handle_insert_mode(key),
                Mode::Command => self.handle_command_mode(key),
            }
        }
        _ => Cmd::none()
    }
}
```

### Mouse Events

```rust
fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match event {
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_click(mouse.column, mouse.row)
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.handle_drag(mouse.column, mouse.row)
            }
            MouseEventKind::ScrollUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(3);
                Cmd::none()
            }
            MouseEventKind::ScrollDown => {
                self.scroll_offset += 3;
                Cmd::none()
            }
            _ => Cmd::none()
        },
        _ => Cmd::none()
    }
}
```

## Async Patterns

### HTTP Requests

```rust
use hojicha_core::async_helpers::http_get;

enum Msg {
    FetchData,
    DataReceived(String),
    RequestFailed(String),
}

fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match event {
        Event::User(Msg::FetchData) => {
            http_get("https://api.example.com/data", |result| {
                match result {
                    Ok(body) => Msg::DataReceived(body),
                    Err(e) => Msg::RequestFailed(e.to_string()),
                }
            })
        }
        _ => Cmd::none()
    }
}
```

### WebSocket Connections

```rust
fn init(&mut self) -> Cmd<Msg> {
    // Subscribe to WebSocket
    let mut program = Program::new(self)?;
    let ws = WebSocketStream::connect("wss://example.com").await?;
    let subscription = program.subscribe(
        ws.map(|msg| Msg::WsMessage(msg))
    );
    self.ws_subscription = Some(subscription);
    Cmd::none()
}
```

### Periodic Tasks

```rust
fn init(&mut self) -> Cmd<Msg> {
    commands::batch(vec![
        // Refresh every 5 seconds
        commands::every(Duration::from_secs(5), |_| Msg::Refresh),
        // Check for updates every minute
        commands::every(Duration::from_secs(60), |_| Msg::CheckUpdates),
    ])
}
```

### Cancellable Operations

```rust
struct App {
    search_handle: Option<AsyncHandle<()>>,
}

fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match event {
        Event::User(Msg::Search(query)) => {
            // Cancel previous search
            if let Some(handle) = self.search_handle.take() {
                handle.cancel();
            }
            
            // Start new search
            let handle = program.spawn_cancellable(|token| async move {
                tokio::select! {
                    _ = token.cancelled() => None,
                    result = search(&query) => Some(Msg::SearchResults(result))
                }
            });
            
            self.search_handle = Some(handle);
            Cmd::none()
        }
        _ => Cmd::none()
    }
}
```

## Component Composition

### Reusable Components

```rust
// Define a reusable component
struct Button {
    label: String,
    focused: bool,
    on_click: Box<dyn Fn() -> Msg>,
}

impl Button {
    fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            focused: false,
            on_click: Box::new(|| Msg::Noop),
        }
    }
    
    fn on_click<F>(mut self, f: F) -> Self 
    where
        F: Fn() -> Msg + 'static
    {
        self.on_click = Box::new(f);
        self
    }
    
    fn render(&self, frame: &mut Frame, area: Rect) {
        let style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        
        let widget = Paragraph::new(self.label.clone())
            .style(style)
            .block(Block::default().borders(Borders::ALL));
            
        frame.render_widget(widget, area);
    }
}
```

### Component Lists

```rust
struct TodoList {
    items: Vec<TodoItem>,
    selected: usize,
}

impl TodoList {
    fn handle_key(&mut self, key: KeyEvent) -> Cmd<Msg> {
        match key.key {
            Key::Up => {
                self.selected = self.selected.saturating_sub(1);
                Cmd::none()
            }
            Key::Down => {
                self.selected = (self.selected + 1).min(self.items.len() - 1);
                Cmd::none()
            }
            Key::Enter => {
                if let Some(item) = self.items.get_mut(self.selected) {
                    item.toggle();
                }
                Cmd::none()
            }
            _ => Cmd::none()
        }
    }
}
```

## Error Handling

### Result Commands

```rust
fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match event {
        Event::User(Msg::SaveFile) => {
            commands::fallible_with_error(
                || {
                    std::fs::write("data.json", &self.data)?;
                    Ok(Some(Msg::SaveSuccess))
                },
                |err| Msg::SaveError(err.to_string())
            )
        }
        _ => Cmd::none()
    }
}
```

### Error Recovery

```rust
struct App {
    last_error: Option<String>,
    retry_count: u32,
}

fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
    match event {
        Event::User(Msg::Error(e)) => {
            self.last_error = Some(e);
            self.retry_count += 1;
            
            if self.retry_count < 3 {
                // Retry with exponential backoff
                let delay = Duration::from_secs(2_u64.pow(self.retry_count));
                commands::tick(delay, || Msg::Retry)
            } else {
                // Give up and show error
                Cmd::none()
            }
        }
        _ => Cmd::none()
    }
}
```

## Testing Patterns

### Unit Testing Models

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use hojicha_core::testing::TestHarness;
    
    #[test]
    fn test_increment() {
        let model = Counter { value: 0 };
        
        let result = TestHarness::new(model)
            .send_event(Event::User(Msg::Increment))
            .run();
            
        assert_eq!(result.model.value, 1);
    }
    
    #[test]
    fn test_keyboard_navigation() {
        let model = Menu { selected: 0, items: vec!["A", "B", "C"] };
        
        let result = TestHarness::new(model)
            .send_event(Event::Key(KeyEvent::new(Key::Down)))
            .send_event(Event::Key(KeyEvent::new(Key::Down)))
            .run();
            
        assert_eq!(result.model.selected, 2);
    }
}
```

### Testing Async Operations

```rust
#[tokio::test]
async fn test_async_command() {
    let model = App::default();
    let mut program = Program::new(model).unwrap();
    
    // Send async message
    program.send_message(Msg::FetchData).unwrap();
    
    // Wait for response
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify state changed
    program.run_until(|model| model.data.is_some()).unwrap();
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_counter_bounds(ops in prop::collection::vec(any::<bool>(), 0..100)) {
        let mut model = Counter { value: 0, max: 100 };
        
        for increment in ops {
            let msg = if increment { Msg::Increment } else { Msg::Decrement };
            model.update(Event::User(msg));
        }
        
        assert!(model.value >= 0);
        assert!(model.value <= model.max);
    }
}
```

## Best Practices

### 1. Keep Update Functions Pure
- Don't perform side effects directly in update
- Return commands for all side effects
- This makes testing easier and behavior predictable

### 2. Use Type-Safe Messages
- Prefer enums over strings for messages
- Use nested enums for component messages
- This catches errors at compile time

### 3. Handle All Events Explicitly
- Always have a catch-all pattern
- Log unexpected events in development
- Return `Cmd::none()` for unhandled events

### 4. Structure Large Applications
- Split into modules by feature
- Use sub-models for complex components
- Keep view functions focused

### 5. Test Critical Paths
- Test state transitions
- Test error handling
- Test async operation cancellation
- Use property-based testing for invariants