//! Command and message tracing utilities

use std::fmt::{self, Debug, Display};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Trace levels for different types of events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TraceLevel(u8);

impl TraceLevel {
    /// No tracing
    pub const NONE: TraceLevel = TraceLevel(0);
    /// Trace commands only
    pub const COMMANDS: TraceLevel = TraceLevel(1);
    /// Trace messages only
    pub const MESSAGES: TraceLevel = TraceLevel(2);
    /// Trace events only
    pub const EVENTS: TraceLevel = TraceLevel(4);
    /// Trace performance metrics
    pub const METRICS: TraceLevel = TraceLevel(8);
    /// Trace everything
    pub const ALL: TraceLevel = TraceLevel(15);

    /// Check if a specific level is enabled
    pub fn contains(&self, other: TraceLevel) -> bool {
        self.0 & other.0 != 0
    }

    /// Combine trace levels
    pub fn combine(&self, other: TraceLevel) -> TraceLevel {
        TraceLevel(self.0 | other.0)
    }

    /// Parse from string (comma-separated)
    pub fn from_str(s: &str) -> TraceLevel {
        let mut level = TraceLevel::NONE;
        for part in s.split(',') {
            match part.trim().to_lowercase().as_str() {
                "commands" | "cmd" => level = level.combine(TraceLevel::COMMANDS),
                "messages" | "msg" => level = level.combine(TraceLevel::MESSAGES),
                "events" | "evt" => level = level.combine(TraceLevel::EVENTS),
                "metrics" | "perf" => level = level.combine(TraceLevel::METRICS),
                "all" => return TraceLevel::ALL,
                _ => {}
            }
        }
        level
    }
}

impl Default for TraceLevel {
    fn default() -> Self {
        TraceLevel::NONE
    }
}

/// Types of events that can be traced
#[derive(Debug, Clone)]
pub enum TraceEvent {
    /// Command execution started
    CommandStart {
        id: u64,
        name: String,
        timestamp: Instant,
    },
    /// Command execution completed
    CommandEnd {
        id: u64,
        duration: Duration,
        result: String,
    },
    /// Message sent
    MessageSent {
        id: u64,
        message: String,
        timestamp: Instant,
    },
    /// Message received
    MessageReceived {
        id: u64,
        message: String,
        timestamp: Instant,
    },
    /// Event processed
    EventProcessed {
        event_type: String,
        timestamp: Instant,
    },
    /// Frame rendered
    FrameRendered {
        duration: Duration,
        fps: f32,
    },
    /// Custom trace event
    Custom {
        label: String,
        data: String,
    },
}

impl Display for TraceEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraceEvent::CommandStart { id, name, .. } => {
                write!(f, "[CMD_START #{:04}] {}", id, name)
            }
            TraceEvent::CommandEnd { id, duration, result } => {
                write!(f, "[CMD_END   #{:04}] {:?} - {}", id, duration, result)
            }
            TraceEvent::MessageSent { id, message, .. } => {
                write!(f, "[MSG_SENT  #{:04}] {}", id, message)
            }
            TraceEvent::MessageReceived { id, message, .. } => {
                write!(f, "[MSG_RECV  #{:04}] {}", id, message)
            }
            TraceEvent::EventProcessed { event_type, .. } => {
                write!(f, "[EVENT     ] {}", event_type)
            }
            TraceEvent::FrameRendered { duration, fps } => {
                write!(f, "[FRAME     ] {:?} @ {:.1} FPS", duration, fps)
            }
            TraceEvent::Custom { label, data } => {
                write!(f, "[{}] {}", label, data)
            }
        }
    }
}

/// Tracer for recording and outputting debug events
pub struct Tracer {
    level: TraceLevel,
    buffer: VecDeque<TraceEvent>,
    buffer_size: usize,
    next_id: u64,
    start_time: Instant,
}

impl Tracer {
    /// Create a new tracer with the specified level
    pub fn new(level: TraceLevel) -> Self {
        Self {
            level,
            buffer: VecDeque::with_capacity(1000),
            buffer_size: 1000,
            next_id: 1,
            start_time: Instant::now(),
        }
    }

    /// Get the next unique ID for tracing
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Trace an event
    pub fn trace(&mut self, event: TraceEvent) {
        // Check if this type of event should be traced
        let should_trace = match &event {
            TraceEvent::CommandStart { .. } | TraceEvent::CommandEnd { .. } => {
                self.level.contains(TraceLevel::COMMANDS)
            }
            TraceEvent::MessageSent { .. } | TraceEvent::MessageReceived { .. } => {
                self.level.contains(TraceLevel::MESSAGES)
            }
            TraceEvent::EventProcessed { .. } => {
                self.level.contains(TraceLevel::EVENTS)
            }
            TraceEvent::FrameRendered { .. } => {
                self.level.contains(TraceLevel::METRICS)
            }
            TraceEvent::Custom { .. } => true,
        };

        if should_trace {
            // Output immediately
            let elapsed = self.start_time.elapsed();
            eprintln!("[{:>8.3}s] {}", elapsed.as_secs_f32(), event);

            // Store in buffer
            if self.buffer.len() >= self.buffer_size {
                self.buffer.pop_front();
            }
            self.buffer.push_back(event);
        }
    }

    /// Get the current trace buffer
    pub fn get_buffer(&self) -> Vec<TraceEvent> {
        self.buffer.iter().cloned().collect()
    }

    /// Clear the trace buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Flush any pending output
    pub fn flush(&mut self) {
        // In the future, this could write to a file or send to a network endpoint
        // For now, we output to stderr immediately so nothing to flush
    }
}