// Shape rendering shader
// Supports instanced rendering of circles, rectangles, triangles, and hexagons.

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct InstanceInput {
    @location(1) world_position: vec2<f32>,
    @location(2) rotation: f32,
    @location(3) scale: f32,
    @location(4) color: u32,
    @location(5) shape_type: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    // Build 2D rotation matrix
    let c = cos(instance.rotation);
    let s = sin(instance.rotation);

    // Rotate and scale the local vertex
    let rotated = vec2<f32>(
        vertex.position.x * c - vertex.position.y * s,
        vertex.position.x * s + vertex.position.y * c
    );
    let scaled = rotated * instance.scale;

    // Translate to world position
    let world_pos = scaled + instance.world_position;

    // Transform to clip space
    var output: VertexOutput;
    output.clip_position = camera.view_proj * vec4<f32>(world_pos, 0.0, 1.0);

    // Unpack color from u32 (RGBA8 format: R in high byte, A in low byte)
    let r = f32((instance.color >> 24u) & 0xFFu) / 255.0;
    let g = f32((instance.color >> 16u) & 0xFFu) / 255.0;
    let b = f32((instance.color >> 8u) & 0xFFu) / 255.0;
    let a = f32(instance.color & 0xFFu) / 255.0;
    output.color = vec4<f32>(r, g, b, a);

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
