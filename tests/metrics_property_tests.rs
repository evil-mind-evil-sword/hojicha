use hojicha::metrics::{MetricsCollector, MetricsConfig};
use hojicha::priority_queue::Priority;
use proptest::prelude::*;
use std::time::Duration;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_metrics_never_negative(
        events in prop::collection::vec((0u64..10000, 0u8..3), 1..1000)
    ) {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        let event_count = events.len();
        for (latency_us, priority_num) in events {
            let priority = match priority_num {
                0 => Priority::High,
                1 => Priority::Normal,
                _ => Priority::Low,
            };
            collector.record_event(priority, Duration::from_micros(latency_us), None);
        }

        let stats = collector.snapshot();

        // All counts should match what we recorded (usize is always >= 0)
        prop_assert_eq!(stats.basic.total_events, event_count);

        // Latency values should be non-negative (u64 is always >= 0)
        // Just verify they exist
        let _ = stats.latency.overall.min;
        let _ = stats.latency.overall.max;
        let _ = stats.latency.overall.p50;
        let _ = stats.latency.overall.p99;

        // Throughput should be non-negative
        prop_assert!(stats.throughput.current_rate >= 0.0);
        prop_assert!(stats.throughput.peak_rate >= 0.0);
        prop_assert!(stats.throughput.avg_processing_time_us >= 0.0);
    }

    #[test]
    fn prop_event_counts_sum_correctly(
        high_count in 0usize..100,
        normal_count in 0usize..100,
        low_count in 0usize..100
    ) {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        // Record exact counts of each priority
        for _ in 0..high_count {
            collector.record_event(Priority::High, Duration::from_micros(100), None);
        }
        for _ in 0..normal_count {
            collector.record_event(Priority::Normal, Duration::from_micros(100), None);
        }
        for _ in 0..low_count {
            collector.record_event(Priority::Low, Duration::from_micros(100), None);
        }

        let stats = collector.snapshot();

        // Total should equal sum of individual priorities
        prop_assert_eq!(
            stats.basic.total_events,
            stats.basic.high_priority_events +
            stats.basic.normal_priority_events +
            stats.basic.low_priority_events
        );

        // Individual counts should match what we recorded
        prop_assert_eq!(stats.basic.high_priority_events, high_count);
        prop_assert_eq!(stats.basic.normal_priority_events, normal_count);
        prop_assert_eq!(stats.basic.low_priority_events, low_count);
    }

    #[test]
    fn prop_percentiles_ordered_correctly(
        latencies in prop::collection::vec(1u64..1000000, 10..1000)
    ) {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        for latency_us in latencies {
            collector.record_event(Priority::Normal, Duration::from_micros(latency_us), None);
        }

        let stats = collector.snapshot();
        let percentiles = &stats.latency.overall;

        // Percentiles should be ordered: min <= p50 <= p75 <= p90 <= p95 <= p99 <= p999 <= max
        prop_assert!(percentiles.min <= percentiles.p50);
        prop_assert!(percentiles.p50 <= percentiles.p75);
        prop_assert!(percentiles.p75 <= percentiles.p90);
        prop_assert!(percentiles.p90 <= percentiles.p95);
        prop_assert!(percentiles.p95 <= percentiles.p99);
        prop_assert!(percentiles.p99 <= percentiles.p999);
        prop_assert!(percentiles.p999 <= percentiles.max);
    }

    #[test]
    fn prop_queue_depth_tracking_consistent(
        depths in prop::collection::vec((0usize..1000, 1000usize..2000), 1..100)
    ) {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        let mut max_depth_seen = 0;
        let mut last_depth = 0;

        for (depth, capacity) in depths {
            collector.update_queue_depth(depth, capacity);
            max_depth_seen = max_depth_seen.max(depth);
            last_depth = depth;
        }

        let stats = collector.snapshot();

        // Current depth should be the last one we set
        prop_assert_eq!(stats.queue.current_depth, last_depth);

        // Max depth should be the maximum we've seen
        prop_assert_eq!(stats.queue.max_depth, max_depth_seen);

        // Average depth should be between 0 and max
        prop_assert!(stats.queue.avg_depth >= 0.0);
        prop_assert!(stats.queue.avg_depth <= max_depth_seen as f64);
    }

    #[test]
    fn prop_dropped_events_counted_accurately(
        drop_count in 0usize..1000
    ) {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        for _ in 0..drop_count {
            collector.record_dropped();
        }

        let stats = collector.snapshot();
        prop_assert_eq!(stats.basic.dropped_events, drop_count);
    }

    #[test]
    fn prop_backpressure_counted_accurately(
        backpressure_count in 0usize..100
    ) {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        for _ in 0..backpressure_count {
            collector.record_backpressure();
        }

        let stats = collector.snapshot();
        prop_assert_eq!(stats.basic.backpressure_activations, backpressure_count);
    }

    #[test]
    fn prop_reset_clears_all_metrics(
        events in prop::collection::vec((1u64..10000, 0u8..3), 1..100),
        drops in 0usize..50,
        backpressures in 0usize..20
    ) {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        // Record various metrics
        for (latency_us, priority_num) in events {
            let priority = match priority_num {
                0 => Priority::High,
                1 => Priority::Normal,
                _ => Priority::Low,
            };
            collector.record_event(priority, Duration::from_micros(latency_us), None);
        }

        for _ in 0..drops {
            collector.record_dropped();
        }

        for _ in 0..backpressures {
            collector.record_backpressure();
        }

        collector.update_queue_depth(500, 1000);

        // Reset everything
        collector.reset();

        let stats = collector.snapshot();

        // Everything should be zero/default
        prop_assert_eq!(stats.basic.total_events, 0);
        prop_assert_eq!(stats.basic.high_priority_events, 0);
        prop_assert_eq!(stats.basic.normal_priority_events, 0);
        prop_assert_eq!(stats.basic.low_priority_events, 0);
        prop_assert_eq!(stats.basic.dropped_events, 0);
        prop_assert_eq!(stats.basic.backpressure_activations, 0);
        prop_assert_eq!(stats.queue.current_depth, 0);
        prop_assert_eq!(stats.queue.max_depth, 0);
    }

    #[test]
    fn prop_sampling_reduces_event_count(
        events in prop::collection::vec(1u64..1000, 100..500),
        sampling_rate in 0.1f64..0.9f64
    ) {
        let mut config = MetricsConfig::default();
        config.sampling_rate = sampling_rate;
        let collector = MetricsCollector::new(config);

        for latency_us in &events {
            collector.record_event(Priority::Normal, Duration::from_micros(*latency_us), None);
        }

        let stats = collector.snapshot();
        let total_events = events.len();

        // Our simple deterministic sampling should sample every N events
        // where N = 1/sampling_rate
        let sample_every = (1.0 / sampling_rate) as usize;
        let expected_sampled = total_events.div_ceil(sample_every); // ceiling division

        // The actual count should be close to expected (within 10% or at least 1)
        let tolerance = (expected_sampled as f64 * 0.1).max(1.0) as usize;
        let min_expected = expected_sampled.saturating_sub(tolerance);
        let max_expected = expected_sampled + tolerance;

        prop_assert!(stats.basic.total_events >= min_expected,
            "Expected at least {} events, got {}", min_expected, stats.basic.total_events);
        prop_assert!(stats.basic.total_events <= max_expected,
            "Expected at most {} events, got {}", max_expected, stats.basic.total_events);
    }

    #[test]
    fn prop_priority_latency_separation_maintained(
        high_latencies in prop::collection::vec(1u64..100, 10..50),
        low_latencies in prop::collection::vec(1000u64..10000, 10..50)
    ) {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        // Record high priority events with low latency
        for latency_us in &high_latencies {
            collector.record_event(Priority::High, Duration::from_micros(*latency_us), None);
        }

        // Record low priority events with high latency
        for latency_us in &low_latencies {
            collector.record_event(Priority::Low, Duration::from_micros(*latency_us), None);
        }

        let stats = collector.snapshot();

        // High priority events should have lower latency than low priority
        if !high_latencies.is_empty() && !low_latencies.is_empty() {
            prop_assert!(stats.latency.high_priority.p50 < stats.latency.low_priority.p50);
            prop_assert!(stats.latency.high_priority.max <= stats.latency.low_priority.min);
        }
    }

    #[test]
    fn prop_export_formats_non_empty(
        events in prop::collection::vec((1u64..10000, 0u8..3), 1..100)
    ) {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);

        for (latency_us, priority_num) in events {
            let priority = match priority_num {
                0 => Priority::High,
                1 => Priority::Normal,
                _ => Priority::Low,
            };
            collector.record_event(priority, Duration::from_micros(latency_us), None);
        }

        // All export formats should produce non-empty output
        let json = collector.export_json();
        prop_assert!(!json.is_empty());
        prop_assert!(json.contains("total_events"));

        let prometheus = collector.export_prometheus();
        prop_assert!(!prometheus.is_empty());
        prop_assert!(prometheus.contains("boba_events_total"));

        let text = collector.export_text();
        prop_assert!(!text.is_empty());
        prop_assert!(text.contains("Total Events"));
    }

    #[test]
    fn prop_concurrent_updates_safe(
        updates in prop::collection::vec(0u8..4, 100..1000)
    ) {
        use std::sync::Arc;
        use std::thread;

        let config = MetricsConfig::default();
        let collector = Arc::new(MetricsCollector::new(config));
        let mut handles = vec![];

        for update_type in updates {
            let collector = Arc::clone(&collector);
            let handle = thread::spawn(move || {
                match update_type {
                    0 => collector.record_event(Priority::Normal, Duration::from_micros(100), None),
                    1 => collector.record_dropped(),
                    2 => collector.record_backpressure(),
                    _ => collector.update_queue_depth(50, 100),
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Should be able to get a snapshot without panic
        let stats = collector.snapshot();

        // Just verify we can get stats without panic (counts are always >= 0 for usize)
        let _ = stats.basic.total_events;
        let _ = stats.basic.dropped_events;
        let _ = stats.basic.backpressure_activations;
    }

    #[test]
    fn prop_event_type_tracking_accurate(
        events in prop::collection::vec(
            (1u64..1000, prop::sample::select(vec!["key", "mouse", "tick", "resize"])),
            10..200
        )
    ) {
        let mut config = MetricsConfig::default();
        config.track_by_type = true;
        let collector = MetricsCollector::new(config);

        let mut type_counts = std::collections::HashMap::new();

        for (latency_us, event_type) in &events {
            collector.record_event(Priority::Normal, Duration::from_micros(*latency_us), Some(event_type));
            *type_counts.entry(event_type.to_string()).or_insert(0) += 1;
        }

        let stats = collector.snapshot();

        // All event types we recorded should be present
        for (event_type, count) in type_counts {
            if count > 0 {
                prop_assert!(stats.latency.by_type.contains_key(&event_type));
                let type_stats = &stats.latency.by_type[&event_type];
                prop_assert!(type_stats.count > 0);
            }
        }
    }
}

#[test]
fn test_metrics_property_compilation() {
    // This test ensures the property tests compile
    assert!(true);
}
