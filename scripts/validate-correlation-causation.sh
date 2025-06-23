#!/usr/bin/env bash
# Validate that all events, commands, and queries implement correlation/causation properly

set -euo pipefail

echo "🔍 Validating Correlation/Causation Implementation..."
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track violations
VIOLATIONS=0

# Check for optional correlation/causation fields (should be required)
echo "Checking for optional correlation/causation fields..."
if grep -r "correlation_id: Option<" --include="*.rs" src/ cim-*/src/ 2>/dev/null | grep -v "test" | grep -v "example"; then
    echo -e "${RED}❌ Found optional correlation_id fields - these should be required!${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
else
    echo -e "${GREEN}✅ No optional correlation_id fields found${NC}"
fi

if grep -r "causation_id: Option<" --include="*.rs" src/ cim-*/src/ 2>/dev/null | grep -v "test" | grep -v "example"; then
    echo -e "${RED}❌ Found optional causation_id fields - these should be required!${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
else
    echo -e "${GREEN}✅ No optional causation_id fields found${NC}"
fi

echo

# Check for direct UUID creation in events (should use MessageFactory)
echo "Checking for direct UUID creation in events..."
if grep -r "Uuid::new_v4()" --include="*.rs" src/ cim-*/src/ 2>/dev/null | grep -E "(correlation|causation)" | grep -v "MessageFactory" | grep -v "test"; then
    echo -e "${RED}❌ Found direct UUID creation for correlation/causation - use MessageFactory!${NC}"
    VIOLATIONS=$((VIOLATIONS + 1))
else
    echo -e "${GREEN}✅ No direct UUID creation for correlation/causation found${NC}"
fi

echo

# Check for NATS publish without headers
echo "Checking for NATS publish without correlation headers..."
if grep -r "\.publish(" --include="*.rs" src/ cim-*/src/ 2>/dev/null | grep -v "publish_with_headers" | grep -v "test" | grep -v "example"; then
    echo -e "${YELLOW}⚠️  Found publish calls without headers - verify they include correlation headers${NC}"
    echo "   (This may be a false positive if headers are added differently)"
fi

echo

# Check for event/command/query structs without correlation fields
echo "Checking for message types without correlation fields..."
MISSING_CORRELATION=0

# Function to check a file for message types
check_file_for_messages() {
    local file=$1
    
    # Skip test and example files
    if [[ "$file" == *"test"* ]] || [[ "$file" == *"example"* ]]; then
        return
    fi
    
    # Look for Event, Command, or Query structs
    if grep -E "^(pub )?struct \w+(Event|Command|Query)" "$file" >/dev/null 2>&1; then
        # Check if the file contains correlation_id
        if ! grep -E "(correlation_id|correlation_id:)" "$file" >/dev/null 2>&1; then
            echo -e "${YELLOW}   Warning: $file may have message types without correlation_id${NC}"
            MISSING_CORRELATION=$((MISSING_CORRELATION + 1))
        fi
    fi
}

# Check all Rust files
for file in $(find src/ cim-*/src/ -name "*.rs" -type f 2>/dev/null); do
    check_file_for_messages "$file"
done

if [ $MISSING_CORRELATION -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Found $MISSING_CORRELATION files that may need correlation fields${NC}"
fi

echo

# Check for correlation tests
echo "Checking for correlation tests..."
CORRELATION_TESTS=$(find . -name "*test*.rs" -type f -exec grep -l "correlation" {} \; 2>/dev/null | wc -l)
if [ $CORRELATION_TESTS -lt 10 ]; then
    echo -e "${YELLOW}⚠️  Only found $CORRELATION_TESTS test files with correlation tests - consider adding more${NC}"
else
    echo -e "${GREEN}✅ Found $CORRELATION_TESTS test files with correlation tests${NC}"
fi

echo

# Summary
echo "========================================="
if [ $VIOLATIONS -eq 0 ]; then
    echo -e "${GREEN}✅ All critical checks passed!${NC}"
    echo
    echo "Next steps:"
    echo "1. Review any warnings above"
    echo "2. Ensure all new events use MessageFactory"
    echo "3. Add correlation tests for new features"
else
    echo -e "${RED}❌ Found $VIOLATIONS critical violations!${NC}"
    echo
    echo "Required actions:"
    echo "1. Make correlation_id and causation_id required (not Option<>)"
    echo "2. Use MessageFactory for all message creation"
    echo "3. Include correlation headers in all NATS publishes"
    exit 1
fi

echo
echo "📚 See /doc/design/event-correlation-causation-algebra.md for details"
echo "📚 See /doc/design/event-correlation-implementation-guide.md for patterns" 