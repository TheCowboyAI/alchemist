# Cursor Format Setup

This document explains how automatic code formatting is configured in this project.

## Automatic Formatting with treefmt

This project uses `treefmt` to automatically format code on save in Cursor/VS Code. The configuration is set up to format all file types according to the rules defined in `flake.nix`.

### What Gets Formatted

- **Rust files** (.rs): Formatted with `rustfmt`
- **Nix files** (.nix): Formatted with `nixpkgs-fmt`

### How It Works

When you save any file in Cursor, the following happens:
1. The save triggers the `emeraldwalk.runonsave` extension
2. It runs `nix develop -c treefmt ${file}` on the saved file
3. `treefmt` checks the file type and applies the appropriate formatter

### Required Extensions

To enable format on save, you need one of these VS Code/Cursor extensions:
- **Run on Save** by emeraldwalk (already configured)
- **Run on Save** by pucelle (alternative, also configured)

### Manual Formatting

You can also manually format files:

```bash
# Format all files in the project
nix develop -c treefmt

# Format specific files
nix develop -c treefmt src/main.rs

# Check formatting without changing files
nix develop -c treefmt --check
```

### Troubleshooting

If formatting on save isn't working:

1. **Check that you're in the Nix development shell**:
   ```bash
   nix develop
   ```

2. **Verify treefmt is available**:
   ```bash
   which treefmt
   ```

3. **Test manual formatting**:
   ```bash
   treefmt --version
   ```

4. **Check VS Code output**:
   - Open Output panel (View → Output)
   - Select "Run on Save" from the dropdown
   - Look for any error messages

### Customizing Format Rules

To modify formatting rules, edit the `treefmt.config` section in `flake.nix`:

```nix
treefmt.config = {
  projectRootFile = "flake.nix";
  programs = {
    rustfmt.enable = true;
    nixpkgs-fmt.enable = true;
    # Add more formatters here
  };
};
```

After changing the configuration, reload your development shell:
```bash
exit
nix develop
```
