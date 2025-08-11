//! Tests to verify API consistency improvements

use hojicha_core::commands;
use hojicha_core::commands_v2;
use hojicha_core::core::{Cmd, Message};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
enum TestMsg {
    A,
    B,
    C,
}

impl Message for TestMsg {}

#[test]
fn test_combine_always_returns_batch() {
    // The new combine() should always return a batch, unlike the old batch()
    
    // Empty vector - should return empty batch
    let cmd = commands_v2::combine::<TestMsg>(vec![]);
    assert!(cmd.is_batch(), "combine([]) should return a batch");
    
    // Single command - should return batch with one command
    let single = Cmd::new(|| Some(TestMsg::A));
    let cmd = commands_v2::combine(vec![single]);
    assert!(cmd.is_batch(), "combine([single]) should return a batch");
    
    // Multiple commands - should return batch
    let cmd = commands_v2::combine(vec![
        Cmd::new(|| Some(TestMsg::A)),
        Cmd::new(|| Some(TestMsg::B)),
    ]);
    assert!(cmd.is_batch(), "combine([a, b]) should return a batch");
}

#[test]
fn test_combine_optimized_matches_old_batch() {
    // combine_optimized() should behave exactly like the old batch()
    
    // Empty vector - returns none
    let old_cmd = commands::batch::<TestMsg>(vec![]);
    let new_cmd = commands_v2::combine_optimized::<TestMsg>(vec![]);
    assert!(old_cmd.is_noop() && new_cmd.is_noop(), 
            "Both should return none for empty vector");
    
    // Multiple commands - returns batch
    let old_cmd = commands::batch(vec![
        Cmd::new(|| Some(TestMsg::A)),
        Cmd::new(|| Some(TestMsg::B)),
    ]);
    let new_cmd = commands_v2::combine_optimized(vec![
        Cmd::new(|| Some(TestMsg::A)),
        Cmd::new(|| Some(TestMsg::B)),
    ]);
    assert!(old_cmd.is_batch() && new_cmd.is_batch(),
            "Both should return batch for multiple commands");
}

#[test]
fn test_timer_api_clarity() {
    // Test that new timer APIs compile and have clear semantics
    
    // One-shot timers - clear from the name
    let _after = commands_v2::after(Duration::from_secs(1), || TestMsg::A);
    let _delay = commands_v2::delay(Duration::from_secs(1), || TestMsg::B);
    
    // Recurring timers - clear from the name
    let _interval = commands_v2::interval(Duration::from_secs(1), |_| TestMsg::A);
    let _repeat = commands_v2::repeat(Duration::from_secs(1), |_| TestMsg::B);
    
    // Old API for comparison (less clear)
    let _tick = commands::tick(Duration::from_secs(1), || TestMsg::A);
    let _every = commands::every(Duration::from_secs(1), |_| TestMsg::B);
}

#[test]
fn test_async_api_unification() {
    // Test that async APIs are more unified
    
    // New API - clear and consistent
    let _async1 = commands_v2::async_cmd(async { Some(TestMsg::A) });
    let _async2 = commands_v2::async_with(|| async { Some(TestMsg::B) });
    
    // Old API - two different names for similar operations
    let _spawn = commands::spawn(async { Some(TestMsg::A) });
    let _custom = commands::custom_async(|| async { Some(TestMsg::B) });
}

#[test]
fn test_utility_commands() {
    // Test new utility commands
    
    // Clear message creation
    let _msg = commands_v2::message(|| Some(TestMsg::A));
    let _send = commands_v2::send(TestMsg::B);
    
    // Control commands remain the same
    let _none = commands_v2::none::<TestMsg>();
    let _quit = commands_v2::quit::<TestMsg>();
}

#[test]
fn test_chain_vs_sequence() {
    // Test that chain is clearer than sequence
    
    // Old API
    let _seq = commands::sequence(vec![
        Cmd::new(|| Some(TestMsg::A)),
        Cmd::new(|| Some(TestMsg::B)),
    ]);
    
    // New API - clearer name
    let _chain = commands_v2::chain(vec![
        Cmd::new(|| Some(TestMsg::A)),
        Cmd::new(|| Some(TestMsg::B)),
    ]);
}

#[cfg(test)]
mod migration_examples {
    use super::*;
    
    /// Example of migrating batch commands
    fn migrate_batch_example() {
        // Before
        let _old = commands::batch(vec![
            Cmd::new(|| Some(TestMsg::A)),
            Cmd::new(|| Some(TestMsg::B)),
        ]);
        
        // After - explicit about optimization
        let _new = commands_v2::combine_optimized(vec![
            Cmd::new(|| Some(TestMsg::A)),
            Cmd::new(|| Some(TestMsg::B)),
        ]);
        
        // Or for predictable behavior
        let _new_strict = commands_v2::combine(vec![
            Cmd::new(|| Some(TestMsg::A)),
            Cmd::new(|| Some(TestMsg::B)),
        ]);
    }
    
    /// Example of migrating timer commands
    fn migrate_timer_example() {
        // Before - unclear semantics
        let _old_tick = commands::tick(Duration::from_secs(1), || TestMsg::A);
        let _old_every = commands::every(Duration::from_secs(1), |_| TestMsg::B);
        
        // After - clear semantics
        let _new_after = commands_v2::after(Duration::from_secs(1), || TestMsg::A);
        let _new_interval = commands_v2::interval(Duration::from_secs(1), |_| TestMsg::B);
    }
    
    /// Example of migrating async commands
    fn migrate_async_example() {
        // Before - two different APIs
        let _old1 = commands::spawn(async { Some(TestMsg::A) });
        let _old2 = commands::custom_async(|| async { Some(TestMsg::B) });
        
        // After - unified API
        let _new1 = commands_v2::async_cmd(async { Some(TestMsg::A) });
        let _new2 = commands_v2::async_with(|| async { Some(TestMsg::B) });
    }
}

/// Test that deprecated aliases work correctly
#[test]
#[allow(deprecated)]
fn test_deprecated_aliases() {
    // These should compile but show deprecation warnings
    let _batch = commands_v2::batch(vec![Cmd::new(|| Some(TestMsg::A))]);
    let _batch_strict = commands_v2::batch_strict(vec![Cmd::new(|| Some(TestMsg::A))]);
    let _tick = commands_v2::tick(Duration::from_secs(1), || TestMsg::A);
    let _every = commands_v2::every(Duration::from_secs(1), |_| TestMsg::B);
    let _spawn = commands_v2::spawn(async { Some(TestMsg::A) });
    let _custom_async = commands_v2::custom_async(|| async { Some(TestMsg::B) });
}