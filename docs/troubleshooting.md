# Alchemist Troubleshooting Guide

## Common Issues and Solutions

### Build Issues

#### 1. Compilation Errors with Bevy
**Problem**: `error: could not compile bevy_render`

**Solution**:
```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Rebuild with specific features
cargo build --release --no-default-features
```

#### 2. Missing System Dependencies
**Problem**: `error: failed to run custom build command for 'x11'`

**Solution** (Linux):
```bash
# Ubuntu/Debian
sudo apt-get install libx11-dev libxi-dev libxcursor-dev

# Fedora
sudo dnf install libX11-devel libXi-devel libXcursor-devel

# Arch
sudo pacman -S libx11 libxi libxcursor
```

### UI Issues

#### 1. Dashboard Window Not Opening
**Problem**: Window doesn't appear when running `ia dashboard-local`

**Solutions**:
- Check display server:
  ```bash
  echo $DISPLAY  # Should show :0 or similar
  ```
- For WSL2:
  ```bash
  export DISPLAY=:0
  # Install X server on Windows (VcXsrv or similar)
  ```
- For SSH:
  ```bash
  ssh -X user@host  # Enable X11 forwarding
  ```

#### 2. Iced Panic on Launch
**Problem**: `thread 'main' panicked at 'Failed to create window'`

**Solution**:
```bash
# Check GPU drivers
glxinfo | grep "OpenGL version"

# Try software rendering
export LIBGL_ALWAYS_SOFTWARE=1
ia dashboard-local
```

### NATS Issues

#### 1. JetStream Subject Overlap
**Problem**: `error: subject overlap with an existing stream`

**Solution**:
```bash
# Run the fix script
./scripts/fix_jetstream_overlap.sh

# Or manually:
nats stream delete CIM-EVENTS -f
nats stream delete DASHBOARD-EVENTS -f
```

#### 2. Connection Refused
**Problem**: `error: connection refused`

**Solution**:
```bash
# Check if NATS is running
ps aux | grep nats-server

# Start NATS with JetStream
nats-server -js

# Verify connection
nats server ping
```

### AI Model Issues

#### 1. API Key Not Found
**Problem**: `error: ANTHROPIC_API_KEY not found`

**Solution**:
```bash
# Create .env file
echo "ANTHROPIC_API_KEY=sk-ant-..." > .env
echo "OPENAI_API_KEY=sk-..." >> .env

# Or export directly
export ANTHROPIC_API_KEY="sk-ant-..."
```

#### 2. Rate Limiting
**Problem**: `error: rate limit exceeded`

**Solution**:
- Add delays between requests
- Use different models
- Implement exponential backoff:
  ```rust
  use tokio::time::sleep;
  use std::time::Duration;
  
  let mut delay = Duration::from_secs(1);
  for attempt in 0..3 {
      match ai_manager.get_completion(...).await {
          Ok(response) => break,
          Err(e) if e.to_string().contains("rate") => {
              sleep(delay).await;
              delay *= 2;
          }
          Err(e) => return Err(e),
      }
  }
  ```

### Dialog Window Issues

#### 1. Streaming Not Working
**Problem**: Tokens don't appear in real-time

**Solution**:
- Check model support for streaming
- Verify event channel isn't blocked
- Enable debug logging:
  ```bash
  RUST_LOG=alchemist::dialog_handler=debug ia dialog new
  ```

#### 2. Export Failing
**Problem**: `error: permission denied`

**Solution**:
```bash
# Check write permissions
ls -la .

# Change export directory
mkdir ~/alchemist-exports
cd ~/alchemist-exports
ia dialog export <dialog-id>
```

### Performance Issues

#### 1. High Memory Usage
**Problem**: Dashboard using too much memory

**Solutions**:
- Limit event history:
  ```rust
  // In dashboard_nats_stream.rs
  if self.current_data.recent_events.len() > 100 {
      self.current_data.recent_events.truncate(100);
  }
  ```
- Reduce update frequency
- Clear old dialogs

#### 2. Slow UI Response
**Problem**: UI feels sluggish

**Solutions**:
- Check CPU usage: `top -p $(pgrep ia)`
- Disable unnecessary features
- Use release build: `cargo build --release`

### Testing Issues

#### 1. Tests Timing Out
**Problem**: `error: test timed out after 60s`

**Solution**:
```bash
# Increase timeout
cargo test -- --test-threads=1 --timeout=300

# Skip slow tests
cargo test --lib  # Unit tests only
```

#### 2. Integration Tests Failing
**Problem**: NATS-dependent tests fail

**Solution**:
```bash
# Ensure NATS is running
nats-server -js &

# Run specific test
cargo test test_event_monitor -- --nocapture
```

## Debug Commands

### Enable Detailed Logging
```bash
# All modules
RUST_LOG=alchemist=trace ia dashboard-local

# Specific module
RUST_LOG=alchemist::dialog_handler=debug ia dialog new

# Multiple modules
RUST_LOG=alchemist::ai=debug,alchemist::dialog=debug ia dialog new
```

### Check System State
```bash
# NATS streams
nats stream list
nats stream info ALCHEMIST-EVENTS

# Process info
ps aux | grep ia
lsof -p $(pgrep ia)  # Open files/connections
```

### Clean State
```bash
# Remove local data
rm -rf ~/.local/share/alchemist
rm -rf ~/.config/alchemist

# Clean build
cargo clean
rm Cargo.lock
```

## Getting More Help

1. **Check logs**: Most errors are logged with context
2. **Run examples**: `cargo run --example <name>`
3. **Read source**: Error messages often point to specific files
4. **Ask community**: Open an issue with:
   - Error message
   - Steps to reproduce
   - System info (`uname -a`, `cargo --version`)

## Platform-Specific Notes

### NixOS
- Use `nix develop` for proper environment
- Check `flake.nix` for dependencies

### macOS
- Install XQuartz for X11 support
- Use `brew install nats-server`

### Windows
- Use WSL2 for best compatibility
- Native builds may have limited features