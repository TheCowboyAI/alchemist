# Bevy 0.16 Experimental Feature Linking Issues - Resolution Guide

## Problem Summary

When building a Bevy 0.16 application on NixOS with Rust nightly, we encountered undefined symbol errors related to experimental Bevy features, even when those features were not explicitly enabled or used in our code.

### Error Manifestation

```
./target/x86_64-unknown-linux-gnu/release/ia: symbol lookup error: ./target/x86_64-unknown-linux-gnu/release/ia: undefined symbol: _ZN120_$LT$bevy_render..experimental..occlusion_culling..OcclusionCullingSubview$u20$as$u20$bevy_ecs..component..Component$GT$28register_required_components17h0471bdf13f749a1eE
```

And:

```
./target/x86_64-unknown-linux-gnu/release/ia: undefined symbol: _ZN113_$LT$bevy_render..view..ViewDepthTexture$u20$as$u20$bevy_ecs..component..Component$GT$28register_required_components17h432436c838bef677
```

### Affected Components

1. `bevy_render::experimental::occlusion_culling::OcclusionCullingSubview`
2. `bevy_render::view::ViewDepthTexture`

## Root Cause Analysis

### 1. Component Derive Macro Issue

The Bevy ECS `#[derive(Component)]` macro is supposed to generate a `register_required_components` method for each component. However, for certain experimental components, this method was not being properly generated or exported when building with dynamic linking.

### 2. Symbol Visibility Problem

When examining the compiled libraries with `nm`:

```bash
nm -D target/x86_64-unknown-linux-gnu/debug/deps/libbevy_dylib-*.so | grep -E "(ViewDepthTexture|OcclusionCullingSubview)"
```

The symbols were marked with 'U' (undefined), indicating they were referenced but not implemented in the dynamic library.

### 3. Version Consistency

This issue affects both Bevy 0.16.0 and 0.16.1, indicating it's not a regression but a fundamental issue with how experimental features are compiled.

## Solution Implementation

### Step 1: Fork/Patch Bevy

Since the issue is within Bevy itself, we need to use a patched version. If you have a local Bevy checkout or submodule:

1. **For ViewDepthTexture** - Edit `bevy-patched/crates/bevy_render/src/view/mod.rs`:

```rust
// Replace the derive macro with manual implementation
// FROM:
#[derive(Component)]
pub struct ViewDepthTexture {
    pub texture: Texture,
    attachment: DepthAttachment,
}

// TO:
// PATCH: Manually implement Component to work around linking issues
pub struct ViewDepthTexture {
    pub texture: Texture,
    attachment: DepthAttachment,
}

impl bevy_ecs::component::Component for ViewDepthTexture {
    const STORAGE_TYPE: bevy_ecs::component::StorageType = bevy_ecs::component::StorageType::Table;
    type Mutability = bevy_ecs::component::Mutable;

    fn register_required_components(
        _requiree: bevy_ecs::component::ComponentId,
        _components: &mut bevy_ecs::component::ComponentsRegistrator,
        _required_components: &mut bevy_ecs::component::RequiredComponents,
        _inheritance_depth: u16,
        _recursion_check_stack: &mut Vec<bevy_ecs::component::ComponentId>
    ) {
        // No required components
    }
}
```

2. **For OcclusionCullingSubview** - Edit `bevy-patched/crates/bevy_render/src/experimental/occlusion_culling/mod.rs`:

First, update the imports:
```rust
use bevy_ecs::{component::{Component, ComponentId, ComponentsRegistrator, RequiredComponents}, entity::Entity, prelude::ReflectComponent};
```

Then replace the derive with manual implementation:
```rust
// FROM:
#[derive(Clone, Component)]
pub struct OcclusionCullingSubview {
    // ...
}

// TO:
// PATCH: Manually implement Component to work around linking issues
#[derive(Clone)]
pub struct OcclusionCullingSubview {
    // ...
}

// Add after OcclusionCullingSubviewEntities struct:
impl Component for OcclusionCullingSubview {
    const STORAGE_TYPE: bevy_ecs::component::StorageType = bevy_ecs::component::StorageType::Table;
    type Mutability = bevy_ecs::component::Mutable;

    fn register_required_components(
        _requiree: bevy_ecs::component::ComponentId,
        _components: &mut bevy_ecs::component::ComponentsRegistrator,
        _required_components: &mut bevy_ecs::component::RequiredComponents,
        _inheritance_depth: u16,
        _recursion_check_stack: &mut Vec<bevy_ecs::component::ComponentId>
    ) {
        // No required components
    }
}
```

### Step 2: Update Cargo.toml

Switch from using the crates.io version to your patched version:

```toml
# FROM:
bevy = { version = "0.16.0", default-features = false, features = [...] }

# TO:
bevy = { path = "bevy-patched", default-features = false, features = [...] }
```

### Step 3: Handle Workspace Conflicts

If you get workspace conflicts, exclude the bevy-patched directory:

```toml
[workspace]
exclude = ["examples/", "bevy-patched"]
resolver = "2"
```

### Step 4: Clean and Rebuild

```bash
cargo clean
cargo build --release
```

## Alternative Solutions (Not Recommended)

### 1. Disable Dynamic Linking

While this avoids the issue, it significantly increases compile times:

```toml
[features]
dev = [
  # "bevy/dynamic_linking",  # Comment out
  "bevy/asset_processor",
  "bevy/file_watcher"
]
```

### 2. Linker Workarounds

You could try to ignore undefined symbols, but this leads to runtime segfaults:

```rust
// build.rs
println!("cargo:rustc-link-arg=-Wl,--allow-shlib-undefined");
```

### 3. Stub Implementations

Creating stub implementations in your own code doesn't work because the symbols need to match exactly what Bevy expects internally.

## Verification

After applying the fix, verify it works:

1. **Check symbols are defined**:
```bash
nm -D target/*/deps/libbevy_render-*.so | grep -E "register_required_components.*ViewDepthTexture"
# Should show 'T' (defined) not 'U' (undefined)
```

2. **Run the application**:
```bash
./target/release/ia
# Should run without symbol lookup errors
```

3. **Test with dynamic linking** (if using dev features):
```bash
cargo build --features dev
./target/debug/ia
```

## Prevention Strategies

1. **Pin Bevy Version**: Use exact version specifications to avoid surprises
2. **CI Testing**: Include symbol verification in CI pipelines
3. **Document Patches**: Keep detailed records of any Bevy modifications
4. **Monitor Upstream**: Check if the issue is fixed in newer Bevy versions

## NixOS-Specific Considerations

On NixOS, ensure your shell environment includes all necessary libraries:

```nix
devShells.default = pkgs.mkShell {
  buildInputs = with pkgs; [
    vulkan-loader
    libxkbcommon
    wayland
    systemd
    alsa-lib
    libX11
    libXcursor
    libXi
    libXrandr
  ];

  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
};
```

## When This Solution Applies

Use this fix when you encounter:
- Undefined symbol errors for Bevy components
- Errors specifically mentioning `register_required_components`
- Issues that persist across Bevy 0.16.0 and 0.16.1
- Problems that only occur with certain experimental or internal Bevy components

## Long-term Resolution

The proper fix should come from upstream Bevy. Consider:
1. Filing an issue with Bevy if one doesn't exist
2. Contributing the fix upstream
3. Using stable Bevy versions once the fix is merged

## References

- Bevy Component derive macro: `bevy_ecs/macros/src/component.rs`
- Component trait definition: `bevy_ecs/src/component/mod.rs`
- Related Bevy issues: Check https://github.com/bevyengine/bevy/issues

---

**Last Updated**: 2025-06-05
**Affected Versions**: Bevy 0.16.0, 0.16.1
**Status**: Workaround implemented, awaiting upstream fix
