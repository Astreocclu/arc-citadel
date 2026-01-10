# Complete Graphics MVP Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the graphical implementation from shape-based MVP to sprite-based rendering with hex terrain.

**Architecture:** Build on existing wgpu renderer (Phases 1-2 complete). Add sprite pipeline for textured units, hex grid for terrain display. Keep shape rendering for debug/fallback.

**Tech Stack:** wgpu 0.19, winit 0.29, image crate for PNG loading, existing sprite data structures.

---

## Current State

**Working:**
- `src/bin/live_sim.rs` - Shape-based live simulation renderer
- `src/renderer/` - Complete shape rendering pipeline
- `src/renderer/sprites/` - Animation state machine, atlas data structures, instance format
- `src/renderer/shaders/sprite.wgsl` - Sprite shader with texture sampling

**Missing:**
- SpritePipeline (GPU pipeline using sprite shader)
- Texture loading (PNG → wgpu::Texture)
- Sprite rendering integration in main Renderer
- Hex terrain rendering

---

## Task 1: Add Image Dependency

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add image crate**

Add to `[dependencies]` section:

```toml
image = "0.24"
```

**Step 2: Verify build**

Run: `cargo build --lib`
Expected: Compiles with new dependency

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "deps: add image crate for texture loading"
```

---

## Task 2: Create Texture Loader

**Files:**
- Create: `src/renderer/gpu/texture.rs`
- Modify: `src/renderer/gpu/mod.rs`

**Step 1: Write texture.rs**

Create `src/renderer/gpu/texture.rs`:

```rust
//! Texture loading and management.

use image::GenericImageView;
use std::path::Path;

/// A loaded GPU texture with its view and sampler.
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    /// Load a texture from a PNG file.
    pub fn from_file(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &Path,
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        let img = image::open(path).map_err(|e| TextureError::Load(e.to_string()))?;
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        Self::from_rgba(device, queue, &rgba, dimensions, label)
    }

    /// Create a texture from raw RGBA bytes.
    pub fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: &[u8],
        dimensions: (u32, u32),
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest, // Pixel art style
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            width: dimensions.0,
            height: dimensions.1,
        })
    }

    /// Create a 1x1 white placeholder texture.
    pub fn white_pixel(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::from_rgba(
            device,
            queue,
            &[255, 255, 255, 255],
            (1, 1),
            Some("White Pixel"),
        )
        .expect("Failed to create white pixel texture")
    }
}

#[derive(Debug)]
pub enum TextureError {
    Load(String),
}

impl std::fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureError::Load(msg) => write!(f, "Failed to load texture: {}", msg),
        }
    }
}

impl std::error::Error for TextureError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_error_display() {
        let err = TextureError::Load("file not found".into());
        assert!(err.to_string().contains("file not found"));
    }
}
```

**Step 2: Update gpu/mod.rs**

Add to `src/renderer/gpu/mod.rs`:

```rust
pub mod texture;
pub use texture::{Texture, TextureError};
```

**Step 3: Run tests**

Run: `cargo test --lib renderer::gpu::texture`
Expected: PASS

**Step 4: Commit**

```bash
git add src/renderer/gpu/texture.rs src/renderer/gpu/mod.rs
git commit -m "feat(renderer): add texture loading from PNG files"
```

---

## Task 3: Create Sprite Pipeline

**Files:**
- Create: `src/renderer/gpu/sprite_pipeline.rs`
- Modify: `src/renderer/gpu/mod.rs`

**Step 1: Write sprite_pipeline.rs**

Create `src/renderer/gpu/sprite_pipeline.rs`:

```rust
//! Sprite rendering pipeline with texture sampling.

use super::{GpuContext, Texture};
use crate::renderer::shapes::Vertex;

/// GPU pipeline for sprite rendering.
pub struct SpritePipeline {
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
}

impl SpritePipeline {
    pub fn new(ctx: &GpuContext) -> Self {
        // Camera uniform buffer
        let camera_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Camera Buffer"),
            size: 64, // mat4x4<f32>
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Camera bind group layout (group 0)
        let camera_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Sprite Camera Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Texture bind group layout (group 1)
        let texture_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Sprite Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        // Load sprite shader
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Sprite Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sprite.wgsl").into()),
            });

        // Pipeline layout
        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sprite Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Sprite instance layout - matches sprite.wgsl expectations
        let sprite_instance_layout = wgpu::VertexBufferLayout {
            array_stride: 24, // 6 x u32
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // location 1: world_position (vec2<f32>)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // location 2: uv_offset (u32)
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint32,
                },
                // location 3: uv_size (u32)
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
                // location 4: color_tint (u32)
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
                // location 5: transform_flags (u32)
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        };

        // Render pipeline
        let render_pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Sprite Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc(), sprite_instance_layout],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        Self {
            render_pipeline,
            camera_bind_group_layout,
            texture_bind_group_layout,
            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, view_proj: glam::Mat4) {
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&view_proj.to_cols_array()),
        );
    }

    /// Create a texture bind group for a specific texture.
    pub fn create_texture_bind_group(
        &self,
        device: &wgpu::Device,
        texture: &Texture,
        label: Option<&str>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        })
    }
}
```

**Step 2: Update gpu/mod.rs**

Add to exports in `src/renderer/gpu/mod.rs`:

```rust
pub mod sprite_pipeline;
pub use sprite_pipeline::SpritePipeline;
```

**Step 3: Build check**

Run: `cargo build --lib`
Expected: Compiles

**Step 4: Commit**

```bash
git add src/renderer/gpu/sprite_pipeline.rs src/renderer/gpu/mod.rs
git commit -m "feat(renderer): add sprite rendering pipeline"
```

---

## Task 4: Create Sprite Instance Type

**Files:**
- Modify: `src/renderer/sprites/instance.rs`

**Step 1: Update SpriteInstance for shader compatibility**

The existing `SpriteInstance` uses 32-byte format, but the shader expects 24 bytes. Create a simpler render-ready instance:

Add to `src/renderer/sprites/instance.rs`:

```rust
/// GPU-ready sprite instance. 24 bytes, matches sprite.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteRenderInstance {
    pub position: [f32; 2],      // 8 bytes - world position
    pub uv_offset: u32,          // 4 bytes - packed u16 u, u16 v (normalized)
    pub uv_size: u32,            // 4 bytes - packed u16 width, u16 height (normalized)
    pub color_tint: u32,         // 4 bytes - RGBA8 packed
    pub transform_flags: u32,    // 4 bytes - rotation(16) + scale(8) + flags(8)
}

impl SpriteRenderInstance {
    /// Create a new sprite render instance.
    ///
    /// * `position` - World position
    /// * `uv_rect` - UV rectangle [u, v, width, height] in 0.0-1.0 range
    /// * `color` - RGBA color as [r, g, b, a] in 0-255
    /// * `rotation` - Rotation in radians
    /// * `scale` - Scale factor (0.0-25.5 range)
    /// * `flip_x` - Flip horizontally
    /// * `flip_y` - Flip vertically
    pub fn new(
        position: [f32; 2],
        uv_rect: [f32; 4],
        color: [u8; 4],
        rotation: f32,
        scale: f32,
        flip_x: bool,
        flip_y: bool,
    ) -> Self {
        // Pack UV offset
        let u_off = (uv_rect[0].clamp(0.0, 1.0) * 65535.0) as u32;
        let v_off = (uv_rect[1].clamp(0.0, 1.0) * 65535.0) as u32;
        let uv_offset = u_off | (v_off << 16);

        // Pack UV size
        let u_size = (uv_rect[2].clamp(0.0, 1.0) * 65535.0) as u32;
        let v_size = (uv_rect[3].clamp(0.0, 1.0) * 65535.0) as u32;
        let uv_size = u_size | (v_size << 16);

        // Pack color
        let color_tint = ((color[0] as u32) << 24)
            | ((color[1] as u32) << 16)
            | ((color[2] as u32) << 8)
            | (color[3] as u32);

        // Pack transform: rotation(16) + scale(8) + flags(8)
        let rot_norm = (rotation / std::f32::consts::TAU).fract();
        let rot_packed = ((rot_norm.abs() * 65535.0) as u32) & 0xFFFF;
        let scale_packed = ((scale / 25.5).clamp(0.0, 1.0) * 255.0) as u32;
        let flags = (flip_x as u32) | ((flip_y as u32) << 1);
        let transform_flags = (rot_packed << 16) | (scale_packed << 8) | flags;

        Self {
            position,
            uv_offset,
            uv_size,
            color_tint,
            transform_flags,
        }
    }

    /// Create a simple sprite at position with full texture and no transform.
    pub fn simple(position: [f32; 2], scale: f32) -> Self {
        Self::new(
            position,
            [0.0, 0.0, 1.0, 1.0], // Full texture
            [255, 255, 255, 255], // White tint
            0.0,                  // No rotation
            scale,
            false,
            false,
        )
    }
}

#[cfg(test)]
mod render_instance_tests {
    use super::*;

    #[test]
    fn test_sprite_render_instance_size() {
        assert_eq!(std::mem::size_of::<SpriteRenderInstance>(), 24);
    }

    #[test]
    fn test_sprite_render_instance_simple() {
        let inst = SpriteRenderInstance::simple([10.0, 20.0], 5.0);
        assert_eq!(inst.position, [10.0, 20.0]);
        // Full texture UV: offset=0, size=1.0 packed as 0xFFFF
        assert_eq!(inst.uv_offset, 0);
        assert_eq!(inst.uv_size, 0xFFFF_FFFF);
    }
}
```

**Step 2: Run tests**

Run: `cargo test --lib renderer::sprites::instance`
Expected: PASS

**Step 3: Commit**

```bash
git add src/renderer/sprites/instance.rs
git commit -m "feat(renderer): add GPU-ready sprite instance type"
```

---

## Task 5: Create Unit Quad Geometry

**Files:**
- Modify: `src/renderer/shapes/vertex.rs`

**Step 1: Add unit quad for sprites**

Add to `src/renderer/shapes/vertex.rs`:

```rust
/// Create a unit quad for sprite rendering.
/// Vertices are in range [-0.5, 0.5] so scale can be applied directly.
pub fn unit_quad_vertices() -> Vec<Vertex> {
    vec![
        Vertex { position: [-0.5, -0.5] }, // Bottom-left
        Vertex { position: [0.5, -0.5] },  // Bottom-right
        Vertex { position: [0.5, 0.5] },   // Top-right
        Vertex { position: [-0.5, 0.5] },  // Top-left
    ]
}

/// Indices for unit quad (two triangles).
pub fn unit_quad_indices() -> Vec<u16> {
    vec![0, 1, 2, 0, 2, 3]
}
```

**Step 2: Run tests**

Run: `cargo test --lib renderer::shapes::vertex`
Expected: PASS (existing tests should still pass)

**Step 3: Commit**

```bash
git add src/renderer/shapes/vertex.rs
git commit -m "feat(renderer): add unit quad geometry for sprites"
```

---

## Task 6: Create Sprite Buffers

**Files:**
- Create: `src/renderer/gpu/sprite_buffers.rs`
- Modify: `src/renderer/gpu/mod.rs`

**Step 1: Write sprite_buffers.rs**

Create `src/renderer/gpu/sprite_buffers.rs`:

```rust
//! Sprite buffer management.

use wgpu::util::DeviceExt;

use super::GpuContext;
use crate::renderer::shapes::vertex::{unit_quad_indices, unit_quad_vertices, Vertex};
use crate::renderer::sprites::SpriteRenderInstance;

/// Buffers for sprite rendering.
pub struct SpriteBuffers {
    /// Unit quad vertex buffer
    pub quad_vertex_buffer: wgpu::Buffer,
    /// Unit quad index buffer
    pub quad_index_buffer: wgpu::Buffer,
    /// Number of indices in quad
    pub quad_index_count: u32,
    /// Dynamic instance buffer
    pub instance_buffer: wgpu::Buffer,
    /// Current instance buffer capacity
    pub instance_capacity: usize,
}

impl SpriteBuffers {
    pub fn new(ctx: &GpuContext, initial_capacity: usize) -> Self {
        let quad_verts = unit_quad_vertices();
        let quad_indices = unit_quad_indices();

        let quad_vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_index_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Quad Index Buffer"),
            contents: bytemuck::cast_slice(&quad_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Instance Buffer"),
            size: (initial_capacity * std::mem::size_of::<SpriteRenderInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            quad_vertex_buffer,
            quad_index_buffer,
            quad_index_count: quad_indices.len() as u32,
            instance_buffer,
            instance_capacity: initial_capacity,
        }
    }

    /// Upload sprite instances, growing buffer if needed.
    pub fn upload_instances(&mut self, ctx: &GpuContext, instances: &[SpriteRenderInstance]) {
        if instances.is_empty() {
            return;
        }

        if instances.len() > self.instance_capacity {
            let new_capacity = (instances.len() * 2).max(1024);
            self.instance_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Sprite Instance Buffer"),
                size: (new_capacity * std::mem::size_of::<SpriteRenderInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_capacity = new_capacity;
        }

        ctx.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(instances),
        );
    }
}
```

**Step 2: Update gpu/mod.rs**

Add to `src/renderer/gpu/mod.rs`:

```rust
pub mod sprite_buffers;
pub use sprite_buffers::SpriteBuffers;
```

**Step 3: Build check**

Run: `cargo build --lib`
Expected: Compiles

**Step 4: Commit**

```bash
git add src/renderer/gpu/sprite_buffers.rs src/renderer/gpu/mod.rs
git commit -m "feat(renderer): add sprite buffer management"
```

---

## Task 7: Integrate Sprites into Renderer

**Files:**
- Modify: `src/renderer/mod.rs`
- Modify: `src/renderer/state.rs`

**Step 1: Add SpriteEntity to state.rs**

Add to `src/renderer/state.rs`:

```rust
/// Sprite render data (for textured entities).
#[derive(Clone, Copy)]
pub struct SpriteEntity {
    pub id: EntityId,
    pub position: Vec2,
    pub uv_rect: [f32; 4],  // [u, v, width, height] normalized
    pub color: [u8; 4],     // RGBA tint
    pub rotation: f32,
    pub scale: f32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub z_order: i32,
}

impl SpriteEntity {
    /// Create a simple sprite using full texture.
    pub fn simple(id: EntityId, position: Vec2, scale: f32) -> Self {
        Self {
            id,
            position,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: [255, 255, 255, 255],
            rotation: 0.0,
            scale,
            flip_x: false,
            flip_y: false,
            z_order: 0,
        }
    }
}
```

Update `RenderState` to include sprites:

```rust
#[derive(Clone)]
pub struct RenderState {
    pub tick: u64,
    pub entities: Vec<RenderEntity>,      // Shape-based entities
    pub sprites: Vec<SpriteEntity>,       // Textured sprites
    pub camera: CameraState,
}
```

**Step 2: Update Renderer to support sprites**

Modify `src/renderer/mod.rs` - add sprite fields to `Renderer`:

```rust
use gpu::{GpuContext, ShapeBuffers, ShapePipeline, SpritePipeline, SpriteBuffers, Texture};
use sprites::SpriteRenderInstance;

pub struct Renderer {
    ctx: GpuContext,

    // Shape rendering
    shape_pipeline: ShapePipeline,
    shape_buffers: ShapeBuffers,
    circle_instances: Vec<ShapeInstance>,
    rectangle_instances: Vec<ShapeInstance>,
    triangle_instances: Vec<ShapeInstance>,
    hexagon_instances: Vec<ShapeInstance>,

    // Sprite rendering
    sprite_pipeline: SpritePipeline,
    sprite_buffers: SpriteBuffers,
    sprite_instances: Vec<SpriteRenderInstance>,
    default_texture: Texture,
    default_texture_bind_group: wgpu::BindGroup,

    metrics: RenderMetrics,
}
```

Update `Renderer::new()`:

```rust
pub async fn new(window: Arc<Window>) -> Self {
    let ctx = GpuContext::new(window).await;
    let shape_pipeline = ShapePipeline::new(&ctx);
    let shape_buffers = ShapeBuffers::new(&ctx, 10000);

    let sprite_pipeline = SpritePipeline::new(&ctx);
    let sprite_buffers = SpriteBuffers::new(&ctx, 1000);

    // Create default white texture for sprites without custom texture
    let default_texture = Texture::white_pixel(&ctx.device, &ctx.queue);
    let default_texture_bind_group = sprite_pipeline.create_texture_bind_group(
        &ctx.device,
        &default_texture,
        Some("Default Texture Bind Group"),
    );

    Self {
        ctx,
        shape_pipeline,
        shape_buffers,
        circle_instances: Vec::with_capacity(10000),
        rectangle_instances: Vec::with_capacity(1000),
        triangle_instances: Vec::with_capacity(1000),
        hexagon_instances: Vec::with_capacity(1000),
        sprite_pipeline,
        sprite_buffers,
        sprite_instances: Vec::with_capacity(1000),
        default_texture,
        default_texture_bind_group,
        metrics: RenderMetrics::new(),
    }
}
```

Add sprite rendering to `render()` method (after shape rendering):

```rust
// Render sprites
if !state.sprites.is_empty() {
    self.sprite_instances.clear();
    for sprite in &state.sprites {
        self.sprite_instances.push(SpriteRenderInstance::new(
            [sprite.position.x, sprite.position.y],
            sprite.uv_rect,
            sprite.color,
            sprite.rotation,
            sprite.scale,
            sprite.flip_x,
            sprite.flip_y,
        ));
    }

    self.sprite_buffers.upload_instances(&self.ctx, &self.sprite_instances);
    self.metrics.record_buffer_upload();

    render_pass.set_pipeline(&self.sprite_pipeline.render_pipeline);
    render_pass.set_bind_group(0, &self.sprite_pipeline.camera_bind_group, &[]);
    render_pass.set_bind_group(1, &self.default_texture_bind_group, &[]);
    render_pass.set_vertex_buffer(0, self.sprite_buffers.quad_vertex_buffer.slice(..));
    render_pass.set_vertex_buffer(1, self.sprite_buffers.instance_buffer.slice(..));
    render_pass.set_index_buffer(
        self.sprite_buffers.quad_index_buffer.slice(..),
        wgpu::IndexFormat::Uint16,
    );
    render_pass.draw_indexed(
        0..self.sprite_buffers.quad_index_count,
        0,
        0..self.sprite_instances.len() as u32,
    );
    self.metrics.record_draw_call();
}
```

**Step 3: Build check**

Run: `cargo build --lib`
Expected: Compiles

**Step 4: Commit**

```bash
git add src/renderer/mod.rs src/renderer/state.rs
git commit -m "feat(renderer): integrate sprite rendering into main renderer"
```

---

## Task 8: Create Test Sprite Binary

**Files:**
- Create: `src/bin/sprite_test.rs`

**Step 1: Write sprite_test.rs**

Create `src/bin/sprite_test.rs`:

```rust
//! Sprite rendering test binary.
//!
//! Tests sprite pipeline with default white texture.

use std::sync::Arc;
use std::time::Instant;

use glam::Vec2;
use winit::{
    event::{ElementState, Event, MouseScrollDelta, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use arc_citadel::core::types::EntityId;
use arc_citadel::renderer::{CameraState, RenderState, Renderer, SpriteEntity};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Sprite Test");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Arc Citadel - Sprite Test")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    let mut renderer = pollster::block_on(Renderer::new(window.clone()));

    // Create test sprites in a grid
    let mut sprites = Vec::new();
    for i in 0..10 {
        for j in 0..10 {
            sprites.push(SpriteEntity {
                id: EntityId::new(),
                position: Vec2::new(i as f32 * 20.0, j as f32 * 20.0),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: [
                    (i * 25) as u8,
                    (j * 25) as u8,
                    128,
                    255,
                ],
                rotation: (i + j) as f32 * 0.3,
                scale: 8.0,
                flip_x: i % 2 == 0,
                flip_y: j % 2 == 0,
                z_order: 0,
            });
        }
    }

    let mut camera = CameraState {
        center: Vec2::new(100.0, 100.0),
        zoom: 2.0,
        viewport_size: Vec2::new(1280.0, 720.0),
    };

    let mut frame_count: u64 = 0;
    let mut last_fps_time = Instant::now();

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(size) => {
                    renderer.resize(size.width, size.height);
                    camera.set_viewport_size(size.width as f32, size.height as f32);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == ElementState::Pressed {
                        let pan_speed = 20.0 * camera.zoom;
                        match event.physical_key {
                            PhysicalKey::Code(KeyCode::KeyW)
                            | PhysicalKey::Code(KeyCode::ArrowUp) => {
                                camera.pan(Vec2::new(0.0, pan_speed));
                            }
                            PhysicalKey::Code(KeyCode::KeyS)
                            | PhysicalKey::Code(KeyCode::ArrowDown) => {
                                camera.pan(Vec2::new(0.0, -pan_speed));
                            }
                            PhysicalKey::Code(KeyCode::KeyA)
                            | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                                camera.pan(Vec2::new(-pan_speed, 0.0));
                            }
                            PhysicalKey::Code(KeyCode::KeyD)
                            | PhysicalKey::Code(KeyCode::ArrowRight) => {
                                camera.pan(Vec2::new(pan_speed, 0.0));
                            }
                            PhysicalKey::Code(KeyCode::Escape) => elwt.exit(),
                            _ => {}
                        }
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let zoom_factor = match delta {
                        MouseScrollDelta::LineDelta(_, y) => {
                            if y > 0.0 {
                                0.9
                            } else {
                                1.1
                            }
                        }
                        MouseScrollDelta::PixelDelta(pos) => {
                            if pos.y > 0.0 {
                                0.95
                            } else {
                                1.05
                            }
                        }
                    };
                    camera.zoom_by(zoom_factor);
                }
                WindowEvent::RedrawRequested => {
                    let state = RenderState {
                        tick: frame_count,
                        entities: vec![], // No shapes
                        sprites: sprites.clone(),
                        camera,
                    };

                    match renderer.render(&state) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            let (w, h) = renderer.size();
                            renderer.resize(w, h);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            tracing::error!("Out of GPU memory!");
                            elwt.exit();
                        }
                        Err(e) => tracing::warn!("Render error: {:?}", e),
                    }

                    frame_count += 1;
                    let elapsed = last_fps_time.elapsed().as_secs_f32();
                    if elapsed >= 1.0 {
                        let metrics = renderer.metrics();
                        window.set_title(&format!(
                            "Arc Citadel - Sprite Test | {} sprites | {:.1} FPS",
                            sprites.len(),
                            metrics.fps(),
                        ));
                        frame_count = 0;
                        last_fps_time = Instant::now();
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        })
        .expect("Event loop error");
}
```

**Step 2: Build check**

Run: `cargo build --bin sprite_test`
Expected: Compiles

**Step 3: Commit**

```bash
git add src/bin/sprite_test.rs
git commit -m "feat(renderer): add sprite test binary"
```

---

## Task 9: Update live_sim to Use Both Shapes and Sprites

**Files:**
- Modify: `src/bin/live_sim.rs`

**Step 1: Add sprites field to RenderState usage**

Update the `RedrawRequested` handler in `live_sim.rs` to include empty sprites vec:

```rust
let state = RenderState {
    tick: frame_count,
    entities,
    sprites: vec![], // No sprites yet - using shapes for entities
    camera,
};
```

**Step 2: Build and test**

Run: `cargo build --bin live_sim`
Expected: Compiles

**Step 3: Commit**

```bash
git add src/bin/live_sim.rs
git commit -m "fix(live_sim): update to use new RenderState with sprites"
```

---

## Task 10: Add Hex Grid Coordinates

**Files:**
- Create: `src/renderer/hex.rs`
- Modify: `src/renderer/mod.rs`

**Step 1: Write hex.rs**

Create `src/renderer/hex.rs`:

```rust
//! Hex grid coordinate system for terrain rendering.
//!
//! Uses axial coordinates (q, r) with pointy-top hexagons.

use glam::Vec2;

/// Hex size constant (distance from center to corner)
pub const HEX_SIZE: f32 = 10.0;

/// Axial hex coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Convert axial to cube coordinates for algorithms.
    pub fn to_cube(&self) -> (i32, i32, i32) {
        let x = self.q;
        let z = self.r;
        let y = -x - z;
        (x, y, z)
    }

    /// Convert hex coordinate to world position (center of hex).
    pub fn to_world(&self) -> Vec2 {
        let x = HEX_SIZE * (3.0_f32.sqrt() * self.q as f32 + 3.0_f32.sqrt() / 2.0 * self.r as f32);
        let y = HEX_SIZE * (3.0 / 2.0 * self.r as f32);
        Vec2::new(x, y)
    }

    /// Get the 6 neighbor coordinates.
    pub fn neighbors(&self) -> [HexCoord; 6] {
        [
            HexCoord::new(self.q + 1, self.r),
            HexCoord::new(self.q + 1, self.r - 1),
            HexCoord::new(self.q, self.r - 1),
            HexCoord::new(self.q - 1, self.r),
            HexCoord::new(self.q - 1, self.r + 1),
            HexCoord::new(self.q, self.r + 1),
        ]
    }

    /// Distance to another hex (in hex steps).
    pub fn distance(&self, other: &HexCoord) -> i32 {
        let (x1, y1, z1) = self.to_cube();
        let (x2, y2, z2) = other.to_cube();
        ((x1 - x2).abs() + (y1 - y2).abs() + (z1 - z2).abs()) / 2
    }
}

/// Convert world position to nearest hex coordinate.
pub fn world_to_hex(pos: Vec2) -> HexCoord {
    let q = (3.0_f32.sqrt() / 3.0 * pos.x - 1.0 / 3.0 * pos.y) / HEX_SIZE;
    let r = (2.0 / 3.0 * pos.y) / HEX_SIZE;

    // Round to nearest hex
    hex_round(q, r)
}

/// Round fractional hex coordinates to integer.
fn hex_round(q: f32, r: f32) -> HexCoord {
    let s = -q - r;

    let mut rq = q.round();
    let mut rr = r.round();
    let rs = s.round();

    let q_diff = (rq - q).abs();
    let r_diff = (rr - r).abs();
    let s_diff = (rs - s).abs();

    if q_diff > r_diff && q_diff > s_diff {
        rq = -rr - rs;
    } else if r_diff > s_diff {
        rr = -rq - rs;
    }

    HexCoord::new(rq as i32, rr as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_world_origin() {
        let hex = HexCoord::new(0, 0);
        let world = hex.to_world();
        assert!((world.x).abs() < 0.001);
        assert!((world.y).abs() < 0.001);
    }

    #[test]
    fn test_hex_distance() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(2, -1);
        assert_eq!(a.distance(&b), 2);
    }

    #[test]
    fn test_world_to_hex_roundtrip() {
        let original = HexCoord::new(3, -2);
        let world = original.to_world();
        let back = world_to_hex(world);
        assert_eq!(original, back);
    }

    #[test]
    fn test_neighbor_count() {
        let hex = HexCoord::new(0, 0);
        assert_eq!(hex.neighbors().len(), 6);
    }
}
```

**Step 2: Update mod.rs**

Add to `src/renderer/mod.rs`:

```rust
pub mod hex;
pub use hex::{HexCoord, world_to_hex, HEX_SIZE};
```

**Step 3: Run tests**

Run: `cargo test --lib renderer::hex`
Expected: PASS

**Step 4: Commit**

```bash
git add src/renderer/hex.rs src/renderer/mod.rs
git commit -m "feat(renderer): add hex grid coordinate system"
```

---

## Verification Commands

After completing all tasks, run:

```bash
# Build all binaries
cargo build --bin live_sim --bin sprite_test --bin renderer

# Run tests
cargo test --lib renderer::

# Run live simulation (requires display)
cargo run --bin live_sim

# Run sprite test (requires display)
cargo run --bin sprite_test
```

Expected: All builds succeed, all tests pass.

---

## Summary

| Task | Files | Purpose |
|------|-------|---------|
| 1 | Cargo.toml | Add image crate |
| 2 | gpu/texture.rs | PNG → GPU texture |
| 3 | gpu/sprite_pipeline.rs | Sprite render pipeline |
| 4 | sprites/instance.rs | GPU-ready sprite format |
| 5 | shapes/vertex.rs | Unit quad geometry |
| 6 | gpu/sprite_buffers.rs | Sprite buffer management |
| 7 | mod.rs, state.rs | Integrate sprites into renderer |
| 8 | bin/sprite_test.rs | Test sprite pipeline |
| 9 | bin/live_sim.rs | Update for new RenderState |
| 10 | hex.rs | Hex grid coordinates |

**Next Steps (Future Plan):**
- Load actual sprite atlases from PNG files
- Add terrain hex rendering using shapes/sprites
- Add fog of war texture
- Add UI overlay with egui
