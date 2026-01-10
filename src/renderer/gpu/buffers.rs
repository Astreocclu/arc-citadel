//! GPU buffer management for shapes.

use wgpu::util::DeviceExt;
use super::context::GpuContext;
use crate::renderer::shapes::ShapeInstance;
use crate::renderer::shapes::vertex::{
    circle_geometry, rectangle_geometry,
    triangle_geometry, hexagon_geometry,
};

/// Static geometry for a shape type.
pub struct ShapeGeometry {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

/// All shape geometry buffers plus dynamic instance buffer.
pub struct ShapeBuffers {
    pub circle: ShapeGeometry,
    pub rectangle: ShapeGeometry,
    pub triangle: ShapeGeometry,
    pub hexagon: ShapeGeometry,

    // Dynamic instance buffer (shared across all shapes)
    pub instance_buffer: wgpu::Buffer,
    pub instance_capacity: usize,
}

impl ShapeBuffers {
    /// Create shape buffers with initial instance capacity.
    pub fn new(ctx: &GpuContext, initial_capacity: usize) -> Self {
        Self {
            circle: Self::create_geometry(ctx, "Circle", circle_geometry()),
            rectangle: Self::create_geometry(ctx, "Rectangle", rectangle_geometry()),
            triangle: Self::create_geometry(ctx, "Triangle", triangle_geometry()),
            hexagon: Self::create_geometry(ctx, "Hexagon", hexagon_geometry()),
            instance_buffer: ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: (initial_capacity * std::mem::size_of::<ShapeInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            instance_capacity: initial_capacity,
        }
    }

    fn create_geometry(
        ctx: &GpuContext,
        name: &str,
        (vertices, indices): (Vec<crate::renderer::shapes::Vertex>, Vec<u16>),
    ) -> ShapeGeometry {
        let vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Index Buffer", name)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        ShapeGeometry {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    /// Ensure instance buffer has enough capacity. Returns true if buffer was reallocated.
    pub fn ensure_capacity(&mut self, ctx: &GpuContext, needed: usize) -> bool {
        if needed <= self.instance_capacity {
            return false;
        }

        let new_capacity = (needed * 2).max(1024);
        self.instance_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (new_capacity * std::mem::size_of::<ShapeInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.instance_capacity = new_capacity;
        tracing::debug!("Grew instance buffer to {} capacity", new_capacity);
        true
    }

    /// Upload instances to the instance buffer.
    pub fn upload_instances(&self, queue: &wgpu::Queue, instances: &[ShapeInstance]) {
        if !instances.is_empty() {
            queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(instances),
            );
        }
    }

    /// Upload all instances in one buffer with offsets (single GPU upload).
    /// Returns BatchedInstances with ranges for each shape type.
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
        self.ensure_capacity(ctx, total);

        // Combine all instances into single upload
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
        if !all_instances.is_empty() {
            ctx.queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&all_instances),
            );
        }

        BatchedInstances {
            circle_range: circle_start..circle_end,
            rectangle_range: rectangle_start..rectangle_end,
            triangle_range: triangle_start..triangle_end,
            hexagon_range: hexagon_start..hexagon_end,
        }
    }
}

/// Result of batched instance upload with ranges for each shape type.
#[derive(Clone, Debug)]
pub struct BatchedInstances {
    pub circle_range: std::ops::Range<u32>,
    pub rectangle_range: std::ops::Range<u32>,
    pub triangle_range: std::ops::Range<u32>,
    pub hexagon_range: std::ops::Range<u32>,
}
