#!/bin/bash
# Run performance benchmarks for Alchemist

set -e

echo "=== Alchemist Performance Benchmarks ==="
echo "Date: $(date)"
echo "Commit: $(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
echo

# Check if criterion is available
if ! cargo bench --no-run 2>/dev/null; then
    echo "Installing benchmark dependencies..."
    cargo build --benches
fi

# Run benchmarks
echo "Running benchmarks..."
echo "This may take a few minutes..."
echo

# Run the benchmarks and save results
BENCH_OUTPUT=$(mktemp)
cargo bench --bench alchemist_benchmarks -- --verbose | tee "$BENCH_OUTPUT"

# Extract summary
echo
echo "=== Benchmark Summary ==="
grep -E "time:.*\[(.*)\]" "$BENCH_OUTPUT" | tail -20 || echo "No timing results found"

# Check for regressions
echo
echo "=== Performance Analysis ==="

# Count improvements vs regressions
IMPROVEMENTS=$(grep -c "improved" "$BENCH_OUTPUT" 2>/dev/null || echo "0")
REGRESSIONS=$(grep -c "regressed" "$BENCH_OUTPUT" 2>/dev/null || echo "0")

echo "Improvements: $IMPROVEMENTS"
echo "Regressions: $REGRESSIONS"

if [ "$REGRESSIONS" -gt 0 ]; then
    echo
    echo "⚠️  Performance regressions detected!"
    grep "regressed" "$BENCH_OUTPUT" | head -10
fi

# Clean up
rm -f "$BENCH_OUTPUT"

echo
echo "Full benchmark results available at: target/criterion/"
echo "Open target/criterion/report/index.html for detailed reports"
echo

# Optional: Run specific performance tests
if [ "$1" == "--with-tests" ]; then
    echo "Running performance integration tests..."
    cargo test --test test_performance_integration -- --nocapture
    cargo test --test test_cache_rate_limit -- --nocapture
fi

echo "Benchmarks complete!"