# Format on Save Setup for Cursor

## Configuration Added

I've created the necessary configuration files to enable proper formatting for each file type:

### Files Created:
- `.vscode/settings.json` - Cursor/VS Code settings
- `.vscode/tasks.json` - Task definition for nix fmt
- `.vscode/keybindings.json` - Keyboard shortcuts

## How It Works

### Rust Files (.rs)
- **Format on Save**: ✅ Enabled by default
- **Formatter**: rustfmt with nightly features
- **No extension needed** - works out of the box!

### Nix & TOML Files (.nix, .toml)
- **Format on Save**: Requires "Run on Save" extension
- **Formatter**: nix fmt (nixpkgs-fmt for .nix, configured formatter for .toml)

## Setup Options

### For Rust Files
Nothing to do! Rust files will automatically format when you save them.

### For Nix/TOML Files

#### Option 1: Using "Run on Save" Extension (Recommended)
1. Install the "Run on Save" extension in Cursor:
   - Open Command Palette (Cmd/Ctrl + Shift + P)
   - Type "Extensions: Install Extensions"
   - Search for "Run on Save" by emeraldwalk
   - Install it

2. Now `.nix` and `.toml` files will automatically format on save

#### Option 2: Manual Formatting
- Use keyboard shortcuts: `Shift+Alt+F` or `Ctrl+K Ctrl+F`
- Or run the task: Cmd/Ctrl + Shift + P → "Tasks: Run Task" → "nix fmt"
- Or use the terminal: `nix fmt`

## What's Configured

- ✅ Rust files use rustfmt directly (with nightly features)
- ✅ Nix and TOML files use `nix fmt`
- ✅ Configured terminal to use zsh (your default shell)
- ✅ Added file cleanup (trim whitespace, add final newline)

## Testing

### Test Rust Formatting:
1. Open any `.rs` file
2. Make a change (e.g., add extra spaces)
3. Save (Cmd/Ctrl + S)
4. File should auto-format using rustfmt

### Test Nix/TOML Formatting:
1. Install "Run on Save" extension first
2. Open a `.nix` or `.toml` file
3. Make a change
4. Save - it should format using nix fmt

## Troubleshooting

If format-on-save isn't working:
- **For Rust**: Check that rust-analyzer extension is installed and running
- **For Nix/TOML**: Ensure you're in the nix devshell and "Run on Save" extension is installed
- Check the Output panel in Cursor for any error messages
