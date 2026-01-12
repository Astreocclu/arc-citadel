//! GPU context - wgpu device, queue, and surface management.

use std::sync::Arc;
use winit::window::Window;

/// Holds the wgpu device, queue, and surface.
pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl GpuContext {
    /// Create a new GPU context for the given window.
    pub async fn new(window: Arc<Window>) -> Self {
        // Create wgpu instance with all backends
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface from window
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        // Request adapter (prefer low power for integrated graphics)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find GPU adapter");

        // Log adapter info
        let info = adapter.get_info();
        tracing::info!("Using GPU: {} ({:?})", info.name, info.backend);

        // Request device with conservative limits for integrated graphics
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Arc Citadel Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // Configure surface
        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);

        // Prefer sRGB format
        let format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
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

    /// Resize the surface (call on window resize).
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Get surface format.
    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Get current surface size.
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}
