pub trait RenderSurface {
    type SurfaceId;
    
    fn create_surface(&mut self, window: &dyn RawWindow) -> Self::SurfaceId;
    fn configure_surface(&mut self, id: &Self::SurfaceId, config: &SurfaceConfiguration);
    fn present(&mut self, id: &Self::SurfaceId);
}

pub trait RawWindow {
    fn window_handle(&self) -> &dyn WindowHandle;
}

/// Opaque handle for platform-specific window data
/// 
/// Implementation crates must provide concrete window handle types
/// that implement this trait to bridge platform-specific APIs.
pub trait WindowHandle: Send + Sync {
    /// Get an opaque identifier for this window handle
    fn id(&self) -> u64;
    
    /// Get platform name for debugging
    fn platform(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct SurfaceConfiguration {
    pub width: u32,
    pub height: u32,
    pub format: SurfaceFormat,
    pub present_mode: PresentMode,
    pub alpha_mode: AlphaMode,
}

#[derive(Debug, Clone, Copy)]
pub enum SurfaceFormat {
    Rgba8Unorm,
    Bgra8Unorm,
    Rgb10a2Unorm,
}

#[derive(Debug, Clone, Copy)]
pub enum PresentMode {
    Immediate,
    Fifo,
    FifoRelaxed,
    Mailbox,
}

#[derive(Debug, Clone, Copy)]
pub enum AlphaMode {
    Auto,
    Opaque,
    PreMultiplied,
    PostMultiplied,
}