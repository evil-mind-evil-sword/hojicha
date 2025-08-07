//! Test backend for capturing program output

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// A test backend that captures all output
#[derive(Clone)]
pub struct TestBackend {
    output: Arc<Mutex<Vec<u8>>>,
    lines: Arc<Mutex<Vec<String>>>,
}

impl TestBackend {
    /// Create a new test backend
    pub fn new() -> Self {
        Self {
            output: Arc::new(Mutex::new(Vec::new())),
            lines: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get all captured output as bytes
    pub fn get_output(&self) -> Vec<u8> {
        self.output.lock().unwrap().clone()
    }

    /// Get all captured output as a string
    pub fn get_output_string(&self) -> String {
        String::from_utf8_lossy(&self.get_output()).to_string()
    }

    /// Get captured lines
    pub fn get_lines(&self) -> Vec<String> {
        self.lines.lock().unwrap().clone()
    }

    /// Check if output contains a string
    pub fn contains(&self, text: &str) -> bool {
        self.get_output_string().contains(text)
    }

    /// Clear captured output
    pub fn clear(&self) {
        self.output.lock().unwrap().clear();
        self.lines.lock().unwrap().clear();
    }
}

impl Write for TestBackend {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut output = self.output.lock().unwrap();
        output.extend_from_slice(buf);

        // Also track lines for easier testing
        if let Ok(s) = std::str::from_utf8(buf) {
            let mut lines = self.lines.lock().unwrap();
            for line in s.lines() {
                lines.push(line.to_string());
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Default for TestBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_capture() {
        let mut backend = TestBackend::new();

        write!(backend, "Hello, ").unwrap();
        write!(backend, "World!").unwrap();

        assert_eq!(backend.get_output_string(), "Hello, World!");
        assert!(backend.contains("World"));
    }

    #[test]
    fn test_backend_lines() {
        let mut backend = TestBackend::new();

        writeln!(backend, "Line 1").unwrap();
        writeln!(backend, "Line 2").unwrap();

        let lines = backend.get_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
    }
}
