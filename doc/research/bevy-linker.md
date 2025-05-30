# Understanding and Resolving ld.lld Version Script Assignment Errors on NixOS

The error you're encountering on NixOS relates to a significant change in the LLVM linker (ld.lld) that impacts how it handles version scripts, particularly when using Bevy's dynamic linking feature. This report explains the error, why it's occurring in your NixOS environment, and provides solutions to resolve it.

## The Error Explained

Your build is failing with errors like this:

```
ld.lld: error: version script assignment of 'global' to symbol '_ZN110_$LT$bevy_render..camera..manual_texture_view..ManualTextureView$u20$as$u20$bevy_ecs..component..Component$GT$28register_required_components17h758a6355b4180310E' failed: symbol not defined
```

This error occurs during the linking phase when compiling `bevy_dylib` in your Rust project. The linker is complaining that symbols declared in version scripts don't actually exist in the code being linked. Before understanding how to fix it, let's explore why this is happening.

### What are Version Scripts?

Version scripts are used in shared libraries to control which symbols are exported and which version of the library they belong to. They essentially act as a filter for which functions should be visible to applications linking against the library[12].

### The Root Cause

Starting with LLVM 16, the linker flag `--no-undefined-version` is enabled by default[17]. This change means that when a version script declares a symbol that doesn't exist in the compiled code, the linker now reports it as an error rather than silently ignoring it[6].

As explained in the LLVM release notes: "--no-undefined-version is now the default; symbols named in version scripts that have no matching symbol in the output will be reported."[17]

This change was made to help developers identify potential bugs during the build phase rather than potentially encountering issues at runtime[6]. However, it has broken compatibility with many existing projects that relied on the previous behavior.

## NixOS Specific Challenges

NixOS presents some unique challenges when it comes to dynamic linking, which compounds this issue.

### NixOS and Dynamic Linking

NixOS uses a different approach to library management than traditional Linux distributions. As explained in the article "RPATH, or why lld doesn't work on NixOS"[10], NixOS doesn't use conventional library paths like `/usr/lib`. Instead, it relies heavily on RPATH (runtime search path) to locate dynamic libraries.

When using ld.lld on NixOS, the default behavior doesn't automatically include the correct RPATH entries, which can lead to linking issues[10]. This particular aspect of NixOS combined with the stricter version script checking in newer ld.lld versions creates a perfect storm for your current error.

### Bevy Dynamic Linking Evolution

Your error specifically involves `bevy_dylib`, which is Bevy's mechanism for enabling dynamic linking. According to Bevy's migration guide, `bevy_dylib` has been replaced by `dynamic_linking` in newer versions[13]:

```
# 0.11
cargo run --features bevy/bevy_dylib

# 0.12
cargo run --features bevy/dynamic_linking
```

This change in Bevy's API, coupled with the stricter linker behavior, is contributing to your build failure.

## Solutions

Based on the analysis, here are several solutions to resolve the issue:

### 1. Switch to the new dynamic_linking feature

If you're using a newer version of Bevy (0.12+), update your code to use the `dynamic_linking` feature instead of `bevy_dylib`[13]:

```
# In Cargo.toml
bevy = { version = "0.16.0", features = ["dynamic_linking"] }

# OR when running
cargo run --features bevy/dynamic_linking
```

### 2. Use the --undefined-version linker flag

You can override the new default behavior by passing the `--undefined-version` flag to the linker[6][17]:

```
# Add to .cargo/config.toml
[target.x86_64-unknown-linux-gnu.rustflags]
linker-args = ["-Wl,--undefined-version"]
```

This tells the linker to revert to the old behavior of ignoring undefined symbols in version scripts.

### 3. Conditional use of bevy_dylib

Another approach, as shown in the `bevy_dylib` documentation[4], is to conditionally enable dynamic linking only in debug builds:

```rust
// In main.rs
#[allow(unused_imports)]
#[cfg(debug_assertions)]
use bevy_dylib;
```

This way, you won't need to ship additional dynamic libraries with your release builds[4].

### 4. For older packages with version script issues

If you're working with older libraries (like ncurses, glusterfs, etc.) that have similar issues, you may need to patch their version scripts to remove undefined symbols or add the `--undefined-version` flag to their build process[5][11][19].

## Conclusion

The error you're experiencing is due to a deliberate change in the LLVM linker's default behavior to be stricter about version scripts, which is exposing pre-existing issues in code that assumes symbols can be listed in version scripts without being defined. This change is particularly challenging on NixOS due to its unique approach to library management.

By either adapting to Bevy's newer API, overriding the linker behavior, or conditionally using dynamic linking, you should be able to resolve this issue and successfully build your project on NixOS.

For long-term maintainability, migrating to the newer `dynamic_linking` feature (if using Bevy 0.12+) is likely the cleanest solution, as it aligns with the project's current recommended practices and will be better supported going forward.

Citations:
[1] https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/attachments/14621406/5777ad49-4e75-4e90-a068-3ac320f098aa/paste.txt
[2] https://github.com/LinuxCNC/linuxcnc/issues/3191
[3] https://bugs.gentoo.org/919840
[4] https://docs.rs/bevy_dylib
[5] https://lists.gnu.org/archive/html/bug-ncurses/2024-05/msg00087.html
[6] https://reviews.llvm.org/D135402
[7] https://stackoverflow.com/questions/2356168/force-gcc-to-notify-about-undefined-references-in-shared-libraries
[8] https://github.com/rust-cross/cargo-zigbuild/issues/162
[9] https://github.com/bevyengine/bevy/discussions/14403
[10] https://matklad.github.io/2022/03/14/rpath-or-why-lld-doesnt-work-on-nixos.html
[11] https://github.com/NixOS/nixpkgs/issues/310727
[12] https://issues.chromium.org/40242425
[13] https://bevyengine.org/learn/migration-guides/0-11-to-0-12/
[14] https://github.com/llvm/llvm-project/issues/61208
[15] https://github.com/ROCm/HIP/issues/3382
[16] https://stackoverflow.com/questions/70131727/lld-undefined-symbol-when-attempting-to-link-glfw
[17] https://releases.llvm.org/16.0.0/tools/lld/docs/ReleaseNotes.html
[18] https://bbs.archlinux.org/viewtopic.php?id=282283
[19] https://lists.samba.org/archive/samba-technical/2024-May/138944.html
[20] https://bugs.gentoo.org/918914
[21] https://nxmnpg.lemoda.net/1/ld.lld
[22] https://doc.rust-lang.org/cargo/reference/build-scripts.html
[23] https://hackmd.io/@bevy
[24] https://discourse.nixos.org/t/c-and-libstdc-not-available/39126
[25] https://www.reddit.com/r/NixOS/comments/1bqwti5/nixn00b_how_do_i_load_default_configs_for/
[26] https://docs.rs/bevy/latest/bevy/?search=PbrBundle
[27] https://stackoverflow.com/questions/68789250/using-an-ld-version-script-in-a-cdylib-rust-crate
[28] https://nixos.wiki/wiki/Nix_Cookbook
[29] https://users.rust-lang.org/t/rustc-equivalent-of-clangs-undefined-dynamic-lookup/60949
[30] https://www.reddit.com/r/bevy/comments/15m50gr/do_i_still_need_to_do_anything_special_to_enable/
[31] https://users.rust-lang.org/t/undefined-symbols-while-compiling-to-x86-64-unknown-uefi/34506
[32] https://users.rust-lang.org/t/hello-world-no-std-linker-error/48868
[33] https://ianthehenry.com/posts/how-to-learn-nix/my-first-nix-bug/
[34] https://bbs.archlinux.org/viewtopic.php?id=298815
[35] https://ferrous-systems.com/blog/defmt-rtt-linker-error/
[36] https://discourse.cmake.org/t/cant-add-linker-options-no-matter-what/12941
[37] https://github.com/bevyengine/bevy/issues/14117
[38] https://docs.rs/bevy/latest/i686-pc-windows-msvc/bevy/reflect/trait.Map.html
[39] https://stackoverflow.com/questions/70283054/compiling-bevy-dylib-v0-5-0-error-linking-with-cc-failed-exit-status-1
[40] https://bevyengine.org/learn/migration-guides/0-14-to-0-15/
[41] https://bevyengine.org/learn/migration-guides/0-13-to-0-14/
[42] https://thisweekinbevy.com/issue/2025-02-10-entity-mapping-cloning-disabling-and-more
[43] https://www.reddit.com/r/rust/comments/1atase8/bevy_013/
[44] https://github.com/NixOS/nixpkgs/issues/321667
[45] https://discourse.nixos.org/t/nixos-rust-usr-bin-clang-not-found/14807
[46] https://discourse.nixos.org/t/gcc11stdenv-and-clang/17734
[47] https://www.reddit.com/r/NixOS/comments/1d7zvgu/nvim_cant_find_standard_library_headers/
[48] https://nix.dev/manual/nix/2.24/command-ref/conf-file.html
[49] https://www.reddit.com/r/NixOS/comments/px1hoq/clang_linker_error_on_macos/
[50] https://nix.dev/manual/nix/2.25/command-ref/nix-env/set-flag
[51] https://www.reddit.com/r/bevy/comments/1d22xw7/bevy_roadmap_to_v1/
[52] https://github.com/ilyvion/bevy_image_font/blob/main/CHANGELOG.md
[53] https://bevy-cheatbook.github.io/platforms/macos.html
[54] https://users.rust-lang.org/t/how-to-use-linker-version-scripts-in-rust-1-54/64033
[55] https://bevyengine.org/learn/contribute/helping-out/writing-docs/
[56] https://users.rust-lang.org/t/any-examples-of-dynamic-linking-of-rust-code/119732
[57] https://nixos.org/nixos/manual/
[58] https://discourse.nixos.org/t/how-to-install-a-specific-version-of-a-package-from-my-configuration-nix/18057
[59] https://mplanchard.com/posts/installing-a-specific-version-of-a-package-with-nix.html
[60] https://www.ertt.ca/nix/shell-scripts/
[61] https://news.ycombinator.com/item?id=30688815
[62] https://www.reddit.com/r/NixOS/comments/1cdh4kv/question_about_rust_llvm_dependencies/
[63] https://github.com/rust-lang/rust/issues/105967
[64] https://github.com/rust-lang/rust/issues/130062
[65] https://www.reddit.com/r/rust/comments/1665lfq/strange_rustlld_error_in_no_std_environment/
[66] https://users.rust-lang.org/t/target-x86-64-unknown-linux-musl-fails-to-link/50401
[67] https://bugs.gentoo.org/715348
[68] https://pyo3.rs/v0.10.1/
[69] https://lib.rs/crates/bevy_dylib
[70] https://users.rust-lang.org/t/llvm-lld-linker-fails-on-mac-os-x/35078
[71] https://internals.rust-lang.org/t/versions-later-than-1-70-builds-cdylib-with-an-error-of-undefined-version/19560
[72] https://users.rust-lang.org/t/undefined-symbol-rust-probestack-when-compiling-executable-from-llvm-ir/67425
[73] https://discourse.nixos.org/t/need-a-good-example-for-specifying-versions-in-nixpkgs/9815
[74] https://discourse.nixos.org/t/how-to-override-package-version/2889
[75] https://stackoverflow.com/questions/60131414/nix-nixlang-undefined-variable-pkgs-in-default-nix-via-nix-build-a-hello-but-w
[76] https://unix.stackexchange.com/questions/741682/how-to-pin-a-package-version-with-nix-shell
[77] https://ryantm.github.io/nixpkgs/stdenv/stdenv/
[78] https://discourse.nixos.org/t/nats-nsc-package-coming-up-as-undefined-variable/26603
[79] https://github.com/rust-lang/rust/issues/111888
[80] https://stackoverflow.com/questions/71372256/why-does-cargo-build-succeed-but-cargo-build-release-fail-with-undefined-r
[81] https://stackoverflow.com/questions/77011647/strange-rust-lld-error-in-no-std-environment
[82] https://reviews.llvm.org/D150637
