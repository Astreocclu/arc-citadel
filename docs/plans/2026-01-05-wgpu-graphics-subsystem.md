# Arc Citadel: wgpu Graphics Subsystem Implementation Plan

> **Date:** 2026-01-05
> **Status:** Ready for Implementation
> **Confidence:** 93%
> **Target Hardware:** Intel UHD 630 (integrated graphics baseline)

## Executive Summary

Complete rewrite of the graphics subsystem from macroquad to wgpu, implementing 6 phases:
1. Debug shape renderer with camera
2. GPU instancing and batching
3. Sprite system with animation
4. Terrain rendering (hex + polygon)
5. Effects system (particles, combat feedback)
6. UI system (hybrid custom + egui)

**Performance Target:** 10,000 entities at 60fps on integrated graphics.

---

## Pre-Implementation: Delete Existing Renderer

Before starting, remove macroquad code:

```bash
# Delete existing renderer
rm -rf src/render/
rm src/bin/renderer.rs

# Remove macroquad dependency from Cargo.toml
# (manual edit - remove `macroquad = "0.4"`)
```

---

## Phase 1: Debug Shape Renderer

### Goal
Render colored shapes (circles, rectangles, triangles, hexagons) with camera controls.

### 1.1 Add Dependencies

**File:** `Cargo.toml`

```toml
[dependencies]
# Graphics
wgpu = "0.19"
winit = "0.29"
bytemuck = { version = "1.14", features = ["derive"] }
pollster = "0.3"  # For blocking on async

# Math (already present)
glam = "0.25"

# Keep existing deps...
```

**Verification:**
```bash
cargo check
# Expected: Compiles with new dependencies
```

### 1.2 Create Module Structure

**Create directories and files:**
```
src/renderer/
├── mod.rs              # Public API
├── state.rs            # RenderState, RenderEntity
├── camera.rs           # CameraState, transforms
├── gpu/
│   ├── mod.rs
│   ├── context.rs      # wgpu Device, Queue, Surface
│   ├── pipeline.rs     # Shape render pipeline
│   └── buffers.rs      # Vertex/instance buffer management
├── shapes/
│   ├── mod.rs
│   ├── vertex.rs       # Vertex format
│   └── instance.rs     # ShapeInstance struct
└── shaders/
    └── shape.wgsl      # Shape shader
```

### 1.3 Core Types

**File:** `src/renderer/state.rs`

```rust
use glam::Vec2;
use crate::core::types::{EntityId, Species};

/// Frozen snapshot of simulation state for rendering.
/// Immutable once created - no references back to simulation.
#[derive(Clone)]
pub struct RenderState {
    pub tick: u64,
    pub entities: Vec<RenderEntity>,
    pub camera: CameraState,
}

/// Minimal render data per entity.
#[derive(Clone, Copy)]
pub struct RenderEntity {
    pub id: EntityId,
    pub position: Vec2,
    pub facing: f32,
    pub shape: ShapeType,
    pub color: Color,
    pub scale: f32,
    pub z_order: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapeType {
    Circle,
    Rectangle,
    Triangle,
    Hexagon,
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_u32(&self) -> u32 {
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        let a = (self.a * 255.0) as u32;
        (r << 24) | (g << 16) | (b << 8) | a
    }
}

#[derive(Clone, Copy)]
pub struct CameraState {
    pub center: Vec2,
    pub zoom: f32,           // World units per screen pixel
    pub viewport_size: Vec2, // Screen dimensions in pixels
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            zoom: 1.0,
            viewport_size: Vec2::new(1280.0, 720.0),
        }
    }
}
```

**Test:** `src/renderer/state.rs` (add at bottom)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_u32() {
        let white = Color::rgba(1.0, 1.0, 1.0, 1.0);
        assert_eq!(white.to_u32(), 0xFFFFFFFF);

        let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
        assert_eq!(red.to_u32(), 0xFF0000FF);
    }
}
```

### 1.4 Camera System

**File:** `src/renderer/camera.rs`

```rust
use glam::{Vec2, Mat4};
use super::state::CameraState;

impl CameraState {
    /// Convert world coordinates to screen coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let relative = world_pos - self.center;
        let scaled = relative / self.zoom;
        Vec2::new(
            self.viewport_size.x / 2.0 + scaled.x,
            self.viewport_size.y / 2.0 - scaled.y, // Y-flip for screen coords
        )
    }

    /// Convert screen coordinates to world coordinates.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let centered = Vec2::new(
            screen_pos.x - self.viewport_size.x / 2.0,
            self.viewport_size.y / 2.0 - screen_pos.y, // Y-flip
        );
        self.center + centered * self.zoom
    }

    /// Generate view-projection matrix for GPU.
    pub fn view_projection_matrix(&self) -> Mat4 {
        let half_width = self.viewport_size.x * self.zoom / 2.0;
        let half_height = self.viewport_size.y * self.zoom / 2.0;

        Mat4::orthographic_rh(
            self.center.x - half_width,
            self.center.x + half_width,
            self.center.y - half_height,
            self.center.y + half_height,
            -1000.0,
            1000.0,
        )
    }

    /// Pan camera by delta in world units.
    pub fn pan(&mut self, delta: Vec2) {
        self.center += delta;
    }

    /// Zoom toward a screen position.
    pub fn zoom_toward(&mut self, screen_pos: Vec2, factor: f32) {
        let world_before = self.screen_to_world(screen_pos);
        self.zoom *= factor;
        self.zoom = self.zoom.clamp(0.1, 100.0);
        let world_after = self.screen_to_world(screen_pos);
        self.center += world_before - world_after;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_screen_roundtrip() {
        let camera = CameraState {
            center: Vec2::new(100.0, 200.0),
            zoom: 2.0,
            viewport_size: Vec2::new(800.0, 600.0),
        };

        let world = Vec2::new(150.0, 250.0);
        let screen = camera.world_to_screen(world);
        let back = camera.screen_to_world(screen);

        assert!((world - back).length() < 0.001);
    }
}
```

### 1.5 GPU Context

**File:** `src/renderer/gpu/context.rs`

```rust
use std::sync::Arc;
use winit::window::Window;

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl GpuContext {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower, // Prefer integrated
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find GPU adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Arc Citadel Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self {
            device,
            queue,
            surface,
            config,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
```

### 1.6 Shape Instance Data

**File:** `src/renderer/shapes/instance.rs`

```rust
use bytemuck::{Pod, Zeroable};

/// GPU instance data for shapes. 24 bytes, tightly packed.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ShapeInstance {
    pub position: [f32; 2],   // 8 bytes
    pub rotation: f32,        // 4 bytes
    pub scale: f32,           // 4 bytes
    pub color: u32,           // 4 bytes (RGBA8 packed)
    pub shape_type: u32,      // 4 bytes
}

impl ShapeInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ShapeInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // rotation
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
                // scale
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                // color
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
                // shape_type
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}
```

### 1.7 Vertex Data

**File:** `src/renderer/shapes/vertex.rs`

```rust
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

/// Unit circle vertices (32 segments).
pub fn circle_vertices() -> Vec<Vertex> {
    let segments = 32;
    let mut vertices = vec![Vertex { position: [0.0, 0.0] }]; // Center
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        vertices.push(Vertex {
            position: [angle.cos(), angle.sin()],
        });
    }
    vertices
}

/// Unit rectangle vertices.
pub fn rectangle_vertices() -> Vec<Vertex> {
    vec![
        Vertex { position: [-0.5, -0.5] },
        Vertex { position: [0.5, -0.5] },
        Vertex { position: [0.5, 0.5] },
        Vertex { position: [-0.5, 0.5] },
    ]
}

/// Rectangle indices.
pub fn rectangle_indices() -> Vec<u16> {
    vec![0, 1, 2, 0, 2, 3]
}

/// Unit triangle vertices (equilateral, pointing up).
pub fn triangle_vertices() -> Vec<Vertex> {
    let h = 0.866; // sqrt(3)/2
    vec![
        Vertex { position: [0.0, h * 0.667] },     // Top
        Vertex { position: [-0.5, -h * 0.333] },   // Bottom left
        Vertex { position: [0.5, -h * 0.333] },    // Bottom right
    ]
}

/// Unit hexagon vertices (pointy-top).
pub fn hexagon_vertices() -> Vec<Vertex> {
    let mut vertices = vec![Vertex { position: [0.0, 0.0] }]; // Center
    for i in 0..=6 {
        let angle = (i as f32 / 6.0) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        vertices.push(Vertex {
            position: [angle.cos(), angle.sin()],
        });
    }
    vertices
}
```

### 1.8 Shape Shader

**File:** `src/renderer/shaders/shape.wgsl`

```wgsl
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
    let rot = mat2x2<f32>(c, -s, s, c);

    // Transform vertex
    let local = rot * vertex.position * instance.scale;
    let world = vec4<f32>(local + instance.world_position, 0.0, 1.0);

    var output: VertexOutput;
    output.clip_position = camera.view_proj * world;

    // Unpack color from u32 (RGBA8)
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
```

### 1.9 Render Pipeline

**File:** `src/renderer/gpu/pipeline.rs`

```rust
use wgpu::util::DeviceExt;
use super::context::GpuContext;
use crate::renderer::shapes::{instance::ShapeInstance, vertex::Vertex};

pub struct ShapePipeline {
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
}

impl ShapePipeline {
    pub fn new(ctx: &GpuContext) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shape Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shape.wgsl").into()),
        });

        let camera_bind_group_layout =
            ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
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

        let camera_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: 64, // mat4x4
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shape Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shape Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), ShapeInstance::desc()],
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
            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, view_proj: glam::Mat4) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&view_proj.to_cols_array()));
    }
}
```

### 1.10 Buffer Management

**File:** `src/renderer/gpu/buffers.rs`

```rust
use wgpu::util::DeviceExt;
use crate::renderer::shapes::{instance::ShapeInstance, vertex::*};
use super::context::GpuContext;

pub struct ShapeBuffers {
    // Static vertex buffers (one per shape type)
    pub circle_vertex_buffer: wgpu::Buffer,
    pub circle_vertex_count: u32,

    pub rectangle_vertex_buffer: wgpu::Buffer,
    pub rectangle_index_buffer: wgpu::Buffer,
    pub rectangle_index_count: u32,

    pub triangle_vertex_buffer: wgpu::Buffer,
    pub triangle_vertex_count: u32,

    pub hexagon_vertex_buffer: wgpu::Buffer,
    pub hexagon_vertex_count: u32,

    // Dynamic instance buffer
    pub instance_buffer: wgpu::Buffer,
    pub instance_capacity: usize,
}

impl ShapeBuffers {
    pub fn new(ctx: &GpuContext, initial_capacity: usize) -> Self {
        let circle_verts = circle_vertices();
        let circle_vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle Vertex Buffer"),
            contents: bytemuck::cast_slice(&circle_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let rect_verts = rectangle_vertices();
        let rect_indices = rectangle_indices();
        let rectangle_vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rectangle Vertex Buffer"),
            contents: bytemuck::cast_slice(&rect_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let rectangle_index_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rectangle Index Buffer"),
            contents: bytemuck::cast_slice(&rect_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let tri_verts = triangle_vertices();
        let triangle_vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle Vertex Buffer"),
            contents: bytemuck::cast_slice(&tri_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let hex_verts = hexagon_vertices();
        let hexagon_vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hexagon Vertex Buffer"),
            contents: bytemuck::cast_slice(&hex_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (initial_capacity * std::mem::size_of::<ShapeInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            circle_vertex_buffer,
            circle_vertex_count: circle_verts.len() as u32,
            rectangle_vertex_buffer,
            rectangle_index_buffer,
            rectangle_index_count: rect_indices.len() as u32,
            triangle_vertex_buffer,
            triangle_vertex_count: tri_verts.len() as u32,
            hexagon_vertex_buffer,
            hexagon_vertex_count: hex_verts.len() as u32,
            instance_buffer,
            instance_capacity: initial_capacity,
        }
    }

    /// Upload instance data. Grows buffer if needed.
    pub fn upload_instances(&mut self, ctx: &GpuContext, instances: &[ShapeInstance]) {
        if instances.len() > self.instance_capacity {
            // Grow buffer 2x
            let new_capacity = (instances.len() * 2).max(1024);
            self.instance_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: (new_capacity * std::mem::size_of::<ShapeInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_capacity = new_capacity;
        }

        ctx.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
    }
}
```

### 1.11 Main Renderer

**File:** `src/renderer/mod.rs`

```rust
pub mod state;
pub mod camera;
pub mod gpu;
pub mod shapes;

use std::sync::Arc;
use winit::window::Window;
use state::{RenderState, ShapeType};
use shapes::instance::ShapeInstance;
use gpu::{context::GpuContext, pipeline::ShapePipeline, buffers::ShapeBuffers};

pub struct Renderer {
    ctx: GpuContext,
    pipeline: ShapePipeline,
    buffers: ShapeBuffers,

    // Batched instances per shape type
    circle_instances: Vec<ShapeInstance>,
    rectangle_instances: Vec<ShapeInstance>,
    triangle_instances: Vec<ShapeInstance>,
    hexagon_instances: Vec<ShapeInstance>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let ctx = GpuContext::new(window).await;
        let pipeline = ShapePipeline::new(&ctx);
        let buffers = ShapeBuffers::new(&ctx, 10000);

        Self {
            ctx,
            pipeline,
            buffers,
            circle_instances: Vec::with_capacity(10000),
            rectangle_instances: Vec::with_capacity(1000),
            triangle_instances: Vec::with_capacity(1000),
            hexagon_instances: Vec::with_capacity(1000),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize(width, height);
    }

    pub fn render(&mut self, state: &RenderState) -> Result<(), wgpu::SurfaceError> {
        // Update camera uniform
        let view_proj = state.camera.view_projection_matrix();
        self.pipeline.update_camera(&self.ctx.queue, view_proj);

        // Batch entities by shape type
        self.circle_instances.clear();
        self.rectangle_instances.clear();
        self.triangle_instances.clear();
        self.hexagon_instances.clear();

        for entity in &state.entities {
            let instance = ShapeInstance {
                position: [entity.position.x, entity.position.y],
                rotation: entity.facing,
                scale: entity.scale,
                color: entity.color.to_u32(),
                shape_type: entity.shape as u32,
            };

            match entity.shape {
                ShapeType::Circle => self.circle_instances.push(instance),
                ShapeType::Rectangle => self.rectangle_instances.push(instance),
                ShapeType::Triangle => self.triangle_instances.push(instance),
                ShapeType::Hexagon => self.hexagon_instances.push(instance),
            }
        }

        // Get surface texture
        let output = self.ctx.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline.render_pipeline);
            render_pass.set_bind_group(0, &self.pipeline.camera_bind_group, &[]);

            // Draw circles (triangle fan)
            if !self.circle_instances.is_empty() {
                self.buffers.upload_instances(&self.ctx, &self.circle_instances);
                render_pass.set_vertex_buffer(0, self.buffers.circle_vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.buffers.instance_buffer.slice(..));
                render_pass.draw(0..self.buffers.circle_vertex_count, 0..self.circle_instances.len() as u32);
            }

            // Draw rectangles (indexed)
            if !self.rectangle_instances.is_empty() {
                self.buffers.upload_instances(&self.ctx, &self.rectangle_instances);
                render_pass.set_vertex_buffer(0, self.buffers.rectangle_vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.buffers.instance_buffer.slice(..));
                render_pass.set_index_buffer(self.buffers.rectangle_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.buffers.rectangle_index_count, 0, 0..self.rectangle_instances.len() as u32);
            }

            // Draw triangles
            if !self.triangle_instances.is_empty() {
                self.buffers.upload_instances(&self.ctx, &self.triangle_instances);
                render_pass.set_vertex_buffer(0, self.buffers.triangle_vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.buffers.instance_buffer.slice(..));
                render_pass.draw(0..self.buffers.triangle_vertex_count, 0..self.triangle_instances.len() as u32);
            }

            // Draw hexagons (triangle fan)
            if !self.hexagon_instances.is_empty() {
                self.buffers.upload_instances(&self.ctx, &self.hexagon_instances);
                render_pass.set_vertex_buffer(0, self.buffers.hexagon_vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.buffers.instance_buffer.slice(..));
                render_pass.draw(0..self.buffers.hexagon_vertex_count, 0..self.hexagon_instances.len() as u32);
            }
        }

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
```

### 1.12 Module Exports

**File:** `src/renderer/gpu/mod.rs`

```rust
pub mod context;
pub mod pipeline;
pub mod buffers;
```

**File:** `src/renderer/shapes/mod.rs`

```rust
pub mod vertex;
pub mod instance;
```

### 1.13 Binary Entry Point

**File:** `src/bin/renderer.rs`

```rust
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};
use glam::Vec2;
use arc_citadel::renderer::{Renderer, state::{RenderState, RenderEntity, CameraState, ShapeType, Color}};

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    camera: CameraState,
    entities: Vec<RenderEntity>,
}

impl App {
    fn new() -> Self {
        // Create test entities
        let mut entities = Vec::new();
        for i in 0..100 {
            for j in 0..100 {
                entities.push(RenderEntity {
                    id: arc_citadel::core::types::EntityId::new(),
                    position: Vec2::new(i as f32 * 10.0, j as f32 * 10.0),
                    facing: 0.0,
                    shape: match (i + j) % 4 {
                        0 => ShapeType::Circle,
                        1 => ShapeType::Rectangle,
                        2 => ShapeType::Triangle,
                        _ => ShapeType::Hexagon,
                    },
                    color: Color::rgba(
                        (i as f32 / 100.0),
                        (j as f32 / 100.0),
                        0.5,
                        1.0,
                    ),
                    scale: 3.0,
                    z_order: 0,
                });
            }
        }

        Self {
            window: None,
            renderer: None,
            camera: CameraState {
                center: Vec2::new(500.0, 500.0),
                zoom: 1.0,
                viewport_size: Vec2::new(1280.0, 720.0),
            },
            entities,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Arc Citadel - wgpu Renderer")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720)),
                )
                .unwrap(),
        );

        let renderer = pollster::block_on(Renderer::new(window.clone()));

        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                    self.camera.viewport_size = Vec2::new(size.width as f32, size.height as f32);
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(key_code),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                let pan_speed = 20.0 * self.camera.zoom;
                match key_code {
                    KeyCode::KeyW | KeyCode::ArrowUp => self.camera.pan(Vec2::new(0.0, pan_speed)),
                    KeyCode::KeyS | KeyCode::ArrowDown => self.camera.pan(Vec2::new(0.0, -pan_speed)),
                    KeyCode::KeyA | KeyCode::ArrowLeft => self.camera.pan(Vec2::new(-pan_speed, 0.0)),
                    KeyCode::KeyD | KeyCode::ArrowRight => self.camera.pan(Vec2::new(pan_speed, 0.0)),
                    KeyCode::Equal => self.camera.zoom *= 0.9,
                    KeyCode::Minus => self.camera.zoom *= 1.1,
                    KeyCode::Escape => event_loop.exit(),
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                let state = RenderState {
                    tick: 0,
                    entities: self.entities.clone(),
                    camera: self.camera,
                };

                if let Some(renderer) = &mut self.renderer {
                    match renderer.render(&state) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(
                            self.camera.viewport_size.x as u32,
                            self.camera.viewport_size.y as u32,
                        ),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("Render error: {:?}", e),
                    }
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
```

### 1.14 Update lib.rs

**File:** `src/lib.rs` — Add at appropriate location:

```rust
pub mod renderer;
```

### 1.15 Verification

```bash
# Build
cargo build --bin renderer
# Expected: Compiles successfully

# Run
cargo run --bin renderer
# Expected: Window opens with 10,000 colored shapes, WASD pans, +/- zooms

# Test
cargo test renderer::
# Expected: All tests pass
```

---

## Phase 2: GPU Instancing and Batching

### Goal
Optimize rendering with proper instancing, dirty tracking, and performance profiling.

### 2.1 Combined Instance Buffer

Instead of re-uploading per shape type, use a single combined buffer with offsets.

**File:** `src/renderer/gpu/buffers.rs` — Replace `upload_instances`:

```rust
pub struct BatchedInstances {
    pub buffer: wgpu::Buffer,
    pub circle_range: std::ops::Range<u32>,
    pub rectangle_range: std::ops::Range<u32>,
    pub triangle_range: std::ops::Range<u32>,
    pub hexagon_range: std::ops::Range<u32>,
}

impl ShapeBuffers {
    /// Upload all instances in one buffer with offsets.
    pub fn upload_batched(
        &mut self,
        ctx: &GpuContext,
        circles: &[ShapeInstance],
        rectangles: &[ShapeInstance],
        triangles: &[ShapeInstance],
        hexagons: &[ShapeInstance],
    ) -> BatchedInstances {
        let total = circles.len() + rectangles.len() + triangles.len() + hexagons.len();

        // Grow if needed
        if total > self.instance_capacity {
            let new_capacity = (total * 2).max(1024);
            self.instance_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: (new_capacity * std::mem::size_of::<ShapeInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_capacity = new_capacity;
        }

        // Combine all instances
        let mut all_instances = Vec::with_capacity(total);
        let circle_start = 0u32;
        all_instances.extend_from_slice(circles);
        let circle_end = all_instances.len() as u32;

        let rectangle_start = circle_end;
        all_instances.extend_from_slice(rectangles);
        let rectangle_end = all_instances.len() as u32;

        let triangle_start = rectangle_end;
        all_instances.extend_from_slice(triangles);
        let triangle_end = all_instances.len() as u32;

        let hexagon_start = triangle_end;
        all_instances.extend_from_slice(hexagons);
        let hexagon_end = all_instances.len() as u32;

        // Single upload
        ctx.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&all_instances));

        BatchedInstances {
            buffer: self.instance_buffer.clone(),
            circle_range: circle_start..circle_end,
            rectangle_range: rectangle_start..rectangle_end,
            triangle_range: triangle_start..triangle_end,
            hexagon_range: hexagon_start..hexagon_end,
        }
    }
}
```

### 2.2 Performance Metrics

**File:** `src/renderer/metrics.rs`

```rust
use std::time::{Duration, Instant};
use std::collections::VecDeque;

pub struct RenderMetrics {
    frame_times: VecDeque<Duration>,
    last_frame_start: Instant,
    pub entity_count: usize,
    pub draw_calls: u32,
}

impl RenderMetrics {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            last_frame_start: Instant::now(),
            entity_count: 0,
            draw_calls: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.last_frame_start = Instant::now();
        self.draw_calls = 0;
    }

    pub fn end_frame(&mut self) {
        let elapsed = self.last_frame_start.elapsed();
        self.frame_times.push_back(elapsed);
        if self.frame_times.len() > 120 {
            self.frame_times.pop_front();
        }
    }

    pub fn record_draw_call(&mut self) {
        self.draw_calls += 1;
    }

    pub fn avg_frame_time_ms(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let sum: Duration = self.frame_times.iter().sum();
        sum.as_secs_f32() * 1000.0 / self.frame_times.len() as f32
    }

    pub fn fps(&self) -> f32 {
        let ms = self.avg_frame_time_ms();
        if ms > 0.0 { 1000.0 / ms } else { 0.0 }
    }
}
```

### 2.3 Verification

```bash
# Run with 10,000 entities
cargo run --bin renderer
# Expected: 60fps, <5 draw calls, <10ms frame time

# Profile
cargo build --release --bin renderer
./target/release/renderer
# Expected: Smooth rendering, low CPU usage
```

---

## Phase 3: Sprite System

### Goal
Replace shapes with textured sprites, add animation state machine.

### 3.1 Sprite Instance Data

**File:** `src/renderer/sprites/instance.rs`

```rust
use bytemuck::{Pod, Zeroable};

/// GPU sprite instance. 32 bytes, 16-byte aligned.
#[repr(C, align(16))]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SpriteInstance {
    pub position: [f32; 2],       // 8 bytes
    pub uv_offset_scale: u32,     // 4 bytes: 16-bit u, 16-bit v (normalized)
    pub color_tint: u32,          // 4 bytes: RGBA8
    pub transform_flags: u32,     // 4 bytes: 16-bit rotation, 8-bit scale, 8-bit flags
    pub atlas_layer_frame: u32,   // 4 bytes: 8-bit atlas, 24-bit frame
    pub _padding: [u32; 2],       // 8 bytes (future: layer_mask, z_order)
}

impl SpriteInstance {
    pub fn new(
        position: [f32; 2],
        uv: [f32; 2],        // 0.0-1.0
        color: [u8; 4],
        rotation: f32,       // radians
        scale: f32,          // 0.0-2.55
        atlas: u8,
        frame: u32,
    ) -> Self {
        let u_packed = (uv[0].clamp(0.0, 1.0) * 65535.0) as u32;
        let v_packed = (uv[1].clamp(0.0, 1.0) * 65535.0) as u32;
        let uv_offset_scale = u_packed | (v_packed << 16);

        let color_tint = (color[0] as u32) << 24
            | (color[1] as u32) << 16
            | (color[2] as u32) << 8
            | color[3] as u32;

        let rotation_packed = ((rotation / std::f32::consts::TAU).fract() * 65535.0) as u32;
        let scale_packed = ((scale / 2.55).clamp(0.0, 1.0) * 255.0) as u32;
        let transform_flags = (rotation_packed << 16) | (scale_packed << 8);

        let atlas_layer_frame = ((atlas as u32) << 24) | (frame & 0x00FFFFFF);

        Self {
            position,
            uv_offset_scale,
            color_tint,
            transform_flags,
            atlas_layer_frame,
            _padding: [0; 2],
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: 32, // Fixed 32 bytes
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
                wgpu::VertexAttribute { offset: 8, shader_location: 2, format: wgpu::VertexFormat::Uint32 },
                wgpu::VertexAttribute { offset: 12, shader_location: 3, format: wgpu::VertexFormat::Uint32 },
                wgpu::VertexAttribute { offset: 16, shader_location: 4, format: wgpu::VertexFormat::Uint32 },
                wgpu::VertexAttribute { offset: 20, shader_location: 5, format: wgpu::VertexFormat::Uint32 },
            ],
        }
    }
}
```

### 3.2 Animation State Machine

**File:** `src/renderer/sprites/animation.rs`

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationState {
    Idle,
    Move,
    Attack,
    Hit,
    Die,
    Rout,
}

pub struct AnimationController {
    pub current_state: AnimationState,
    pub current_frame: u8,
    pub frame_timer: f32,
    pub direction: u8, // 0-7 for 8 directions
}

impl AnimationController {
    pub fn new() -> Self {
        Self {
            current_state: AnimationState::Idle,
            current_frame: 0,
            frame_timer: 0.0,
            direction: 0,
        }
    }

    /// Update animation, returns true if frame changed.
    pub fn update(&mut self, dt: f32, animation_data: &AnimationData) -> bool {
        self.frame_timer += dt;

        let frame_duration = animation_data.frame_duration(self.current_state);
        let frame_count = animation_data.frame_count(self.current_state);

        if self.frame_timer >= frame_duration {
            self.frame_timer -= frame_duration;
            self.current_frame = (self.current_frame + 1) % frame_count;
            return true;
        }
        false
    }

    pub fn set_state(&mut self, state: AnimationState) {
        if self.current_state != state {
            self.current_state = state;
            self.current_frame = 0;
            self.frame_timer = 0.0;
        }
    }

    /// Get frame index in sprite atlas.
    pub fn atlas_frame(&self, animation_data: &AnimationData) -> u32 {
        animation_data.base_frame(self.current_state) + self.current_frame as u32
    }
}

pub struct AnimationData {
    pub idle_frames: u8,
    pub move_frames: u8,
    pub attack_frames: u8,
    pub hit_frames: u8,
    pub die_frames: u8,
    pub rout_frames: u8,
    pub frame_duration: f32, // seconds per frame
}

impl AnimationData {
    pub fn frame_count(&self, state: AnimationState) -> u8 {
        match state {
            AnimationState::Idle => self.idle_frames,
            AnimationState::Move => self.move_frames,
            AnimationState::Attack => self.attack_frames,
            AnimationState::Hit => self.hit_frames,
            AnimationState::Die => self.die_frames,
            AnimationState::Rout => self.rout_frames,
        }
    }

    pub fn frame_duration(&self, _state: AnimationState) -> f32 {
        self.frame_duration
    }

    pub fn base_frame(&self, state: AnimationState) -> u32 {
        let mut offset = 0u32;
        match state {
            AnimationState::Idle => offset,
            AnimationState::Move => { offset += self.idle_frames as u32; offset }
            AnimationState::Attack => { offset += self.idle_frames as u32 + self.move_frames as u32; offset }
            AnimationState::Hit => { offset += self.idle_frames as u32 + self.move_frames as u32 + self.attack_frames as u32; offset }
            AnimationState::Die => { offset += self.idle_frames as u32 + self.move_frames as u32 + self.attack_frames as u32 + self.hit_frames as u32; offset }
            AnimationState::Rout => { offset += self.idle_frames as u32 + self.move_frames as u32 + self.attack_frames as u32 + self.hit_frames as u32 + self.die_frames as u32; offset }
        }
    }
}
```

### 3.3 Texture Atlas

**File:** `src/renderer/sprites/atlas.rs`

```rust
use std::collections::HashMap;

pub struct TextureAtlas {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
    pub sprites: HashMap<String, SpriteRegion>,
}

#[derive(Clone, Copy)]
pub struct SpriteRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl SpriteRegion {
    pub fn uv(&self, atlas_width: u32, atlas_height: u32) -> ([f32; 2], [f32; 2]) {
        let u0 = self.x as f32 / atlas_width as f32;
        let v0 = self.y as f32 / atlas_height as f32;
        let u1 = (self.x + self.width) as f32 / atlas_width as f32;
        let v1 = (self.y + self.height) as f32 / atlas_height as f32;
        ([u0, v0], [u1, v1])
    }
}

impl TextureAtlas {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Self {
        let img = image::load_from_memory(bytes).expect("Failed to load image");
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            width,
            height,
            sprites: HashMap::new(),
        }
    }
}
```

### 3.4 Verification

```bash
cargo test renderer::sprites::
# Expected: Animation state machine tests pass

cargo run --bin renderer
# Expected: Sprites render with animation (requires test atlas)
```

---

## Phase 4: Terrain Rendering

### Goal
Render hex grids (campaign) and feature polygons (battle), with fog of war.

### 4.1 Hex Coordinate Utilities

**File:** `src/renderer/terrain/hex.rs`

```rust
use glam::Vec2;

/// Axial hex coordinates.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Convert to world coordinates (pointy-top, 100m hexes).
    pub fn to_world(&self, hex_size: f32) -> Vec2 {
        let sqrt3 = 3.0_f32.sqrt();
        let x = hex_size * (sqrt3 * self.q as f32 + sqrt3 / 2.0 * self.r as f32);
        let y = hex_size * (3.0 / 2.0 * self.r as f32);
        Vec2::new(x, y)
    }

    /// Convert world coordinates to nearest hex.
    pub fn from_world(world: Vec2, hex_size: f32) -> Self {
        let sqrt3 = 3.0_f32.sqrt();
        let q = (sqrt3 / 3.0 * world.x - 1.0 / 3.0 * world.y) / hex_size;
        let r = (2.0 / 3.0 * world.y) / hex_size;
        Self::round(q, r)
    }

    fn round(q: f32, r: f32) -> Self {
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

        Self::new(rq as i32, rr as i32)
    }

    /// Get the 6 neighbors.
    pub fn neighbors(&self) -> [HexCoord; 6] {
        [
            Self::new(self.q + 1, self.r - 1), // NE
            Self::new(self.q + 1, self.r),     // E
            Self::new(self.q, self.r + 1),     // SE
            Self::new(self.q - 1, self.r + 1), // SW
            Self::new(self.q - 1, self.r),     // W
            Self::new(self.q, self.r - 1),     // NW
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_roundtrip() {
        let hex = HexCoord::new(5, -3);
        let world = hex.to_world(50.0);
        let back = HexCoord::from_world(world, 50.0);
        assert_eq!(hex, back);
    }
}
```

### 4.2 Fog of War Texture

**File:** `src/renderer/terrain/fog.rs`

```rust
pub struct FogOfWar {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FogState {
    Unexplored = 0,
    Explored = 77,   // ~30% visible
    Visible = 255,   // 100% visible
}

impl FogOfWar {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let data = vec![FogState::Unexplored as u8; (width * height) as usize];

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fog of War Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view, width, height, data }
    }

    pub fn set_visibility(&mut self, x: u32, y: u32, state: FogState) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = state as u8;
        }
    }

    pub fn upload(&self, queue: &wgpu::Queue) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.width),
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
        );
    }
}
```

### 4.3 Verification

```bash
cargo test renderer::terrain::
# Expected: Hex coordinate tests pass

cargo run --bin renderer
# Expected: Hex grid renders with fog overlay
```

---

## Phase 5: Effects System

### Goal
Particle system for combat effects, hit feedback, status indicators.

### 5.1 Particle System

**File:** `src/renderer/effects/particles.rs`

```rust
use glam::Vec2;

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub color: [f32; 4],
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct ParticleEmitter {
    pub position: Vec2,
    pub particles: Vec<Particle>,
    pub spawn_rate: f32,
    pub spawn_timer: f32,
    pub particle_lifetime: f32,
    pub initial_velocity: Vec2,
    pub velocity_variance: f32,
    pub color: [f32; 4],
    pub size: f32,
    pub gravity: Vec2,
    pub active: bool,
}

impl ParticleEmitter {
    pub fn blood_burst(position: Vec2) -> Self {
        Self {
            position,
            particles: Vec::with_capacity(20),
            spawn_rate: 0.0, // Burst mode
            spawn_timer: 0.0,
            particle_lifetime: 0.5,
            initial_velocity: Vec2::ZERO,
            velocity_variance: 50.0,
            color: [0.8, 0.1, 0.1, 1.0],
            size: 2.0,
            gravity: Vec2::new(0.0, -100.0),
            active: true,
        }
    }

    pub fn dust_trail(position: Vec2) -> Self {
        Self {
            position,
            particles: Vec::with_capacity(10),
            spawn_rate: 30.0, // Particles per second
            spawn_timer: 0.0,
            particle_lifetime: 0.3,
            initial_velocity: Vec2::new(0.0, 5.0),
            velocity_variance: 10.0,
            color: [0.6, 0.5, 0.4, 0.5],
            size: 1.5,
            gravity: Vec2::ZERO,
            active: true,
        }
    }

    pub fn update(&mut self, dt: f32, rng: &mut impl rand::Rng) {
        // Update existing particles
        self.particles.retain_mut(|p| {
            p.lifetime -= dt;
            if p.lifetime <= 0.0 {
                return false;
            }
            p.velocity += self.gravity * dt;
            p.position += p.velocity * dt;
            p.color[3] = p.lifetime / p.max_lifetime; // Fade out
            true
        });

        // Spawn new particles
        if self.spawn_rate > 0.0 && self.active {
            self.spawn_timer += dt;
            let spawn_interval = 1.0 / self.spawn_rate;
            while self.spawn_timer >= spawn_interval {
                self.spawn_timer -= spawn_interval;
                self.spawn_particle(rng);
            }
        }
    }

    pub fn burst(&mut self, count: usize, rng: &mut impl rand::Rng) {
        for _ in 0..count {
            self.spawn_particle(rng);
        }
    }

    fn spawn_particle(&mut self, rng: &mut impl rand::Rng) {
        use rand::Rng;
        let angle = rng.gen::<f32>() * std::f32::consts::TAU;
        let speed = rng.gen::<f32>() * self.velocity_variance;
        let velocity = self.initial_velocity + Vec2::new(angle.cos(), angle.sin()) * speed;

        self.particles.push(Particle {
            position: self.position,
            velocity,
            color: self.color,
            size: self.size,
            lifetime: self.particle_lifetime,
            max_lifetime: self.particle_lifetime,
        });
    }
}

pub struct ParticleSystem {
    pub emitters: Vec<ParticleEmitter>,
    pub max_particles: usize,
}

impl ParticleSystem {
    pub fn new(max_particles: usize) -> Self {
        Self {
            emitters: Vec::new(),
            max_particles,
        }
    }

    pub fn update(&mut self, dt: f32, rng: &mut impl rand::Rng) {
        for emitter in &mut self.emitters {
            emitter.update(dt, rng);
        }
        // Remove dead emitters
        self.emitters.retain(|e| e.active || !e.particles.is_empty());
    }

    pub fn total_particles(&self) -> usize {
        self.emitters.iter().map(|e| e.particles.len()).sum()
    }

    pub fn spawn_blood(&mut self, position: Vec2, rng: &mut impl rand::Rng) {
        let mut emitter = ParticleEmitter::blood_burst(position);
        emitter.burst(15, rng);
        emitter.active = false; // One-shot
        self.emitters.push(emitter);
    }
}
```

### 5.2 Combat Feedback

**File:** `src/renderer/effects/feedback.rs`

```rust
use glam::Vec2;

pub struct HitFlash {
    pub entity_id: crate::core::types::EntityId,
    pub timer: f32,
    pub duration: f32,
    pub color: [f32; 4],
}

pub struct DamageNumber {
    pub position: Vec2,
    pub velocity: Vec2,
    pub value: i32,
    pub timer: f32,
    pub duration: f32,
    pub color: [f32; 4],
}

pub struct CombatFeedbackSystem {
    pub hit_flashes: Vec<HitFlash>,
    pub damage_numbers: Vec<DamageNumber>,
}

impl CombatFeedbackSystem {
    pub fn new() -> Self {
        Self {
            hit_flashes: Vec::new(),
            damage_numbers: Vec::new(),
        }
    }

    pub fn flash_entity(&mut self, entity_id: crate::core::types::EntityId) {
        self.hit_flashes.push(HitFlash {
            entity_id,
            timer: 0.0,
            duration: 0.15,
            color: [1.0, 0.3, 0.3, 1.0],
        });
    }

    pub fn spawn_damage_number(&mut self, position: Vec2, damage: i32) {
        let color = if damage >= 50 {
            [1.0, 0.2, 0.2, 1.0] // Red for heavy
        } else if damage >= 20 {
            [1.0, 0.8, 0.2, 1.0] // Yellow for medium
        } else {
            [1.0, 1.0, 1.0, 1.0] // White for light
        };

        self.damage_numbers.push(DamageNumber {
            position,
            velocity: Vec2::new(0.0, 30.0),
            value: damage,
            timer: 0.0,
            duration: 1.0,
            color,
        });
    }

    pub fn update(&mut self, dt: f32) {
        // Update hit flashes
        self.hit_flashes.retain_mut(|f| {
            f.timer += dt;
            f.timer < f.duration
        });

        // Update damage numbers
        self.damage_numbers.retain_mut(|d| {
            d.timer += dt;
            d.position += d.velocity * dt;
            d.color[3] = 1.0 - (d.timer / d.duration); // Fade out
            d.timer < d.duration
        });
    }

    pub fn get_flash_intensity(&self, entity_id: crate::core::types::EntityId) -> f32 {
        for flash in &self.hit_flashes {
            if flash.entity_id == entity_id {
                return 1.0 - (flash.timer / flash.duration);
            }
        }
        0.0
    }
}
```

### 5.3 Verification

```bash
cargo test renderer::effects::
# Expected: Particle and feedback tests pass
```

---

## Phase 6: UI System

### Goal
Hybrid custom + egui UI for HUD, minimap, selection panel.

### 6.1 Add egui Dependency

**File:** `Cargo.toml` — Add:

```toml
egui = "0.27"
egui-wgpu = "0.27"
egui-winit = "0.27"
```

### 6.2 egui Integration

**File:** `src/renderer/ui/egui_integration.rs`

```rust
use egui_wgpu::ScreenDescriptor;
use egui_winit::State as EguiWinitState;
use winit::window::Window;

pub struct EguiRenderer {
    pub context: egui::Context,
    pub state: EguiWinitState,
    pub renderer: egui_wgpu::Renderer,
}

impl EguiRenderer {
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        window: &Window,
    ) -> Self {
        let context = egui::Context::default();
        let viewport_id = context.viewport_id();
        let state = EguiWinitState::new(
            context.clone(),
            viewport_id,
            window,
            None,
            None,
            None,
        );
        let renderer = egui_wgpu::Renderer::new(device, output_format, None, 1, false);

        Self { context, state, renderer }
    }

    pub fn handle_input(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.state.take_egui_input(window);
        self.context.begin_pass(raw_input);
    }

    pub fn end_frame_and_render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        screen_descriptor: ScreenDescriptor,
        window: &Window,
    ) {
        let full_output = self.context.end_pass();
        self.state.handle_platform_output(window, full_output.platform_output);

        let tris = self.context.tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, image_delta);
        }
        self.renderer.update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear, draw on top
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.renderer.render(&mut render_pass, &tris, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
```

### 6.3 Custom Health Bars

**File:** `src/renderer/ui/health_bars.rs`

```rust
use glam::Vec2;
use crate::renderer::shapes::instance::ShapeInstance;
use crate::renderer::state::CameraState;

pub struct HealthBarRenderer {
    instances: Vec<ShapeInstance>,
}

impl HealthBarRenderer {
    pub fn new() -> Self {
        Self { instances: Vec::with_capacity(1000) }
    }

    pub fn prepare(
        &mut self,
        entities: &[(Vec2, f32)], // (position, health 0-1)
        camera: &CameraState,
    ) {
        self.instances.clear();

        for (world_pos, health) in entities {
            let screen_pos = camera.world_to_screen(*world_pos);

            // Background bar (dark)
            self.instances.push(ShapeInstance {
                position: [screen_pos.x, screen_pos.y - 10.0],
                rotation: 0.0,
                scale: 20.0,
                color: 0x333333FF,
                shape_type: 1, // Rectangle
            });

            // Health bar (colored by health)
            let color = if *health > 0.6 {
                0x44FF44FF // Green
            } else if *health > 0.3 {
                0xFFFF44FF // Yellow
            } else {
                0xFF4444FF // Red
            };

            self.instances.push(ShapeInstance {
                position: [screen_pos.x - 10.0 + health * 10.0, screen_pos.y - 10.0],
                rotation: 0.0,
                scale: 20.0 * health,
                color,
                shape_type: 1,
            });
        }
    }

    pub fn instances(&self) -> &[ShapeInstance] {
        &self.instances
    }
}
```

### 6.4 Minimap

**File:** `src/renderer/ui/minimap.rs`

```rust
use glam::Vec2;

pub struct Minimap {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: u32,
    pub world_bounds: (Vec2, Vec2), // min, max
}

impl Minimap {
    pub fn new(device: &wgpu::Device, size: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Minimap Texture"),
            size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            size,
            world_bounds: (Vec2::ZERO, Vec2::new(1000.0, 1000.0)),
        }
    }

    pub fn world_to_minimap(&self, world_pos: Vec2) -> Vec2 {
        let normalized = (world_pos - self.world_bounds.0)
            / (self.world_bounds.1 - self.world_bounds.0);
        normalized * self.size as f32
    }
}
```

### 6.5 Verification

```bash
cargo build --bin renderer
# Expected: Compiles with egui integration

cargo run --bin renderer
# Expected: egui panels render on top of game
```

---

## Validation Checklist

### Phase 1 Complete When:
- [ ] Window opens with wgpu context
- [ ] 10,000 shapes render at 60fps
- [ ] Camera pans with WASD
- [ ] Camera zooms with +/-
- [ ] All 4 shape types render correctly

### Phase 2 Complete When:
- [ ] Single buffer upload per frame
- [ ] <5 draw calls for 10,000 entities
- [ ] Frame time <10ms

### Phase 3 Complete When:
- [ ] Sprites render from atlas
- [ ] Animation state machine works
- [ ] Equipment layers composite correctly

### Phase 4 Complete When:
- [ ] Hex grid renders for campaign
- [ ] Feature polygons render for battle
- [ ] Fog of war overlay works

### Phase 5 Complete When:
- [ ] Blood particles spawn on hit
- [ ] Hit flash visible
- [ ] Damage numbers float up

### Phase 6 Complete When:
- [ ] egui panels render
- [ ] Minimap shows entities
- [ ] Health bars above entities

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| wgpu version incompatibility | Pin to wgpu 0.19, test on multiple backends |
| Integrated GPU performance | Profile early, reduce particle count if needed |
| Shader compilation failures | Test on Vulkan, DX12, Metal, and GL backends |
| egui performance | Use custom UI for hot paths, limit egui to panels |

---

## Dependencies Summary

```toml
[dependencies]
wgpu = "0.19"
winit = "0.29"
bytemuck = { version = "1.14", features = ["derive"] }
pollster = "0.3"
glam = "0.25"
image = "0.24"
egui = "0.27"
egui-wgpu = "0.27"
egui-winit = "0.27"
rand = "0.8"
```

---

*Plan generated 2026-01-05. Execute phases sequentially, validating each before proceeding.*
