//! Tests for debugging and tracing features

use hojicha_core::{
    commands, 
    core::{Cmd, Model},
    debug::{DebugConfig, DebugContext, TraceLevel, TraceEvent, inspector::Inspectable},
    event::Event,
};
use ratatui::{Frame, layout::Rect};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone)]
struct TestModel {
    counter: i32,
    messages: Vec<String>,
}

#[derive(Clone, Debug)]
enum TestMsg {
    Increment,
    Decrement,
    SetValue(i32),
}

impl Model for TestModel {
    type Message = TestMsg;

    fn init(&mut self) -> Cmd<Self::Message> {
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        if let Some(msg) = event.into_user() {
            match msg {
                TestMsg::Increment => {
                    self.counter += 1;
                    Cmd::none()
                }
                TestMsg::Decrement => {
                    self.counter -= 1;
                    Cmd::none()
                }
                TestMsg::SetValue(v) => {
                    self.counter = v;
                    Cmd::none()
                }
            }
        } else {
            Cmd::none()
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

#[test]
fn test_trace_level_parsing() {
    let level = TraceLevel::from_str("commands,messages");
    assert!(level.contains(TraceLevel::COMMANDS));
    assert!(level.contains(TraceLevel::MESSAGES));
    assert!(!level.contains(TraceLevel::EVENTS));

    let all = TraceLevel::from_str("all");
    assert_eq!(all, TraceLevel::ALL);
}

#[test]
fn test_debug_config_from_env() {
    // Test with no env vars set
    let config = DebugConfig::disabled();
    assert!(!config.enabled);
    assert_eq!(config.trace_level, TraceLevel::NONE);

    // Test full debug config
    let config = DebugConfig::full_debug();
    assert!(config.enabled);
    assert_eq!(config.trace_level, TraceLevel::ALL);
    assert!(config.collect_metrics);
}

#[test]
fn test_cmd_inspection() {
    let inspected = Arc::new(Mutex::new(Vec::new()));
    let inspected_clone = Arc::clone(&inspected);

    let cmd = Cmd::<TestMsg>::none()
        .inspect(move |cmd| {
            inspected_clone.lock().unwrap().push(cmd.debug_name().to_string());
        });

    // Inspection should have been called
    assert_eq!(inspected.lock().unwrap().len(), 1);
    assert_eq!(inspected.lock().unwrap()[0], "NoOp");
}

#[test]
fn test_conditional_inspection() {
    let inspected = Arc::new(Mutex::new(0));
    
    // Should inspect
    let inspected_clone = Arc::clone(&inspected);
    let _cmd = Cmd::<TestMsg>::none()
        .inspect_if(true, move |_| {
            *inspected_clone.lock().unwrap() += 1;
        });
    assert_eq!(*inspected.lock().unwrap(), 1);

    // Should not inspect
    let inspected_clone = Arc::clone(&inspected);
    let _cmd = Cmd::<TestMsg>::none()
        .inspect_if(false, move |_| {
            *inspected_clone.lock().unwrap() += 1;
        });
    assert_eq!(*inspected.lock().unwrap(), 1); // Still 1
}

#[test]
fn test_cmd_debug_names() {
    assert_eq!(Cmd::<TestMsg>::none().debug_name(), "NoOp");
    assert_eq!(commands::quit::<TestMsg>().debug_name(), "Quit");
    
    let batch = commands::batch::<TestMsg>(vec![
        Cmd::none(),
        Cmd::none(),
    ]);
    assert_eq!(batch.debug_name(), "Batch");

    let tick = commands::tick(Duration::from_millis(100), || TestMsg::Increment);
    assert_eq!(tick.debug_name(), "Tick");
}

#[test]
fn test_inspectable_trait() {
    let value = 42i32;
    let inspected = Arc::new(Mutex::new(false));
    let inspected_clone = Arc::clone(&inspected);
    
    let result = value
        .tap(move |v: &i32| {
            assert_eq!(*v, 42);
            *inspected_clone.lock().unwrap() = true;
        });
    
    assert!(*inspected.lock().unwrap());
    assert_eq!(result, 42);
}

#[test]
fn test_debug_context() {
    let config = DebugConfig::full_debug();
    let context = DebugContext::with_config(config);
    
    assert!(context.is_enabled());
    assert_eq!(context.trace_level(), TraceLevel::ALL);
    
    // Test tracing
    context.trace_event(TraceEvent::Custom {
        label: "TEST".to_string(),
        data: "test data".to_string(),
    });
    
    // Test metrics
    use hojicha_core::debug::FrameMetrics;
    let metrics = FrameMetrics {
        update_duration: Duration::from_millis(10),
        view_duration: Duration::from_millis(5),
        frame_duration: Duration::from_millis(16),
        events_processed: 3,
        commands_executed: 2,
        timestamp: std::time::Instant::now(),
    };
    
    context.record_frame(metrics);
    
    let perf = context.get_metrics();
    assert!(perf.is_some());
}

#[test]
fn test_performance_metrics() {
    use hojicha_core::debug::{PerformanceMetrics, FrameMetrics};
    
    let mut metrics = PerformanceMetrics::new();
    
    // Record some frames
    for i in 0..5 {
        let frame = FrameMetrics {
            update_duration: Duration::from_millis(5),
            view_duration: Duration::from_millis(3),
            frame_duration: Duration::from_millis(16),
            events_processed: i,
            commands_executed: i * 2,
            timestamp: std::time::Instant::now(),
        };
        metrics.record_frame(frame);
    }
    
    let summary = metrics.summary();
    assert_eq!(summary.frame_count, 5);
    assert!(summary.average_fps > 0.0);
    assert!(summary.total_events > 0);
    assert!(summary.total_commands > 0);
}

#[test]
fn test_trace_event_display() {
    use std::time::Instant;
    
    let event = TraceEvent::CommandStart {
        id: 1,
        name: "test_command".to_string(),
        timestamp: Instant::now(),
    };
    
    let display = format!("{}", event);
    assert!(display.contains("CMD_START"));
    assert!(display.contains("test_command"));
    
    let event = TraceEvent::MessageSent {
        id: 2,
        message: "TestMsg::Increment".to_string(),
        timestamp: Instant::now(),
    };
    
    let display = format!("{}", event);
    assert!(display.contains("MSG_SENT"));
    assert!(display.contains("TestMsg::Increment"));
}