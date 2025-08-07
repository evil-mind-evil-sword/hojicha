use hojicha::event::{Event, KeyEvent};
use hojicha::priority_queue::{Priority, PriorityEventQueue, ResizeError};
use hojicha::queue_scaling::{AutoScaleConfig, QueueAutoScaler, ScalingStrategy};

#[derive(Debug, Clone, PartialEq)]
struct TestMsg(String);

#[test]
fn test_queue_resize_grow() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    // Fill queue partially
    for i in 0..50 {
        queue
            .push(Event::User(TestMsg(format!("msg{}", i))))
            .unwrap();
    }

    assert_eq!(queue.capacity(), 100);
    assert_eq!(queue.len(), 50);

    // Resize to larger capacity
    queue.resize(200).unwrap();

    assert_eq!(queue.capacity(), 200);
    assert_eq!(queue.len(), 50); // No events lost

    // Can now add more events
    for i in 50..150 {
        queue
            .push(Event::User(TestMsg(format!("msg{}", i))))
            .unwrap();
    }

    assert_eq!(queue.len(), 150);
}

#[test]
fn test_queue_resize_shrink() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    // Add mixed priority events
    for i in 0..30 {
        queue.push(Event::Tick).unwrap(); // Low priority
    }
    for i in 0..20 {
        queue
            .push(Event::User(TestMsg(format!("msg{}", i))))
            .unwrap(); // Normal
    }
    for _ in 0..10 {
        queue.push(Event::Quit).unwrap(); // High priority
    }

    assert_eq!(queue.len(), 60);

    // Shrink queue - should drop low priority events first
    queue.resize(40).unwrap();

    assert_eq!(queue.capacity(), 40);
    assert_eq!(queue.len(), 40);

    // High priority events should still be there
    let mut high_count = 0;
    let mut normal_count = 0;
    let mut low_count = 0;

    while let Some(event) = queue.pop() {
        match Priority::from_event(&event) {
            Priority::High => high_count += 1,
            Priority::Normal => normal_count += 1,
            Priority::Low => low_count += 1,
        }
    }

    assert_eq!(high_count, 10); // All high priority preserved
    assert_eq!(normal_count, 20); // All normal priority preserved
    assert_eq!(low_count, 10); // Some low priority dropped
}

#[test]
fn test_queue_resize_zero_fails() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    let result = queue.resize(0);
    assert!(result.is_err());
    assert!(matches!(result, Err(ResizeError::InvalidSize(_))));
}

#[test]
fn test_try_grow_and_shrink() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    // Try growing
    let new_size = queue.try_grow(50).unwrap();
    assert_eq!(new_size, 150);
    assert_eq!(queue.capacity(), 150);

    // Try shrinking
    let new_size = queue.try_shrink(75).unwrap();
    assert_eq!(new_size, 75);
    assert_eq!(queue.capacity(), 75);

    // Shrinking to zero should give us 1
    let new_size = queue.try_shrink(100).unwrap();
    assert_eq!(new_size, 1);
    assert_eq!(queue.capacity(), 1);
}

#[test]
fn test_queue_stats() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    for i in 0..50 {
        queue
            .push(Event::User(TestMsg(format!("msg{}", i))))
            .unwrap();
    }

    let stats = queue.stats();
    assert_eq!(stats.current_size, 50);
    assert_eq!(stats.max_size, 100);
    assert_eq!(stats.utilization, 0.5);
    assert!(!stats.backpressure_active);

    // Fill to trigger backpressure
    for i in 50..85 {
        queue
            .push(Event::User(TestMsg(format!("msg{}", i))))
            .unwrap();
    }

    let stats = queue.stats();
    assert_eq!(stats.current_size, 85);
    assert!(stats.backpressure_active); // Should be active at 80%+
}

#[test]
fn test_auto_scaler_conservative_strategy() {
    let config = AutoScaleConfig {
        min_size: 50,
        max_size: 500,
        target_utilization: 0.5,
        evaluation_interval: 10,
        strategy: ScalingStrategy::Conservative,
        cooldown: std::time::Duration::from_millis(0), // No cooldown for testing
        debug: false,
    };

    let mut scaler = QueueAutoScaler::new(config);
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    // Fill queue to high utilization
    for i in 0..92 {
        queue
            .push(Event::User(TestMsg(format!("msg{}", i))))
            .unwrap();
    }

    // Process enough events to trigger evaluation
    for _ in 0..10 {
        scaler.on_event_processed(&mut queue);
    }

    // Should have grown the queue
    assert!(queue.capacity() > 100);
}

#[test]
fn test_auto_scaler_aggressive_strategy() {
    let config = AutoScaleConfig {
        min_size: 50,
        max_size: 1000,
        target_utilization: 0.5,
        evaluation_interval: 10,
        strategy: ScalingStrategy::Aggressive,
        cooldown: std::time::Duration::from_millis(0),
        debug: false,
    };

    let mut scaler = QueueAutoScaler::new(config);
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    // Fill queue to high utilization
    for i in 0..85 {
        queue
            .push(Event::User(TestMsg(format!("msg{}", i))))
            .unwrap();
    }

    // Process events to trigger evaluation
    for _ in 0..10 {
        scaler.on_event_processed(&mut queue);
    }

    // Aggressive should double the size
    assert_eq!(queue.capacity(), 200);
}

#[test]
fn test_auto_scaler_respects_min_max() {
    let config = AutoScaleConfig {
        min_size: 50,
        max_size: 150,
        target_utilization: 0.5,
        evaluation_interval: 10,
        strategy: ScalingStrategy::Aggressive,
        cooldown: std::time::Duration::from_millis(0),
        debug: false,
    };

    let mut scaler = QueueAutoScaler::new(config);
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    // Fill queue to trigger growth
    for i in 0..85 {
        queue
            .push(Event::User(TestMsg(format!("msg{}", i))))
            .unwrap();
    }

    for _ in 0..10 {
        scaler.on_event_processed(&mut queue);
    }

    // Should not exceed max_size
    assert_eq!(queue.capacity(), 150);

    // Clear queue and trigger shrink
    queue.clear();

    for _ in 0..10 {
        scaler.on_event_processed(&mut queue);
    }

    // Should not go below min_size
    assert_eq!(queue.capacity(), 50);
}

#[test]
fn test_auto_scaler_cooldown() {
    use std::thread;
    use std::time::Duration;

    let config = AutoScaleConfig {
        min_size: 50,
        max_size: 500,
        target_utilization: 0.5,
        evaluation_interval: 5,
        strategy: ScalingStrategy::Conservative,
        cooldown: Duration::from_millis(100),
        debug: false,
    };

    let mut scaler = QueueAutoScaler::new(config);
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    // Fill queue to trigger scaling
    for i in 0..92 {
        queue
            .push(Event::User(TestMsg(format!("msg{}", i))))
            .unwrap();
    }

    // First scaling should work
    for _ in 0..5 {
        scaler.on_event_processed(&mut queue);
    }
    let size_after_first = queue.capacity();
    assert!(size_after_first > 100);

    // Immediate second attempt should be blocked by cooldown
    for _ in 0..5 {
        scaler.on_event_processed(&mut queue);
    }
    assert_eq!(queue.capacity(), size_after_first);

    // Wait for cooldown
    thread::sleep(Duration::from_millis(150));

    // Now scaling should work again
    for _ in 0..5 {
        scaler.on_event_processed(&mut queue);
    }
    assert!(queue.capacity() > size_after_first);
}

#[test]
fn test_resize_preserves_priority_order() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    // Add events in specific order
    queue.push(Event::Tick).unwrap(); // Low
    queue
        .push(Event::User(TestMsg("normal".to_string())))
        .unwrap(); // Normal
    queue.push(Event::Quit).unwrap(); // High

    // Resize shouldn't affect order
    queue.resize(200).unwrap();

    assert_eq!(queue.pop(), Some(Event::Quit));
    assert_eq!(
        queue.pop(),
        Some(Event::User(TestMsg("normal".to_string())))
    );
    assert_eq!(queue.pop(), Some(Event::Tick));
}

#[test]
fn test_backpressure_threshold_updates_on_resize() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

    // Fill to just below 80%
    for i in 0..79 {
        queue
            .push(Event::User(TestMsg(format!("msg{}", i))))
            .unwrap();
    }
    assert!(!queue.is_backpressure_active());

    // One more should trigger backpressure
    queue
        .push(Event::User(TestMsg("trigger".to_string())))
        .unwrap();
    assert!(queue.is_backpressure_active());

    // Resize larger - backpressure should deactivate
    queue.resize(200).unwrap();
    assert!(!queue.is_backpressure_active()); // 80/200 = 40%

    // Resize smaller - backpressure should activate again
    queue.resize(90).unwrap();
    assert!(queue.is_backpressure_active()); // 80/90 = 88%
}
