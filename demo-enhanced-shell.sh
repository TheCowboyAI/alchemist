#!/usr/bin/env bash
# Demo script for enhanced interactive shell with tab completion and syntax highlighting

echo "=== Alchemist Enhanced Shell Demo ==="
echo ""
echo "The interactive shell now includes:"
echo "  ✓ Tab completion for commands and subcommands"
echo "  ✓ Syntax highlighting (commands in green, keywords in cyan)"
echo "  ✓ Command history (UP/DOWN arrows)"
echo "  ✓ ESC key to clear current input"
echo ""
echo "Try these features:"
echo "  1. Type 'ai' and press TAB to see subcommands"
echo "  2. Type 'd' and press TAB to see all commands starting with 'd'"
echo "  3. Use UP/DOWN arrows to navigate command history"
echo "  4. Commands are highlighted in green, subcommands in cyan"
echo ""
echo "Starting interactive shell..."
echo ""

# Run the interactive shell
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

./target/x86_64-unknown-linux-gnu/debug/ia -i