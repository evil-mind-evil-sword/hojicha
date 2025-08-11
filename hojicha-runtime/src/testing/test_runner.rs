//! Test runner utilities for testing hojicha applications

use crate::program::{Program, ProgramOptions};
use hojicha_core::{
    core::Model,
    error::Result,
    event::Event,
};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

/// A test runner that provides utilities for testing hojicha applications
pub struct TestRunner<M: Model> {
    program: crate::program::Program<M>,
    timeout: Option<Duration>,
    message_log: Arc<Mutex<Vec<Event<M::Message>>>>,
}

impl<M: Model> TestRunner<M>
where
    M::Message: Clone + Send + 'static,
{
    /// Create a new test runner with the given model
    pub fn new(model: M) -> Result<Self> {
        Self::with_options(model, ProgramOptions::default().headless())
    }

    /// Create a new test runner with custom options
    pub fn with_options(model: M, options: ProgramOptions) -> Result<Self> {
        let program = Program::with_options(model, options)?;
        Ok(Self {
            program,
            timeout: Some(Duration::from_secs(5)), // Default 5 second timeout
            message_log: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Set the timeout for test execution
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Run without any timeout (be careful - may hang!)
    pub fn without_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    /// Initialize the async bridge and return a sender
    pub fn init_async_bridge(&mut self) -> mpsc::SyncSender<Event<M::Message>> {
        self.program.init_async_bridge()
    }

    /// Get a sender if async bridge was initialized
    pub fn sender(&self) -> Option<mpsc::SyncSender<Event<M::Message>>> {
        self.program.sender()
    }

    /// Send a message to the program
    pub fn send_message(&self, msg: M::Message) -> Result<()> {
        self.program.send_message(msg)
    }

    /// Run the program with the configured timeout
    pub fn run(self) -> Result<()> {
        match self.timeout {
            Some(timeout) => self.program.run_with_timeout(timeout),
            None => self.program.run(),
        }
    }

    /// Run until a condition is met
    pub fn run_until<F>(self, condition: F) -> Result<()>
    where
        F: FnMut(&M) -> bool + 'static,
    {
        self.program.run_until(condition)
    }

    /// Run for a specific number of update cycles
    pub fn run_for_updates(self, count: usize) -> Result<()> {
        let counter = Arc::new(Mutex::new(0));
        let target = count;

        self.program.run_until(move |_| {
            let mut c = counter.lock().unwrap();
            *c += 1;
            *c >= target
        })
    }
}

/// Macro to simplify test setup
#[macro_export]
macro_rules! test_runner {
    ($model:expr) => {
        $crate::testing::TestRunner::new($model).unwrap()
    };
    ($model:expr, $timeout:expr) => {
        $crate::testing::TestRunner::new($model)
            .unwrap()
            .with_timeout($timeout)
    };
}
