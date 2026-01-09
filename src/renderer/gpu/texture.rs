//! Texture loading and management.

use image::GenericImageView;
use std::path::Path;

/// Bytes per pixel for RGBA textures.
const BYTES_PER_PIXEL: u32 = 4;

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
        // Validate dimensions
        if dimensions.0 == 0 || dimensions.1 == 0 {
            return Err(TextureError::InvalidDimensions {
                width: dimensions.0,
                height: dimensions.1,
            });
        }

        // Validate buffer size matches dimensions
        let expected_size = (dimensions.0 as usize)
            .checked_mul(dimensions.1 as usize)
            .and_then(|pixels| pixels.checked_mul(BYTES_PER_PIXEL as usize))
            .ok_or(TextureError::InvalidDimensions {
                width: dimensions.0,
                height: dimensions.1,
            })?;

        if rgba.len() != expected_size {
            return Err(TextureError::BufferSizeMismatch {
                expected: expected_size,
                actual: rgba.len(),
            });
        }

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
                bytes_per_row: Some(BYTES_PER_PIXEL * dimensions.0),
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
    InvalidDimensions { width: u32, height: u32 },
    BufferSizeMismatch { expected: usize, actual: usize },
}

impl std::fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureError::Load(msg) => write!(f, "Failed to load texture: {}", msg),
            TextureError::InvalidDimensions { width, height } => {
                write!(f, "Invalid texture dimensions: {}x{}", width, height)
            }
            TextureError::BufferSizeMismatch { expected, actual } => {
                write!(
                    f,
                    "Buffer size mismatch: expected {} bytes, got {}",
                    expected, actual
                )
            }
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

    #[test]
    fn test_invalid_dimensions_error_display() {
        let err = TextureError::InvalidDimensions {
            width: 0,
            height: 100,
        };
        let msg = err.to_string();
        assert!(msg.contains("0x100"));
        assert!(msg.contains("Invalid"));
    }

    #[test]
    fn test_buffer_size_mismatch_error_display() {
        let err = TextureError::BufferSizeMismatch {
            expected: 400,
            actual: 200,
        };
        let msg = err.to_string();
        assert!(msg.contains("400"));
        assert!(msg.contains("200"));
        assert!(msg.contains("mismatch"));
    }

    #[test]
    fn test_bytes_per_pixel_constant() {
        // RGBA = 4 bytes per pixel
        assert_eq!(BYTES_PER_PIXEL, 4);
    }
}
