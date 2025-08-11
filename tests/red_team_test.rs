//! Red team test for Hojicha library
//! 
//! This test creates a complex TUI application to test the library's functionality
//! and document any issues found during usage.

use hojicha_core::prelude::*;
use hojicha_core::event::{MouseEventKind, MouseButton};
use hojicha_runtime::{Event, Key, KeyEvent, KeyModifiers, MouseEvent, Program, ProgramOptions};
use std::time::Duration;

// Complex application state with multiple views and async operations
#[derive(Debug, Clone)]
struct TaskManager {
    // Current view
    current_view: View,
    
    // Data models
    tasks: Vec<Task>,
    selected_task: usize,
    
    // Input state
    input_buffer: String,
    edit_mode: bool,
    
    // Async state
    loading: bool,
    error_message: Option<String>,
    
    // Stats
    stats: Stats,
}

#[derive(Debug, Clone, PartialEq)]
enum View {
    TaskList,
    TaskDetail,
    CreateTask,
    Settings,
}

#[derive(Debug, Clone)]
struct Task {
    id: usize,
    title: String,
    description: String,
    completed: bool,
    priority: Priority,
}

#[derive(Debug, Clone, PartialEq)]
enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Default)]
struct Stats {
    total_tasks: usize,
    completed_tasks: usize,
    avg_completion_time: Duration,
}

// Messages for the application
#[derive(Debug, Clone)]
enum Msg {
    // Navigation
    ChangeView(View),
    SelectTask(usize),
    ScrollUp,
    ScrollDown,
    
    // Task operations
    CreateTask(String, String),
    UpdateTask(usize, Task),
    DeleteTask(usize),
    ToggleComplete(usize),
    
    // Input handling
    InputChar(char),
    InputBackspace,
    InputSubmit,
    ToggleEditMode,
    
    // Async operations
    LoadTasksStart,
    LoadTasksComplete(Vec<Task>),
    SaveTasksStart,
    SaveTasksComplete(std::result::Result<(), String>),
    
    // Timer/Periodic
    Tick,
    UpdateStats,
    
    // Error handling
    ShowError(String),
    ClearError,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            current_view: View::TaskList,
            tasks: vec![
                Task {
                    id: 1,
                    title: "Test async operations".to_string(),
                    description: "Test that async commands work properly".to_string(),
                    completed: false,
                    priority: Priority::High,
                },
                Task {
                    id: 2,
                    title: "Test component state".to_string(),
                    description: "Ensure components maintain state correctly".to_string(),
                    completed: false,
                    priority: Priority::Medium,
                },
            ],
            selected_task: 0,
            input_buffer: String::new(),
            edit_mode: false,
            loading: false,
            error_message: None,
            stats: Stats::default(),
        }
    }
}

impl Model for TaskManager {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        // Test initialization with multiple commands
        batch(vec![
            // Start with loading tasks
            custom(|| Some(Msg::LoadTasksStart)),
            // Set up periodic updates
            tick(Duration::from_secs(1), || Msg::Tick),
            // Update stats periodically
            every(Duration::from_secs(5), |_| Msg::UpdateStats),
        ])
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::Key(key_event) => self.handle_key(key_event),
            Event::Mouse(mouse_event) => self.handle_mouse(mouse_event),
            Event::User(msg) => self.handle_message(msg),
            Event::Tick => {
                // Handle tick events
                custom(|| Some(Msg::UpdateStats))
            }
            Event::Resize { width: _, height: _ } => {
                // Handle resize
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Simple text representation for headless testing
        let status = format!(
            "TaskManager - View: {:?}, Tasks: {}, Selected: {}, Loading: {}, Error: {:?}",
            self.current_view,
            self.tasks.len(),
            self.selected_task,
            self.loading,
            self.error_message
        );
        
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status),
            area,
        );
    }
}

impl TaskManager {
    fn handle_key(&mut self, key_event: KeyEvent) -> Cmd<Msg> {
        match key_event.key {
            Key::Char('q') if !self.edit_mode => quit(),
            Key::Char('j') | Key::Down => {
                self.selected_task = (self.selected_task + 1).min(self.tasks.len().saturating_sub(1));
                Cmd::none()
            }
            Key::Char('k') | Key::Up => {
                self.selected_task = self.selected_task.saturating_sub(1);
                Cmd::none()
            }
            Key::Char('n') => {
                self.current_view = View::CreateTask;
                self.edit_mode = true;
                Cmd::none()
            }
            Key::Char('d') if !self.edit_mode => {
                let task_id = self.selected_task;
                custom(move || Some(Msg::DeleteTask(task_id)))
            }
            Key::Char(' ') if !self.edit_mode => {
                let task_id = self.selected_task;
                custom(move || Some(Msg::ToggleComplete(task_id)))
            }
            Key::Char('1') => {
                self.current_view = View::TaskList;
                Cmd::none()
            }
            Key::Char('2') => {
                self.current_view = View::TaskDetail;
                Cmd::none()
            }
            Key::Char('3') => {
                self.current_view = View::Settings;
                Cmd::none()
            }
            Key::Char(c) if self.edit_mode => {
                self.input_buffer.push(c);
                Cmd::none()
            }
            Key::Backspace if self.edit_mode => {
                self.input_buffer.pop();
                Cmd::none()
            }
            Key::Enter if self.edit_mode => {
                let title = self.input_buffer.clone();
                self.input_buffer.clear();
                self.edit_mode = false;
                custom(move || Some(Msg::CreateTask(title, "New task".to_string())))
            }
            Key::Esc => {
                self.edit_mode = false;
                self.input_buffer.clear();
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn handle_mouse(&mut self, mouse_event: MouseEvent) -> Cmd<Msg> {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Simulate selecting a task based on y position
                let new_selection = mouse_event.row as usize / 2;
                if new_selection < self.tasks.len() {
                    self.selected_task = new_selection;
                }
                Cmd::none()
            }
            MouseEventKind::ScrollUp => {
                self.selected_task = self.selected_task.saturating_sub(1);
                Cmd::none()
            }
            MouseEventKind::ScrollDown => {
                self.selected_task = (self.selected_task + 1).min(self.tasks.len().saturating_sub(1));
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn handle_message(&mut self, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::LoadTasksStart => {
                self.loading = true;
                // Simulate async loading
                custom_async(|| async {
                    // Simulate network delay
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    
                    let tasks = vec![
                        Task {
                            id: 3,
                            title: "Loaded task 1".to_string(),
                            description: "Async loaded".to_string(),
                            completed: false,
                            priority: Priority::High,
                        },
                        Task {
                            id: 4,
                            title: "Loaded task 2".to_string(),
                            description: "Also async loaded".to_string(),
                            completed: true,
                            priority: Priority::Low,
                        },
                    ];
                    
                    Some(Msg::LoadTasksComplete(tasks))
                })
            }
            Msg::LoadTasksComplete(tasks) => {
                self.loading = false;
                self.tasks.extend(tasks);
                Cmd::none()
            }
            Msg::CreateTask(title, description) => {
                let new_task = Task {
                    id: self.tasks.len() + 1,
                    title,
                    description,
                    completed: false,
                    priority: Priority::Medium,
                };
                self.tasks.push(new_task);
                self.current_view = View::TaskList;
                
                // Trigger save
                custom(|| Some(Msg::SaveTasksStart))
            }
            Msg::DeleteTask(idx) => {
                if idx < self.tasks.len() {
                    self.tasks.remove(idx);
                    if self.selected_task >= self.tasks.len() && self.selected_task > 0 {
                        self.selected_task -= 1;
                    }
                }
                Cmd::none()
            }
            Msg::ToggleComplete(idx) => {
                if let Some(task) = self.tasks.get_mut(idx) {
                    task.completed = !task.completed;
                }
                Cmd::none()
            }
            Msg::SaveTasksStart => {
                self.loading = true;
                custom_async(|| async {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    // Simulate save success (removed rand dependency)
                    Some(Msg::SaveTasksComplete(Ok(())))
                })
            }
            Msg::SaveTasksComplete(result) => {
                self.loading = false;
                match result {
                    Ok(()) => Cmd::none(),
                    Err(err) => {
                        self.error_message = Some(err);
                        // Clear error after 3 seconds
                        tick(Duration::from_secs(3), || Msg::ClearError)
                    }
                }
            }
            Msg::UpdateStats => {
                self.stats.total_tasks = self.tasks.len();
                self.stats.completed_tasks = self.tasks.iter().filter(|t| t.completed).count();
                Cmd::none()
            }
            Msg::ShowError(err) => {
                self.error_message = Some(err);
                tick(Duration::from_secs(3), || Msg::ClearError)
            }
            Msg::ClearError => {
                self.error_message = None;
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_program_creation() {
        let model = TaskManager::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let program = Program::with_options(model, options);
        assert!(program.is_ok(), "Failed to create headless program");
    }

    #[test]
    fn test_async_bridge_initialization() {
        let model = TaskManager::default();
        let options = ProgramOptions::default()
            .headless()
            .without_renderer();
        
        let mut program = Program::with_options(model, options).unwrap();
        let sender = program.init_async_bridge();
        
        // Test sending events through the bridge
        let result = sender.send(Event::User(Msg::Tick));
        assert!(result.is_ok(), "Failed to send event through async bridge");
    }

    #[test]
    fn test_model_initialization() {
        let mut model = TaskManager::default();
        let init_cmd = model.init();
        
        // The init should return a batch command
        assert!(init_cmd.is_batch(), "Init should return a batch command");
    }

    #[test]
    fn test_keyboard_navigation() {
        let mut model = TaskManager::default();
        
        // Test navigation keys
        let cmd = model.update(Event::Key(KeyEvent {
            key: Key::Down,
            modifiers: KeyModifiers::empty(),
        }));
        assert!(cmd.is_noop(), "Down key should return no-op");
        assert_eq!(model.selected_task, 1, "Selected task should increment");
        
        let cmd = model.update(Event::Key(KeyEvent {
            key: Key::Up,
            modifiers: KeyModifiers::empty(),
        }));
        assert!(cmd.is_noop(), "Up key should return no-op");
        assert_eq!(model.selected_task, 0, "Selected task should decrement");
    }

    #[test]
    fn test_task_creation() {
        let mut model = TaskManager::default();
        let initial_count = model.tasks.len();
        
        // Enter create mode
        model.update(Event::Key(KeyEvent {
            key: Key::Char('n'),
            modifiers: KeyModifiers::empty(),
        }));
        assert!(model.edit_mode, "Should enter edit mode");
        assert_eq!(model.current_view, View::CreateTask, "Should switch to create view");
        
        // Type task title
        model.update(Event::Key(KeyEvent {
            key: Key::Char('T'),
            modifiers: KeyModifiers::empty(),
        }));
        model.update(Event::Key(KeyEvent {
            key: Key::Char('e'),
            modifiers: KeyModifiers::empty(),
        }));
        model.update(Event::Key(KeyEvent {
            key: Key::Char('s'),
            modifiers: KeyModifiers::empty(),
        }));
        model.update(Event::Key(KeyEvent {
            key: Key::Char('t'),
            modifiers: KeyModifiers::empty(),
        }));
        assert_eq!(model.input_buffer, "Test", "Input buffer should contain typed text");
        
        // Submit task
        let cmd = model.update(Event::Key(KeyEvent {
            key: Key::Enter,
            modifiers: KeyModifiers::empty(),
        }));
        assert!(!model.edit_mode, "Should exit edit mode");
        assert_eq!(model.input_buffer, "", "Input buffer should be cleared");
        
        // Execute the returned command to create the task
        if let Ok(Some(msg)) = cmd.execute() {
            model.update(Event::User(msg));
        }
        
        assert_eq!(model.tasks.len(), initial_count + 1, "Should have one more task");
    }

    #[test]
    fn test_async_loading() {
        let mut model = TaskManager::default();
        let initial_count = model.tasks.len();
        
        // Start loading
        let _cmd = model.update(Event::User(Msg::LoadTasksStart));
        assert!(model.loading, "Should be in loading state");
        
        // Simulate completion
        let loaded_tasks = vec![
            Task {
                id: 100,
                title: "Test task".to_string(),
                description: "Test".to_string(),
                completed: false,
                priority: Priority::Low,
            }
        ];
        model.update(Event::User(Msg::LoadTasksComplete(loaded_tasks)));
        assert!(!model.loading, "Should not be loading anymore");
        assert_eq!(model.tasks.len(), initial_count + 1, "Should have added loaded task");
    }

    #[test]
    fn test_error_handling() {
        let mut model = TaskManager::default();
        
        // Trigger an error
        model.update(Event::User(Msg::ShowError("Test error".to_string())));
        assert_eq!(model.error_message, Some("Test error".to_string()), "Should have error message");
        
        // Clear error
        model.update(Event::User(Msg::ClearError));
        assert_eq!(model.error_message, None, "Error should be cleared");
    }

    #[test]
    fn test_mouse_interaction() {
        let mut model = TaskManager::default();
        model.tasks = vec![
            Task {
                id: 1,
                title: "Task 1".to_string(),
                description: "".to_string(),
                completed: false,
                priority: Priority::Low,
            },
            Task {
                id: 2,
                title: "Task 2".to_string(),
                description: "".to_string(),
                completed: false,
                priority: Priority::Low,
            },
            Task {
                id: 3,
                title: "Task 3".to_string(),
                description: "".to_string(),
                completed: false,
                priority: Priority::Low,
            },
        ];
        
        // Test click selection
        model.update(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 0,
            row: 4,
            modifiers: KeyModifiers::empty(),
        }));
        assert_eq!(model.selected_task, 2, "Should select third task");
        
        // Test scroll
        model.update(Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::empty(),
        }));
        assert_eq!(model.selected_task, 1, "Should scroll up");
        
        model.update(Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::empty(),
        }));
        assert_eq!(model.selected_task, 2, "Should scroll down");
    }

    #[test]
    fn test_quit_command() {
        let mut model = TaskManager::default();
        
        let cmd = model.update(Event::Key(KeyEvent {
            key: Key::Char('q'),
            modifiers: KeyModifiers::empty(),
        }));
        
        assert!(cmd.is_quit(), "Should return quit command");
    }
}