use engine_geometry_core::MeshHandle;
use engine_materials_core::MaterialHandle;
use glam::{Mat4, Vec4};

pub trait CommandBuffer {
    fn begin_render_pass(&mut self, pass: &RenderPassDescriptor);
    fn end_render_pass(&mut self);
    fn draw_mesh(&mut self, mesh: &MeshHandle, material: &MaterialHandle, transform: &Mat4);
    fn set_viewport(&mut self, viewport: &Viewport);
    fn clear(&mut self, clear_value: &ClearValue);
    fn submit(self);
}

#[derive(Debug, Clone)]
pub struct RenderPassDescriptor {
    pub color_attachments: Vec<ColorAttachment>,
    pub depth_attachment: Option<DepthAttachment>,
}

#[derive(Debug, Clone)]
pub struct ColorAttachment {
    pub target: RenderTarget,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub clear_value: Vec4,
}

#[derive(Debug, Clone)]
pub struct DepthAttachment {
    pub target: RenderTarget,
    pub depth_load_op: LoadOp,
    pub depth_store_op: StoreOp,
    pub clear_depth: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadOp {
    Clear,
    Load,
    DontCare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOp {
    Store,
    DontCare,
}

#[derive(Debug, Clone)]
pub enum RenderTarget {
    Surface,
    Texture { width: u32, height: u32 },
}

#[derive(Debug, Clone)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[derive(Debug, Clone)]
pub struct ClearValue {
    pub color: Vec4,
    pub depth: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
}

impl Default for ClearValue {
    fn default() -> Self {
        Self {
            color: Vec4::new(0.0, 0.0, 0.0, 1.0), // Black
            depth: 1.0,
        }
    }
}

impl Viewport {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }

    pub fn with_depth(mut self, min_depth: f32, max_depth: f32) -> Self {
        self.min_depth = min_depth;
        self.max_depth = max_depth;
        self
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width / self.height
    }
}

impl ClearValue {
    pub fn color_only(color: Vec4) -> Self {
        Self { color, depth: 1.0 }
    }

    pub fn depth_only(depth: f32) -> Self {
        Self {
            color: Vec4::ZERO,
            depth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_default() {
        let viewport = Viewport::default();
        assert_eq!(viewport.x, 0.0);
        assert_eq!(viewport.y, 0.0);
        assert_eq!(viewport.width, 1920.0);
        assert_eq!(viewport.height, 1080.0);
        assert_eq!(viewport.min_depth, 0.0);
        assert_eq!(viewport.max_depth, 1.0);
    }

    #[test]
    fn test_viewport_creation() {
        let viewport = Viewport::new(10.0, 20.0, 800.0, 600.0);
        assert_eq!(viewport.x, 10.0);
        assert_eq!(viewport.y, 20.0);
        assert_eq!(viewport.width, 800.0);
        assert_eq!(viewport.height, 600.0);
        assert_eq!(viewport.min_depth, 0.0);
        assert_eq!(viewport.max_depth, 1.0);
    }

    #[test]
    fn test_viewport_with_depth() {
        let viewport = Viewport::new(0.0, 0.0, 1024.0, 768.0).with_depth(0.1, 0.9);
        assert_eq!(viewport.min_depth, 0.1);
        assert_eq!(viewport.max_depth, 0.9);
    }

    #[test]
    fn test_viewport_aspect_ratio() {
        let viewport = Viewport::new(0.0, 0.0, 1920.0, 1080.0);
        assert!((viewport.aspect_ratio() - 16.0 / 9.0).abs() < 0.001);

        let square_viewport = Viewport::new(0.0, 0.0, 512.0, 512.0);
        assert_eq!(square_viewport.aspect_ratio(), 1.0);
    }

    #[test]
    fn test_clear_value_default() {
        let clear = ClearValue::default();
        assert_eq!(clear.color, Vec4::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(clear.depth, 1.0);
    }

    #[test]
    fn test_clear_value_color_only() {
        let clear = ClearValue::color_only(Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(clear.color, Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(clear.depth, 1.0);
    }

    #[test]
    fn test_clear_value_depth_only() {
        let clear = ClearValue::depth_only(0.5);
        assert_eq!(clear.color, Vec4::ZERO);
        assert_eq!(clear.depth, 0.5);
    }

    #[test]
    fn test_load_op_enum() {
        assert_eq!(LoadOp::Clear, LoadOp::Clear);
        assert_ne!(LoadOp::Clear, LoadOp::Load);
        assert_ne!(LoadOp::Load, LoadOp::DontCare);
    }

    #[test]
    fn test_store_op_enum() {
        assert_eq!(StoreOp::Store, StoreOp::Store);
        assert_ne!(StoreOp::Store, StoreOp::DontCare);
    }

    #[test]
    fn test_render_target_enum() {
        let surface_target = RenderTarget::Surface;
        let texture_target = RenderTarget::Texture {
            width: 512,
            height: 512,
        };

        match surface_target {
            RenderTarget::Surface => {} // Success
            _ => panic!("Should be surface target"),
        }

        match texture_target {
            RenderTarget::Texture { width, height } => {
                assert_eq!(width, 512);
                assert_eq!(height, 512);
            }
            _ => panic!("Should be texture target"),
        }
    }

    #[test]
    fn test_color_attachment_creation() {
        let attachment = ColorAttachment {
            target: RenderTarget::Surface,
            load_op: LoadOp::Clear,
            store_op: StoreOp::Store,
            clear_value: Vec4::new(0.2, 0.3, 0.4, 1.0),
        };

        assert_eq!(attachment.load_op, LoadOp::Clear);
        assert_eq!(attachment.store_op, StoreOp::Store);
        assert_eq!(attachment.clear_value, Vec4::new(0.2, 0.3, 0.4, 1.0));
    }

    #[test]
    fn test_depth_attachment_creation() {
        let attachment = DepthAttachment {
            target: RenderTarget::Texture {
                width: 1024,
                height: 1024,
            },
            depth_load_op: LoadOp::Clear,
            depth_store_op: StoreOp::Store,
            clear_depth: 0.5,
        };

        assert_eq!(attachment.depth_load_op, LoadOp::Clear);
        assert_eq!(attachment.depth_store_op, StoreOp::Store);
        assert_eq!(attachment.clear_depth, 0.5);
    }

    #[test]
    fn test_render_pass_descriptor() {
        let descriptor = RenderPassDescriptor {
            color_attachments: vec![ColorAttachment {
                target: RenderTarget::Surface,
                load_op: LoadOp::Clear,
                store_op: StoreOp::Store,
                clear_value: Vec4::ONE,
            }],
            depth_attachment: Some(DepthAttachment {
                target: RenderTarget::Surface,
                depth_load_op: LoadOp::Clear,
                depth_store_op: StoreOp::Store,
                clear_depth: 1.0,
            }),
        };

        assert_eq!(descriptor.color_attachments.len(), 1);
        assert!(descriptor.depth_attachment.is_some());
    }
}
