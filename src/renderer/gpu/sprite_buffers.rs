//! Sprite buffer management.

use wgpu::util::DeviceExt;

use super::GpuContext;
use crate::renderer::shapes::vertex::{unit_quad_indices, unit_quad_vertices};
use crate::renderer::sprites::SpriteInstance;

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
    /// Create sprite buffers with initial instance capacity.
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
            size: (initial_capacity * std::mem::size_of::<SpriteInstance>()) as u64,
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
    pub fn upload_instances(&mut self, ctx: &GpuContext, instances: &[SpriteInstance]) {
        if instances.is_empty() {
            return;
        }

        if instances.len() > self.instance_capacity {
            let new_capacity = (instances.len() * 2).max(1024);
            self.instance_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Sprite Instance Buffer"),
                size: (new_capacity * std::mem::size_of::<SpriteInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_capacity = new_capacity;
            tracing::debug!("Grew sprite instance buffer to {} capacity", new_capacity);
        }

        ctx.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(instances),
        );
    }

    /// Ensure instance buffer has enough capacity. Returns true if buffer was reallocated.
    pub fn ensure_capacity(&mut self, ctx: &GpuContext, needed: usize) -> bool {
        if needed <= self.instance_capacity {
            return false;
        }

        let new_capacity = (needed * 2).max(1024);
        self.instance_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Instance Buffer"),
            size: (new_capacity * std::mem::size_of::<SpriteInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.instance_capacity = new_capacity;
        tracing::debug!("Grew sprite instance buffer to {} capacity", new_capacity);
        true
    }
}
