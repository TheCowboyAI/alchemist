#!/bin/bash
# Run all user story tests with categorized output

set -e

echo "================================================="
echo "     Alchemist User Story Test Suite"
echo "================================================="
echo

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test counters
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

run_test_category() {
    local category=$1
    local test_pattern=$2
    
    echo -e "${YELLOW}Running $category tests...${NC}"
    
    if cargo test --test comprehensive_user_story_tests $test_pattern -- --nocapture 2>&1; then
        echo -e "${GREEN}✓ $category tests passed${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ $category tests failed${NC}"
        ((FAILED++))
    fi
    ((TOTAL++))
    echo
}

# Run tests by category
echo "1. AI Management Tests"
run_test_category "AI Management" "test_ai_"

echo "2. Dialog Management Tests"
run_test_category "Dialog Management" "test_dialog_"

echo "3. Policy Management Tests"
run_test_category "Policy Management" "test_policy_"
run_test_category "Claims Management" "test_claims_"

echo "4. Domain Management Tests"
run_test_category "Domain Management" "test_domain_"

echo "5. Deployment Tests"
run_test_category "Deployment" "test_deployment_"
run_test_category "Nix Deployment" "test_nix_"

echo "6. Workflow Management Tests"
run_test_category "Workflow Management" "test_workflow_"

echo "7. Event Monitoring Tests"
run_test_category "Event Monitoring" "test_event_"

echo "8. Rendering Tests"
run_test_category "Graph Rendering" "test_graph_3d_"
run_test_category "Document Rendering" "test_document_"
run_test_category "Text Editor" "test_text_"
run_test_category "Chart Rendering" "test_chart_"
run_test_category "Markdown Rendering" "test_markdown_"

echo "9. Dashboard Tests"
run_test_category "Dashboard" "test_dashboard_"
run_test_category "NATS Dashboard" "test_nats_dashboard"
run_test_category "Performance Monitor" "test_performance_monitoring"

echo "10. Graph Processing Tests"
run_test_category "Graph Loading" "test_graph_file_"
run_test_category "Graph Persistence" "test_graph_persistence"
run_test_category "Graph Components" "test_graph_components"
run_test_category "Graph Algorithms" "test_graph_algorithms"

echo "11. System Integration Tests"
run_test_category "NATS Integration" "test_nats_integration"
run_test_category "Cross-Domain Events" "test_cross_domain"
run_test_category "Error Handling" "test_error_"

echo "12. Progress Tracking Tests"
run_test_category "Progress Tracking" "test_progress_"

echo "13. Configuration Tests"
run_test_category "Configuration" "test_configuration_"

echo "14. Performance Tests"
run_test_category "Performance" "test_performance_"

# Summary
echo "================================================="
echo "              TEST SUMMARY"
echo "================================================="
echo -e "Total test categories: $TOTAL"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo -e "Skipped: ${YELLOW}$SKIPPED${NC}"
echo

# Additional test suites
echo "Running additional test suites..."
echo

# Run other test files
OTHER_TESTS=(
    "ai_model_tests"
    "ai_streaming_tests"
    "policy_engine_tests"
    "shell_command_tests"
    "event_driven_tests"
    "graph_integration_test"
    "deployment_automation_tests"
)

for test in "${OTHER_TESTS[@]}"; do
    if cargo test --test $test -- --nocapture 2>&1; then
        echo -e "${GREEN}✓ $test passed${NC}"
    else
        echo -e "${RED}✗ $test failed${NC}"
    fi
done

echo
echo "================================================="
echo "         COVERAGE REPORT GENERATION"
echo "================================================="
echo

# Generate coverage report if tarpaulin is installed
if command -v cargo-tarpaulin &> /dev/null; then
    echo "Generating code coverage report..."
    cargo tarpaulin --out Html --output-dir target/coverage
    echo "Coverage report generated at: target/coverage/index.html"
else
    echo "Install cargo-tarpaulin for code coverage:"
    echo "  cargo install cargo-tarpaulin"
fi

echo
echo "Test run complete!"

# Exit with failure if any tests failed
if [ $FAILED -gt 0 ]; then
    exit 1
fi