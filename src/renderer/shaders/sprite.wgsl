// Sprite shader with texture sampling, rotation, and color tinting.

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var sprite_texture: texture_2d<f32>;

@group(1) @binding(1)
var sprite_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,  // Unit quad vertex
}

struct InstanceInput {
    @location(1) world_position: vec2<f32>,
    @location(2) uv_offset: u32,       // Packed 16-bit u, v offset
    @location(3) uv_size: u32,         // Packed 16-bit u, v size
    @location(4) color_tint: u32,      // RGBA8 packed
    @location(5) transform_flags: u32, // rotation(16) + scale(8) + flags(8)
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    // Unpack UV offset (normalized 0-65535 to 0-1)
    let u_off = f32(instance.uv_offset & 0xFFFFu) / 65535.0;
    let v_off = f32((instance.uv_offset >> 16u) & 0xFFFFu) / 65535.0;

    // Unpack UV size
    let u_size = f32(instance.uv_size & 0xFFFFu) / 65535.0;
    let v_size = f32((instance.uv_size >> 16u) & 0xFFFFu) / 65535.0;

    // Unpack transform: rotation(16) + scale(8) + flags(8)
    let rotation_packed = f32((instance.transform_flags >> 16u) & 0xFFFFu) / 65535.0;
    let rotation = rotation_packed * 6.28318530718; // TAU
    let scale = f32((instance.transform_flags >> 8u) & 0xFFu) / 255.0 * 25.5;
    let flags = instance.transform_flags & 0xFFu;
    let flip_x = (flags & 1u) != 0u;
    let flip_y = (flags & 2u) != 0u;

    // Build rotation matrix
    let c = cos(rotation);
    let s = sin(rotation);
    let rot = mat2x2<f32>(c, -s, s, c);

    // Transform vertex position
    var local_pos = vertex.position;

    // Apply flip before rotation
    if flip_x {
        local_pos.x = -local_pos.x;
    }
    if flip_y {
        local_pos.y = -local_pos.y;
    }

    // Apply rotation and scale
    local_pos = rot * local_pos * scale;

    // Transform to world space
    let world = vec4<f32>(local_pos + instance.world_position, 0.0, 1.0);

    var output: VertexOutput;
    output.clip_position = camera.view_proj * world;

    // Calculate UV coordinates
    // Vertex position is in range [-0.5, 0.5], map to [0, 1]
    var uv = vertex.position + vec2<f32>(0.5, 0.5);

    // Apply flip to UV
    if flip_x {
        uv.x = 1.0 - uv.x;
    }
    if flip_y {
        uv.y = 1.0 - uv.y;
    }

    // Map to atlas region
    output.uv = vec2<f32>(u_off, v_off) + uv * vec2<f32>(u_size, v_size);

    // Unpack color tint
    let r = f32((instance.color_tint >> 24u) & 0xFFu) / 255.0;
    let g = f32((instance.color_tint >> 16u) & 0xFFu) / 255.0;
    let b = f32((instance.color_tint >> 8u) & 0xFFu) / 255.0;
    let a = f32(instance.color_tint & 0xFFu) / 255.0;
    output.color = vec4<f32>(r, g, b, a);

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(sprite_texture, sprite_sampler, input.uv);

    // Multiply by tint color
    let final_color = tex_color * input.color;

    // Discard fully transparent pixels
    if final_color.a < 0.01 {
        discard;
    }

    return final_color;
}
