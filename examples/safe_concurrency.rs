//! Examples of safe concurrency patterns in Hojicha
//!
//! This example demonstrates:
//! - Message passing instead of shared state
//! - Request-response pattern with IDs
//! - Cancellation of operations
//! - State machine design

use hojicha::{commands::*, core::*, event::*, prelude::*};
use std::collections::HashMap;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

// ============================================================================
// Messages
// ============================================================================

#[derive(Debug, Clone)]
enum Msg {
    // User actions
    StartDataFetch,
    StartComputation(u32),
    CancelCurrentOperation,
    
    // Async responses
    DataFetched(RequestId, Result<Data, String>),
    ComputationComplete(RequestId, u32),
    
    // State transitions
    Reset,
}

type RequestId = u64;

#[derive(Debug, Clone)]
struct Data {
    items: Vec<String>,
}

// ============================================================================
// State Machine
// ============================================================================

#[derive(Debug, Clone)]
enum AppState {
    Idle,
    FetchingData { request_id: RequestId },
    Computing { request_id: RequestId, value: u32 },
    Ready { data: Data },
    Error { message: String },
}

// ============================================================================
// Model - No shared mutable state!
// ============================================================================

struct SafeConcurrencyModel {
    state: AppState,
    next_request_id: RequestId,
    pending_requests: HashMap<RequestId, RequestInfo>,
    cancellation_token: Option<CancellationToken>,
    computation_results: Vec<u32>,
}

#[derive(Debug)]
struct RequestInfo {
    request_type: RequestType,
    started_at: std::time::Instant,
}

#[derive(Debug)]
enum RequestType {
    DataFetch,
    Computation(u32),
}

impl SafeConcurrencyModel {
    fn new() -> Self {
        Self {
            state: AppState::Idle,
            next_request_id: 1,
            pending_requests: HashMap::new(),
            cancellation_token: None,
            computation_results: Vec::new(),
        }
    }
    
    fn get_next_request_id(&mut self) -> RequestId {
        let id = self.next_request_id;
        self.next_request_id += 1;
        id
    }
    
    fn is_valid_request(&self, id: RequestId) -> bool {
        self.pending_requests.contains_key(&id)
    }
    
    fn complete_request(&mut self, id: RequestId) -> Option<RequestInfo> {
        self.pending_requests.remove(&id)
    }
}

// ============================================================================
// Update - All state changes happen here
// ============================================================================

impl Model for SafeConcurrencyModel {
    type Message = Msg;
    
    fn init(&mut self) -> Cmd<Self::Message> {
        // No shared state initialization needed!
        Cmd::none()
    }
    
    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            // ----------------------------------------------------------------
            // User Actions
            // ----------------------------------------------------------------
            Event::User(Msg::StartDataFetch) => {
                // Cancel any existing operation
                if let Some(token) = &self.cancellation_token {
                    token.cancel();
                }
                
                let request_id = self.get_next_request_id();
                self.pending_requests.insert(
                    request_id,
                    RequestInfo {
                        request_type: RequestType::DataFetch,
                        started_at: std::time::Instant::now(),
                    },
                );
                
                // Update state machine
                self.state = AppState::FetchingData { request_id };
                
                // Create cancellable async operation
                let token = CancellationToken::new();
                self.cancellation_token = Some(token.clone());
                
                // Safe async operation - returns message, no shared state!
                custom_async(move || async move {
                    tokio::select! {
                        _ = token.cancelled() => {
                            // Operation was cancelled
                            None
                        }
                        _ = tokio::time::sleep(Duration::from_secs(2)) => {
                            // Simulate async data fetch
                            let data = Data {
                                items: vec!["Item 1".into(), "Item 2".into()],
                            };
                            Some(Msg::DataFetched(request_id, Ok(data)))
                        }
                    }
                })
            }
            
            Event::User(Msg::StartComputation(value)) => {
                let request_id = self.get_next_request_id();
                self.pending_requests.insert(
                    request_id,
                    RequestInfo {
                        request_type: RequestType::Computation(value),
                        started_at: std::time::Instant::now(),
                    },
                );
                
                self.state = AppState::Computing { request_id, value };
                
                // Concurrent computation - no shared state!
                custom_async(move || async move {
                    // Simulate expensive computation
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let result = value * value;
                    Some(Msg::ComputationComplete(request_id, result))
                })
            }
            
            Event::User(Msg::CancelCurrentOperation) => {
                if let Some(token) = &self.cancellation_token {
                    token.cancel();
                }
                self.cancellation_token = None;
                self.state = AppState::Idle;
                self.pending_requests.clear();
                Cmd::none()
            }
            
            // ----------------------------------------------------------------
            // Async Responses - Only process if request is still valid
            // ----------------------------------------------------------------
            Event::User(Msg::DataFetched(request_id, result)) => {
                // Verify this is a response to a request we're still tracking
                if let Some(info) = self.complete_request(request_id) {
                    // Valid response - update state
                    match result {
                        Ok(data) => {
                            self.state = AppState::Ready { data };
                            println!("Data fetched successfully after {:?}", 
                                     info.started_at.elapsed());
                        }
                        Err(error) => {
                            self.state = AppState::Error { message: error };
                        }
                    }
                } else {
                    // Ignore responses for cancelled/unknown requests
                    println!("Ignoring response for unknown request {}", request_id);
                }
                Cmd::none()
            }
            
            Event::User(Msg::ComputationComplete(request_id, result)) => {
                if let Some(info) = self.complete_request(request_id) {
                    self.computation_results.push(result);
                    self.state = AppState::Idle;
                    println!("Computation {} completed: {} (took {:?})",
                             request_id, result, info.started_at.elapsed());
                }
                Cmd::none()
            }
            
            Event::User(Msg::Reset) => {
                // Clean reset - no shared state to worry about!
                *self = Self::new();
                Cmd::none()
            }
            
            // ----------------------------------------------------------------
            // Keyboard handling
            // ----------------------------------------------------------------
            Event::Key(key) => match key.key {
                Key::Char('f') => self.update(Event::User(Msg::StartDataFetch)),
                Key::Char('c') => self.update(Event::User(Msg::CancelCurrentOperation)),
                Key::Char('r') => self.update(Event::User(Msg::Reset)),
                Key::Char('q') => quit(),
                Key::Char(n) if n.is_ascii_digit() => {
                    let value = n.to_digit(10).unwrap();
                    self.update(Event::User(Msg::StartComputation(value)))
                }
                _ => Cmd::none(),
            },
            
            _ => Cmd::none(),
        }
    }
    
    fn view(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        use ratatui::{
            style::{Color, Style},
            text::{Line, Span},
            widgets::{Block, Borders, Paragraph},
        };
        
        let state_color = match &self.state {
            AppState::Idle => Color::Green,
            AppState::FetchingData { .. } => Color::Yellow,
            AppState::Computing { .. } => Color::Cyan,
            AppState::Ready { .. } => Color::Blue,
            AppState::Error { .. } => Color::Red,
        };
        
        let state_text = match &self.state {
            AppState::Idle => "Idle - Press 'f' to fetch data, 1-9 to compute".into(),
            AppState::FetchingData { request_id } => 
                format!("Fetching data (request {})...", request_id),
            AppState::Computing { request_id, value } => 
                format!("Computing {}² (request {})...", value, request_id),
            AppState::Ready { data } => 
                format!("Ready - {} items loaded", data.items.len()),
            AppState::Error { message } => 
                format!("Error: {}", message),
        };
        
        let mut lines = vec![
            Line::from(vec![
                Span::raw("State: "),
                Span::styled(state_text, Style::default().fg(state_color)),
            ]),
            Line::from(""),
            Line::from(format!("Pending requests: {}", self.pending_requests.len())),
            Line::from(format!("Computation results: {:?}", self.computation_results)),
            Line::from(""),
            Line::from("Controls:"),
            Line::from("  f - Fetch data"),
            Line::from("  1-9 - Start computation"),
            Line::from("  c - Cancel current operation"),
            Line::from("  r - Reset"),
            Line::from("  q - Quit"),
        ];
        
        // Show pending requests
        if !self.pending_requests.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from("Pending:"));
            for (id, info) in &self.pending_requests {
                lines.push(Line::from(format!(
                    "  Request {}: {:?} ({}ms ago)",
                    id,
                    info.request_type,
                    info.started_at.elapsed().as_millis()
                )));
            }
        }
        
        let widget = Paragraph::new(lines)
            .block(Block::default()
                .title("Safe Concurrency Example")
                .borders(Borders::ALL));
        
        frame.render_widget(widget, area);
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Create and run the program
    let model = SafeConcurrencyModel::new();
    Program::new(model)?.run()
}

// ============================================================================
// Tests demonstrating safety
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_no_shared_state() {
        // This model has no Arc<Mutex<T>> or other shared state
        let model = SafeConcurrencyModel::new();
        
        // All fields are owned by the model
        assert_eq!(model.next_request_id, 1);
        assert!(model.pending_requests.is_empty());
    }
    
    #[test]
    fn test_request_tracking() {
        let mut model = SafeConcurrencyModel::new();
        
        // Generate request IDs
        let id1 = model.get_next_request_id();
        let id2 = model.get_next_request_id();
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_state_transitions() {
        let mut model = SafeConcurrencyModel::new();
        
        // State should start as Idle
        assert!(matches!(model.state, AppState::Idle));
        
        // Transition to fetching
        let _ = model.update(Event::User(Msg::StartDataFetch));
        assert!(matches!(model.state, AppState::FetchingData { .. }));
        
        // Reset should return to idle
        let _ = model.update(Event::User(Msg::Reset));
        assert!(matches!(model.state, AppState::Idle));
    }
}