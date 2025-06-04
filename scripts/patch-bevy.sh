#!/usr/bin/env bash
set -euo pipefail

# Clone Bevy if not already cloned
if [ ! -d "bevy-patched" ]; then
    echo "Cloning Bevy v0.16.1..."
    git clone --depth 1 --branch v0.16.1 https://github.com/bevyengine/bevy.git bevy-patched
fi

cd bevy-patched

# Create a patch to remove experimental occlusion culling
cat > remove-experimental-occlusion.patch << 'EOF'
diff --git a/crates/bevy_render/src/view/mod.rs b/crates/bevy_render/src/view/mod.rs
index abc..def 100644
--- a/crates/bevy_render/src/view/mod.rs
+++ b/crates/bevy_render/src/view/mod.rs
@@ -100,6 +100,7 @@ pub struct ViewUniform {

 /// Texture used to store depth data from the prepass.
 #[derive(Component, Default, Deref)]
+#[cfg(feature = "experimental_occlusion_culling")]
 pub struct ViewDepthTexture {
     pub texture: Texture,
 }
@@ -107,6 +108,15 @@ pub struct ViewDepthTexture {
+// Stub for when experimental_occlusion_culling is disabled
+#[cfg(not(feature = "experimental_occlusion_culling"))]
+#[derive(Component, Default)]
+pub struct ViewDepthTexture;
+
diff --git a/crates/bevy_render/src/experimental/occlusion_culling/mod.rs b/crates/bevy_render/src/experimental/occlusion_culling/mod.rs
index abc..def 100644
--- a/crates/bevy_render/src/experimental/occlusion_culling/mod.rs
+++ b/crates/bevy_render/src/experimental/occlusion_culling/mod.rs
@@ -50,6 +50,7 @@ use bevy_ecs::{

 /// Marks a view as supporting experimental occlusion culling.
 #[derive(Component)]
+#[cfg(feature = "experimental_occlusion_culling")]
 pub struct OcclusionCullingSubview {
     pub view: TextureView,
 }
@@ -57,6 +58,11 @@ pub struct OcclusionCullingSubview {
+// Stub for when experimental_occlusion_culling is disabled
+#[cfg(not(feature = "experimental_occlusion_culling"))]
+#[derive(Component)]
+pub struct OcclusionCullingSubview;
+
EOF

echo "Applying patch..."
git apply remove-experimental-occlusion.patch

echo "Bevy patched successfully!"
echo "Update your Cargo.toml to use:"
echo '[patch.crates-io]'
echo 'bevy = { path = "./bevy-patched" }'
