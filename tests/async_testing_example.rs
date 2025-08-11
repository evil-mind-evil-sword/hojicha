//! Example of testing async and timing-dependent behavior

use hojicha_core::commands;
use hojicha_core::core::{Cmd, Model};
use hojicha_core::event::Event;
use hojicha_runtime::testing::{AsyncTestHarness, CmdTestExt};
use ratatui::layout::Rect;
use ratatui::Frame;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
enum AnimationMsg {
    Tick(String),
    Frame(usize),
    Complete,
}

struct AnimationModel {
    frames: Vec<String>,
    current_frame: usize,
}

impl Model for AnimationModel {
    type Message = AnimationMsg;

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(AnimationMsg::Tick(label)) => {
                self.frames.push(label);
                Cmd::none()
            }
            Event::User(AnimationMsg::Frame(n)) => {
                self.current_frame = n;
                if n < 10 {
                    // Continue animation
                    commands::tick(Duration::from_millis(16), move || {
                        AnimationMsg::Frame(n + 1)
                    })
                } else {
                    // Animation complete
                    commands::custom(|| Some(AnimationMsg::Complete))
                }
            }
            Event::User(AnimationMsg::Complete) => {
                self.frames.push("DONE".to_string());
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut Frame, _area: Rect) {}
}

#[test]
fn test_tick_command_with_harness() {
    let harness = AsyncTestHarness::new();
    
    // Create a tick command
    let tick_cmd = commands::tick(Duration::from_millis(10), || {
        AnimationMsg::Tick("tick_10".to_string())
    });
    
    // Execute and collect messages
    let messages = harness.execute_command(tick_cmd);
    
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0], AnimationMsg::Tick("tick_10".to_string()));
}

#[test]
fn test_batch_with_different_timings() {
    let harness = AsyncTestHarness::new();
    
    // Create a batch with different timing commands
    let batch_cmd = commands::batch(vec![
        commands::tick(Duration::from_millis(5), || {
            AnimationMsg::Tick("tick_5".to_string())
        }),
        commands::custom(|| Some(AnimationMsg::Tick("immediate".to_string()))),
        commands::tick(Duration::from_millis(10), || {
            AnimationMsg::Tick("tick_10".to_string())
        }),
    ]);
    
    // Execute and wait for all ticks to complete
    let messages = harness.execute_and_wait(batch_cmd, Duration::from_millis(20));
    
    // Should have all three messages (batch executes concurrently)
    assert_eq!(messages.len(), 3);
    assert!(messages.contains(&AnimationMsg::Tick("immediate".to_string())));
    assert!(messages.contains(&AnimationMsg::Tick("tick_5".to_string())));
    assert!(messages.contains(&AnimationMsg::Tick("tick_10".to_string())));
}

#[test]
fn test_async_command_execution() {
    let harness = AsyncTestHarness::new();
    
    // Create an async command
    let async_cmd = commands::spawn(async {
        tokio::time::sleep(Duration::from_millis(5)).await;
        Some(AnimationMsg::Complete)
    });
    
    // Execute and wait for result
    let messages = harness.execute_command(async_cmd);
    
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0], AnimationMsg::Complete);
}

#[test]
fn test_immediate_tick_execution() {
    let harness = AsyncTestHarness::new();
    
    // In tests, we can execute tick callbacks immediately
    let msg = harness.execute_tick_now(Duration::from_secs(100), || {
        AnimationMsg::Tick("immediate_tick".to_string())
    });
    
    assert_eq!(msg, AnimationMsg::Tick("immediate_tick".to_string()));
}

#[test]
fn test_cmd_sync_extension() {
    // Test the sync execution extension for simple commands
    let cmd = commands::custom(|| Some(AnimationMsg::Complete));
    let result = cmd.execute_sync();
    
    assert_eq!(result, Some(AnimationMsg::Complete));
}

#[test]
fn test_sequence_with_timing() {
    let harness = AsyncTestHarness::new();
    
    // Create a sequence with timing
    let seq_cmd = commands::sequence(vec![
        commands::custom(|| Some(AnimationMsg::Frame(0))),
        commands::tick(Duration::from_millis(5), || AnimationMsg::Frame(1)),
        commands::tick(Duration::from_millis(5), || AnimationMsg::Frame(2)),
        commands::custom(|| Some(AnimationMsg::Complete)),
    ]);
    
    // Execute and wait for sequence to complete
    // Sequences run one after another, so we need to wait for the sum of all delays
    let messages = harness.execute_and_wait(seq_cmd, Duration::from_millis(30));
    
    // Should have at least the expected messages
    assert!(messages.len() >= 3);
    
    // Check that we got the expected frame messages
    let frame_messages: Vec<_> = messages
        .iter()
        .filter_map(|m| match m {
            AnimationMsg::Frame(n) => Some(*n),
            _ => None,
        })
        .collect();
    
    // Frames should be in order (0, 1, 2)
    assert!(!frame_messages.is_empty());
    for i in 1..frame_messages.len() {
        assert!(frame_messages[i] >= frame_messages[i - 1]);
    }
}