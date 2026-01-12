//! wgpu-based renderer for Arc Citadel.
//!
//! The renderer reads from `RenderState` snapshots and never accesses
//! live simulation data directly. This ensures thread safety and
//! clean separation of concerns.

pub mod camera;
pub mod gpu;
pub mod hex;
pub mod metrics;
pub mod shapes;
pub mod sprites;
pub mod state;

use std::sync::Arc;
use winit::window::Window;

use gpu::{GpuContext, ShapeBuffers, ShapePipeline, SpriteBuffers, SpritePipeline, Texture};
use shapes::ShapeInstance;
use sprites::SpriteInstance;

/// Main renderer struct.
pub struct Renderer {
    ctx: GpuContext,
    pipeline: ShapePipeline,
    buffers: ShapeBuffers,

    // Batched instances per shape type
    circle_instances: Vec<ShapeInstance>,
    rectangle_instances: Vec<ShapeInstance>,
    triangle_instances: Vec<ShapeInstance>,
    hexagon_instances: Vec<ShapeInstance>,

    // Sprite rendering
    sprite_pipeline: SpritePipeline,
    sprite_buffers: SpriteBuffers,
    sprite_instances: Vec<SpriteInstance>,
    #[allow(dead_code)] // Stored to keep texture alive for bind group
    default_texture: Texture,
    default_texture_bind_group: wgpu::BindGroup,

    // Performance tracking
    metrics: RenderMetrics,
}

impl Renderer {
    /// Create a new renderer for the given window.
    pub async fn new(window: Arc<Window>) -> Self {
        let ctx = GpuContext::new(window).await;
        let pipeline = ShapePipeline::new(&ctx);
        let buffers = ShapeBuffers::new(&ctx, 10000);

        // Sprite rendering setup
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
            pipeline,
            buffers,
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

    /// Handle window resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize(width, height);
    }

    /// Get current surface size.
    pub fn size(&self) -> (u32, u32) {
        self.ctx.size()
    }

    /// Get draw call count from last frame.
    pub fn draw_calls(&self) -> u32 {
        self.metrics.draw_calls
    }

    /// Get render metrics.
    pub fn metrics(&self) -> &RenderMetrics {
        &self.metrics
    }

    /// Render a frame from the given state.
    pub fn render(&mut self, state: &RenderState) -> Result<(), wgpu::SurfaceError> {
        self.metrics.begin_frame();
        self.metrics.entity_count = state.entities.len();

        // Update camera uniform for both pipelines
        let view_proj = state.camera.view_projection_matrix();
        self.pipeline.update_camera(&self.ctx.queue, view_proj);
        self.sprite_pipeline
            .update_camera(&self.ctx.queue, view_proj);

        // Batch entities by shape type
        self.circle_instances.clear();
        self.rectangle_instances.clear();
        self.triangle_instances.clear();
        self.hexagon_instances.clear();

        for entity in &state.entities {
            let instance = ShapeInstance::new(
                [entity.position.x, entity.position.y],
                entity.facing,
                entity.scale,
                entity.color.to_u32(),
                entity.shape as u32,
            );

            match entity.shape {
                ShapeType::Circle => self.circle_instances.push(instance),
                ShapeType::Rectangle => self.rectangle_instances.push(instance),
                ShapeType::Triangle => self.triangle_instances.push(instance),
                ShapeType::Hexagon => self.hexagon_instances.push(instance),
            }
        }

        // Single batched upload for all instances
        let batched = self.buffers.upload_batched(
            &self.ctx,
            &self.circle_instances,
            &self.rectangle_instances,
            &self.triangle_instances,
            &self.hexagon_instances,
        );
        self.metrics.record_buffer_upload();

        // Get surface texture
        let output = self.ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

            // All shapes share the same instance buffer, use ranges from batched upload
            render_pass.set_vertex_buffer(1, self.buffers.instance_buffer.slice(..));

            // Draw circles (indexed)
            if !batched.circle_range.is_empty() {
                render_pass.set_vertex_buffer(0, self.buffers.circle.vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.buffers.circle.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(
                    0..self.buffers.circle.index_count,
                    0,
                    batched.circle_range.clone(),
                );
                self.metrics.record_draw_call();
            }

            // Draw hexagons (indexed)
            if !batched.hexagon_range.is_empty() {
                render_pass.set_vertex_buffer(0, self.buffers.hexagon.vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.buffers.hexagon.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(
                    0..self.buffers.hexagon.index_count,
                    0,
                    batched.hexagon_range.clone(),
                );
                self.metrics.record_draw_call();
            }

            // Draw triangles (indexed)
            if !batched.triangle_range.is_empty() {
                render_pass.set_vertex_buffer(0, self.buffers.triangle.vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.buffers.triangle.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(
                    0..self.buffers.triangle.index_count,
                    0,
                    batched.triangle_range.clone(),
                );
                self.metrics.record_draw_call();
            }

            // Draw rectangles (indexed)
            if !batched.rectangle_range.is_empty() {
                render_pass.set_vertex_buffer(0, self.buffers.rectangle.vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.buffers.rectangle.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(
                    0..self.buffers.rectangle.index_count,
                    0,
                    batched.rectangle_range.clone(),
                );
                self.metrics.record_draw_call();
            }

            // Render sprites
            if !state.sprites.is_empty() {
                self.sprite_instances.clear();
                for sprite in &state.sprites {
                    self.sprite_instances.push(SpriteInstance::new(
                        [sprite.position.x, sprite.position.y],
                        [sprite.uv_rect[0], sprite.uv_rect[1]],
                        [sprite.uv_rect[2], sprite.uv_rect[3]],
                        sprite.color,
                        sprite.rotation,
                        sprite.scale,
                        sprite.flip_x,
                        sprite.flip_y,
                    ));
                }

                self.sprite_buffers
                    .upload_instances(&self.ctx, &self.sprite_instances);
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
        }

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.metrics.end_frame();

        Ok(())
    }
}

// Re-export commonly used types
pub use hex::{world_to_hex, HexCoord, HEX_SIZE};
pub use metrics::RenderMetrics;
pub use state::{CameraState, Color, RenderEntity, RenderState, ShapeType, SpriteEntity};
