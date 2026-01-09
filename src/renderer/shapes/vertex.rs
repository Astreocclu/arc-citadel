//! Vertex data for shapes.

use bytemuck::{Pod, Zeroable};

/// Basic vertex with 2D position.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl Vertex {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { position: [x, y] }
    }

    /// Vertex buffer layout descriptor.
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

/// Generate unit circle as triangle list (32 segments).
/// Returns (vertices, indices) for indexed drawing.
pub fn circle_geometry() -> (Vec<Vertex>, Vec<u16>) {
    const SEGMENTS: usize = 32;
    let mut vertices = Vec::with_capacity(SEGMENTS + 1);
    let mut indices = Vec::with_capacity(SEGMENTS * 3);

    // Center vertex
    vertices.push(Vertex::new(0.0, 0.0));

    // Perimeter vertices
    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
        vertices.push(Vertex::new(angle.cos(), angle.sin()));
    }

    // Triangle indices (fan pattern)
    for i in 0..SEGMENTS {
        indices.push(0); // Center
        indices.push((i + 1) as u16);
        indices.push(((i + 1) % SEGMENTS + 1) as u16);
    }

    (vertices, indices)
}

/// Generate unit rectangle as triangle list.
pub fn rectangle_geometry() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        Vertex::new(-0.5, -0.5), // 0: Bottom-left
        Vertex::new(0.5, -0.5),  // 1: Bottom-right
        Vertex::new(0.5, 0.5),   // 2: Top-right
        Vertex::new(-0.5, 0.5),  // 3: Top-left
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    (vertices, indices)
}

/// Generate unit equilateral triangle as triangle list (pointing up).
pub fn triangle_geometry() -> (Vec<Vertex>, Vec<u16>) {
    let h = 0.866_f32; // sqrt(3)/2
    let vertices = vec![
        Vertex::new(0.0, h * 0.667),     // Top
        Vertex::new(-0.5, -h * 0.333),   // Bottom-left
        Vertex::new(0.5, -h * 0.333),    // Bottom-right
    ];
    let indices = vec![0, 1, 2];
    (vertices, indices)
}

/// Create a unit quad for sprite rendering.
/// Vertices are in range [-0.5, 0.5] so scale can be applied directly.
pub fn unit_quad_vertices() -> Vec<Vertex> {
    vec![
        Vertex::new(-0.5, -0.5), // Bottom-left
        Vertex::new(0.5, -0.5),  // Bottom-right
        Vertex::new(0.5, 0.5),   // Top-right
        Vertex::new(-0.5, 0.5),  // Top-left
    ]
}

/// Indices for unit quad (two triangles).
pub fn unit_quad_indices() -> Vec<u16> {
    vec![0, 1, 2, 0, 2, 3]
}

/// Generate unit hexagon as triangle list (pointy-top).
pub fn hexagon_geometry() -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::with_capacity(7);
    let mut indices = Vec::with_capacity(6 * 3);

    // Center vertex
    vertices.push(Vertex::new(0.0, 0.0));

    // Corner vertices (pointy-top: start at top)
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        vertices.push(Vertex::new(angle.cos(), angle.sin()));
    }

    // Triangle indices (fan pattern from center)
    for i in 0..6 {
        indices.push(0); // Center
        indices.push((i + 1) as u16);
        indices.push((i % 6 + 2) as u16);
    }
    // Fix last triangle to wrap around
    indices[17] = 1;

    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_geometry() {
        let (verts, indices) = circle_geometry();
        assert_eq!(verts.len(), 33); // 1 center + 32 perimeter
        assert_eq!(indices.len(), 96); // 32 triangles * 3

        // Center at origin
        assert_eq!(verts[0].position, [0.0, 0.0]);

        // All perimeter vertices at distance 1
        for v in &verts[1..] {
            let dist = (v.position[0].powi(2) + v.position[1].powi(2)).sqrt();
            assert!((dist - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_rectangle_geometry() {
        let (verts, indices) = rectangle_geometry();
        assert_eq!(verts.len(), 4);
        assert_eq!(indices.len(), 6); // 2 triangles

        // Check corners
        assert_eq!(verts[0].position, [-0.5, -0.5]);
        assert_eq!(verts[2].position, [0.5, 0.5]);
    }

    #[test]
    fn test_hexagon_geometry() {
        let (verts, indices) = hexagon_geometry();
        assert_eq!(verts.len(), 7); // 1 center + 6 corners
        assert_eq!(indices.len(), 18); // 6 triangles * 3

        // Center at origin
        assert_eq!(verts[0].position, [0.0, 0.0]);

        // All corner vertices at distance 1
        for v in &verts[1..] {
            let dist = (v.position[0].powi(2) + v.position[1].powi(2)).sqrt();
            assert!((dist - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_unit_quad() {
        let verts = unit_quad_vertices();
        let indices = unit_quad_indices();

        assert_eq!(verts.len(), 4);
        assert_eq!(indices.len(), 6); // 2 triangles

        // Check corners are at [-0.5, 0.5] range
        assert_eq!(verts[0].position, [-0.5, -0.5]); // Bottom-left
        assert_eq!(verts[1].position, [0.5, -0.5]);  // Bottom-right
        assert_eq!(verts[2].position, [0.5, 0.5]);   // Top-right
        assert_eq!(verts[3].position, [-0.5, 0.5]);  // Top-left

        // Check indices form two triangles
        assert_eq!(indices, vec![0, 1, 2, 0, 2, 3]);
    }
}
