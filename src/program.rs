//! The main Program struct that runs the application

// Module components
mod command_executor;
mod event_processor;
mod fps_limiter;
mod priority_event_processor;
mod terminal_manager;

pub use command_executor::CommandExecutor;
pub use event_processor::EventProcessor;
pub use fps_limiter::FpsLimiter;
pub use priority_event_processor::{
    EventStats, PriorityConfig, PriorityEventProcessor, get_event_stats,
};
pub use terminal_manager::{TerminalConfig, TerminalManager};

// Re-export the main types from program_old.rs for backward compatibility
// We'll gradually migrate the implementation to use the extracted components

use crate::async_handle::AsyncHandle;
use crate::core::Model;
use crate::error::{Error, Result};
use crate::event::Event;
use crate::subscription::Subscription;
use crossterm::event::{self};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

/// Type alias for message filter function
type MessageFilter<M> = Box<
    dyn Fn(&M, Event<<M as Model>::Message>) -> Option<Event<<M as Model>::Message>> + Send + Sync,
>;

/// Mouse tracking mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MouseMode {
    /// No mouse tracking
    #[default]
    None,
    /// Track mouse events only when buttons are pressed
    CellMotion,
    /// Track all mouse movement, even without button presses
    AllMotion,
}

/// Options for configuring the program
#[derive(Default)]
pub struct ProgramOptions {
    /// Whether to use alternate screen
    pub alt_screen: bool,
    /// Mouse tracking mode
    pub mouse_mode: MouseMode,
    /// Enable bracketed paste mode
    pub bracketed_paste: bool,
    /// Enable focus reporting
    pub focus_reporting: bool,
    /// Frames per second (0 = unlimited)
    pub fps: u16,
    /// Run in headless mode without rendering
    pub headless: bool,
    /// Disable signal handlers
    pub install_signal_handler: bool,
    /// Disable renderer
    pub without_renderer: bool,
    /// Custom output writer
    pub output: Option<Box<dyn Write + Send + Sync>>,
}

impl ProgramOptions {
    /// Create new default options
    pub fn default() -> Self {
        Self {
            alt_screen: true,
            mouse_mode: MouseMode::None,
            bracketed_paste: false,
            focus_reporting: false,
            fps: 60,
            headless: false,
            install_signal_handler: true,
            without_renderer: false,
            output: None,
        }
    }

    /// Enable or disable alternate screen
    pub fn with_alt_screen(mut self, enable: bool) -> Self {
        self.alt_screen = enable;
        self
    }

    /// Set mouse tracking mode
    pub fn with_mouse_mode(mut self, mode: MouseMode) -> Self {
        self.mouse_mode = mode;
        self
    }

    /// Enable bracketed paste mode
    pub fn with_bracketed_paste(mut self, enable: bool) -> Self {
        self.bracketed_paste = enable;
        self
    }

    /// Enable focus reporting
    pub fn with_focus_reporting(mut self, enable: bool) -> Self {
        self.focus_reporting = enable;
        self
    }

    /// Set frames per second
    pub fn with_fps(mut self, fps: u16) -> Self {
        self.fps = fps;
        self
    }

    /// Run in headless mode
    pub fn headless(mut self) -> Self {
        self.headless = true;
        self
    }

    /// Disable signal handlers
    pub fn without_signal_handler(mut self) -> Self {
        self.install_signal_handler = false;
        self
    }

    /// Disable renderer
    pub fn without_renderer(mut self) -> Self {
        self.without_renderer = true;
        self
    }

    /// Set custom output
    pub fn with_output(mut self, output: Box<dyn Write + Send + Sync>) -> Self {
        self.output = Some(output);
        self
    }
}

/// The main program that runs your application
pub struct Program<M: Model> {
    model: M,
    options: ProgramOptions,
    terminal_manager: TerminalManager,
    command_executor: CommandExecutor,
    fps_limiter: FpsLimiter,
    message_tx: Option<mpsc::SyncSender<Event<M::Message>>>,
    message_rx: Option<mpsc::Receiver<Event<M::Message>>>,
    priority_processor: PriorityEventProcessor<M::Message>,
    filter: Option<MessageFilter<M>>,
    running: Arc<AtomicBool>,
    force_quit: Arc<AtomicBool>,
    input_thread: Option<thread::JoinHandle<()>>,
}

impl<M: Model> Program<M>
where
    M::Message: Clone,
{
    /// Create a new program with the given model
    pub fn new(model: M) -> Result<Self> {
        Self::with_options(model, ProgramOptions::default())
    }

    /// Create a new program with custom options
    pub fn with_options(model: M, options: ProgramOptions) -> Result<Self> {
        // Create terminal manager
        let terminal_config = TerminalConfig {
            alt_screen: options.alt_screen,
            mouse_mode: options.mouse_mode,
            bracketed_paste: options.bracketed_paste,
            focus_reporting: options.focus_reporting,
            headless: options.headless || options.without_renderer,
        };
        let terminal_manager = TerminalManager::new(terminal_config)?;

        // Create command executor
        let command_executor = CommandExecutor::new()?;

        // Create FPS limiter
        let fps_limiter = FpsLimiter::new(options.fps);

        // Create priority event processor with default config
        let priority_processor = PriorityEventProcessor::new();

        log::info!("Boba program initialized with priority event processing");

        Ok(Self {
            model,
            options,
            terminal_manager,
            command_executor,
            fps_limiter,
            message_tx: None,
            message_rx: None,
            priority_processor,
            filter: None,
            running: Arc::new(AtomicBool::new(false)),
            force_quit: Arc::new(AtomicBool::new(false)),
            input_thread: None,
        })
    }

    /// Set a message filter function
    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&M, Event<M::Message>) -> Option<Event<M::Message>> + Send + Sync + 'static,
    {
        self.filter = Some(Box::new(filter));
        self
    }

    /// Configure the priority event processor
    pub fn with_priority_config(mut self, config: PriorityConfig) -> Self {
        self.priority_processor = PriorityEventProcessor::with_config(config);
        log::debug!("Priority processor configured with custom settings");
        self
    }

    /// Get current event processing statistics
    pub fn event_stats(&self) -> EventStats {
        self.priority_processor.stats()
    }

    /// Get a formatted string of event statistics (useful for debugging)
    pub fn event_stats_string(&self) -> String {
        get_event_stats(&self.priority_processor)
    }

    /// Print formatted text to stderr
    pub fn printf(&self, args: std::fmt::Arguments) {
        eprint!("{args}");
        let _ = io::stderr().flush();
    }

    /// Print a line to stderr
    pub fn println(&self, text: &str) {
        eprintln!("{text}");
        let _ = io::stderr().flush();
    }

    /// Send a quit message to the running program
    pub fn quit(&self) {
        if let Some(tx) = &self.message_tx {
            let _ = tx.send(Event::Quit);
        }
    }

    /// Force kill the program immediately
    pub fn kill(&self) {
        self.force_quit.store(true, Ordering::SeqCst);
        self.running.store(false, Ordering::SeqCst);
    }

    /// Get advanced performance metrics
    pub fn metrics(&self) -> crate::metrics::AdvancedEventStats {
        self.priority_processor.advanced_metrics()
    }

    /// Export metrics in JSON format
    pub fn metrics_json(&self) -> String {
        self.priority_processor.metrics_collector().export_json()
    }

    /// Export metrics in Prometheus format
    pub fn metrics_prometheus(&self) -> String {
        self.priority_processor
            .metrics_collector()
            .export_prometheus()
    }

    /// Export metrics in plain text format
    pub fn metrics_text(&self) -> String {
        self.priority_processor.metrics_collector().export_text()
    }

    /// Display metrics dashboard (for debugging)
    pub fn metrics_dashboard(&self) -> String {
        let stats = self.priority_processor.advanced_metrics();
        crate::metrics::display_dashboard(&stats)
    }

    /// Reset all metrics
    pub fn reset_metrics(&self) {
        self.priority_processor.reset_stats();
    }

    /// Dynamically resize the event queue
    pub fn resize_queue(&self, new_size: usize) -> Result<()> {
        self.priority_processor
            .resize_queue(new_size)
            .map_err(|e| Error::Custom(Box::new(e)))
    }

    /// Get current queue capacity
    pub fn queue_capacity(&self) -> usize {
        self.priority_processor.queue_capacity()
    }

    /// Enable auto-scaling with the specified configuration
    pub fn enable_auto_scaling(
        &mut self,
        config: crate::queue_scaling::AutoScaleConfig,
    ) -> &mut Self {
        self.priority_processor.enable_auto_scaling(config);
        self
    }

    /// Enable auto-scaling with default configuration
    pub fn with_auto_scaling(mut self) -> Self {
        self.priority_processor
            .enable_auto_scaling(crate::queue_scaling::AutoScaleConfig::default());
        self
    }

    /// Disable auto-scaling
    pub fn disable_auto_scaling(&mut self) -> &mut Self {
        self.priority_processor.disable_auto_scaling();
        self
    }

    /// Get a sender for injecting events from external sources
    ///
    /// This allows external async tasks to send messages to the program's event loop.
    /// The sender is thread-safe and can be cloned for use in multiple threads.
    ///
    /// # Example
    /// ```ignore
    /// let mut program = Program::new(model)?;
    /// let sender = program.sender();
    ///
    /// std::thread::spawn(move || {
    ///     loop {
    ///         std::thread::sleep(Duration::from_secs(1));
    ///         let _ = sender.send(Event::User(Msg::Tick));
    ///     }
    /// });
    ///
    /// program.run()?;
    /// ```
    pub fn sender(&self) -> Option<mpsc::SyncSender<Event<M::Message>>> {
        self.message_tx.clone()
    }

    /// Send a user message to the program
    ///
    /// Convenience method that wraps the message in Event::User.
    ///
    /// # Example
    /// ```ignore
    /// program.send_message(Msg::DataLoaded(data))?;
    /// ```
    pub fn send_message(&self, msg: M::Message) -> Result<()> {
        self.message_tx
            .as_ref()
            .ok_or_else(|| {
                Error::from(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "Program not running",
                ))
            })?
            .send(Event::User(msg))
            .map_err(|_| {
                Error::from(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Receiver disconnected",
                ))
            })
    }

    /// Wait for the program to finish
    pub fn wait(&self) {
        while !self.running.load(Ordering::SeqCst) && !self.force_quit.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(1));
        }
        while self.running.load(Ordering::SeqCst) && !self.force_quit.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
        }
    }

    /// Release the terminal
    pub fn release_terminal(&mut self) -> Result<()> {
        self.terminal_manager.release().map_err(Error::from)
    }

    /// Restore the terminal
    pub fn restore_terminal(&mut self) -> Result<()> {
        self.terminal_manager.restore().map_err(Error::from)
    }

    /// Initialize the async message bridge for external event injection
    ///
    /// This sets up channels for external systems to send messages into the program's
    /// event loop. Must be called before `run()` if you need external message injection.
    ///
    /// # Use Cases
    ///
    /// - **WebSocket Integration**: Forward messages from WebSocket connections
    /// - **File Watchers**: Send events when files change
    /// - **Timers**: Implement custom scheduling beyond tick/every
    /// - **Database Listeners**: Forward change notifications
    /// - **IPC**: Receive messages from other processes
    /// - **HTTP Responses**: Send results from async HTTP requests
    ///
    /// # Thread Safety
    ///
    /// The returned sender can be cloned and shared across multiple threads safely.
    /// Messages are queued with a capacity of 100 by default.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut program = Program::new(model)?;
    /// let sender = program.init_async_bridge();
    ///
    /// // WebSocket integration example
    /// let ws_sender = sender.clone();
    /// tokio::spawn(async move {
    ///     let mut ws = connect_websocket().await;
    ///     while let Some(msg) = ws.recv().await {
    ///         ws_sender.send(Event::User(Msg::WsMessage(msg))).ok();
    ///     }
    /// });
    ///
    /// // File watcher example
    /// let fs_sender = sender.clone();
    /// thread::spawn(move || {
    ///     let mut watcher = FileWatcher::new();
    ///     for event in watcher.events() {
    ///         fs_sender.send(Event::User(Msg::FileChanged(event))).ok();
    ///     }
    /// });
    ///
    /// program.run()?;
    /// ```
    pub fn init_async_bridge(&mut self) -> mpsc::SyncSender<Event<M::Message>> {
        if self.message_tx.is_none() {
            let (message_tx, message_rx) = mpsc::sync_channel(100);
            self.message_tx = Some(message_tx.clone());
            self.message_rx = Some(message_rx);
            message_tx
        } else {
            self.message_tx.as_ref().unwrap().clone()
        }
    }

    /// Subscribe to an async stream of events
    ///
    /// Connects any `futures::Stream` to your program's event loop. Each item from
    /// the stream is automatically wrapped in `Event::User` and sent to your model's
    /// update function.
    ///
    /// # Use Cases
    ///
    /// - **WebSocket/SSE**: Stream real-time messages
    /// - **File Watching**: Monitor file system changes
    /// - **Periodic Tasks**: Custom intervals and scheduling
    /// - **Database Changes**: Listen to change streams
    /// - **Sensor Data**: Process continuous data streams
    ///
    /// # Cancellation
    ///
    /// The returned `Subscription` handle allows graceful cancellation. The subscription
    /// is also automatically cancelled when dropped.
    ///
    /// # Examples
    ///
    /// ## Timer Stream
    /// ```ignore
    /// use tokio_stream::wrappers::IntervalStream;
    /// use std::time::Duration;
    ///
    /// let interval = tokio::time::interval(Duration::from_secs(1));
    /// let stream = IntervalStream::new(interval)
    ///     .map(|_| Msg::Tick);
    /// let subscription = program.subscribe(stream);
    /// ```
    ///
    /// ## WebSocket Stream
    /// ```ignore
    /// let ws_stream = websocket.messages()
    ///     .filter_map(|msg| msg.ok())
    ///     .map(|msg| Msg::WebSocketMessage(msg));
    /// let subscription = program.subscribe(ws_stream);
    /// ```
    ///
    /// ## File Watcher Stream
    /// ```ignore
    /// let file_stream = watch_file("/path/to/file")
    ///     .map(|event| Msg::FileChanged(event));
    /// let subscription = program.subscribe(file_stream);
    /// ```
    pub fn subscribe<S>(&mut self, stream: S) -> Subscription
    where
        S: futures::Stream<Item = M::Message> + Send + 'static,
        M::Message: Send + 'static,
    {
        use futures::StreamExt;
        use tokio_util::sync::CancellationToken;

        // Ensure we have a message channel
        if self.message_tx.is_none() {
            self.init_async_bridge();
        }

        let sender = self.message_tx.as_ref().unwrap().clone();
        let cancel_token = CancellationToken::new();
        let cancel_clone = cancel_token.clone();

        // Spawn task to poll the stream
        let handle = self.command_executor.spawn(async move {
            tokio::pin!(stream);

            loop {
                tokio::select! {
                    _ = cancel_clone.cancelled() => {
                        break; // Subscription was cancelled
                    }
                    item = stream.next() => {
                        match item {
                            Some(msg) => {
                                if sender.send(Event::User(msg)).is_err() {
                                    break; // Program has shut down
                                }
                            }
                            None => {
                                break; // Stream completed
                            }
                        }
                    }
                }
            }
        });

        Subscription::new(handle, cancel_token)
    }

    /// Spawn a cancellable async operation
    ///
    /// Spawns a long-running async task with cooperative cancellation support.
    /// The task receives a `CancellationToken` for checking cancellation status.
    ///
    /// # Use Cases
    ///
    /// - **Background Processing**: Data analysis, file processing
    /// - **Network Operations**: Long polling, streaming downloads
    /// - **Periodic Tasks**: Health checks, metrics collection
    /// - **Resource Monitoring**: CPU/memory monitoring
    /// - **Cleanup Tasks**: Temporary file cleanup, cache management
    ///
    /// # Cancellation Pattern
    ///
    /// Tasks should periodically check the cancellation token and exit gracefully
    /// when cancelled. Use `tokio::select!` for responsive cancellation.
    ///
    /// # Examples
    ///
    /// ## Background File Processing
    /// ```ignore
    /// let handle = program.spawn_cancellable(|token| async move {
    ///     for file in large_file_list {
    ///         if token.is_cancelled() {
    ///             return Err("Cancelled");
    ///         }
    ///         process_file(file).await?;
    ///     }
    ///     Ok("All files processed")
    /// });
    /// ```
    ///
    /// ## Long Polling
    /// ```ignore
    /// let handle = program.spawn_cancellable(|token| async move {
    ///     loop {
    ///         tokio::select! {
    ///             _ = token.cancelled() => {
    ///                 break Ok("Polling cancelled");
    ///             }
    ///             result = poll_server() => {
    ///                 handle_result(result)?;
    ///             }
    ///         }
    ///     }
    /// });
    ///
    /// // Cancel when user navigates away
    /// if user_navigated_away {
    ///     handle.cancel().await;
    /// }
    /// ```
    pub fn spawn_cancellable<F, Fut, T>(&self, f: F) -> AsyncHandle<T>
    where
        F: FnOnce(tokio_util::sync::CancellationToken) -> Fut,
        Fut: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        use tokio_util::sync::CancellationToken;

        let cancel_token = CancellationToken::new();
        let token_clone = cancel_token.clone();

        let handle = self.command_executor.spawn(f(token_clone));

        AsyncHandle::new(handle, cancel_token)
    }

    /// Spawn a cancellable async operation that sends messages
    ///
    /// Similar to spawn_cancellable but specifically for operations that produce
    /// messages for the program.
    ///
    /// # Example
    /// ```ignore
    /// let handle = program.spawn_cancellable_cmd(|token, sender| async move {
    ///     while !token.is_cancelled() {
    ///         let data = fetch_data().await;
    ///         let _ = sender.send(Event::User(Msg::DataReceived(data)));
    ///         tokio::time::sleep(Duration::from_secs(1)).await;
    ///     }
    /// });
    /// ```
    pub fn spawn_cancellable_cmd<F, Fut>(&mut self, f: F) -> AsyncHandle<()>
    where
        F: FnOnce(tokio_util::sync::CancellationToken, mpsc::SyncSender<Event<M::Message>>) -> Fut,
        Fut: std::future::Future<Output = ()> + Send + 'static,
        M::Message: Send + 'static,
    {
        use tokio_util::sync::CancellationToken;

        // Ensure we have a message channel
        if self.message_tx.is_none() {
            self.init_async_bridge();
        }

        let sender = self.message_tx.as_ref().unwrap().clone();
        let cancel_token = CancellationToken::new();
        let token_clone = cancel_token.clone();

        let handle = self.command_executor.spawn(f(token_clone, sender));

        AsyncHandle::new(handle, cancel_token)
    }

    /// Run the program with a timeout (useful for testing)
    ///
    /// The program will automatically exit after the specified duration.
    /// Returns Ok(()) if the timeout was reached or the program exited normally.
    pub fn run_with_timeout(self, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        self.run_internal(Some(timeout), Some(start), None)
    }

    /// Run the program until a condition is met (useful for testing)
    ///
    /// The condition function is called after each update with the current model state.
    /// The program exits when the condition returns true.
    pub fn run_until<F>(self, condition: F) -> Result<()>
    where
        F: FnMut(&M) -> bool + 'static,
    {
        self.run_with_condition(Some(Box::new(condition)))
    }

    /// Run the program until it exits
    pub fn run(self) -> Result<()> {
        self.run_internal(None, None, None)
    }

    /// Run with a condition check
    fn run_with_condition(self, condition: Option<Box<dyn FnMut(&M) -> bool>>) -> Result<()> {
        self.run_internal(None, None, condition)
    }

    /// Internal run implementation with optional timeout and condition
    fn run_internal(
        mut self,
        timeout: Option<Duration>,
        start_time: Option<Instant>,
        mut condition: Option<Box<dyn FnMut(&M) -> bool>>,
    ) -> Result<()> {
        // Mark as running
        self.running.store(true, Ordering::SeqCst);

        let (crossterm_tx, crossterm_rx) = mpsc::sync_channel(100);

        // Use existing message channel if initialized, otherwise create new one
        let (message_tx, message_rx) = if let Some(rx) = self.message_rx.take() {
            // Channel was already initialized via init_async_bridge()
            (self.message_tx.as_ref().unwrap().clone(), rx)
        } else {
            let (tx, rx) = mpsc::sync_channel(100);
            self.message_tx = Some(tx.clone());
            (tx, rx)
        };

        // Spawn input thread if not headless
        if !self.options.headless && !self.options.without_renderer {
            let running = Arc::clone(&self.running);
            let force_quit = Arc::clone(&self.force_quit);

            let input_thread = thread::spawn(move || {
                loop {
                    if !running.load(Ordering::SeqCst) || force_quit.load(Ordering::SeqCst) {
                        break;
                    }

                    if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                        if let Ok(event) = event::read() {
                            let _ = crossterm_tx.send(event);
                        }
                    }
                }
            });
            self.input_thread = Some(input_thread);
        }

        // Run initial command
        if let Some(cmd) = self.model.init() {
            self.command_executor.execute(cmd, message_tx.clone());
        }

        // Main event loop
        let tick_rate = Duration::from_millis(250);
        loop {
            if self.force_quit.load(Ordering::SeqCst) {
                break;
            }

            // Check timeout if specified
            if let (Some(timeout), Some(start)) = (timeout, start_time) {
                if start.elapsed() >= timeout {
                    break; // Timeout reached
                }
            }

            // Process events with shorter timeout to allow checking conditions
            let event_timeout = if timeout.is_some() {
                Duration::from_millis(10) // Check more frequently when timeout is set
            } else {
                tick_rate
            };
            let event = if self.options.headless {
                self.priority_processor
                    .process_events_headless(&message_rx, event_timeout)
            } else {
                self.priority_processor
                    .process_events(&message_rx, &crossterm_rx, event_timeout)
            };

            if let Some(event) = event {
                // Check for quit
                if matches!(event, Event::Quit) {
                    break;
                }

                // Apply filter if present
                let event = if let Some(ref filter) = self.filter {
                    filter(&self.model, event)
                } else {
                    Some(event)
                };

                // Update model
                if let Some(event) = event {
                    let result = self.model.update(event);

                    // Check if model returned None (quit)
                    if result.is_none() {
                        break;
                    }

                    if let Some(cmd) = result {
                        self.command_executor.execute(cmd, message_tx.clone());
                    }
                }
            }

            // Check condition if specified
            if let Some(ref mut cond) = condition {
                if cond(&self.model) {
                    break; // Condition met
                }
            }

            // Render if needed and FPS allows
            if !self.options.without_renderer && self.fps_limiter.should_render() {
                self.terminal_manager
                    .draw(|f| {
                        let area = f.area();
                        self.model.view(f, area);
                    })
                    .map_err(Error::from)?;
                self.fps_limiter.mark_rendered();
            }
        }

        // Log final statistics before cleanup
        log::info!(
            "Program shutting down. Final stats: {}",
            self.event_stats_string()
        );

        // Cleanup
        self.running.store(false, Ordering::SeqCst);
        self.terminal_manager.cleanup().map_err(Error::from)?;

        Ok(())
    }
}

impl<M: Model> Drop for Program<M> {
    fn drop(&mut self) {
        let _ = self.terminal_manager.cleanup();

        // Stop the input thread
        self.running.store(false, Ordering::SeqCst);
        self.force_quit.store(true, Ordering::SeqCst);

        if let Some(thread) = self.input_thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_options_all_methods() {
        let options = ProgramOptions::default()
            .with_mouse_mode(MouseMode::CellMotion)
            .with_alt_screen(true)
            .with_bracketed_paste(true)
            .with_focus_reporting(true)
            .with_fps(120)
            .headless()
            .without_signal_handler()
            .without_renderer();

        assert_eq!(options.mouse_mode, MouseMode::CellMotion);
        assert!(options.alt_screen);
        assert!(options.bracketed_paste);
        assert!(options.focus_reporting);
        assert_eq!(options.fps, 120);
        assert!(options.headless);
        assert!(!options.install_signal_handler);
        assert!(options.without_renderer);
    }

    #[test]
    fn test_mouse_mode_default() {
        assert_eq!(MouseMode::default(), MouseMode::None);
    }

    #[test]
    fn test_program_drop() {
        use crate::core::Cmd;

        struct TestModel;
        impl Model for TestModel {
            type Message = ();
            fn update(&mut self, _: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
                None
            }
            fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
        }

        let options = ProgramOptions::default().headless();
        {
            let _program = Program::with_options(TestModel, options).unwrap();
            // Program should clean up when dropped
        }
    }

    #[test]
    fn test_program_methods() {
        use crate::core::Cmd;

        struct TestModel;
        impl Model for TestModel {
            type Message = String;
            fn update(&mut self, _: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
                None
            }
            fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
        }

        let options = ProgramOptions::default().headless();
        let mut program = Program::with_options(TestModel, options).unwrap();

        // Test println and printf
        program.println("test");
        program.printf(format_args!("test {}", 42));

        // Test quit and kill
        program.quit();
        program.kill();

        // Test release and restore terminal
        let _ = program.release_terminal();
        let _ = program.restore_terminal();
    }

    #[test]
    fn test_program_with_filter() {
        use crate::core::Cmd;

        struct TestModel;
        impl Model for TestModel {
            type Message = i32;
            fn update(&mut self, _: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
                None
            }
            fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
        }

        let options = ProgramOptions::default().headless();
        let program = Program::with_options(TestModel, options).unwrap();

        let _filtered = program.with_filter(|_, event| match event {
            Event::User(n) if n > 5 => None,
            _ => Some(event),
        });
    }
}
