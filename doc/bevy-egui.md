# Bevy 0.16 and Egui Compatibility Report

## Overview

Bevy 0.16, released on April 24, 2025[1], brings significant upgrades including GPU-driven rendering, procedural atmospheric scattering, and ECS relationships. For developers looking to use Egui (the immediate mode GUI library) with this latest Bevy release, there is good news: compatibility is maintained through the bevy_egui integration crate, which was updated promptly following Bevy 0.16's release.

## Compatible Versions

The primary Egui integration for Bevy 0.16 is handled through the following crate:

- **bevy_egui 0.34.0/0.34.1**: Released on April 25-26, 2025, specifically updated to support Bevy 0.16[6][4]

For developers using keyboard and gamepad navigation enhancements for Egui:

- **bevy-egui-kbgp 0.24**: Supports Bevy 0.16 and bevy_egui 0.34[9]

## Breaking Changes and Migration Notes

The update to bevy_egui 0.34.0 introduces some breaking changes that developers should be aware of:

### Multi-Pass Support
The most significant change is the introduction of multi-pass support, adding a new `enable_multipass_for_primary_context` field to the `EguiPlugin`[10][6]. This affects how you initialize the plugin in your applications.

```rust
// New initialization pattern
.add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
```

### System Scheduling Changes

If you add UI systems, they should now go into the `EguiContextPass` schedule to ensure your plugin supports both single-pass and multi-pass modes[5].

```rust
// New recommended system scheduling
.add_systems(EguiContextPass, ui_example_system)
```

Rather than:

```rust
// Old approach
.add_systems(Update, ui_example_system)
```

### Additional Changes

- `bevy_picking` support has been feature-gated behind the `picking` feature[6]
- Non-latin hotkey issues have been fixed[6]
- Multiple window handling has been improved[6]

## Features of bevy_egui 0.34

The 0.34 version of bevy_egui brings several notable features:

1. **Multi-pass support**: Enables more flexible rendering approaches[10]
2. **Absorbing inputs and Egui input run conditions**: Improved input handling mechanisms[6]
3. **AccessKit integration**: Though currently disabled until the next Egui release[6]
4. **Fixed EguiOutput updates**: Resolving previous issues with output handling[6]

## Usage Example

Here's a minimal example of using bevy_egui 0.34 with Bevy 0.16:

```rust
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiContextPass};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
        .add_systems(EguiContextPass, ui_example_system)
        .run();
}

fn ui_example_system(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}
```

## Conclusion

The Bevy ecosystem has responded quickly to the release of Bevy 0.16, with bevy_egui 0.34 providing full compatibility just one day after the main engine update[6][1]. While there are breaking changes to consider, particularly around the new multi-pass system, the migration path is straightforward and well-documented.

For developers creating new applications or updating existing ones, using bevy_egui 0.34.1 (the latest patch version) with Bevy 0.16 is recommended. Those with specialized needs such as keyboard/gamepad navigation should use bevy-egui-kbgp 0.24.

The quick update cycle demonstrates the health of the Bevy ecosystem, ensuring that critical integrations like Egui remain available for the latest engine version with minimal delay.

Citations:
[1] https://bevyengine.org/news/bevy-0-16/
[2] https://bevyengine.org/learn/migration-guides/0-15-to-0-16/
[3] https://www.youtube.com/watch?v=l13mPxDvKLQ
[4] https://docs.rs/crate/bevy_egui/latest/source/CHANGELOG.md
[5] https://docs.rs/bevy_egui/latest/bevy_egui/
[6] https://github.com/vladbat00/bevy_egui/releases
[7] https://github.com/idanarye/bevy-egui-kbgp/blob/main/CHANGELOG.md
[8] https://github.com/bevyengine/bevy/releases
[9] https://github.com/idanarye/bevy-egui-kbgp
[10] https://thisweekinbevy.com/issue/2025-04-28-bevy-016-is-out-now
[11] https://www.reddit.com/r/bevy/comments/1k72g9u/bevy_016_released/
[12] https://websets.exa.ai/articles-bevy-016-major-updates-new-features-cm8wrbm1g042zfz0iy4f150n7
[13] https://www.youtube.com/watch?v=xVoRmcGI2AM
[14] https://github.com/bevyengine/bevy
[15] https://www.youtube.com/watch?v=L6sBHp1AaGM
[16] https://www.reddit.com/r/rust/comments/1k721w1/bevy_016/
[17] https://github.com/mvlabat/bevy_egui/blob/main/Cargo.toml
[18] https://crates.io/crates/bevy_egui/0.34.0-rc.3
[19] https://deps.rs/crate/bevy_egui/0.34.1
[20] https://docs.rs/bevy_egui
[21] https://crates.io/crates/bevy_egui
[22] https://crates.io/crates/bevy_egui/versions
[23] https://crates.io/crates/bevy_egui/dependencies
[24] https://crates.io/crates/bevy-egui-kbgp
[25] https://crates.io/crates/bevy_egui_ime
[26] https://crates.io/crates/bevy_egui/0.18.0/dependencies
[27] https://docs.rs/crate/bevy_egui/latest/source/README.md
[28] https://github.com/aecsocket/aeronet/blob/main/Cargo.toml
[29] https://lib.rs/crates/bevy_egui/features
[30] https://www.reddit.com/r/rust/comments/17gavcx/cargo_dependencies_hell/
[31] https://github.com/vladbat00/bevy_egui
[32] https://github.com/mvlabat/bevy_egui/blob/main/src/lib.rs
[33] https://github.com/mvlabat/bevy_egui/issues
[34] https://github.com/bevyengine/bevy/blob/main/release-content/migration_guides.md
[35] https://bevyengine.org/learn/migration-guides/0-14-to-0-15/
