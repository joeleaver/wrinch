# Rendering Pipeline

Rinch uses a multi-stage rendering pipeline that transforms HTML/CSS into GPU-rendered pixels.

## Pipeline Stages

```
┌───────────────────────────────────────────────────────────────┐
│                      1. HTML/CSS Input                         │
│  RSX generates HTML strings that are parsed by blitz-html     │
└───────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌───────────────────────────────────────────────────────────────┐
│                      2. DOM Construction                       │
│  blitz-dom creates a DOM tree from the parsed HTML            │
└───────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌───────────────────────────────────────────────────────────────┐
│                      3. Style Resolution                       │
│  Stylo (Firefox's CSS engine) computes styles for each node   │
└───────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌───────────────────────────────────────────────────────────────┐
│                      4. Layout                                 │
│  Taffy computes the position and size of each element         │
└───────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌───────────────────────────────────────────────────────────────┐
│                      5. Painting                               │
│  blitz-paint generates paint commands for the layout          │
└───────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌───────────────────────────────────────────────────────────────┐
│                      6. Scene Construction                     │
│  Commands are converted to a Vello scene graph                │
└───────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌───────────────────────────────────────────────────────────────┐
│                      7. GPU Rendering                          │
│  Vello renders the scene using wgpu                           │
└───────────────────────────────────────────────────────────────┘
                              │
                              ▼
                          Display
```

## Key Technologies

### Blitz

Blitz is a modular HTML/CSS rendering engine:

- **blitz-html** - HTML parser, produces a DOM tree
- **blitz-dom** - DOM implementation with Stylo integration
- **blitz-traits** - Shared traits for rendering backends
- **blitz-paint** - Converts styled DOM to paint commands

### Stylo

Mozilla's CSS engine (from Firefox) provides:

- Full CSS specification support
- Efficient style computation
- Media query handling
- CSS custom properties

### Taffy

A flexbox/grid layout engine that computes:

- Element positions (x, y)
- Element sizes (width, height)
- Flexbox alignment and distribution
- CSS Grid support

### Vello

A GPU-accelerated 2D graphics library:

- Scene graph-based rendering
- Efficient batching
- High-quality anti-aliasing
- Path rendering (beziers, fills, strokes)
- Text rendering with proper shaping

### wgpu

Cross-platform GPU abstraction:

- Works on Vulkan, Metal, DX12, WebGPU
- Handles surface creation and management
- Provides compute shaders for Vello

## Window Rendering Flow

```rust
// Simplified rendering flow in window_manager.rs

impl ManagedWindow {
    fn paint_scene(&mut self) {
        // 1. Get the document's scene from blitz
        let scene = self.doc.render();

        // 2. Set up render parameters
        let params = RenderParams {
            width: self.size.width,
            height: self.size.height,
            base_color: Color::WHITE,
            antialiasing: AaConfig::default(),
        };

        // 3. Submit to Vello renderer
        self.renderer.render_to_surface(&scene, &params, &self.surface);
    }
}
```

## Incremental Updates

When content changes, the pipeline can skip unchanged stages:

1. **Style cache** - Styles are cached per element selector
2. **Layout cache** - Layout is only recomputed for affected subtrees
3. **Scene diffing** - Only changed primitives are re-rendered

## Performance Characteristics

| Stage | Complexity | Caching |
|-------|------------|---------|
| HTML Parse | O(n) | N/A (one-time) |
| DOM Build | O(n) | Incremental |
| Style Resolve | O(n × rules) | Selector cache |
| Layout | O(n) | Subtree cache |
| Paint | O(visible) | Command cache |
| GPU Render | O(primitives) | GPU buffers |

## Future Optimizations

Planned improvements to the rendering pipeline:

- **Dirty tracking** - Only re-style/re-layout changed subtrees
- **Layer compositing** - GPU layers for transformed content
- **Text caching** - Glyph atlas for repeated text
- **Viewport culling** - Skip off-screen content
