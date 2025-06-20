#!/usr/bin/env bash

# CIM Alchemist Conversation Test
echo "CIM Alchemist Agent - Interactive Conversation"
echo "============================================="
echo ""

# Function to send a question and wait for response
ask_question() {
    local question="$1"
    local msg_id=$(uuidgen || echo "test-$(date +%s)")
    
    echo "You: $question"
    echo ""
    
    # Create the message
    local message=$(jq -n \
        --arg id "$msg_id" \
        --arg content "$question" \
        --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        '{id: $id, content: $content, timestamp: $timestamp}')
    
    # Send the message
    echo "$message" | nats --server localhost:4222 pub cim.dialog.alchemist.request
    
    # Wait a moment for processing
    echo "Thinking..."
    sleep 2
    
    # Get the response using the consumer
    echo ""
    echo "Alchemist:"
    nats --server localhost:4222 consumer next CIM-AGENT-EVENTS DIALOG-RESPONSES --count 1 2>/dev/null | \
        grep -A100 '"content"' | \
        jq -r '.content' 2>/dev/null || echo "No response received"
    
    echo ""
    echo "---"
    echo ""
}

# Test conversation
ask_question "What is event sourcing in CIM?"
sleep 3

ask_question "How does CIM use NATS JetStream for event persistence?"
sleep 3

ask_question "Can you explain the relationship between ECS and DDD in CIM?"
sleep 3

ask_question "What are the 8 production-ready domains in CIM?"

echo ""
echo "Conversation complete!" 