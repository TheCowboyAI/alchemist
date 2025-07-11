#!/bin/bash
# Test AI APIs with real keys

echo "Testing AI Model Connections..."
echo "==============================="

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Test Anthropic models
echo -e "\n1. Testing Anthropic Models..."
cargo test --test test_ai_real_api test_anthropic_connection -- --ignored --nocapture

# Test OpenAI models
echo -e "\n2. Testing OpenAI Models..."
cargo test --test test_ai_real_api test_openai_connection -- --ignored --nocapture

# Test streaming
echo -e "\n3. Testing Streaming Response..."
cargo test --test test_ai_real_api test_streaming_response -- --ignored --nocapture

# Test dialog conversation
echo -e "\n4. Testing Dialog Conversation..."
cargo test --test test_ai_real_api test_dialog_conversation -- --ignored --nocapture

echo -e "\n✅ All AI tests completed!"