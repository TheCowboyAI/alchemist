#!/bin/bash

# QA Domain Review Script
# This script checks the consistency of all CIM domains

echo "=== CIM Domain QA Review ==="
echo "Date: $(date)"
echo ""

# Define expected structure
EXPECTED_DIRS=(
    "src/aggregate"
    "src/commands"
    "src/events"
    "src/handlers"
    "src/value_objects"
    "src/queries"
    "src/projections"
    "tests"
    "examples"
    "doc"
)

EXPECTED_FILES=(
    "src/lib.rs"
    "Cargo.toml"
    "README.md"
)

EXPECTED_DOC_FILES=(
    "doc/user-stories.md"
    "doc/api.md"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Find all domain directories
DOMAINS=$(find . -name "cim-domain-*" -type d -maxdepth 1 | sort)

# Summary counters
TOTAL_DOMAINS=0
COMPLETE_DOMAINS=0
MISSING_ITEMS=""

echo "Checking domain structure consistency..."
echo "========================================"

for domain in $DOMAINS; do
    TOTAL_DOMAINS=$((TOTAL_DOMAINS + 1))
    domain_name=$(basename "$domain")
    echo ""
    echo "Checking: $domain_name"
    echo "----------------------------"
    
    DOMAIN_COMPLETE=true
    MISSING_IN_DOMAIN=""
    
    # Check expected directories
    for dir in "${EXPECTED_DIRS[@]}"; do
        if [ -d "$domain/$dir" ]; then
            echo -e "${GREEN}✓${NC} $dir"
        else
            echo -e "${RED}✗${NC} $dir (missing)"
            DOMAIN_COMPLETE=false
            MISSING_IN_DOMAIN="$MISSING_IN_DOMAIN\n  - $dir"
        fi
    done
    
    # Check expected files
    for file in "${EXPECTED_FILES[@]}"; do
        if [ -f "$domain/$file" ]; then
            echo -e "${GREEN}✓${NC} $file"
        else
            echo -e "${RED}✗${NC} $file (missing)"
            DOMAIN_COMPLETE=false
            MISSING_IN_DOMAIN="$MISSING_IN_DOMAIN\n  - $file"
        fi
    done
    
    # Check documentation files
    for doc in "${EXPECTED_DOC_FILES[@]}"; do
        if [ -f "$domain/$doc" ]; then
            echo -e "${GREEN}✓${NC} $doc"
        else
            echo -e "${YELLOW}⚠${NC} $doc (missing documentation)"
            MISSING_IN_DOMAIN="$MISSING_IN_DOMAIN\n  - $doc"
        fi
    done
    
    # Check for test files
    test_count=$(find "$domain/tests" -name "*.rs" 2>/dev/null | wc -l)
    if [ $test_count -gt 0 ]; then
        echo -e "${GREEN}✓${NC} Tests found: $test_count test files"
    else
        echo -e "${YELLOW}⚠${NC} No test files found"
    fi
    
    # Check for examples
    example_count=$(find "$domain/examples" -name "*.rs" 2>/dev/null | wc -l)
    if [ $example_count -gt 0 ]; then
        echo -e "${GREEN}✓${NC} Examples found: $example_count example files"
    else
        echo -e "${YELLOW}⚠${NC} No example files found"
    fi
    
    if [ "$DOMAIN_COMPLETE" = true ]; then
        COMPLETE_DOMAINS=$((COMPLETE_DOMAINS + 1))
        echo -e "${GREEN}Domain structure complete!${NC}"
    else
        echo -e "${RED}Domain structure incomplete!${NC}"
        MISSING_ITEMS="$MISSING_ITEMS\n\n$domain_name:$MISSING_IN_DOMAIN"
    fi
done

echo ""
echo "========================================"
echo "Summary:"
echo "Total domains: $TOTAL_DOMAINS"
echo "Complete domains: $COMPLETE_DOMAINS"
echo "Incomplete domains: $((TOTAL_DOMAINS - COMPLETE_DOMAINS))"

if [ -n "$MISSING_ITEMS" ]; then
    echo ""
    echo "Missing items by domain:"
    echo -e "$MISSING_ITEMS"
fi

# Check for cross-domain consistency
echo ""
echo "========================================"
echo "Cross-Domain Consistency Checks:"
echo ""

# Check if all domains follow the same event/command naming pattern
echo "Checking naming conventions..."
for domain in $DOMAINS; do
    domain_name=$(basename "$domain" | sed 's/cim-domain-//')
    
    # Check command naming
    if [ -d "$domain/src/commands" ]; then
        commands=$(find "$domain/src/commands" -name "*.rs" -exec basename {} \; 2>/dev/null | grep -v "mod.rs" | sort)
        if [ -n "$commands" ]; then
            echo ""
            echo "$domain_name commands:"
            echo "$commands" | sed 's/^/  - /'
        fi
    fi
done

echo ""
echo "========================================"
echo "QA Review Complete!" 