use engine_graphics_traits::GraphicsError;
use thiserror::Error;

/// WGPU-specific errors
#[derive(Error, Debug)]
pub enum WgpuError {
    /// Request adapter failed
    #[error("Failed to request adapter")]
    AdapterRequest,
    
    /// Request device failed
    #[error("Failed to request device: {0}")]
    DeviceRequest(#[from] wgpu::RequestDeviceError),
    
    /// Buffer async error
    #[error("Buffer async error")]
    BufferAsync(#[from] wgpu::BufferAsyncError),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Convert WGPU errors to graphics errors
impl From<WgpuError> for GraphicsError {
    fn from(err: WgpuError) -> Self {
        match err {
            WgpuError::AdapterRequest => GraphicsError::DeviceCreation("No adapter found".to_string()),
            WgpuError::DeviceRequest(e) => GraphicsError::DeviceCreation(e.to_string()),
            WgpuError::BufferAsync(e) => GraphicsError::BackendError(e.to_string()),
            WgpuError::Validation(msg) => GraphicsError::InvalidOperation(msg),
        }
    }
}