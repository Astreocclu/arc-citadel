//! Texture atlas for sprite rendering.

use std::collections::HashMap;

/// A region within a texture atlas.
#[derive(Clone, Copy, Debug)]
pub struct SpriteRegion {
    /// X position in pixels.
    pub x: u32,
    /// Y position in pixels.
    pub y: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl SpriteRegion {
    /// Create a new sprite region.
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Get UV coordinates (offset and size) for this region.
    pub fn uv(&self, atlas_width: u32, atlas_height: u32) -> ([f32; 2], [f32; 2]) {
        let u0 = self.x as f32 / atlas_width as f32;
        let v0 = self.y as f32 / atlas_height as f32;
        let u_size = self.width as f32 / atlas_width as f32;
        let v_size = self.height as f32 / atlas_height as f32;
        ([u0, v0], [u_size, v_size])
    }
}

/// A texture atlas containing multiple sprites.
pub struct TextureAtlas {
    /// GPU texture.
    pub texture: wgpu::Texture,
    /// Texture view for binding.
    pub view: wgpu::TextureView,
    /// Sampler for the texture.
    pub sampler: wgpu::Sampler,
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Named sprite regions.
    pub sprites: HashMap<String, SpriteRegion>,
}

impl TextureAtlas {
    /// Create a texture atlas from raw RGBA bytes.
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, image::ImageError> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
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
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{} Sampler", label)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            width,
            height,
            sprites: HashMap::new(),
        })
    }

    /// Create a placeholder 1x1 white texture for testing.
    pub fn placeholder(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let bytes = [255u8, 255, 255, 255]; // Single white pixel

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Placeholder Atlas"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
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
            &bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Placeholder Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            width: 1,
            height: 1,
            sprites: HashMap::new(),
        }
    }

    /// Add a named sprite region.
    pub fn add_sprite(&mut self, name: impl Into<String>, region: SpriteRegion) {
        self.sprites.insert(name.into(), region);
    }

    /// Get a sprite region by name.
    pub fn get_sprite(&self, name: &str) -> Option<&SpriteRegion> {
        self.sprites.get(name)
    }

    /// Define a grid of uniform sprites (e.g., 8x8 grid of 32x32 sprites).
    pub fn define_grid(
        &mut self,
        prefix: &str,
        cols: u32,
        rows: u32,
        sprite_width: u32,
        sprite_height: u32,
    ) {
        for row in 0..rows {
            for col in 0..cols {
                let name = format!("{}_{}", prefix, row * cols + col);
                let region = SpriteRegion::new(
                    col * sprite_width,
                    row * sprite_height,
                    sprite_width,
                    sprite_height,
                );
                self.add_sprite(name, region);
            }
        }
    }

    /// Get UV for a grid-based frame index.
    pub fn grid_uv(
        &self,
        frame: u32,
        cols: u32,
        sprite_width: u32,
        sprite_height: u32,
    ) -> ([f32; 2], [f32; 2]) {
        let col = frame % cols;
        let row = frame / cols;
        let region = SpriteRegion::new(
            col * sprite_width,
            row * sprite_height,
            sprite_width,
            sprite_height,
        );
        region.uv(self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_region_uv() {
        let region = SpriteRegion::new(64, 32, 32, 32);
        let (offset, size) = region.uv(256, 256);

        assert_eq!(offset[0], 0.25); // 64/256
        assert_eq!(offset[1], 0.125); // 32/256
        assert_eq!(size[0], 0.125); // 32/256
        assert_eq!(size[1], 0.125); // 32/256
    }

    #[test]
    fn test_grid_uv() {
        // Simulate a 4x4 grid atlas of 32x32 sprites in a 128x128 texture
        let region = SpriteRegion::new(32, 32, 32, 32); // Frame 5 (col 1, row 1)
        let (offset, size) = region.uv(128, 128);

        assert_eq!(offset[0], 0.25);
        assert_eq!(offset[1], 0.25);
        assert_eq!(size[0], 0.25);
        assert_eq!(size[1], 0.25);
    }
}
