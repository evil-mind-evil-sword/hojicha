//! Message and command inspection utilities

use std::fmt::Debug;

/// Inspector for examining values as they flow through the system
pub struct Inspector {
    enabled: bool,
    prefix: String,
}

impl Inspector {
    /// Create a new inspector
    pub fn new() -> Self {
        Self {
            enabled: true,
            prefix: String::from("[INSPECT]"),
        }
    }

    /// Create an inspector with a custom prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            enabled: true,
            prefix: prefix.into(),
        }
    }

    /// Enable or disable the inspector
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Inspect a value, printing it to stderr
    pub fn inspect<T: Debug>(&self, label: &str, value: &T) {
        if self.enabled {
            eprintln!("{} {}: {:?}", self.prefix, label, value);
        }
    }

    /// Inspect a value with a custom formatter
    pub fn inspect_with<T, F>(&self, label: &str, value: &T, formatter: F)
    where
        F: FnOnce(&T) -> String,
    {
        if self.enabled {
            eprintln!("{} {}: {}", self.prefix, label, formatter(value));
        }
    }

    /// Create a scoped inspector for a specific operation
    pub fn scope(&self, scope_name: &str) -> ScopedInspector {
        ScopedInspector {
            inspector: self,
            scope_name: scope_name.to_string(),
            depth: 0,
        }
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}

/// A scoped inspector that adds context to inspection output
pub struct ScopedInspector<'a> {
    inspector: &'a Inspector,
    scope_name: String,
    depth: usize,
}

impl<'a> ScopedInspector<'a> {
    /// Inspect a value within this scope
    pub fn inspect<T: Debug>(&self, label: &str, value: &T) {
        if self.inspector.enabled {
            let indent = "  ".repeat(self.depth);
            eprintln!(
                "{} {}{}/{}: {:?}",
                self.inspector.prefix, indent, self.scope_name, label, value
            );
        }
    }

    /// Create a nested scope
    pub fn nested(&self, name: &str) -> ScopedInspector {
        ScopedInspector {
            inspector: self.inspector,
            scope_name: format!("{}/{}", self.scope_name, name),
            depth: self.depth + 1,
        }
    }
}

/// Extension trait for adding inspection to command chains
pub trait Inspectable: Sized {
    /// Inspect this value with a label
    fn inspect(self, label: &str) -> Self
    where
        Self: Debug,
    {
        eprintln!("[INSPECT] {}: {:?}", label, self);
        self
    }

    /// Conditionally inspect this value
    fn inspect_if(self, condition: bool, label: &str) -> Self
    where
        Self: Debug,
    {
        if condition {
            eprintln!("[INSPECT] {}: {:?}", label, self);
        }
        self
    }

    /// Inspect with a custom formatter
    fn inspect_with<F>(self, label: &str, f: F) -> Self
    where
        F: FnOnce(&Self) -> String,
    {
        eprintln!("[INSPECT] {}: {}", label, f(&self));
        self
    }

    /// Tap into the value for side effects
    fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self),
    {
        f(&self);
        self
    }

    /// Conditionally tap into the value
    fn tap_if<F>(self, condition: bool, f: F) -> Self
    where
        F: FnOnce(&Self),
    {
        if condition {
            f(&self);
        }
        self
    }
}

// Implement for all types
impl<T> Inspectable for T {}