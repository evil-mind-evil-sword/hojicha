//! Integration tests demonstrating the improved test infrastructure

use hojicha_core::{commands, core::{Cmd, Model}, event::{Event, Key}};
use hojicha_runtime::testing::{
    EventTestHarness, TimeControlledHarness, EventScenarioBuilder, TestScenarioBuilder
};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
enum AppMsg {
    StartTimer,
    TimerFired,
    StartRecurring,
    RecurringTick(u32),
    StartAsync,
    AsyncComplete(String),
    Increment,
    Decrement,
    Reset,
}

#[derive(Clone)]
struct TestApp {
    counter: u32,
    timer_fired_count: u32,
    recurring_count: u32,
    async_results: Vec<String>,
}

impl Model for TestApp {
    type Message = AppMsg;
    
    fn init(&mut self) -> Cmd<Self::Message> {
        Cmd::none()
    }
    
    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(AppMsg::StartTimer) => {
                commands::tick(Duration::from_secs(1), || AppMsg::TimerFired)
            }
            Event::User(AppMsg::TimerFired) => {
                self.timer_fired_count += 1;
                Cmd::none()
            }
            Event::User(AppMsg::StartRecurring) => {
                let count = self.recurring_count;
                commands::every(Duration::from_secs(1), move |_| {
                    AppMsg::RecurringTick(count)
                })
            }
            Event::User(AppMsg::RecurringTick(n)) => {
                self.recurring_count = n + 1;
                Cmd::none()
            }
            Event::User(AppMsg::StartAsync) => {
                commands::custom_async(|| async {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Some(AppMsg::AsyncComplete("Done".to_string()))
                })
            }
            Event::User(AppMsg::AsyncComplete(result)) => {
                self.async_results.push(result);
                Cmd::none()
            }
            Event::User(AppMsg::Increment) => {
                self.counter += 1;
                Cmd::none()
            }
            Event::User(AppMsg::Decrement) => {
                self.counter -= 1;
                Cmd::none()
            }
            Event::User(AppMsg::Reset) => {
                self.counter = 0;
                self.timer_fired_count = 0;
                self.recurring_count = 0;
                self.async_results.clear();
                Cmd::none()
            }
            Event::Key(key) => match key.key {
                Key::Up => self.update(Event::User(AppMsg::Increment)),
                Key::Down => self.update(Event::User(AppMsg::Decrement)),
                Key::Char('r') => self.update(Event::User(AppMsg::Reset)),
                Key::Char('t') => self.update(Event::User(AppMsg::StartTimer)),
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }
    
    fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
}

impl Default for TestApp {
    fn default() -> Self {
        Self {
            counter: 0,
            timer_fired_count: 0,
            recurring_count: 0,
            async_results: Vec::new(),
        }
    }
}

#[test]
#[ignore = "Time control requires tokio-test or similar mock timer infrastructure"]
fn test_deterministic_timer_control() {
    let mut harness = TimeControlledHarness::new(TestApp::default());
    
    // Pause time for full control
    harness.pause_time();
    
    // Start a timer
    harness.send_message(AppMsg::StartTimer);
    
    // Timer should not fire immediately
    assert_eq!(harness.model().timer_fired_count, 0);
    assert!(!harness.assert_received(AppMsg::TimerFired));
    
    // Advance time by half the duration
    harness.advance_time(Duration::from_millis(500));
    assert_eq!(harness.model().timer_fired_count, 0);
    
    // Advance to exactly when timer should fire
    harness.advance_time(Duration::from_millis(500));
    
    // Timer should have fired exactly once
    assert!(harness.assert_received(AppMsg::TimerFired));
    assert_eq!(harness.model().timer_fired_count, 1);
}

#[test]
#[ignore = "Time control requires tokio-test or similar mock timer infrastructure"]
fn test_recurring_timer_control() {
    let mut harness = TimeControlledHarness::new(TestApp::default());
    harness.pause_time();
    
    // Start recurring timer
    harness.send_message(AppMsg::StartRecurring);
    
    // Advance time in steps and check each tick
    for expected in 0..5 {
        harness.advance_time(Duration::from_secs(1));
        assert!(harness.assert_received(AppMsg::RecurringTick(expected)));
    }
    
    assert_eq!(harness.model().recurring_count, 5);
}

#[test]
fn test_event_harness_with_ticks() {
    let mut harness = EventTestHarness::new(TestApp::default());
    
    // Send event that creates a tick command
    harness.send_message(AppMsg::StartTimer);
    
    // Verify tick command was created
    assert!(harness.executed_command("tick"));
    
    // Manually trigger the tick
    harness.trigger_ticks();
    
    // Verify the timer fired
    assert!(harness.received(&AppMsg::TimerFired));
    assert_eq!(harness.model().timer_fired_count, 1);
}

#[test]
fn test_event_harness_keyboard_input() {
    let mut harness = EventTestHarness::new(TestApp::default());
    
    // Send keyboard events
    harness.send_key(Key::Up);
    harness.send_key(Key::Up);
    harness.send_key(Key::Down);
    
    // Check the counter state
    assert_eq!(harness.model().counter, 1);
    
    // Send reset
    harness.send_char('r');
    assert_eq!(harness.model().counter, 0);
}

#[test]
#[ignore = "Time control requires tokio-test or similar mock timer infrastructure"]
fn test_scenario_builder_time_controlled() {
    let result = TestScenarioBuilder::new(TestApp::default())
        .send(AppMsg::StartTimer)
        .advance(Duration::from_millis(500))
        .advance(Duration::from_millis(500))
        .expect(AppMsg::TimerFired)
        .send(AppMsg::Increment)
        .send(AppMsg::Increment)
        .run();
    
    assert!(result.is_ok());
}

#[test]
fn test_scenario_builder_events() {
    let result = EventScenarioBuilder::new(TestApp::default())
        .key(Key::Up)
        .key(Key::Up)
        .key(Key::Down)
        .char('t')  // Start timer
        .trigger_ticks()
        .expect_message(AppMsg::TimerFired)
        .run();
    
    match result {
        Ok(harness) => {
            assert_eq!(harness.model().counter, 1);
            assert_eq!(harness.model().timer_fired_count, 1);
        }
        Err(e) => panic!("Scenario failed: {}", e),
    }
}

#[test]
#[ignore = "Time control requires tokio-test or similar mock timer infrastructure"]
fn test_run_until_condition() {
    let mut harness = TimeControlledHarness::new(TestApp::default());
    harness.pause_time();
    
    // Send multiple increment messages
    for _ in 0..5 {
        harness.send_message(AppMsg::Increment);
    }
    
    // Run until counter reaches 5
    let reached = harness.run_until(|model| model.counter >= 5, Duration::from_secs(1));
    assert!(reached);
    assert_eq!(harness.model().counter, 5);
}

#[test]
fn test_message_history() {
    let mut harness = EventTestHarness::new(TestApp::default());
    
    harness.send_message(AppMsg::Increment);
    harness.send_message(AppMsg::Decrement);
    harness.send_message(AppMsg::Reset);
    
    let messages = harness.messages();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0], AppMsg::Increment);
    assert_eq!(messages[1], AppMsg::Decrement);
    assert_eq!(messages[2], AppMsg::Reset);
}

#[test]
fn test_command_history() {
    let mut harness = EventTestHarness::new(TestApp::default());
    
    harness.send_message(AppMsg::StartTimer);
    harness.send_message(AppMsg::StartRecurring);
    harness.send_message(AppMsg::StartAsync);
    
    let history = harness.command_history();
    assert!(history.contains(&"tick".to_string()));
    assert!(history.contains(&"every".to_string()));
    assert!(history.contains(&"async".to_string()));
}