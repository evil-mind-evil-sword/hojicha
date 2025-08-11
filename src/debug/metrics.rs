//! Performance metrics collection

use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Metrics for a single frame
#[derive(Debug, Clone)]
pub struct FrameMetrics {
    /// Time taken to process update
    pub update_duration: Duration,
    /// Time taken to render view
    pub view_duration: Duration,
    /// Total frame time
    pub frame_duration: Duration,
    /// Number of events processed
    pub events_processed: usize,
    /// Number of commands executed
    pub commands_executed: usize,
    /// Frame timestamp
    pub timestamp: Instant,
}

impl FrameMetrics {
    /// Calculate FPS from frame duration
    pub fn fps(&self) -> f32 {
        if self.frame_duration.as_secs_f32() > 0.0 {
            1.0 / self.frame_duration.as_secs_f32()
        } else {
            0.0
        }
    }
}

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Recent frame metrics (rolling window)
    frames: VecDeque<FrameMetrics>,
    /// Maximum number of frames to keep
    max_frames: usize,
    /// Total events processed
    total_events: usize,
    /// Total commands executed
    total_commands: usize,
    /// Start time
    start_time: Instant,
    /// Current frame being measured
    current_frame: Option<FrameBuilder>,
}

#[derive(Debug, Clone)]
struct FrameBuilder {
    update_start: Option<Instant>,
    update_duration: Option<Duration>,
    view_start: Option<Instant>,
    view_duration: Option<Duration>,
    frame_start: Instant,
    events_processed: usize,
    commands_executed: usize,
}

impl PerformanceMetrics {
    /// Create a new performance metrics collector
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create with a specific frame buffer capacity
    pub fn with_capacity(max_frames: usize) -> Self {
        Self {
            frames: VecDeque::with_capacity(max_frames),
            max_frames,
            total_events: 0,
            total_commands: 0,
            start_time: Instant::now(),
            current_frame: None,
        }
    }

    /// Start measuring a new frame
    pub fn start_frame(&mut self) {
        self.current_frame = Some(FrameBuilder {
            update_start: None,
            update_duration: None,
            view_start: None,
            view_duration: None,
            frame_start: Instant::now(),
            events_processed: 0,
            commands_executed: 0,
        });
    }

    /// Start measuring update phase
    pub fn start_update(&mut self) {
        if let Some(frame) = &mut self.current_frame {
            frame.update_start = Some(Instant::now());
        }
    }

    /// End measuring update phase
    pub fn end_update(&mut self) {
        if let Some(frame) = &mut self.current_frame {
            if let Some(start) = frame.update_start {
                frame.update_duration = Some(start.elapsed());
            }
        }
    }

    /// Start measuring view phase
    pub fn start_view(&mut self) {
        if let Some(frame) = &mut self.current_frame {
            frame.view_start = Some(Instant::now());
        }
    }

    /// End measuring view phase
    pub fn end_view(&mut self) {
        if let Some(frame) = &mut self.current_frame {
            if let Some(start) = frame.view_start {
                frame.view_duration = Some(start.elapsed());
            }
        }
    }

    /// Record an event being processed
    pub fn record_event(&mut self) {
        self.total_events += 1;
        if let Some(frame) = &mut self.current_frame {
            frame.events_processed += 1;
        }
    }

    /// Record a command being executed
    pub fn record_command(&mut self) {
        self.total_commands += 1;
        if let Some(frame) = &mut self.current_frame {
            frame.commands_executed += 1;
        }
    }

    /// End the current frame and record metrics
    pub fn end_frame(&mut self) {
        if let Some(builder) = self.current_frame.take() {
            let metrics = FrameMetrics {
                update_duration: builder.update_duration.unwrap_or_default(),
                view_duration: builder.view_duration.unwrap_or_default(),
                frame_duration: builder.frame_start.elapsed(),
                events_processed: builder.events_processed,
                commands_executed: builder.commands_executed,
                timestamp: builder.frame_start,
            };
            self.record_frame(metrics);
        }
    }

    /// Record frame metrics
    pub fn record_frame(&mut self, metrics: FrameMetrics) {
        // Update totals
        self.total_events += metrics.events_processed;
        self.total_commands += metrics.commands_executed;
        
        // Store frame
        if self.frames.len() >= self.max_frames {
            self.frames.pop_front();
        }
        self.frames.push_back(metrics);
    }

    /// Get average FPS over recent frames
    pub fn average_fps(&self) -> f32 {
        if self.frames.is_empty() {
            return 0.0;
        }

        let total_duration: Duration = self.frames
            .iter()
            .map(|f| f.frame_duration)
            .sum();
        
        if total_duration.as_secs_f32() > 0.0 {
            self.frames.len() as f32 / total_duration.as_secs_f32()
        } else {
            0.0
        }
    }

    /// Get current FPS (based on last frame)
    pub fn current_fps(&self) -> f32 {
        self.frames
            .back()
            .map(|f| f.fps())
            .unwrap_or(0.0)
    }

    /// Get average frame time
    pub fn average_frame_time(&self) -> Duration {
        if self.frames.is_empty() {
            return Duration::ZERO;
        }

        let total: Duration = self.frames
            .iter()
            .map(|f| f.frame_duration)
            .sum();
        
        total / self.frames.len() as u32
    }

    /// Get statistics summary
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            average_fps: self.average_fps(),
            current_fps: self.current_fps(),
            average_frame_time: self.average_frame_time(),
            total_events: self.total_events,
            total_commands: self.total_commands,
            uptime: self.start_time.elapsed(),
            frame_count: self.frames.len(),
        }
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.frames.clear();
        self.total_events = 0;
        self.total_commands = 0;
        self.start_time = Instant::now();
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of performance metrics
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub average_fps: f32,
    pub current_fps: f32,
    pub average_frame_time: Duration,
    pub total_events: usize,
    pub total_commands: usize,
    pub uptime: Duration,
    pub frame_count: usize,
}

impl std::fmt::Display for MetricsSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FPS: {:.1} (avg: {:.1}) | Frame: {:?} | Events: {} | Commands: {} | Uptime: {:?}",
            self.current_fps,
            self.average_fps,
            self.average_frame_time,
            self.total_events,
            self.total_commands,
            self.uptime
        )
    }
}