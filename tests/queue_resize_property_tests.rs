use hojicha::event::Event;
use hojicha::priority_queue::{Priority, PriorityEventQueue};
use hojicha::queue_scaling::{AutoScaleConfig, QueueAutoScaler, ScalingStrategy};
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq)]
struct TestMsg(u32);

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_resize_never_loses_high_priority_events(
        initial_size in 10usize..1000,
        new_size in 1usize..1000,
        high_events in 0usize..50,
        normal_events in 0usize..50,
        low_events in 0usize..100
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(initial_size);

        // Add events of each priority
        for _ in 0..high_events.min(initial_size / 3) {
            queue.push(Event::Quit).ok(); // High priority
        }
        for i in 0..normal_events.min(initial_size / 3) {
            queue.push(Event::User(TestMsg(i as u32))).ok();
        }
        for i in 0..low_events.min(initial_size / 3) {
            queue.push(Event::Tick).ok();
        }

        // Count high priority events before resize
        let high_before = high_events.min(initial_size / 3);

        // Resize the queue
        let _ = queue.resize(new_size);

        // Count high priority events after resize
        let high_after = count_priority_events(&mut queue, Priority::High);

        // High priority events should never be lost unless queue is too small
        if new_size >= high_before {
            prop_assert_eq!(high_after, high_before);
        }
    }

    #[test]
    fn prop_resize_maintains_capacity_invariants(
        initial_size in 1usize..10000,
        resize_operations in prop::collection::vec(1usize..10000, 1..20)
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(initial_size);

        for new_size in resize_operations {
            let result = queue.resize(new_size);

            if new_size == 0 {
                prop_assert!(result.is_err());
            } else {
                prop_assert!(result.is_ok());
                prop_assert_eq!(queue.capacity(), new_size);
            }

            // Queue length should never exceed capacity
            prop_assert!(queue.len() <= queue.capacity());
        }
    }

    #[test]
    fn prop_grow_never_drops_events(
        initial_size in 10usize..100,
        growth in 1usize..1000,
        events in 0usize..100
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(initial_size);

        // Add events up to capacity
        let events_to_add = events.min(initial_size);
        for i in 0..events_to_add {
            queue.push(Event::User(TestMsg(i as u32))).unwrap();
        }

        let len_before = queue.len();

        // Grow the queue
        queue.resize(initial_size + growth).unwrap();

        let len_after = queue.len();

        // Growing should never drop events
        prop_assert_eq!(len_after, len_before);
    }

    #[test]
    fn prop_shrink_drops_lowest_priority_first(
        initial_size in 100usize..500,
        shrink_to in 10usize..99,
        high in 0usize..30,
        normal in 0usize..30,
        low in 0usize..100
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(initial_size);

        // Add events ensuring we don't exceed capacity
        let total_events = (high + normal + low).min(initial_size);
        let high_to_add = high.min(total_events / 3);
        let normal_to_add = normal.min(total_events / 3);
        let low_to_add = low.min(total_events / 3);

        for _ in 0..high_to_add {
            queue.push(Event::Quit).unwrap();
        }
        for i in 0..normal_to_add {
            queue.push(Event::User(TestMsg(i as u32))).unwrap();
        }
        for _ in 0..low_to_add {
            queue.push(Event::Tick).unwrap();
        }

        // Track what we added
        let high_before = high_to_add;
        let normal_before = normal_to_add;
        let low_before = low_to_add;

        // Shrink the queue
        queue.resize(shrink_to).unwrap();

        // Count all priorities after resize
        let mut high_after = 0;
        let mut normal_after = 0;
        let mut low_after = 0;
        let mut events = Vec::new();
        while let Some(event) = queue.pop() {
            match Priority::from_event(&event) {
                Priority::High => high_after += 1,
                Priority::Normal => normal_after += 1,
                Priority::Low => low_after += 1,
            }
            events.push(event);
        }
        // Restore events
        for event in events {
            let _ = queue.push(event);
        }

        // High priority should be preserved more than normal, normal more than low
        if shrink_to >= high_before {
            prop_assert_eq!(high_after, high_before);
        }

        if shrink_to >= high_before + normal_before {
            prop_assert_eq!(normal_after, normal_before);
        }
    }

    #[test]
    fn prop_backpressure_threshold_correct(
        size in 10usize..10000
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(size);

        // Fill to just below 80%
        let below_threshold = (size as f64 * 0.79) as usize;
        for i in 0..below_threshold {
            queue.push(Event::User(TestMsg(i as u32))).unwrap();
        }

        prop_assert!(!queue.is_backpressure_active());

        // Add events to go above 80%
        let above_threshold = (size as f64 * 0.81) as usize;
        for i in below_threshold..above_threshold.min(size) {
            queue.push(Event::User(TestMsg(i as u32))).unwrap();
        }

        if queue.len() >= (size as f64 * 0.8) as usize {
            prop_assert!(queue.is_backpressure_active());
        }
    }

    #[test]
    fn prop_auto_scaler_respects_bounds(
        min_size in 10usize..100,
        max_size in 101usize..1000,
        initial_size in 50usize..500,
        operations in 100usize..1000
    ) {
        let min = min_size.min(initial_size);
        let max = max_size.max(initial_size);

        let config = AutoScaleConfig {
            min_size: min,
            max_size: max,
            target_utilization: 0.5,
            evaluation_interval: 10,
            strategy: ScalingStrategy::Aggressive,
            cooldown: std::time::Duration::from_millis(0),
            debug: false,
        };

        let mut scaler = QueueAutoScaler::new(config);
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(initial_size);

        // Simulate random load patterns
        for i in 0..operations {
            if i % 20 < 15 {
                // High load period
                while queue.len() < queue.capacity() * 9 / 10 {
                    queue.push(Event::User(TestMsg(i as u32))).ok();
                }
            } else {
                // Low load period
                while queue.len() > queue.capacity() / 10 {
                    queue.pop();
                }
            }

            scaler.on_event_processed(&mut queue);

            // Capacity should always be within bounds
            prop_assert!(queue.capacity() >= min);
            prop_assert!(queue.capacity() <= max);
        }
    }

    #[test]
    fn prop_concurrent_resize_safe(
        initial_size in 50usize..200,
        resize_ops in prop::collection::vec(10usize..500, 2..10)
    ) {
        use std::sync::Arc;
        use std::sync::Mutex;
        use std::thread;

        let queue = Arc::new(Mutex::new(PriorityEventQueue::<TestMsg>::new(initial_size)));
        let mut handles = vec![];

        for new_size in resize_ops {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                let mut q = queue_clone.lock().unwrap();

                // Try to resize
                let _ = q.resize(new_size);

                // Try to add events
                for i in 0..10 {
                    let _ = q.push(Event::User(TestMsg(i)));
                }

                // Try to pop events
                for _ in 0..5 {
                    q.pop();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Queue should still be in valid state
        let q = queue.lock().unwrap();
        prop_assert!(q.len() <= q.capacity());
        prop_assert!(q.capacity() > 0);
    }

    #[test]
    fn prop_try_grow_shrink_operations(
        initial in 10usize..1000,
        operations in prop::collection::vec((prop::bool::ANY, 1usize..500), 1..50)
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(initial);

        for (is_grow, amount) in operations {
            let old_capacity = queue.capacity();

            let result = if is_grow {
                queue.try_grow(amount)
            } else {
                queue.try_shrink(amount)
            };

            prop_assert!(result.is_ok());
            let new_capacity = result.unwrap();

            prop_assert_eq!(queue.capacity(), new_capacity);

            if is_grow {
                prop_assert!(new_capacity >= old_capacity);
            } else {
                prop_assert!(new_capacity <= old_capacity);
                prop_assert!(new_capacity >= 1); // Never shrink to 0
            }
        }
    }

    #[test]
    fn prop_stats_consistency(
        size in 10usize..1000,
        events in 0usize..1000
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(size);

        let events_added = events.min(size);
        for i in 0..events_added {
            queue.push(Event::User(TestMsg(i as u32))).unwrap();
        }

        let stats = queue.stats();

        prop_assert_eq!(stats.current_size, queue.len());
        prop_assert_eq!(stats.max_size, queue.capacity());
        prop_assert!((stats.utilization - (queue.len() as f64 / queue.capacity() as f64)).abs() < 0.001);

        if queue.len() >= (queue.capacity() as f64 * 0.8) as usize {
            prop_assert!(stats.backpressure_active);
        } else {
            prop_assert!(!stats.backpressure_active);
        }
    }
}

// Helper function to count events of a specific priority
fn count_priority_events<M: Clone + Send + 'static>(
    queue: &mut PriorityEventQueue<M>,
    priority: Priority,
) -> usize {
    let mut count = 0;
    let mut events = Vec::new();

    while let Some(event) = queue.pop() {
        if Priority::from_event(&event) == priority {
            count += 1;
        }
        events.push(event);
    }

    // Restore events (though order might change)
    for event in events {
        let _ = queue.push(event);
    }

    count
}

#[test]
fn test_property_tests_compile() {
    // This ensures the property tests compile
    assert!(true);
}
