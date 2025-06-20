#!/usr/bin/env bash

echo "=== Information Alchemist UI Debug ==="
echo

# Check if the app is running
if pgrep -f "cargo run --bin ia" > /dev/null; then
    echo "✅ App is running"
else
    echo "❌ App is not running"
    echo "Starting the app..."
    nix develop -c cargo run --bin ia &
    sleep 5
fi

echo
echo "=== Instructions ==="
echo "1. The app should be running now"
echo "2. Press F1 to open the AI Assistant chat window"
echo "3. Type a question like 'What is CIM?' and press Send"
echo "4. The agent should respond with information about CIM"
echo
echo "=== Troubleshooting ==="
echo "- Make sure you have a graphical window open"
echo "- The app window title should be 'Information Alchemist'"
echo "- If you don't see a window, check if Wayland/X11 is working"
echo
echo "=== Event Flow ==="
echo "1. F1 key → Toggle chat window visibility"
echo "2. Send button → AgentQuestionEvent"
echo "3. Agent processes → AgentResponseEvent"
echo "4. UI displays response in chat"
echo
echo "Press Ctrl+C to exit" 