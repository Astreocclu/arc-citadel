# Renderer Module

> wgpu-based graphics rendering with shapes, sprites, and animations.

## Module Structure (1962 LOC total)

```
renderer/
├── mod.rs              # Core Renderer struct and exports
├── gpu/                # GPU abstraction layer
│   ├── mod.rs          # GPU module exports
│   ├── context.rs      # wgpu device/queue context
│   ├── pipeline.rs     # Render pipeline setup
│   ├── buffers.rs      # Vertex/index buffer management
│   ├── texture.rs      # Texture loading and management
│   ├── sprite_buffers.rs   # Sprite-specific buffers
│   └── sprite_pipeline.rs  # Sprite render pipeline
├── shapes/             # Shape rendering
│   ├── mod.rs          # Shape exports
│   ├── vertex.rs       # Shape vertex definitions
│   └── instance.rs     # Shape instancing
└── sprites/            # Sprite rendering
    ├── mod.rs          # Sprite exports
    ├── atlas.rs        # Texture atlas management
    ├── animation.rs    # Sprite animation system
    └── instance.rs     # Sprite instancing
```

## Status: COMPLETE IMPLEMENTATION

Full wgpu renderer with:
- Shape rendering (circles, rectangles)
- Sprite rendering with texture atlases
- Animation support
- Camera controls (pan, zoom)
- egui integration for UI overlay

## Core Types

### Renderer

```rust
pub struct Renderer {
    context: GpuContext,
    shape_pipeline: ShapePipeline,
    sprite_pipeline: SpritePipeline,
    camera: CameraState,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self
    pub fn render(&mut self, entities: &[RenderEntity]) -> Result<(), SurfaceError>
    pub fn device(&self) -> &Device
    pub fn surface_format(&self) -> TextureFormat
}
```

### RenderEntity

```rust
pub struct RenderEntity {
    pub position: Vec2,
    pub shape: ShapeType,
    pub color: Color,
    pub size: f32,
}

pub enum ShapeType {
    Circle,
    Square,
    Triangle,
}
```

### CameraState

```rust
pub struct CameraState {
    pub position: Vec2,
    pub zoom: f32,
}
```

### Color

```rust
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
```

## Sprite Animation

```rust
pub struct SpriteAnimation {
    pub frames: Vec<SpriteFrame>,
    pub frame_duration: f32,
    pub loop_mode: LoopMode,
}

pub enum LoopMode {
    Once,
    Loop,
    PingPong,
}
```

## Texture Atlas

```rust
pub struct TextureAtlas {
    pub texture: Texture,
    pub regions: HashMap<String, AtlasRegion>,
}

pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
```

## Usage in live_sim

The renderer is used by `bin/live_sim.rs`:

```rust
// Create renderer
let mut renderer = pollster::block_on(Renderer::new(window.clone()));

// Render loop
for entity in world.humans.iter_living() {
    let render_entity = RenderEntity {
        position: to_render_pos(world.humans.positions[idx]),
        shape: ShapeType::Circle,
        color: Color::new(0.2, 0.6, 0.9, 1.0),
        size: 5.0,
    };
    entities.push(render_entity);
}

renderer.render(&entities)?;
```

## Camera Controls

From `live_sim.rs`:

- **WASD / Arrow keys**: Pan camera
- **+/-**: Zoom in/out
- **Mouse wheel**: Zoom
- **Space**: Pause/resume simulation
- **Escape**: Quit

## GPU Pipeline

The renderer uses wgpu with:

1. **Vertex shader**: Transforms positions with camera
2. **Fragment shader**: Colors shapes/sprites
3. **Instancing**: Efficient rendering of many entities

## Integration Points

### With `simulation/`
- Renders entity positions from world state
- Updates each frame based on simulation tick

### With `ui/`
- egui integration for overlay UI
- Shares wgpu context with UI renderer

## Testing

```bash
cargo run --bin renderer  # Standalone renderer test
cargo run --bin live_sim  # Full simulation with rendering
```
