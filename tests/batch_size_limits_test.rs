//! Test batch command size limits and chunking

use hojicha_core::commands;
use hojicha_core::core::Cmd;

#[derive(Debug, Clone, PartialEq)]
enum TestMsg {
    Item(usize),
}

#[test]
fn test_small_batch_unchanged() {
    // Small batches should work normally
    let cmds: Vec<Cmd<TestMsg>> = (0..10)
        .map(|i| commands::custom(move || Some(TestMsg::Item(i))))
        .collect();
    
    let batch_cmd = commands::batch(cmds);
    assert!(batch_cmd.is_batch());
}

#[test]
fn test_empty_batch() {
    // Empty batch should return none
    let cmds: Vec<Cmd<TestMsg>> = vec![];
    let batch_cmd = commands::batch(cmds);
    assert!(batch_cmd.is_noop());
}

#[test]
fn test_single_item_batch() {
    // Single item should be returned directly (optimization)
    let cmds = vec![commands::custom(|| Some(TestMsg::Item(0)))];
    let batch_cmd = commands::batch(cmds);
    // Single item is returned directly, not as a batch
    assert!(!batch_cmd.is_batch());
}

#[test]
fn test_batch_with_limit() {
    // Test explicit limit function
    let cmds: Vec<Cmd<TestMsg>> = (0..200)
        .map(|i| commands::custom(move || Some(TestMsg::Item(i))))
        .collect();
    
    let batch_cmd = commands::batch_with_limit(cmds, 50);
    // Should be chunked into multiple batches
    assert!(batch_cmd.is_batch());
}

#[test]
#[cfg(debug_assertions)]
fn test_large_batch_warning() {
    // In debug mode, large batches should trigger a warning
    // We can't easily test stderr output, but we can verify the batch is created
    let cmds: Vec<Cmd<TestMsg>> = (0..150)
        .map(|i| commands::custom(move || Some(TestMsg::Item(i))))
        .collect();
    
    let batch_cmd = commands::batch(cmds);
    assert!(batch_cmd.is_batch());
}

#[test]
fn test_very_large_batch_chunking() {
    // Very large batches (>1000) should be automatically chunked
    let cmds: Vec<Cmd<TestMsg>> = (0..1500)
        .map(|i| commands::custom(move || Some(TestMsg::Item(i))))
        .collect();
    
    let batch_cmd = commands::batch(cmds);
    // Should still be a batch (of batches)
    assert!(batch_cmd.is_batch());
}

#[test]
fn test_batch_strict_always_batches() {
    // batch_strict should always return a batch
    let cmds = vec![];
    let batch_cmd = commands::batch_strict::<TestMsg>(cmds);
    assert!(batch_cmd.is_batch());
    
    let cmds = vec![commands::custom(|| Some(TestMsg::Item(0)))];
    let batch_cmd = commands::batch_strict(cmds);
    assert!(batch_cmd.is_batch());
}

#[test]
fn test_sequence_no_size_limit() {
    // Sequences don't have size limits (they run sequentially anyway)
    let cmds: Vec<Cmd<TestMsg>> = (0..1500)
        .map(|i| commands::custom(move || Some(TestMsg::Item(i))))
        .collect();
    
    let seq_cmd = commands::sequence(cmds);
    // Large sequences are allowed (no chunking)
    assert!(seq_cmd.is_sequence());
}