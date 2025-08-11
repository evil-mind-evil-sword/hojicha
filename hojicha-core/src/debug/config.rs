//! Debug configuration from environment variables

use super::TraceLevel;
use std::env;

/// Debug configuration
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Whether debugging is enabled
    pub enabled: bool,
    /// Trace level for output
    pub trace_level: TraceLevel,
    /// Whether to collect performance metrics
    pub collect_metrics: bool,
    /// Whether to show debug overlay (future feature)
    pub show_overlay: bool,
    /// Output format
    pub format: OutputFormat,
}

/// Output format for debug information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable text
    Text,
    /// JSON format (for tooling)
    Json,
    /// Compact format
    Compact,
}

impl DebugConfig {
    /// Create configuration from environment variables
    ///
    /// Environment variables:
    /// - `HOJICHA_DEBUG`: Enable debugging (1, true, yes)
    /// - `HOJICHA_TRACE`: Trace level (commands,messages,events,metrics,all)
    /// - `HOJICHA_METRICS`: Enable metrics collection (1, true, yes)
    /// - `HOJICHA_DEBUG_FORMAT`: Output format (text, json, compact)
    pub fn from_env() -> Self {
        let enabled = env::var("HOJICHA_DEBUG")
            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

        let trace_level = env::var("HOJICHA_TRACE")
            .map(|v| TraceLevel::from_str(&v))
            .unwrap_or_else(|_| {
                if enabled {
                    TraceLevel::COMMANDS.combine(TraceLevel::MESSAGES)
                } else {
                    TraceLevel::NONE
                }
            });

        let collect_metrics = env::var("HOJICHA_METRICS")
            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(enabled);

        let format = env::var("HOJICHA_DEBUG_FORMAT")
            .ok()
            .and_then(|v| match v.to_lowercase().as_str() {
                "json" => Some(OutputFormat::Json),
                "compact" => Some(OutputFormat::Compact),
                "text" => Some(OutputFormat::Text),
                _ => None,
            })
            .unwrap_or(OutputFormat::Text);

        Self {
            enabled,
            trace_level,
            collect_metrics,
            show_overlay: false, // Future feature
            format,
        }
    }

    /// Create a default configuration with debugging disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            trace_level: TraceLevel::NONE,
            collect_metrics: false,
            show_overlay: false,
            format: OutputFormat::Text,
        }
    }

    /// Create a configuration with all debugging enabled
    pub fn full_debug() -> Self {
        Self {
            enabled: true,
            trace_level: TraceLevel::ALL,
            collect_metrics: true,
            show_overlay: false,
            format: OutputFormat::Text,
        }
    }

    /// Builder method to enable debugging
    pub fn with_debugging(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Builder method to set trace level
    pub fn with_trace_level(mut self, level: TraceLevel) -> Self {
        self.trace_level = level;
        self
    }

    /// Builder method to enable metrics
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.collect_metrics = enabled;
        self
    }

    /// Builder method to set output format
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self::from_env()
    }
}