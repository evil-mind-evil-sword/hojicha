# Integration Test Curation Plan

## Tests to KEEP (Core Behavioral Tests)

### Essential Integration Tests
1. **integration_tests.rs** - Basic integration test demonstrating core TEA behavior
2. **headless_program_test.rs** - Tests headless mode (important for CI)
3. **api_refactor_tests.rs** - Tests public API behavior
4. **readme_examples_test.rs** - Ensures README examples work (documentation accuracy)

### Event Handling Behavior
1. **event_loop_integration_tests.rs** - Tests event loop behavior
2. **filter_test.rs** - Tests event filtering
3. **focus_test.rs** - Tests focus/blur events
4. **mouse_test.rs** - Tests mouse event handling
5. **paste_test.rs** - Tests paste event handling
6. **suspend_test.rs** - Tests suspend/resume

### Async Behavior
1. **async_simple_test.rs** - Simple async test
2. **async_bridge_integration.rs** - Async bridge integration
3. **cancellable_operations_tests.rs** - Tests cancellation behavior
4. **stream_integration_tests.rs** - Stream subscription behavior

### Component Behavior
1. **component_integration_test_simple.rs** - Component integration
2. **snapshot_tests.rs** - Visual regression tests

### Error Handling
1. **error_handling_tests.rs** - Error handling behavior

## Tests to REMOVE (Redundant/Implementation-focused)

### Coverage-focused Tests (Remove)
- **program_coverage_tests.rs** - 596 lines, just for coverage
- **additional_coverage_tests.rs** - 311 lines, coverage padding
- **program_comprehensive_tests.rs** - 499 lines, overlaps with other tests

### Duplicate Program Tests (Consolidate)
- **program_unit_tests.rs** - Move essential tests to integration_tests.rs
- **program_basic_tests.rs** - Redundant with integration_tests.rs

### Property Tests (Move to separate suite or remove)
- **program_property_tests.rs** - 460 lines, slow
- **event_command_property_tests.rs** - 461 lines, slow
- **metrics_property_tests.rs** - 374 lines, implementation detail
- **priority_queue_property_tests.rs** - 333 lines, implementation detail
- **queue_resize_property_tests.rs** - 354 lines, implementation detail
- **property_tests.rs** - Generic property tests

### Stress Tests (Move to separate suite)
- **async_bridge_stress_tests.rs** - Already ignored
- **priority_concurrent_stress_tests.rs** - Already ignored

### Implementation Detail Tests (Remove)
- **queue_resizing_test.rs** - 333 lines, tests internal queue
- **metrics_test.rs** - Internal metrics
- **event_priority_tests.rs** - Internal priority system
- **priority_default_test.rs** - Internal priority
- **event_queue_test.rs** - Internal queue

### Redundant Async Tests (Consolidate)
- **async_bridge_tests.rs** - Consolidate with async_bridge_integration.rs
- **async_bridge_working_tests.rs** - Redundant
- **refactored_async_bridge_test.rs** - Redundant
- **stream_subscription_tests.rs** - Consolidate with stream_integration_tests.rs
- **deterministic_test_example.rs** - Example code, not a real test

### Misc Tests to Review
- **send_test.rs** - Review if essential
- **input_and_logging_test.rs** - May be redundant
- **lifecycle_test.rs** - May be covered elsewhere

## Target State

From 45 test files → ~15 focused behavioral test files

## Benefits
1. **Faster test runs** - Remove 30+ redundant/slow tests
2. **Clearer purpose** - Each test has a specific behavioral focus
3. **Better maintenance** - Less duplicate code to maintain
4. **CI-friendly** - Quick feedback loop

## Next Steps
1. Create `tests/behavioral/` directory for core tests
2. Move property tests to `tests/property/` (run separately)
3. Move stress tests to `benches/` or remove
4. Consolidate redundant tests
5. Ensure remaining tests compile and pass