# Editor Viewport Rendering Design

## Goal

Render sprites in the editor viewport using real wgpu rendering, allowing users to see their game scene both in Scene mode and Play mode.

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| wgpu version | Downgrade to 22 | Editor (egui) uses wgpu 22; sharing GPU context is cleaner than version bridging |
| Render approach | Render-to-texture | Standard editor pattern; clean separation; egui displays texture |
| Initial sprites | Embedded test texture | No asset loading complexity; proves rendering works |
| Movement | Static for now | Focus on rendering; movement comes with scripting later |
| Camera | Fixed at origin | Simplest; shows ~800x600 world units |

## Architecture

```
Editor Frame:
1. EditorViewportRenderer.render() → off-screen texture
2. Register texture with egui_wgpu::Renderer
3. ViewportPanel displays texture via egui::Image
4. egui renders UI (including viewport image) → screen
```

## EditorViewportRenderer

New struct in `longhorn-editor`:

```rust
pub struct EditorViewportRenderer {
    // Render target
    render_texture: wgpu::Texture,
    render_view: wgpu::TextureView,
    egui_texture_id: egui::TextureId,
    size: (u32, u32),

    // Sprite rendering (reused from longhorn-renderer)
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,

    // Test texture (embedded)
    test_texture: GpuTexture,
}
```

**Methods:**
- `new(device, queue, egui_renderer)` - Creates pipeline, buffers, registers with egui
- `resize(device, egui_renderer, width, height)` - Recreates render texture
- `render(encoder, world, camera)` - Renders sprites to off-screen texture
- `texture_id()` - Returns egui TextureId for display

## Integration

**Editor binary (`editor/src/main.rs`):**
- Add `viewport_renderer: Option<EditorViewportRenderer>` to EditorApp
- Create after GPU init
- Call render each frame before egui
- Pass texture ID to viewport panel

**ViewportPanel:**
- Display game texture with `egui::Image::new(texture_id, size)`
- Track size for resize detection

## Implementation Steps

1. **Downgrade wgpu**: Change workspace from 23 → 22, fix API differences
2. **Create test sprite**: Add 32x32 PNG to `crates/longhorn-editor/src/assets/`
3. **Build EditorViewportRenderer**: Copy sprite pipeline, add render-to-texture
4. **Wire up in editor**: Create renderer, call each frame, pass to viewport
5. **Update ViewportPanel**: Display texture with egui::Image
6. **Test**: Verify sprites appear, Play/Pause/Stop work, resize handles correctly
