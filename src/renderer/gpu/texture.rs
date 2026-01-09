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
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::ImageDataLayout {
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
