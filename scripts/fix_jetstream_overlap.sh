#!/bin/bash
# Fix JetStream subject overlap by cleaning up conflicting streams

echo "Fixing JetStream Subject Overlap..."
echo "==================================="

# Check if NATS is running
if ! command -v nats &> /dev/null; then
    echo "❌ NATS CLI not found. Please install it first."
    exit 1
fi

# Default NATS URL
NATS_URL="${NATS_URL:-nats://localhost:4222}"

echo "Using NATS server: $NATS_URL"

# List existing streams
echo -e "\nExisting JetStream streams:"
nats stream list -s $NATS_URL

# Check for conflicting streams
echo -e "\nChecking for conflicting streams..."

# Delete old CIM-EVENTS stream if it exists
if nats stream info CIM-EVENTS -s $NATS_URL &> /dev/null; then
    echo "Found old CIM-EVENTS stream. Deleting..."
    nats stream delete CIM-EVENTS -f -s $NATS_URL
    echo "✅ Deleted CIM-EVENTS stream"
fi

# Delete old DASHBOARD-EVENTS stream if it exists
if nats stream info DASHBOARD-EVENTS -s $NATS_URL &> /dev/null; then
    echo "Found old DASHBOARD-EVENTS stream. Deleting..."
    nats stream delete DASHBOARD-EVENTS -f -s $NATS_URL
    echo "✅ Deleted DASHBOARD-EVENTS stream"
fi

# Create new unified event stream with proper configuration
echo -e "\nCreating unified ALCHEMIST-EVENTS stream..."
cat << EOF | nats stream add ALCHEMIST-EVENTS -s $NATS_URL
Subjects: cim.>, dashboard.>, alchemist.>
Storage: file
Retention: limits
Discard Policy: old
Max Messages: 1000000
Max Bytes: 1GB
Max Age: 7d
Max Message Size: 1MB
Duplicate Window: 2m
Allow Rollup: no
Deny Delete: no
Deny Purge: no
EOF

echo -e "\n✅ Created ALCHEMIST-EVENTS stream"

# Show final stream configuration
echo -e "\nFinal stream configuration:"
nats stream info ALCHEMIST-EVENTS -s $NATS_URL

echo -e "\n✅ JetStream overlap issue resolved!"
echo "The system will now use ALCHEMIST-EVENTS for all event storage."