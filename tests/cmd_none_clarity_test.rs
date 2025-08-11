//! Test to clarify the difference between Cmd::none() and Cmd::new(|| None)

use hojicha_core::core::Cmd;
use hojicha_core::prelude::*;

#[derive(Clone)]
enum TestMsg {
    Test,
}

#[test]
fn test_cmd_none_vs_new_none_difference() {
    // Cmd::none() creates a NoOp variant
    let cmd_none = Cmd::<TestMsg>::none();
    assert!(cmd_none.is_noop());
    
    // Cmd::new(|| None) creates a Function variant that returns None
    let cmd_new_none = Cmd::new(|| None);
    assert!(!cmd_new_none.is_noop());
    
    // Both return None when executed, but have different internal representations
    // This is documented in the API to avoid confusion
}

#[test]
fn test_cmd_none_is_idiomatic_for_no_op() {
    // The idiomatic way to return "no command" is Cmd::none()
    let no_op = Cmd::<TestMsg>::none();
    
    // It's specially optimized and clearly indicates intent
    assert!(no_op.is_noop());
    
    // The execute method returns Ok(None) for NoOp
    let result = no_op.execute();
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}