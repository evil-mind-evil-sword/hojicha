#\!/bin/bash
set -e

echo "🔍 Running quick CI checks..."

echo "📝 Checking formatting..."
if cargo fmt --all -- --check; then
    echo "✅ Formatting OK"
else
    echo "❌ Formatting issues found. Run 'cargo fmt --all' to fix."
    exit 1
fi

echo "📎 Running clippy..."
if cargo clippy --all-targets --all-features 2>&1 | grep -q "error:"; then
    echo "❌ Clippy found errors"
    cargo clippy --all-targets --all-features 2>&1 | grep "error:"
    exit 1
else
    echo "✅ Clippy OK (warnings allowed)"
fi

echo "🧪 Running lib tests only..."
if cargo test --lib --all-features; then
    echo "✅ Lib tests passed"
else
    echo "❌ Lib tests failed"
    exit 1
fi

echo "📚 Checking documentation..."
if cargo doc --no-deps --all-features --quiet; then
    echo "✅ Documentation builds"
else
    echo "❌ Documentation failed"
    exit 1
fi

echo "✅ All quick checks passed\!"
