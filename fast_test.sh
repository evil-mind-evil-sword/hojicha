#\!/bin/bash
set -e

echo "Running fast test suite (excludes long-running property tests)..."

echo "📚 Library tests..."
cargo test --lib --all-features

echo "🧪 Quick integration tests..."
# Run specific fast tests
cargo test --test additional_coverage_tests
cargo test --test async_simple_test
cargo test --test deterministic_test_example
cargo test --test component_integration_test_simple

echo "✅ Fast tests complete\!"
echo ""
echo "To run ALL tests (including slow property tests): cargo test --all-features"
echo "To run ignored stress tests: cargo test --all-features -- --ignored"
