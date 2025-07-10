#!/usr/bin/env bash
# Test script to demonstrate dialog UI functionality

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "=== Alchemist Dialog UI Test ==="
echo ""
echo "This test demonstrates the dialog UI functionality."
echo ""

# Test 1: Check renderer binary exists
echo "1. Checking renderer binary..."
RENDERER_BIN="./target/x86_64-unknown-linux-gnu/debug/alchemist-renderer"
if [ ! -f "$RENDERER_BIN" ]; then
    # Try alternative location
    RENDERER_BIN="./target/debug/alchemist-renderer"
fi

if [ -f "$RENDERER_BIN" ]; then
    echo "   ✓ Renderer binary found at: $RENDERER_BIN"
else
    echo "   ✗ Renderer binary not found"
    echo "   Building renderer..."
    cd alchemist-renderer && cargo build --no-default-features && cd ..
    RENDERER_BIN="./target/x86_64-unknown-linux-gnu/debug/alchemist-renderer"
fi

# Test 2: Test dialog UI spawn command
echo ""
echo "2. Testing dialog UI spawn (would open window in GUI environment)..."
echo ""
echo "Command that would be run:"
echo "  ia dialog ui test-dialog-123"
echo ""

# Test 3: Show renderer help
echo "3. Renderer help:"
"$RENDERER_BIN" --help

# Test 4: Show what happens when dialog is launched
echo ""
echo "4. Simulating dialog launch..."
TEMP_FILE=$(mktemp)
cat > "$TEMP_FILE" << EOF
{
  "id": "test-123",
  "renderer": "Iced",
  "title": "AI Dialog Test",
  "data": {
    "type": "Dialog",
    "dialog_id": "test-dialog-123",
    "ai_model": "gpt-4",
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant.",
        "timestamp": "2024-01-10T10:00:00Z"
      },
      {
        "role": "user",
        "content": "Hello, how are you?",
        "timestamp": "2024-01-10T10:00:10Z"
      },
      {
        "role": "assistant",
        "content": "I'm doing well, thank you! How can I help you today?",
        "timestamp": "2024-01-10T10:00:15Z"
      }
    ],
    "system_prompt": "You are a helpful assistant."
  },
  "config": {
    "width": 800,
    "height": 600,
    "position": null,
    "fullscreen": false,
    "resizable": true,
    "always_on_top": false
  }
}
EOF

echo "Launching renderer with dialog data..."
"$RENDERER_BIN" iced --data-file "$TEMP_FILE" --id test-123

# Cleanup
rm -f "$TEMP_FILE"

echo ""
echo "=== Test Complete ==="
echo ""
echo "In a GUI environment, this would open a dialog window with:"
echo "  - AI model selector showing 'gpt-4'"
echo "  - Message history with system, user, and assistant messages"
echo "  - Input field for new messages"
echo "  - Send button to submit messages"
echo "  - Clear and Close buttons"
echo ""
echo "The renderer API provides:"
echo "  - spawn_dialog() method to create new dialog windows"
echo "  - Message streaming support for real-time AI responses"
echo "  - IPC communication between main process and renderer"
echo "  - Event handling for user interactions"