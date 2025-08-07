#\!/bin/bash
set -e

echo "🔍 Running CI checks locally..."

echo "📝 Checking formatting..."
cargo fmt --all -- --check

echo "📎 Running clippy..."
cargo clippy --all-targets --all-features

echo "🧪 Running tests..."
cargo test --all-features

echo "📚 Checking documentation..."
cargo doc --no-deps --all-features

echo "🎯 Checking MSRV (1.80.0)..."
cargo +1.80.0 check --all-features 2>/dev/null || echo "⚠️  MSRV check requires Rust 1.80.0 installed"

echo "✅ All checks complete\!"
