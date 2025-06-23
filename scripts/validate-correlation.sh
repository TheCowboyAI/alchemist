#!/usr/bin/env bash
# Validate that correlation and causation IDs are properly implemented

set -euo pipefail

echo "=== Correlation/Causation Validation ==="
echo

# Check for optional correlation/causation fields (should be required)
echo "Checking for optional correlation/causation fields..."
if grep -r "correlation_id: Option<" --include="*.rs" --exclude-dir=target --exclude-dir=doc/archive 2>/dev/null | grep -v "test" | grep -v "example"; then
    echo "❌ ERROR: Found optional correlation_id fields - these should be required!"
    exit 1
else
    echo "✅ No optional correlation_id fields found"
fi

if grep -r "causation_id: Option<" --include="*.rs" --exclude-dir=target --exclude-dir=doc/archive 2>/dev/null | grep -v "test" | grep -v "example"; then
    echo "❌ ERROR: Found optional causation_id fields - these should be required!"
    exit 1
else
    echo "✅ No optional causation_id fields found"
fi

echo

# Check for direct UUID creation in correlation/causation context
echo "Checking for direct UUID creation in correlation context..."
if grep -r "correlation_id.*Uuid::new_v4()" --include="*.rs" --exclude-dir=target --exclude-dir=doc/archive 2>/dev/null | grep -v "MessageFactory" | grep -v "test" | grep -v "example"; then
    echo "⚠️  WARNING: Found direct UUID creation for correlation_id - should use MessageFactory"
fi

echo

# Check for NATS publish without headers
echo "Checking for NATS publish without correlation headers..."
if grep -r "\.publish(" --include="*.rs" --exclude-dir=target --exclude-dir=doc/archive 2>/dev/null | grep -v "publish_with_headers" | grep -v "test" | grep -v "example" | head -5; then
    echo "⚠️  WARNING: Found publish calls without headers - correlation headers may be missing"
fi

echo

# Check for proper MessageFactory usage
echo "Checking for MessageFactory pattern..."
if grep -r "MessageFactory" --include="*.rs" --exclude-dir=target 2>/dev/null | head -5; then
    echo "✅ MessageFactory pattern found in codebase"
else
    echo "⚠️  WARNING: MessageFactory pattern not found - may need implementation"
fi

echo
echo "=== Validation Complete ===" 