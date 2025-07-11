#!/bin/bash
# Comprehensive test runner for Alchemist

set -e

echo "🧪 Alchemist Comprehensive Test Suite"
echo "====================================="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run tests for a specific category
run_test_category() {
    local category=$1
    local test_pattern=$2
    
    echo -e "${YELLOW}Running $category tests...${NC}"
    
    if cargo test $test_pattern -- --nocapture; then
        echo -e "${GREEN}✅ $category tests passed${NC}"
    else
        echo -e "${RED}❌ $category tests failed${NC}"
        exit 1
    fi
    echo
}

# Check if NATS is running (for integration tests)
check_nats() {
    if nc -z localhost 4222 2>/dev/null; then
        echo -e "${GREEN}✅ NATS is running${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  NATS is not running. Some integration tests will be skipped.${NC}"
        return 1
    fi
}

# Start NATS if requested
if [ "$1" = "--with-nats" ]; then
    echo "Starting NATS server..."
    if command -v nats-server &> /dev/null; then
        nats-server -js &
        NATS_PID=$!
        sleep 2
        trap "kill $NATS_PID 2>/dev/null" EXIT
    else
        echo -e "${RED}nats-server not found. Install it with: nix-env -iA nixpkgs.nats-server${NC}"
        exit 1
    fi
fi

# Check environment
echo "Environment Check:"
echo "=================="
cargo --version
rustc --version
check_nats
echo

# Run unit tests
echo -e "${YELLOW}Running Unit Tests${NC}"
echo "=================="

run_test_category "Shell Commands" "shell_command_tests"
run_test_category "Event System" "event_driven_tests"
run_test_category "Policy Engine" "policy_engine_tests"
run_test_category "AI Integration" "ai_model_tests"
run_test_category "Deployment Automation" "deployment_automation_tests"

# Run integration tests
echo -e "${YELLOW}Running Integration Tests${NC}"
echo "========================"

run_test_category "Basic Integration" "basic_integration_test"
run_test_category "Cross-Domain Integration" "cross_domain_integration_test"
run_test_category "Renderer Integration" "renderer_integration_tests"
run_test_category "Workflow Execution" "test_workflow_execution"
run_test_category "Comprehensive Tests" "comprehensive_alchemist_tests"

# Run performance tests
echo -e "${YELLOW}Running Performance Tests${NC}"
echo "========================"

run_test_category "Performance Benchmarks" "performance_benchmark_test"
run_test_category "Stress Tests" "stress_tests"

# Run domain-specific tests
echo -e "${YELLOW}Running Domain Tests${NC}"
echo "==================="

# Graph domain
echo "Testing Graph Domain..."
cargo test -p cim-domain-graph -- --nocapture

# Workflow domain
echo "Testing Workflow Domain..."
cargo test -p cim-domain-workflow -- --nocapture

# Agent domain
echo "Testing Agent Domain..."
cargo test -p cim-domain-agent -- --nocapture

# Document domain
echo "Testing Document Domain..."
cargo test -p cim-domain-document -- --nocapture

echo

# Generate test coverage report if requested
if [ "$1" = "--coverage" ] || [ "$2" = "--coverage" ]; then
    echo -e "${YELLOW}Generating Coverage Report${NC}"
    echo "=========================="
    
    if command -v cargo-tarpaulin &> /dev/null; then
        cargo tarpaulin --out Html --output-dir coverage/
        echo -e "${GREEN}✅ Coverage report generated in coverage/tarpaulin-report.html${NC}"
    else
        echo -e "${YELLOW}cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin${NC}"
    fi
fi

# Summary
echo
echo -e "${GREEN}🎉 All tests completed successfully!${NC}"
echo
echo "Test Summary:"
echo "============="
echo "✅ Unit tests passed"
echo "✅ Integration tests passed"
echo "✅ Performance tests passed"
echo "✅ Domain tests passed"

# Count total tests
TOTAL_TESTS=$(cargo test -- --list 2>/dev/null | grep -E "test::" | wc -l)
echo
echo "Total tests run: ~$TOTAL_TESTS"

# Check for any warnings
echo
echo "Checking for warnings..."
if cargo check --all-features 2>&1 | grep -q "warning:"; then
    echo -e "${YELLOW}⚠️  Some warnings were found. Run 'cargo check --all-features' for details.${NC}"
else
    echo -e "${GREEN}✅ No warnings found${NC}"
fi

echo
echo "Done! 🚀"