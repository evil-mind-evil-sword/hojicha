//! Debugging and developer ergonomics tools
//!
//! This module provides comprehensive debugging utilities for hojicha applications,
//! including command tracing, message inspection, and performance metrics.

pub mod tracer;
pub mod inspector;
pub mod metrics;
pub mod config;

pub use tracer::{Tracer, TraceLevel, TraceEvent};
pub use inspector::Inspector;
pub use metrics::{PerformanceMetrics, FrameMetrics};
pub use config::DebugConfig;

use std::sync::{Arc, Mutex};
use std::fmt::Debug;

/// Global debug context that can be shared across the application
#[derive(Clone)]
pub struct DebugContext {
    tracer: Arc<Mutex<Tracer>>,
    metrics: Arc<Mutex<PerformanceMetrics>>,
    config: Arc<DebugConfig>,
}

impl DebugContext {
    /// Create a new debug context with default configuration
    pub fn new() -> Self {
        Self::with_config(DebugConfig::from_env())
    }

    /// Create a new debug context with specific configuration
    pub fn with_config(config: DebugConfig) -> Self {
        Self {
            tracer: Arc::new(Mutex::new(Tracer::new(config.trace_level))),
            metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
            config: Arc::new(config),
        }
    }

    /// Check if debugging is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the current trace level
    pub fn trace_level(&self) -> TraceLevel {
        self.config.trace_level
    }

    /// Trace an event
    pub fn trace_event(&self, event: TraceEvent) {
        if self.is_enabled() {
            if let Ok(mut tracer) = self.tracer.lock() {
                tracer.trace(event);
            }
        }
    }

    /// Record frame metrics
    pub fn record_frame(&self, metrics: FrameMetrics) {
        if self.is_enabled() && self.config.collect_metrics {
            if let Ok(mut perf) = self.metrics.lock() {
                perf.record_frame(metrics);
            }
        }
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> Option<PerformanceMetrics> {
        if self.is_enabled() && self.config.collect_metrics {
            self.metrics.lock().ok().map(|m| m.clone())
        } else {
            None
        }
    }

    /// Flush all debug output
    pub fn flush(&self) {
        if let Ok(mut tracer) = self.tracer.lock() {
            tracer.flush();
        }
    }
}

impl Default for DebugContext {
    fn default() -> Self {
        Self::new()
    }
}

