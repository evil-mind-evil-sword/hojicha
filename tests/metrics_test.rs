use hojicha::metrics::{MetricsCollector, MetricsConfig};
use hojicha::priority_queue::Priority;
use std::thread;
use std::time::Duration;

#[test]
fn test_metrics_collector_creation() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);
    let stats = collector.snapshot();

    assert_eq!(stats.basic.total_events, 0);
    assert_eq!(stats.basic.dropped_events, 0);
}

#[test]
fn test_event_recording() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);

    // Record some events
    collector.record_event(Priority::High, Duration::from_micros(100), Some("key"));
    collector.record_event(Priority::Normal, Duration::from_micros(200), Some("user"));
    collector.record_event(Priority::Low, Duration::from_micros(300), Some("tick"));

    let stats = collector.snapshot();
    assert_eq!(stats.basic.total_events, 3);
    assert_eq!(stats.basic.high_priority_events, 1);
    assert_eq!(stats.basic.normal_priority_events, 1);
    assert_eq!(stats.basic.low_priority_events, 1);
}

#[test]
fn test_latency_tracking() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);

    // Record events with different latencies
    for i in 1..=100 {
        let latency = Duration::from_micros(i * 10);
        let priority = match i % 3 {
            0 => Priority::High,
            1 => Priority::Normal,
            _ => Priority::Low,
        };
        collector.record_event(priority, latency, None);
    }

    let stats = collector.snapshot();

    // Check that percentiles are calculated
    assert!(stats.latency.overall.p50 > 0);
    assert!(stats.latency.overall.p99 > stats.latency.overall.p50);
    assert!(stats.latency.overall.max >= stats.latency.overall.p99);
    assert!(stats.latency.overall.min <= stats.latency.overall.p50);
}

#[test]
fn test_queue_depth_tracking() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);

    // Update queue depth multiple times
    collector.update_queue_depth(10, 100);
    collector.update_queue_depth(50, 100);
    collector.update_queue_depth(80, 100);
    collector.update_queue_depth(30, 100);

    let stats = collector.snapshot();
    assert_eq!(stats.queue.current_depth, 30);
    assert_eq!(stats.queue.max_depth, 80);
    assert!(stats.queue.avg_depth > 0.0);
}

#[test]
fn test_dropped_events_tracking() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);

    collector.record_dropped();
    collector.record_dropped();
    collector.record_dropped();

    let stats = collector.snapshot();
    assert_eq!(stats.basic.dropped_events, 3);
}

#[test]
fn test_backpressure_tracking() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);

    collector.record_backpressure();
    collector.record_backpressure();

    let stats = collector.snapshot();
    assert_eq!(stats.basic.backpressure_activations, 2);
}

#[test]
fn test_throughput_calculation() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);

    // Record events over time
    for _ in 0..10 {
        collector.record_event(Priority::Normal, Duration::from_micros(100), None);
        thread::sleep(Duration::from_millis(10));
    }

    let stats = collector.snapshot();
    // Throughput should be greater than 0
    assert!(stats.throughput.current_rate > 0.0);
}

#[test]
fn test_metrics_reset() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);

    // Record some events
    collector.record_event(Priority::High, Duration::from_micros(100), None);
    collector.record_dropped();
    collector.update_queue_depth(50, 100);

    // Reset metrics
    collector.reset();

    let stats = collector.snapshot();
    assert_eq!(stats.basic.total_events, 0);
    assert_eq!(stats.basic.dropped_events, 0);
    assert_eq!(stats.queue.current_depth, 0);
}

#[test]
fn test_json_export() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);

    collector.record_event(Priority::High, Duration::from_micros(100), Some("test"));

    let json = collector.export_json();
    // JSON uses snake_case for field names
    assert!(json.contains("\"total_events\": 1") || json.contains("\"total_events\":1"));
    assert!(
        json.contains("\"high_priority_events\": 1") || json.contains("\"high_priority_events\":1")
    );
}

#[test]
fn test_prometheus_export() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);

    collector.record_event(Priority::Normal, Duration::from_micros(200), None);
    collector.record_dropped();

    let prometheus = collector.export_prometheus();
    assert!(prometheus.contains("hojicha_events_total"));
    assert!(prometheus.contains("hojicha_events_dropped"));
    assert!(prometheus.contains("# TYPE"));
    assert!(prometheus.contains("# HELP"));
}

#[test]
fn test_text_export() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);

    collector.record_event(Priority::Low, Duration::from_micros(300), None);

    let text = collector.export_text();
    assert!(text.contains("Total Events: 1"));
    assert!(text.contains("Low Priority: 1"));
}

#[test]
fn test_sampling_rate() {
    let config = MetricsConfig {
        sampling_rate: 0.5, // Sample 50% of events
        ..Default::default()
    };

    let collector = MetricsCollector::new(config);

    // Record many events
    for _ in 0..100 {
        collector.record_event(Priority::Normal, Duration::from_micros(100), None);
    }

    let stats = collector.snapshot();
    // With 50% sampling, we should have roughly 50 events
    // Allow for some variance due to the simple sampling algorithm
    assert!(stats.basic.total_events >= 40);
    assert!(stats.basic.total_events <= 60);
}

#[test]
fn test_event_type_tracking() {
    let config = MetricsConfig {
        track_by_type: true, // Enable event type tracking
        ..Default::default()
    };
    let collector = MetricsCollector::new(config);

    // Record events of different types
    collector.record_event(Priority::High, Duration::from_micros(100), Some("key"));
    collector.record_event(Priority::Normal, Duration::from_micros(200), Some("mouse"));
    collector.record_event(Priority::Low, Duration::from_micros(300), Some("tick"));
    collector.record_event(Priority::Normal, Duration::from_micros(150), Some("key"));

    let stats = collector.snapshot();

    // Check that event types are tracked
    assert!(stats.latency.by_type.contains_key("key"));
    assert!(stats.latency.by_type.contains_key("mouse"));
    assert!(stats.latency.by_type.contains_key("tick"));
}

#[test]
fn test_priority_latency_separation() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config);

    // Record high priority events with low latency
    for _ in 0..50 {
        collector.record_event(Priority::High, Duration::from_micros(10), None);
    }

    // Record low priority events with high latency
    for _ in 0..50 {
        collector.record_event(Priority::Low, Duration::from_micros(1000), None);
    }

    let stats = collector.snapshot();

    // High priority should have lower latency than low priority
    assert!(stats.latency.high_priority.p50 < stats.latency.low_priority.p50);
    assert!(stats.latency.high_priority.p99 < stats.latency.low_priority.p99);
}
