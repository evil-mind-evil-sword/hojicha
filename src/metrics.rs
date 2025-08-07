//! Advanced performance metrics and percentile latency tracking
//!
//! This module provides comprehensive performance monitoring capabilities
//! for production applications, including percentile latencies, throughput
//! metrics, and queue utilization statistics.

use hdrhistogram::Histogram;
use log::trace;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Advanced event processing statistics with percentile tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedEventStats {
    /// Basic event counts
    pub basic: BasicStats,

    /// Latency percentiles by priority
    pub latency: LatencyStats,

    /// Throughput metrics
    pub throughput: ThroughputStats,

    /// Queue utilization statistics
    pub queue: QueueStats,

    /// Time-windowed statistics
    pub windows: WindowedStats,
}

/// Basic event statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BasicStats {
    pub total_events: usize,
    pub high_priority_events: usize,
    pub normal_priority_events: usize,
    pub low_priority_events: usize,
    pub dropped_events: usize,
    pub backpressure_activations: usize,
}

/// Latency statistics with percentiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    /// High priority event latencies
    pub high_priority: LatencyPercentiles,

    /// Normal priority event latencies
    pub normal_priority: LatencyPercentiles,

    /// Low priority event latencies
    pub low_priority: LatencyPercentiles,

    /// Overall latencies across all priorities
    pub overall: LatencyPercentiles,

    /// Event type specific latencies
    pub by_type: HashMap<String, LatencyPercentiles>,
}

/// Latency percentiles in microseconds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    pub min: u64,
    pub p50: u64,
    pub p75: u64,
    pub p90: u64,
    pub p95: u64,
    pub p99: u64,
    pub p999: u64,
    pub max: u64,
    pub mean: f64,
    pub std_dev: f64,
    pub count: u64,
}

impl Default for LatencyPercentiles {
    fn default() -> Self {
        Self {
            min: 0,
            p50: 0,
            p75: 0,
            p90: 0,
            p95: 0,
            p99: 0,
            p999: 0,
            max: 0,
            mean: 0.0,
            std_dev: 0.0,
            count: 0,
        }
    }
}

/// Throughput metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputStats {
    /// Current events per second
    pub current_rate: f64,

    /// Peak rate observed
    pub peak_rate: f64,

    /// Average rate over last minute
    pub avg_rate_1m: f64,

    /// Average rate over last 5 minutes
    pub avg_rate_5m: f64,

    /// Average processing time per event (microseconds)
    pub avg_processing_time_us: f64,
}

impl Default for ThroughputStats {
    fn default() -> Self {
        Self {
            current_rate: 0.0,
            peak_rate: 0.0,
            avg_rate_1m: 0.0,
            avg_rate_5m: 0.0,
            avg_processing_time_us: 0.0,
        }
    }
}

/// Queue utilization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    /// Current queue depth
    pub current_depth: usize,

    /// Maximum depth reached
    pub max_depth: usize,

    /// Average depth over time
    pub avg_depth: f64,

    /// Percentage of time at capacity
    pub saturation_percentage: f64,

    /// Queue growth rate (events/sec)
    pub growth_rate: f64,
}

impl Default for QueueStats {
    fn default() -> Self {
        Self {
            current_depth: 0,
            max_depth: 0,
            avg_depth: 0.0,
            saturation_percentage: 0.0,
            growth_rate: 0.0,
        }
    }
}

/// Time-windowed statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowedStats {
    /// Last 60 seconds (1-second buckets)
    pub last_minute: Vec<BucketStats>,

    /// Last hour (1-minute buckets)
    pub last_hour: Vec<BucketStats>,
}

impl Default for WindowedStats {
    fn default() -> Self {
        Self {
            last_minute: Vec::new(),
            last_hour: Vec::new(),
        }
    }
}

/// Statistics for a time bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketStats {
    pub timestamp: u64,
    pub events_processed: usize,
    pub events_dropped: usize,
    pub avg_latency_us: f64,
    pub p99_latency_us: u64,
}

/// Configuration for metrics collection
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Enable percentile tracking
    pub track_percentiles: bool,

    /// Track metrics by event type
    pub track_by_type: bool,

    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,

    /// Maximum histogram size (limits memory usage)
    pub max_histogram_size: u64,

    /// Window size for rate calculations
    pub rate_window: Duration,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            track_percentiles: true,
            track_by_type: false,
            sampling_rate: 1.0,
            max_histogram_size: 100_000,
            rate_window: Duration::from_secs(60),
        }
    }
}

/// Latency tracker using HDR Histogram for efficient percentile calculation
struct LatencyTracker {
    histogram: Histogram<u64>,
}

impl LatencyTracker {
    fn new(max_value: u64) -> Self {
        let histogram = Histogram::new_with_max(max_value, 3).expect("Failed to create histogram");
        Self { histogram }
    }

    fn record(&mut self, latency_us: u64) {
        let _ = self.histogram.record(latency_us);
    }

    fn percentiles(&self) -> LatencyPercentiles {
        if self.histogram.len() == 0 {
            return LatencyPercentiles::default();
        }

        LatencyPercentiles {
            min: self.histogram.min(),
            p50: self.histogram.value_at_percentile(50.0),
            p75: self.histogram.value_at_percentile(75.0),
            p90: self.histogram.value_at_percentile(90.0),
            p95: self.histogram.value_at_percentile(95.0),
            p99: self.histogram.value_at_percentile(99.0),
            p999: self.histogram.value_at_percentile(99.9),
            max: self.histogram.max(),
            mean: self.histogram.mean(),
            std_dev: self.histogram.stdev(),
            count: self.histogram.len(),
        }
    }

    fn reset(&mut self) {
        self.histogram.reset();
    }
}

/// Metrics collector for event processing
pub struct MetricsCollector {
    config: MetricsConfig,
    start_time: Instant,

    // Basic counters
    basic: Arc<Mutex<BasicStats>>,

    // Latency tracking
    high_priority_latency: Arc<Mutex<LatencyTracker>>,
    normal_priority_latency: Arc<Mutex<LatencyTracker>>,
    low_priority_latency: Arc<Mutex<LatencyTracker>>,
    overall_latency: Arc<Mutex<LatencyTracker>>,
    by_type_latency: Arc<Mutex<HashMap<String, LatencyTracker>>>,

    // Throughput tracking
    event_times: Arc<Mutex<Vec<Instant>>>,
    processing_times: Arc<Mutex<Vec<Duration>>>,
    peak_rate: Arc<Mutex<f64>>,

    // Queue tracking
    queue_depths: Arc<Mutex<Vec<(Instant, usize)>>>,
    max_queue_depth: Arc<Mutex<usize>>,
    time_at_capacity: Arc<Mutex<Duration>>,
    last_capacity_check: Arc<Mutex<Instant>>,

    // Windowed stats
    minute_buckets: Arc<Mutex<Vec<BucketStats>>>,
    hour_buckets: Arc<Mutex<Vec<BucketStats>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        let max_latency = 10_000_000; // 10 seconds in microseconds

        Self {
            config,
            start_time: Instant::now(),
            basic: Arc::new(Mutex::new(BasicStats::default())),
            high_priority_latency: Arc::new(Mutex::new(LatencyTracker::new(max_latency))),
            normal_priority_latency: Arc::new(Mutex::new(LatencyTracker::new(max_latency))),
            low_priority_latency: Arc::new(Mutex::new(LatencyTracker::new(max_latency))),
            overall_latency: Arc::new(Mutex::new(LatencyTracker::new(max_latency))),
            by_type_latency: Arc::new(Mutex::new(HashMap::new())),
            event_times: Arc::new(Mutex::new(Vec::new())),
            processing_times: Arc::new(Mutex::new(Vec::new())),
            peak_rate: Arc::new(Mutex::new(0.0)),
            queue_depths: Arc::new(Mutex::new(Vec::new())),
            max_queue_depth: Arc::new(Mutex::new(0)),
            time_at_capacity: Arc::new(Mutex::new(Duration::ZERO)),
            last_capacity_check: Arc::new(Mutex::new(Instant::now())),
            minute_buckets: Arc::new(Mutex::new(Vec::new())),
            hour_buckets: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record an event being processed
    pub fn record_event(
        &self,
        priority: crate::priority_queue::Priority,
        latency: Duration,
        event_type: Option<&str>,
    ) {
        // Apply sampling
        if self.config.sampling_rate < 1.0 {
            // Simplified sampling: skip based on a simple counter
            use std::sync::atomic::{AtomicUsize, Ordering};
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let count = COUNTER.fetch_add(1, Ordering::Relaxed);
            let sample_every = (1.0 / self.config.sampling_rate) as usize;
            if count % sample_every != 0 {
                return;
            }
        }

        let latency_us = latency.as_micros() as u64;

        // Update basic stats
        {
            let mut basic = self.basic.lock().unwrap();
            basic.total_events += 1;
            match priority {
                crate::priority_queue::Priority::High => basic.high_priority_events += 1,
                crate::priority_queue::Priority::Normal => basic.normal_priority_events += 1,
                crate::priority_queue::Priority::Low => basic.low_priority_events += 1,
            }
        }

        // Update latency histograms
        if self.config.track_percentiles {
            self.overall_latency.lock().unwrap().record(latency_us);

            match priority {
                crate::priority_queue::Priority::High => {
                    self.high_priority_latency
                        .lock()
                        .unwrap()
                        .record(latency_us);
                }
                crate::priority_queue::Priority::Normal => {
                    self.normal_priority_latency
                        .lock()
                        .unwrap()
                        .record(latency_us);
                }
                crate::priority_queue::Priority::Low => {
                    self.low_priority_latency.lock().unwrap().record(latency_us);
                }
            }

            if self.config.track_by_type {
                if let Some(event_type) = event_type {
                    let mut by_type = self.by_type_latency.lock().unwrap();
                    by_type
                        .entry(event_type.to_string())
                        .or_insert_with(|| LatencyTracker::new(10_000_000))
                        .record(latency_us);
                }
            }
        }

        // Update throughput tracking
        {
            let mut event_times = self.event_times.lock().unwrap();
            let now = Instant::now();
            event_times.push(now);

            // Keep only events in the rate window
            let cutoff = now - self.config.rate_window;
            event_times.retain(|t| *t > cutoff);

            // Calculate current rate
            if event_times.len() > 1 {
                let duration = now.duration_since(event_times[0]).as_secs_f64();
                if duration > 0.0 {
                    let rate = event_times.len() as f64 / duration;
                    let mut peak = self.peak_rate.lock().unwrap();
                    if rate > *peak {
                        *peak = rate;
                    }
                }
            }
        }

        // Record processing time
        self.processing_times.lock().unwrap().push(latency);

        trace!(
            "Recorded event: priority={:?}, latency={}μs",
            priority, latency_us
        );
    }

    /// Record a dropped event
    pub fn record_dropped(&self) {
        self.basic.lock().unwrap().dropped_events += 1;
    }

    /// Record backpressure activation
    pub fn record_backpressure(&self) {
        self.basic.lock().unwrap().backpressure_activations += 1;
    }

    /// Update queue depth
    pub fn update_queue_depth(&self, depth: usize, capacity: usize) {
        let now = Instant::now();

        // Track queue depths over time
        {
            let mut depths = self.queue_depths.lock().unwrap();
            depths.push((now, depth));

            // Keep only recent depths
            let cutoff = now - Duration::from_secs(300); // 5 minutes
            depths.retain(|(t, _)| *t > cutoff);
        }

        // Update max depth
        {
            let mut max_depth = self.max_queue_depth.lock().unwrap();
            if depth > *max_depth {
                *max_depth = depth;
            }
        }

        // Track time at capacity
        if depth >= capacity {
            let mut last_check = self.last_capacity_check.lock().unwrap();
            let duration = now.duration_since(*last_check);
            *self.time_at_capacity.lock().unwrap() += duration;
            *last_check = now;
        }
    }

    /// Take a snapshot of current metrics
    pub fn snapshot(&self) -> AdvancedEventStats {
        let now = Instant::now();
        let elapsed = now.duration_since(self.start_time).as_secs_f64();

        // Calculate latency stats
        let latency = LatencyStats {
            high_priority: self.high_priority_latency.lock().unwrap().percentiles(),
            normal_priority: self.normal_priority_latency.lock().unwrap().percentiles(),
            low_priority: self.low_priority_latency.lock().unwrap().percentiles(),
            overall: self.overall_latency.lock().unwrap().percentiles(),
            by_type: self
                .by_type_latency
                .lock()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.percentiles()))
                .collect(),
        };

        // Calculate throughput stats
        let throughput = {
            let event_times = self.event_times.lock().unwrap();
            let processing_times = self.processing_times.lock().unwrap();

            let current_rate = if event_times.len() > 1 {
                let duration = now.duration_since(event_times[0]).as_secs_f64();
                if duration > 0.0 {
                    event_times.len() as f64 / duration
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let avg_processing = if !processing_times.is_empty() {
                let sum: Duration = processing_times.iter().sum();
                sum.as_micros() as f64 / processing_times.len() as f64
            } else {
                0.0
            };

            ThroughputStats {
                current_rate,
                peak_rate: *self.peak_rate.lock().unwrap(),
                avg_rate_1m: current_rate, // Simplified for now
                avg_rate_5m: current_rate, // Simplified for now
                avg_processing_time_us: avg_processing,
            }
        };

        // Calculate queue stats
        let queue = {
            let depths = self.queue_depths.lock().unwrap();
            let current_depth = depths.last().map(|(_, d)| *d).unwrap_or(0);

            let avg_depth = if !depths.is_empty() {
                depths.iter().map(|(_, d)| *d).sum::<usize>() as f64 / depths.len() as f64
            } else {
                0.0
            };

            let growth_rate = if depths.len() > 1 {
                let first = depths.first().unwrap().1 as f64;
                let last = depths.last().unwrap().1 as f64;
                let duration = depths
                    .last()
                    .unwrap()
                    .0
                    .duration_since(depths.first().unwrap().0)
                    .as_secs_f64();
                if duration > 0.0 {
                    (last - first) / duration
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let saturation = if elapsed > 0.0 {
                self.time_at_capacity.lock().unwrap().as_secs_f64() / elapsed * 100.0
            } else {
                0.0
            };

            QueueStats {
                current_depth,
                max_depth: *self.max_queue_depth.lock().unwrap(),
                avg_depth,
                saturation_percentage: saturation,
                growth_rate,
            }
        };

        AdvancedEventStats {
            basic: self.basic.lock().unwrap().clone(),
            latency,
            throughput,
            queue,
            windows: WindowedStats::default(), // Simplified for now
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        *self.basic.lock().unwrap() = BasicStats::default();
        self.high_priority_latency.lock().unwrap().reset();
        self.normal_priority_latency.lock().unwrap().reset();
        self.low_priority_latency.lock().unwrap().reset();
        self.overall_latency.lock().unwrap().reset();
        self.by_type_latency.lock().unwrap().clear();
        self.event_times.lock().unwrap().clear();
        self.processing_times.lock().unwrap().clear();
        *self.peak_rate.lock().unwrap() = 0.0;
        self.queue_depths.lock().unwrap().clear();
        *self.max_queue_depth.lock().unwrap() = 0;
        *self.time_at_capacity.lock().unwrap() = Duration::ZERO;
        self.minute_buckets.lock().unwrap().clear();
        self.hour_buckets.lock().unwrap().clear();
    }

    /// Export metrics in JSON format
    pub fn export_json(&self) -> String {
        let stats = self.snapshot();
        stats.export(ExportFormat::Json)
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let stats = self.snapshot();
        stats.export(ExportFormat::Prometheus)
    }

    /// Export metrics in plain text format
    pub fn export_text(&self) -> String {
        let stats = self.snapshot();
        stats.export(ExportFormat::PlainText)
    }
}

/// Export format for metrics
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Prometheus,
    PlainText,
}

impl AdvancedEventStats {
    /// Export metrics in the specified format
    pub fn export(&self, format: ExportFormat) -> String {
        match format {
            ExportFormat::Json => self.to_json(),
            ExportFormat::Prometheus => self.to_prometheus(),
            ExportFormat::PlainText => self.to_plain_text(),
        }
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self)
            .unwrap_or_else(|e| format!("Failed to serialize metrics: {}", e))
    }

    fn to_prometheus(&self) -> String {
        let mut output = String::new();

        // Basic metrics
        output.push_str(&format!(
            "# HELP boba_events_total Total events processed\n"
        ));
        output.push_str(&format!("# TYPE boba_events_total counter\n"));
        output.push_str(&format!(
            "boba_events_total {{}} {}\n",
            self.basic.total_events
        ));
        output.push_str(&format!(
            "boba_events_total {{priority=\"high\"}} {}\n",
            self.basic.high_priority_events
        ));
        output.push_str(&format!(
            "boba_events_total {{priority=\"normal\"}} {}\n",
            self.basic.normal_priority_events
        ));
        output.push_str(&format!(
            "boba_events_total {{priority=\"low\"}} {}\n",
            self.basic.low_priority_events
        ));

        // Dropped events
        output.push_str(&format!(
            "# HELP boba_events_dropped Total events dropped\n"
        ));
        output.push_str(&format!("# TYPE boba_events_dropped counter\n"));
        output.push_str(&format!(
            "boba_events_dropped {{}} {}\n",
            self.basic.dropped_events
        ));

        // Latency metrics
        output.push_str(&format!(
            "# HELP boba_event_latency_microseconds Event processing latency\n"
        ));
        output.push_str(&format!("# TYPE boba_event_latency_microseconds summary\n"));

        for (priority, stats) in [
            ("high", &self.latency.high_priority),
            ("normal", &self.latency.normal_priority),
            ("low", &self.latency.low_priority),
        ] {
            output.push_str(&format!(
                "boba_event_latency_microseconds {{priority=\"{}\",quantile=\"0.5\"}} {}\n",
                priority, stats.p50
            ));
            output.push_str(&format!(
                "boba_event_latency_microseconds {{priority=\"{}\",quantile=\"0.9\"}} {}\n",
                priority, stats.p90
            ));
            output.push_str(&format!(
                "boba_event_latency_microseconds {{priority=\"{}\",quantile=\"0.95\"}} {}\n",
                priority, stats.p95
            ));
            output.push_str(&format!(
                "boba_event_latency_microseconds {{priority=\"{}\",quantile=\"0.99\"}} {}\n",
                priority, stats.p99
            ));
            output.push_str(&format!(
                "boba_event_latency_microseconds {{priority=\"{}\",quantile=\"0.999\"}} {}\n",
                priority, stats.p999
            ));
        }

        // Throughput metrics
        output.push_str(&format!("# HELP boba_throughput_rate Events per second\n"));
        output.push_str(&format!("# TYPE boba_throughput_rate gauge\n"));
        output.push_str(&format!(
            "boba_throughput_rate {{type=\"current\"}} {}\n",
            self.throughput.current_rate
        ));
        output.push_str(&format!(
            "boba_throughput_rate {{type=\"peak\"}} {}\n",
            self.throughput.peak_rate
        ));

        // Queue metrics
        output.push_str(&format!("# HELP boba_queue_depth Current queue depth\n"));
        output.push_str(&format!("# TYPE boba_queue_depth gauge\n"));
        output.push_str(&format!(
            "boba_queue_depth {{}} {}\n",
            self.queue.current_depth
        ));

        output.push_str(&format!(
            "# HELP boba_queue_saturation Queue saturation percentage\n"
        ));
        output.push_str(&format!("# TYPE boba_queue_saturation gauge\n"));
        output.push_str(&format!(
            "boba_queue_saturation {{}} {}\n",
            self.queue.saturation_percentage
        ));

        output
    }

    fn to_plain_text(&self) -> String {
        format!(
            "Event Processing Metrics\n\
            ========================\n\
            Total Events: {}\n\
            - High Priority: {}\n\
            - Normal Priority: {}\n\
            - Low Priority: {}\n\
            - Dropped: {}\n\n\
            Latency (μs):\n\
            - High Priority:   p50={} p95={} p99={} max={}\n\
            - Normal Priority: p50={} p95={} p99={} max={}\n\
            - Low Priority:    p50={} p95={} p99={} max={}\n\n\
            Throughput:\n\
            - Current Rate: {:.1} events/sec\n\
            - Peak Rate: {:.1} events/sec\n\
            - Avg Processing Time: {:.1} μs\n\n\
            Queue:\n\
            - Current Depth: {}\n\
            - Max Depth: {}\n\
            - Saturation: {:.1}%\n\
            - Growth Rate: {:.1} events/sec",
            self.basic.total_events,
            self.basic.high_priority_events,
            self.basic.normal_priority_events,
            self.basic.low_priority_events,
            self.basic.dropped_events,
            self.latency.high_priority.p50,
            self.latency.high_priority.p95,
            self.latency.high_priority.p99,
            self.latency.high_priority.max,
            self.latency.normal_priority.p50,
            self.latency.normal_priority.p95,
            self.latency.normal_priority.p99,
            self.latency.normal_priority.max,
            self.latency.low_priority.p50,
            self.latency.low_priority.p95,
            self.latency.low_priority.p99,
            self.latency.low_priority.max,
            self.throughput.current_rate,
            self.throughput.peak_rate,
            self.throughput.avg_processing_time_us,
            self.queue.current_depth,
            self.queue.max_depth,
            self.queue.saturation_percentage,
            self.queue.growth_rate,
        )
    }
}

/// Print a formatted metrics dashboard
pub fn print_metrics_dashboard(stats: &AdvancedEventStats) {
    eprintln!("╔══════════════════════════════════════════════════════════════╗");
    eprintln!("║                  Event Processing Metrics Dashboard          ║");
    eprintln!("╠══════════════════════════════════════════════════════════════╣");
    eprintln!("║ Throughput:                                                  ║");
    eprintln!(
        "║   Current: {:>8.1} evt/s   Peak: {:>8.1} evt/s           ║",
        stats.throughput.current_rate, stats.throughput.peak_rate
    );
    eprintln!(
        "║   Processing Time: {:>8.1} μs average                       ║",
        stats.throughput.avg_processing_time_us
    );
    eprintln!("║                                                              ║");
    eprintln!("║ Latencies (μs):      P50      P95      P99      Max         ║");
    eprintln!(
        "║   High Priority:  {:>7} {:>7} {:>7} {:>7}          ║",
        stats.latency.high_priority.p50,
        stats.latency.high_priority.p95,
        stats.latency.high_priority.p99,
        stats.latency.high_priority.max
    );
    eprintln!(
        "║   Normal Priority:{:>7} {:>7} {:>7} {:>7}          ║",
        stats.latency.normal_priority.p50,
        stats.latency.normal_priority.p95,
        stats.latency.normal_priority.p99,
        stats.latency.normal_priority.max
    );
    eprintln!(
        "║   Low Priority:   {:>7} {:>7} {:>7} {:>7}          ║",
        stats.latency.low_priority.p50,
        stats.latency.low_priority.p95,
        stats.latency.low_priority.p99,
        stats.latency.low_priority.max
    );
    eprintln!("║                                                              ║");
    eprintln!("║ Queue:                                                       ║");
    eprintln!(
        "║   Depth: {:>5} (max: {:>5})   Saturation: {:>5.1}%         ║",
        stats.queue.current_depth, stats.queue.max_depth, stats.queue.saturation_percentage
    );
    eprintln!(
        "║   Growth Rate: {:>8.1} events/sec                          ║",
        stats.queue.growth_rate
    );
    eprintln!("║                                                              ║");
    eprintln!("║ Events:                                                      ║");
    eprintln!(
        "║   Total: {:>8}  Dropped: {:>6}  Backpressure: {:>6}  ║",
        stats.basic.total_events, stats.basic.dropped_events, stats.basic.backpressure_activations
    );
    eprintln!("╚══════════════════════════════════════════════════════════════╝");
}

/// Display metrics dashboard  
pub fn display_dashboard(stats: &AdvancedEventStats) -> String {
    let mut output = String::new();
    use std::fmt::Write;

    let _ = writeln!(
        output,
        "╔══════════════════════════════════════════════════════════════╗"
    );
    let _ = writeln!(
        output,
        "║                    METRICS DASHBOARD                         ║"
    );
    let _ = writeln!(
        output,
        "╠══════════════════════════════════════════════════════════════╣"
    );

    // Event counts
    let _ = writeln!(
        output,
        "║ Events Processed:  {:>10}                                ║",
        stats.basic.total_events
    );
    let _ = writeln!(
        output,
        "║ Events Dropped:    {:>10}                                ║",
        stats.basic.dropped_events
    );
    let _ = writeln!(
        output,
        "║ Backpressure:      {:>10}                                ║",
        stats.basic.backpressure_activations
    );

    let _ = writeln!(
        output,
        "╠══════════════════════════════════════════════════════════════╣"
    );

    // Latency percentiles (overall)
    let _ = writeln!(
        output,
        "║ LATENCY (μs) - Overall                                       ║"
    );
    let _ = writeln!(
        output,
        "║   p50:  {:>8.1}    p75:  {:>8.1}    p90:  {:>8.1}      ║",
        stats.latency.overall.p50, stats.latency.overall.p75, stats.latency.overall.p90
    );
    let _ = writeln!(
        output,
        "║   p95:  {:>8.1}    p99:  {:>8.1}    p999: {:>8.1}      ║",
        stats.latency.overall.p95, stats.latency.overall.p99, stats.latency.overall.p999
    );
    let _ = writeln!(
        output,
        "║   min:  {:>8.1}    max:  {:>8.1}                        ║",
        stats.latency.overall.min, stats.latency.overall.max
    );

    let _ = writeln!(
        output,
        "╠══════════════════════════════════════════════════════════════╣"
    );

    // Throughput
    let _ = writeln!(
        output,
        "║ THROUGHPUT                                                   ║"
    );
    let _ = writeln!(
        output,
        "║   Current rate:   {:>10.1} events/sec                   ║",
        stats.throughput.current_rate
    );
    let _ = writeln!(
        output,
        "║   Peak rate:      {:>10.1} events/sec                   ║",
        stats.throughput.peak_rate
    );

    let _ = writeln!(
        output,
        "╠══════════════════════════════════════════════════════════════╣"
    );

    // Queue stats
    let _ = writeln!(
        output,
        "║ QUEUE                                                        ║"
    );
    let _ = writeln!(
        output,
        "║   Current depth:    {:>6}                                  ║",
        stats.queue.current_depth
    );
    let _ = writeln!(
        output,
        "║   Max depth:        {:>6}                                  ║",
        stats.queue.max_depth
    );
    let _ = writeln!(
        output,
        "║   Avg depth:        {:>6.1}                                ║",
        stats.queue.avg_depth
    );
    let _ = writeln!(
        output,
        "║   Saturation:       {:>6.1}%                                ║",
        stats.queue.saturation_percentage
    );

    let _ = writeln!(
        output,
        "╚══════════════════════════════════════════════════════════════╝"
    );

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::priority_queue::Priority;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new(MetricsConfig::default());
        let stats = collector.snapshot();

        assert_eq!(stats.basic.total_events, 0);
        assert_eq!(stats.latency.high_priority.count, 0);
    }

    #[test]
    fn test_event_recording() {
        let collector = MetricsCollector::new(MetricsConfig::default());

        // Record some events
        collector.record_event(Priority::High, Duration::from_micros(100), Some("test"));
        collector.record_event(Priority::Normal, Duration::from_micros(200), Some("test"));
        collector.record_event(Priority::Low, Duration::from_micros(300), None);

        let stats = collector.snapshot();

        assert_eq!(stats.basic.total_events, 3);
        assert_eq!(stats.basic.high_priority_events, 1);
        assert_eq!(stats.basic.normal_priority_events, 1);
        assert_eq!(stats.basic.low_priority_events, 1);

        // Check that latencies were recorded
        assert!(stats.latency.high_priority.count > 0);
        assert!(stats.latency.high_priority.mean > 0.0);
    }

    #[test]
    fn test_percentile_calculation() {
        let collector = MetricsCollector::new(MetricsConfig::default());

        // Record many events with known latencies
        for i in 1..=100 {
            collector.record_event(Priority::High, Duration::from_micros(i * 10), None);
        }

        let stats = collector.snapshot();
        let percentiles = &stats.latency.high_priority;

        // P50 should be around 500μs (50th value * 10)
        assert!(percentiles.p50 >= 490 && percentiles.p50 <= 510);

        // P99 should be around 990μs (99th value * 10)
        assert!(percentiles.p99 >= 980 && percentiles.p99 <= 1000);

        assert_eq!(percentiles.count, 100);
    }

    #[test]
    fn test_export_formats() {
        let collector = MetricsCollector::new(MetricsConfig::default());
        collector.record_event(Priority::High, Duration::from_micros(100), None);

        let stats = collector.snapshot();

        // Test JSON export
        let json = stats.export(ExportFormat::Json);
        assert!(json.contains("\"total_events\": 1"));

        // Test Prometheus export
        let prometheus = stats.export(ExportFormat::Prometheus);
        assert!(prometheus.contains("boba_events_total"));
        assert!(prometheus.contains("boba_event_latency_microseconds"));

        // Test plain text export
        let text = stats.export(ExportFormat::PlainText);
        assert!(text.contains("Total Events: 1"));
    }

    #[test]
    fn test_metrics_reset() {
        let collector = MetricsCollector::new(MetricsConfig::default());

        // Record some events
        for _ in 0..10 {
            collector.record_event(Priority::High, Duration::from_micros(100), None);
        }

        let stats = collector.snapshot();
        assert_eq!(stats.basic.total_events, 10);

        // Reset and verify
        collector.reset();
        let stats = collector.snapshot();
        assert_eq!(stats.basic.total_events, 0);
        assert_eq!(stats.latency.high_priority.count, 0);
    }
}
