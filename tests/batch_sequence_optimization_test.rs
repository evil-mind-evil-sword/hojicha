//! Test to verify and document batch/sequence optimization behavior

use hojicha_core::commands::{batch, batch_strict, sequence, sequence_strict};
use hojicha_core::core::Cmd;
use hojicha_core::prelude::*;

#[derive(Clone)]
enum TestMsg {
    Test,
}

#[test]
fn test_batch_optimization_behavior() {
    // Empty batch returns Cmd::none() for performance
    let empty_batch = batch::<TestMsg>(vec![]);
    assert!(empty_batch.is_noop());
    
    // Single-element batch returns the element directly
    let single = batch(vec![Cmd::<TestMsg>::none()]);
    assert!(single.is_noop()); // Because it returns Cmd::none() directly
    assert!(!single.is_batch()); // NOT a batch!
    
    // Multiple elements return an actual batch
    let multiple = batch(vec![Cmd::<TestMsg>::none(), Cmd::none()]);
    assert!(multiple.is_batch());
}

#[test]
fn test_sequence_optimization_behavior() {
    // Empty sequence returns Cmd::none() for performance
    let empty_seq = sequence::<TestMsg>(vec![]);
    assert!(empty_seq.is_noop());
    
    // Single-element sequence returns the element directly
    let single = sequence(vec![Cmd::<TestMsg>::none()]);
    assert!(single.is_noop()); // Because it returns Cmd::none() directly
    assert!(!single.is_sequence()); // NOT a sequence!
    
    // Multiple elements return an actual sequence
    let multiple = sequence(vec![Cmd::<TestMsg>::none(), Cmd::none()]);
    assert!(multiple.is_sequence());
}

#[test]
fn test_batch_strict_always_returns_batch() {
    // batch_strict always returns a batch, even for 0 or 1 elements
    let empty = batch_strict::<TestMsg>(vec![]);
    assert!(empty.is_batch());
    
    let single = batch_strict(vec![Cmd::<TestMsg>::none()]);
    assert!(single.is_batch());
    
    let multiple = batch_strict(vec![Cmd::<TestMsg>::none(), Cmd::none()]);
    assert!(multiple.is_batch());
}

#[test]
fn test_sequence_strict_always_returns_sequence() {
    // sequence_strict always returns a sequence, even for 0 or 1 elements
    let empty = sequence_strict::<TestMsg>(vec![]);
    assert!(empty.is_sequence());
    
    let single = sequence_strict(vec![Cmd::<TestMsg>::none()]);
    assert!(single.is_sequence());
    
    let multiple = sequence_strict(vec![Cmd::<TestMsg>::none(), Cmd::none()]);
    assert!(multiple.is_sequence());
}