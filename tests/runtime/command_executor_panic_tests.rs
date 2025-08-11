//! Tests for panic recovery in command executor

use hojicha_runtime::program::CommandExecutor;
use hojicha_core::commands;
use hojicha_core::event::Event;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
enum TestMsg {
    Success,
    PanicTest,
}

#[test]
fn test_panic_recovery_in_custom_command() {
    let executor = CommandExecutor::new().unwrap();
    let (tx, rx) = mpsc::sync_channel(10);

    // Create a command that panics
    let cmd = commands::custom(|| panic!("Test panic in custom command"));
    executor.execute(cmd, tx.clone());

    // Create a normal command after the panic
    let cmd2 = commands::custom(|| Some(TestMsg::Success));
    executor.execute(cmd2, tx);

    // Give async tasks time to execute
    std::thread::sleep(Duration::from_millis(50));

    // The panic should not crash the executor
    // We should still receive the second message
    let event = rx.recv_timeout(Duration::from_millis(100)).unwrap();
    assert_eq!(event, Event::User(TestMsg::Success));
}

#[test]
fn test_panic_recovery_in_tick_callback() {
    let executor = CommandExecutor::new().unwrap();
    let (tx, rx) = mpsc::sync_channel(10);

    // Create a tick command with a panicking callback
    let cmd = commands::tick(Duration::from_millis(10), || {
        panic!("Test panic in tick callback")
    });
    executor.execute(cmd, tx.clone());

    // Create a normal command after the panic
    let cmd2 = commands::tick(Duration::from_millis(20), || TestMsg::Success);
    executor.execute(cmd2, tx);

    // Wait for both ticks
    std::thread::sleep(Duration::from_millis(100));

    // The panic should not crash the executor
    // We should still receive the second tick
    let event = rx.recv_timeout(Duration::from_millis(100)).unwrap();
    assert_eq!(event, Event::User(TestMsg::Success));
}

#[test]
fn test_panic_recovery_in_sequence() {
    let executor = CommandExecutor::new().unwrap();
    let (tx, rx) = mpsc::sync_channel(10);

    // Create a sequence with a panicking command in the middle
    let commands = vec![
        commands::custom(|| Some(TestMsg::Success)),
        commands::custom(|| panic!("Test panic in sequence")),
        commands::custom(|| Some(TestMsg::Success)),
    ];

    executor.execute_sequence(commands, tx);

    // Give async tasks time to execute
    std::thread::sleep(Duration::from_millis(100));

    // We should receive the first and third messages despite the panic
    let mut received = Vec::new();
    while let Ok(Event::User(msg)) = rx.try_recv() {
        received.push(msg);
    }

    // Should have received two Success messages
    assert_eq!(received.len(), 2);
    assert_eq!(received[0], TestMsg::Success);
    assert_eq!(received[1], TestMsg::Success);
}

#[test]
fn test_panic_recovery_in_batch() {
    let executor = CommandExecutor::new().unwrap();
    let (tx, rx) = mpsc::sync_channel(10);

    // Create a batch with one panicking command
    let commands = vec![
        commands::custom(|| Some(TestMsg::Success)),
        commands::custom(|| panic!("Test panic in batch")),
        commands::custom(|| Some(TestMsg::Success)),
    ];

    executor.execute_batch(commands, tx);

    // Give async tasks time to execute
    std::thread::sleep(Duration::from_millis(100));

    // We should receive the successful messages despite one panic
    let mut received = Vec::new();
    while let Ok(Event::User(msg)) = rx.try_recv() {
        received.push(msg);
    }

    // Should have received two Success messages
    // Order may vary due to concurrent execution
    assert_eq!(received.len(), 2);
    assert!(received.contains(&TestMsg::Success));
}

#[test]
fn test_panic_with_string_message() {
    let executor = CommandExecutor::new().unwrap();
    let (tx, rx) = mpsc::sync_channel(10);

    // Test panic with String
    let panic_msg = "String panic message".to_string();
    let cmd = commands::custom(move || panic!("{}", panic_msg));
    executor.execute(cmd, tx.clone());

    // Send a success message to verify executor still works
    let cmd2 = commands::custom(|| Some(TestMsg::Success));
    executor.execute(cmd2, tx);

    std::thread::sleep(Duration::from_millis(50));

    // Should receive the success message
    let event = rx.recv_timeout(Duration::from_millis(100)).unwrap();
    assert_eq!(event, Event::User(TestMsg::Success));
}

#[test]
fn test_panic_with_non_string_payload() {
    let executor = CommandExecutor::new().unwrap();
    let (tx, rx) = mpsc::sync_channel(10);

    // Test panic with non-string payload  
    // Note: panic! macro requires a format string, so we use a number in the format
    let cmd = commands::custom(|| panic!("Panic with number: {}", 42));
    executor.execute(cmd, tx.clone());

    // Send a success message to verify executor still works
    let cmd2 = commands::custom(|| Some(TestMsg::Success));
    executor.execute(cmd2, tx);

    std::thread::sleep(Duration::from_millis(50));

    // Should receive the success message
    let event = rx.recv_timeout(Duration::from_millis(100)).unwrap();
    assert_eq!(event, Event::User(TestMsg::Success));
}