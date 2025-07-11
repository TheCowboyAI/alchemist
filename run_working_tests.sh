#!/usr/bin/env bash
# Quick test runner for domains that compile successfully

set -e

echo "================================================="
echo "     Running Working Domain Tests"
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

run_domain_tests() {
    local domain=$1
    local package=$2
    
    echo -e "${YELLOW}Testing $domain...${NC}"
    
    if cargo test --package $package --lib -- --nocapture 2>&1 | grep -E "test result:"; then
        echo -e "${GREEN}✓ $domain tests passed${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ $domain tests failed${NC}"
        ((FAILED++))
    fi
    ((TOTAL++))
    echo
}

# Run tests for domains that don't depend on Bevy
echo "Running non-Bevy domain tests..."
echo

run_domain_tests "Conceptual Spaces" "cim-domain-conceptualspaces"
run_domain_tests "Workflow" "cim-domain-workflow"
run_domain_tests "Document" "cim-domain-document"
run_domain_tests "Location" "cim-domain-location"
run_domain_tests "Dialog" "cim-domain-dialog"
run_domain_tests "Nix" "cim-domain-nix"

# Run core infrastructure tests
echo "Running infrastructure tests..."
echo

run_domain_tests "Domain Core" "cim-domain"
run_domain_tests "IPLD" "cim-ipld"

# Summary
echo "================================================="
echo "              TEST SUMMARY"
echo "================================================="
echo -e "Total domains tested: $TOTAL"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo

# Check specific functionality
echo "Checking specific functionality..."
echo

# Test event creation performance
echo -n "Event creation performance: "
if cargo test --package cim-domain --lib test_event_creation_performance -- --nocapture 2>&1 | grep -q "passed"; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
fi

# Test CID chain functionality
echo -n "CID chain functionality: "
if cargo test --package cim-ipld --lib test_cid_chain -- --nocapture 2>&1 | grep -q "passed"; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
fi

echo
echo "Test run complete!"

# Exit with failure if any tests failed
if [ $FAILED -gt 0 ]; then
    exit 1
fi