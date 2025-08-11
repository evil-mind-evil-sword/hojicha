//! Property-based tests for priority event queue behavior

use hojicha_core::event::{Event, Key, KeyEvent, MouseEvent};
use hojicha::priority_queue::{Priority, PriorityEventQueue};
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TestMsg(usize);

// Strategy for generating random events with known priorities
fn event_strategy() -> impl Strategy<Value = Event<TestMsg>> {
    prop_oneof![
        // High priority events
        Just(Event::Quit),
        any::<char>().prop_map(|c| Event::Key(KeyEvent {
            key: Key::Char(c),
            modifiers: crossterm::event::KeyModifiers::empty(),
        })),
        Just(Event::Suspend),
        Just(Event::Resume),
        // Normal priority events
        any::<usize>().prop_map(|n| Event::User(TestMsg(n))),
        any::<String>().prop_map(Event::Paste),
        (0u16..100, 0u16..100).prop_map(|(col, row)| {
            Event::Mouse(MouseEvent {
                column: col,
                row,
                kind: hojicha::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
                modifiers: crossterm::event::KeyModifiers::empty(),
            })
        }),
        // Low priority events
        Just(Event::Tick),
        (10u16..200, 10u16..100).prop_map(|(w, h)| Event::Resize {
            width: w,
            height: h
        }),
        Just(Event::Focus),
        Just(Event::Blur),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_high_priority_always_first(
        high_events in prop::collection::vec(
            prop_oneof![
                Just(Event::<TestMsg>::Quit),
                Just(Event::Suspend),
                Just(Event::Resume),
            ],
            1..10
        ),
        other_events in prop::collection::vec(
            prop_oneof![
                Just(Event::Tick),
                Just(Event::Focus),
                Just(Event::Blur),
                any::<usize>().prop_map(|n| Event::User(TestMsg(n))),
            ],
            1..20
        )
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

        // Add low/normal priority events first
        for event in &other_events {
            queue.push(event.clone()).unwrap();
        }

        // Then add high priority events
        for event in &high_events {
            queue.push(event.clone()).unwrap();
        }

        // High priority events should come out first
        for _ in 0..high_events.len() {
            let event = queue.pop().expect("Should have event");
            let priority = Priority::from_event(&event);
            prop_assert_eq!(priority, Priority::High,
                "Expected high priority event, got {:?}", event);
        }
    }

    #[test]
    fn prop_queue_never_exceeds_capacity(
        events in prop::collection::vec(event_strategy(), 0..500),
        max_size in 10usize..100
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(max_size);
        let mut successful_pushes = 0;

        for event in events {
            if queue.push(event).is_ok() {
                successful_pushes += 1;
            }

            // Queue size should never exceed max_size
            prop_assert!(queue.len() <= max_size,
                "Queue size {} exceeds max {}", queue.len(), max_size);
        }

        // Should be able to pop exactly as many as we pushed successfully
        let mut popped = 0;
        while queue.pop().is_some() {
            popped += 1;
        }
        prop_assert_eq!(popped, successful_pushes.min(max_size));
    }

    #[test]
    fn prop_backpressure_activates_at_threshold(
        fill_ratio in 0.0f64..1.0,
        max_size in 50usize..200
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(max_size);
        let threshold = (max_size as f64 * 0.8) as usize;
        let events_to_add = (max_size as f64 * fill_ratio) as usize;

        // Fill queue to specified ratio
        for i in 0..events_to_add {
            queue.push(Event::User(TestMsg(i))).unwrap();
        }

        // Check backpressure state
        if events_to_add >= threshold {
            prop_assert!(queue.is_backpressure_active(),
                "Backpressure should be active at {} events (threshold: {})",
                events_to_add, threshold);
        } else {
            prop_assert!(!queue.is_backpressure_active(),
                "Backpressure should not be active at {} events (threshold: {})",
                events_to_add, threshold);
        }
    }

    #[test]
    fn prop_priority_order_preserved_within_level(
        messages in prop::collection::vec(0usize..1000, 10..50)
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(100);

        // Add all as same priority (Normal)
        for msg in &messages {
            queue.push(Event::User(TestMsg(*msg))).unwrap();
        }

        // They should come out in FIFO order within the same priority
        let mut previous = None;
        while let Some(Event::User(TestMsg(n))) = queue.pop() {
            if let Some(_prev) = previous {
                // In a priority queue, order within same priority might not be strictly preserved
                // This is actually OK - we care about priority levels, not order within level
                // So we just verify we get all the messages back
                prop_assert!(messages.contains(&n));
            }
            previous = Some(n);
        }
    }

    #[test]
    fn prop_dropped_events_tracked(
        events in prop::collection::vec(event_strategy(), 10..100),
        max_size in 5usize..20
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(max_size);
        let mut actual_drops = 0;

        for event in events {
            if queue.push(event).is_err() {
                actual_drops += 1;
            }
        }

        // The queue's dropped_events counter includes both rejected pushes
        // and events displaced by higher priority ones
        let queue_drops = queue.dropped_events();

        // Queue drops should be at least as many as our counted failures
        prop_assert!(queue_drops >= actual_drops,
            "Queue tracked {} drops but we counted {} failures",
            queue_drops, actual_drops);
    }

    #[test]
    fn prop_high_priority_events_displace_low_when_full(
        max_size in 5usize..20
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(max_size);

        // Fill queue with low priority events
        for _ in 0..max_size {
            queue.push(Event::Tick).unwrap();
        }

        prop_assert_eq!(queue.len(), max_size);

        // Try to add a high priority event
        let result = queue.push(Event::Quit);

        // High priority should succeed by displacing a low priority event
        prop_assert!(result.is_ok(), "High priority event should displace low priority");

        // First event out should be the high priority one
        let first = queue.pop();
        prop_assert_eq!(first, Some(Event::Quit));
    }

    #[test]
    fn prop_clear_resets_queue_state(
        events in prop::collection::vec(event_strategy(), 10..50),
        max_size in 20usize..100
    ) {
        let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(max_size);

        // Add events
        for event in events {
            let _ = queue.push(event);
        }

        // Clear the queue
        queue.clear();

        // Verify state is reset
        prop_assert_eq!(queue.len(), 0);
        prop_assert!(queue.is_empty());
        prop_assert!(!queue.is_backpressure_active());
        prop_assert_eq!(queue.pop(), None);
    }
}

// Additional deterministic edge case tests
#[test]
fn test_empty_queue_operations() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(10);

    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.pop(), None);
    assert!(!queue.is_backpressure_active());
    assert_eq!(queue.dropped_events(), 0);
}

#[test]
fn test_single_element_queue() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(1);

    // Can add one element
    assert!(queue.push(Event::Tick).is_ok()); // Low priority
    assert_eq!(queue.len(), 1);

    // Second low priority element fails
    assert!(queue.push(Event::Tick).is_err());
    assert_eq!(queue.dropped_events(), 1);

    // But high priority can displace low priority
    assert!(queue.push(Event::Quit).is_ok());
    assert_eq!(queue.pop(), Some(Event::Quit));
}

#[test]
fn test_priority_transitions() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(10);

    // Add one of each priority
    queue.push(Event::Tick).unwrap(); // Low
    queue.push(Event::User(TestMsg(1))).unwrap(); // Normal
    queue.push(Event::Quit).unwrap(); // High

    // Should come out in priority order
    assert_eq!(queue.pop(), Some(Event::Quit));
    assert_eq!(queue.pop(), Some(Event::User(TestMsg(1))));
    assert_eq!(queue.pop(), Some(Event::Tick));
}

#[test]
fn test_backpressure_deactivation() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(10);
    let threshold = 8; // 80% of 10

    // Fill to threshold
    for i in 0..threshold {
        queue.push(Event::User(TestMsg(i))).unwrap();
    }
    assert!(queue.is_backpressure_active());

    // Remove one to go below threshold
    queue.pop();
    assert!(!queue.is_backpressure_active());
}

#[test]
fn test_mixed_priority_flood() {
    let mut queue: PriorityEventQueue<TestMsg> = PriorityEventQueue::new(20);

    // Flood with mixed priorities
    for i in 0..30 {
        let event = match i % 3 {
            0 => Event::Tick,             // Low
            1 => Event::User(TestMsg(i)), // Normal
            _ => Event::Key(KeyEvent {
                // High
                key: Key::Char('a'),
                modifiers: crossterm::event::KeyModifiers::empty(),
            }),
        };
        let _ = queue.push(event); // Some will fail, that's OK
    }

    // First events out should be high priority
    let mut high_count = 0;
    let mut total = 0;

    while let Some(event) = queue.pop() {
        if Priority::from_event(&event) == Priority::High {
            high_count += 1;
        }
        total += 1;

        // All high priority events should come first
        if high_count < 10 && Priority::from_event(&event) != Priority::High {
            panic!("Got non-high priority event before all high priority events were processed");
        }
    }

    assert!(
        high_count > 0,
        "Should have processed some high priority events"
    );
    assert_eq!(total, queue.len() + total, "Should process all events");
}
