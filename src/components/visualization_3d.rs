//! 3D visualization components

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Node visualization component for 3D rendering
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct NodeVisual3D {
    /// Shape of the node
    pub shape: NodeShape,
    /// Size of the node
    pub size: f32,
    /// Color of the node
    pub color: Color,
    /// Metallic property
    pub metallic: f32,
    /// Roughness property  
    pub roughness: f32,
    /// Emissive color
    pub emissive: Color,
}

/// Shape types for nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeShape {
    /// Sphere shape
    Sphere,
    /// Cube shape
    Cube,
    /// Cylinder shape
    Cylinder,
}

/// Edge visualization component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EdgeVisual3D {
    /// Width of the edge
    pub width: f32,
    /// Color of the edge
    pub color: Color,
}

/// Graph layout component
#[derive(Component, Debug, Clone)]
pub struct GraphLayout3D {
    /// Force strength for layout
    pub force_strength: f32,
}

/// Highlight component for selected nodes
#[derive(Component, Debug, Clone)]
pub struct Highlight3D;

/// Node label component
#[derive(Component, Debug, Clone)]
pub struct NodeLabel3D {
    /// Label text
    pub text: String,
}

/// LOD controller component
#[derive(Component, Debug, Clone)]
pub struct LODController {
    /// Distance thresholds for LOD levels
    pub thresholds: Vec<f32>,
}

/// Culling sphere component
#[derive(Component, Debug, Clone)]
pub struct CullingSphere {
    /// Radius of the culling sphere
    pub radius: f32,
}

/// Outline style component
#[derive(Component, Debug, Clone)]
pub struct OutlineStyle {
    /// Outline color
    pub color: Color,
    /// Outline width
    pub width: f32,
}
