#!/usr/bin/env bash

# Script to interact with CIM Alchemist agent via NATS CLI

echo "CIM Alchemist Agent - NATS CLI Interaction"
echo "=========================================="
echo ""
echo "This script allows you to send questions to the CIM Alchemist agent"
echo "and receive AI-powered responses about CIM architecture."
echo ""
echo "Make sure the agent is running first!"
echo ""

# Check if NATS CLI is available
if ! command -v nats &> /dev/null; then
    echo "Error: NATS CLI not found. Please install it first."
    exit 1
fi

# Function to send a message and wait for response
send_message() {
    local question="$1"
    local message_id=$(uuidgen || echo "$(date +%s)-$$")
    
    # Create JSON message
    local json_message=$(cat <<EOF
{
    "id": "$message_id",
    "content": "$question",
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ")"
}
EOF
)
    
    echo "Q: $question"
    echo ""
    
    # Send message and wait for response - use local NATS server
    echo "$json_message" | nats --server localhost:4222 req cim.dialog.alchemist.request cim.dialog.alchemist.response --timeout 30s | jq -r '.content' 2>/dev/null || echo "Timeout waiting for response"
    
    echo ""
    echo "---"
    echo ""
}

# Example questions
echo "Sending example questions to the agent..."
echo ""

send_message "What is the Composable Information Machine (CIM)?"
send_message "How does CIM implement event sourcing?"
send_message "What are the benefits of using graph workflows in CIM?"
send_message "How does CIM integrate Domain-Driven Design principles?"

# Interactive mode
echo "You can now ask your own questions. Type 'exit' to quit."
echo ""

while true; do
    read -p "Your question: " question
    
    if [[ "$question" == "exit" ]]; then
        echo "Goodbye!"
        break
    fi
    
    if [[ -n "$question" ]]; then
        send_message "$question"
    fi
done 