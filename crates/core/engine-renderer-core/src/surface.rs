pub trait RenderSurface {
    type SurfaceId;
    
    fn create_surface(&mut self, window: &dyn RawWindow) -> Self::SurfaceId;
    fn configure_surface(&mut self, id: &Self::SurfaceId, config: &SurfaceConfiguration);
    fn present(&mut self, id: &Self::SurfaceId);
}

pub trait RawWindow {
    fn raw_handle(&self) -> RawWindowHandle;
}

#[derive(Debug)]
pub enum RawWindowHandle {
    Win32 { hwnd: *mut std::ffi::c_void },
    Cocoa { ns_window: *mut std::ffi::c_void },
    X11 { window: u64, display: *mut std::ffi::c_void },
    Wayland { surface: *mut std::ffi::c_void },
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