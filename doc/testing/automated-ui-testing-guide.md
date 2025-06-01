# Automated UI Testing Guide for Bevy on Wayland

## Overview

Testing UI interactions in Bevy applications on Wayland presents unique challenges compared to traditional X11-based testing. This guide covers multiple approaches, from simple to complex, each with specific trade-offs.

## Comparison of Approaches

| Approach | Pros | Cons | Best For |
|----------|------|------|----------|
| **Headless Testing** | Fast, deterministic, CI-friendly | No visual validation, limited to simulated events | Unit/integration tests |
| **Enigo** | Cross-platform, real input simulation | Requires window focus, timing issues | End-to-end tests |
| **Custom Framework** | Full control, deterministic, visual validation | Complex to build, more code | Comprehensive test suites |
| **Manual Testing** | Real user experience | Time-consuming, not repeatable | Final validation |

## 1. Headless Testing (Recommended)

The most practical approach for CI/CD pipelines and rapid testing.

### Advantages
- ✅ Runs without display server
- ✅ Fast execution (no rendering overhead)
- ✅ Deterministic results
- ✅ Easy CI/CD integration
- ✅ No Wayland permission issues

### Disadvantages
- ❌ No visual validation
- ❌ May miss rendering-specific bugs
- ❌ Limited to event simulation

### Implementation
```rust
// See src/testing/headless_integration_test.rs
app.add_plugins(
    DefaultPlugins
        .set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: None,
                ..default()
            }),
            ..default()
        })
        .disable::<WinitPlugin>()
);
```

## 2. Enigo - External Input Simulation

Uses the Enigo library to simulate real mouse and keyboard input.

### Advantages
- ✅ Simulates real user input
- ✅ Works with visible windows
- ✅ Cross-platform (Linux/Windows/macOS)
- ✅ Supports Wayland through virtual input devices

### Disadvantages
- ❌ Requires window focus
- ❌ Timing-dependent (needs sleeps)
- ❌ Can be flaky on CI
- ❌ Limited to ASCII input on Wayland
- ❌ Requires elevated permissions

### Installation
```toml
[dev-dependencies]
enigo = "0.3"
```

### Wayland Setup
On Wayland, Enigo uses `/dev/uinput` which requires permissions:
```bash
# Add user to input group
sudo usermod -a -G input $USER
# Or run tests with sudo (not recommended)
```

## 3. Custom Bevy Test Framework

Build testing directly into your Bevy app for maximum control.

### Advantages
- ✅ Full control over test flow
- ✅ Can validate internal state
- ✅ Deterministic execution
- ✅ Visual validation possible
- ✅ No external dependencies

### Disadvantages
- ❌ More complex to implement
- ❌ Tests run inside production code
- ❌ Requires test/prod separation

### Key Features
- Action-based test scenarios
- Frame-perfect timing control
- State assertions
- Screenshot capabilities

## 4. Alternative Tools

### ydotool
- Command-line tool for Wayland
- Limited to ASCII characters
- Good for simple automation

### wtype
- Wayland-native typing tool
- Better Unicode support than ydotool
- Limited to keyboard input

### libei (Experimental)
- New Wayland input injection protocol
- Better security model
- Not yet widely supported

## Recommended Testing Strategy

### 1. **Unit Tests** (70%)
- Test individual systems and components
- Use headless Bevy app
- Fast and reliable

### 2. **Integration Tests** (20%)
- Test feature interactions
- Use custom test framework
- Run in headless mode

### 3. **E2E Tests** (10%)
- Full user scenarios
- Use Enigo or similar
- Run on real display

## Example Test Pipeline

```yaml
# CI/CD Pipeline (e.g., GitHub Actions)
test:
  steps:
    # Fast unit tests
    - name: Unit Tests
      run: cargo test --lib

    # Headless integration tests
    - name: Integration Tests
      run: cargo test --test '*' --features headless

    # Visual E2E tests (optional)
    - name: E2E Tests
      run: |
        # Start virtual display
        export DISPLAY=:99
        Xvfb :99 -screen 0 1920x1080x24 &
        cargo test --test e2e -- --ignored
```

## Best Practices

1. **Start with headless tests** - They're fast and reliable
2. **Use deterministic timing** - Count frames, not wall time
3. **Test behavior, not implementation** - Focus on user outcomes
4. **Keep tests independent** - Reset state between tests
5. **Mock external dependencies** - Network, file I/O, etc.
6. **Visual regression testing** - Capture screenshots for comparison

## Common Pitfalls

1. **Window focus issues** - Enigo requires window focus
2. **Timing problems** - Use frame counts, not sleep()
3. **Wayland permissions** - Input simulation needs special access
4. **Non-deterministic tests** - Avoid relying on system state

## Conclusion

For most Bevy projects:
1. Use **headless testing** for the majority of tests
2. Add a **custom test framework** for complex scenarios
3. Use **Enigo** sparingly for true E2E validation
4. Always prefer deterministic, fast tests over "realistic" slow ones

The key is finding the right balance between test coverage, execution speed, and maintenance burden.
