// Vertex shader
struct VertexOutput {
    @builtin(position) position: vec4<f32>, // Position in clip space
    @location(0) color: vec3<f32>,          // Color to pass to fragment shader
};

@vertex
fn main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),        // Top vertex
        vec2<f32>(-0.5, -0.5),      // Bottom-left vertex
        vec2<f32>(0.5, -0.5)        // Bottom-right vertex
    );
    
    var colors = array<vec3<f32>, 3>(
        vec3<f32>(1.0, 0.0, 0.0),   // Red color for the top vertex
        vec3<f32>(0.0, 1.0, 0.0),   // Green color for the bottom-left vertex
        vec3<f32>(0.0, 0.0, 1.0)    // Blue color for the bottom-right vertex
    );
    
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0); // Set position in clip space
    output.color = colors[vertex_index];                            // Set the corresponding color
    return output;
}

// Fragment shader
@fragment
fn main(@location(0) color: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0); // Output the interpolated color with full opacity
}
