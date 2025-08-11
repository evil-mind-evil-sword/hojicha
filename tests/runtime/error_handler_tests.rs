//! Tests for error handling in fallible commands

use hojicha_runtime::program::CommandExecutor;
use hojicha_runtime::program::error_handler::{EventErrorHandler, ErrorHandler};
use hojicha_core::commands;
use hojicha_core::error::Error;
use hojicha_core::event::Event;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
enum TestMsg {
    Success,
    ErrorOccurred(String),
}

#[test]
fn test_fallible_command_error_handling() {
    let (tx, rx) = mpsc::sync_channel(10);
    
    // Create executor with custom error handler that converts errors to events
    let error_handler = EventErrorHandler::new(|err| {
        TestMsg::ErrorOccurred(format!("Handled: {}", err))
    });
    let executor = CommandExecutor::with_error_handler(error_handler).unwrap();

    // Execute a fallible command that fails
    let cmd = commands::custom_fallible(|| {
        Err(Error::Model("Test error from fallible command".to_string()))
    });
    executor.execute(cmd, tx.clone());

    // Execute a successful command to verify the executor still works
    let cmd2 = commands::custom(|| Some(TestMsg::Success));
    executor.execute(cmd2, tx);

    // Give async tasks time to execute
    std::thread::sleep(Duration::from_millis(100));

    // Collect all events (order may vary due to async execution)
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }

    // We should have received both events
    assert_eq!(events.len(), 2);
    
    // Check that we got both the error and success events (order may vary)
    let has_error = events.iter().any(|e| matches!(e, 
        Event::User(TestMsg::ErrorOccurred(msg)) if msg.contains("Test error from fallible command")
    ));
    let has_success = events.iter().any(|e| matches!(e, Event::User(TestMsg::Success)));
    
    assert!(has_error, "Should have received error event");
    assert!(has_success, "Should have received success event");
}

#[test]
fn test_fallible_command_with_default_handler() {
    let (tx, rx) = mpsc::sync_channel(10);
    
    // Create executor with default error handler (just logs)
    let executor = CommandExecutor::<TestMsg>::new().unwrap();

    // Execute a fallible command that fails
    let cmd = commands::custom_fallible(|| {
        Err(Error::Model("Test error - should be logged".to_string()))
    });
    executor.execute(cmd, tx.clone());

    // Execute a successful command
    let cmd2 = commands::custom(|| Some(TestMsg::Success));
    executor.execute(cmd2, tx);

    // Give async tasks time to execute
    std::thread::sleep(Duration::from_millis(50));

    // With default handler, we should only receive the success message
    // The error is logged but not sent as an event
    let event = rx.recv_timeout(Duration::from_millis(100)).unwrap();
    assert_eq!(event, Event::User(TestMsg::Success));

    // Should be no more events
    assert!(rx.try_recv().is_err());
}

#[test]
fn test_sequence_with_fallible_commands() {
    let (tx, rx) = mpsc::sync_channel(10);
    
    // Create executor with event error handler
    let error_handler = EventErrorHandler::new(|err| {
        TestMsg::ErrorOccurred(err.to_string())
    });
    let executor = CommandExecutor::with_error_handler(error_handler).unwrap();

    // Create a sequence with both successful and failing commands
    let commands = vec![
        commands::custom(|| Some(TestMsg::Success)),
        commands::custom_fallible(|| {
            Err(Error::Model("Error in sequence".to_string()))
        }),
        commands::custom(|| Some(TestMsg::Success)),
    ];

    executor.execute_sequence(commands, tx);

    // Give async tasks time to execute
    std::thread::sleep(Duration::from_millis(100));

    // Collect all events
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if let Event::User(msg) = event {
            events.push(msg);
        }
    }

    // Should have all three results: success, error, success
    assert_eq!(events.len(), 3);
    assert_eq!(events[0], TestMsg::Success);
    assert_eq!(events[1], TestMsg::ErrorOccurred("Model error: Error in sequence".to_string()));
    assert_eq!(events[2], TestMsg::Success);
}

#[test]
fn test_batch_with_fallible_commands() {
    let (tx, rx) = mpsc::sync_channel(10);
    
    // Create executor with event error handler
    let error_handler = EventErrorHandler::new(|err| {
        TestMsg::ErrorOccurred(err.to_string())
    });
    let executor = CommandExecutor::with_error_handler(error_handler).unwrap();

    // Create a batch with both successful and failing commands
    let commands = vec![
        commands::custom(|| Some(TestMsg::Success)),
        commands::custom_fallible(|| {
            Err(Error::Model("Error in batch".to_string()))
        }),
        commands::custom(|| Some(TestMsg::Success)),
    ];

    executor.execute_batch(commands, tx);

    // Give async tasks time to execute
    std::thread::sleep(Duration::from_millis(100));

    // Collect all events
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if let Event::User(msg) = event {
            events.push(msg);
        }
    }

    // Should have all three results (order may vary due to concurrent execution)
    assert_eq!(events.len(), 3);
    assert!(events.contains(&TestMsg::Success));
    assert!(events.contains(&TestMsg::ErrorOccurred("Model error: Error in batch".to_string())));
}

#[test]
fn test_error_chain_handling() {
    let (tx, rx) = mpsc::sync_channel(10);
    
    // Create executor with event error handler
    let error_handler = EventErrorHandler::new(|err| {
        TestMsg::ErrorOccurred(err.to_string())
    });
    let executor = CommandExecutor::with_error_handler(error_handler).unwrap();

    // Create an IO error with a cause chain
    let io_error = std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "File not found: test.txt"
    );
    let cmd = commands::custom_fallible(move || {
        Err(Error::Io(io_error))
    });
    
    executor.execute(cmd, tx);

    // Give async task time to execute
    std::thread::sleep(Duration::from_millis(50));

    // Should receive the error event
    let event = rx.recv_timeout(Duration::from_millis(100)).unwrap();
    if let Event::User(TestMsg::ErrorOccurred(msg)) = event {
        // IO errors have a different format
        assert!(msg.contains("not found") || msg.contains("Not found"));
    } else {
        panic!("Expected error event");
    }
}