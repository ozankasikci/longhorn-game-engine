use engine_graphics_traits::{GraphicsBuffer, BufferMappedRange, Result};
use std::sync::Arc;

/// WGPU buffer implementation
pub struct WgpuBuffer {
    buffer: Arc<wgpu::Buffer>,
    size: u64,
}

impl WgpuBuffer {
    /// Create a new WGPU buffer
    pub fn new(buffer: wgpu::Buffer) -> Self {
        let size = buffer.size();
        Self {
            buffer: Arc::new(buffer),
            size,
        }
    }
    
    /// Get the underlying WGPU buffer
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

impl GraphicsBuffer for WgpuBuffer {
    fn write(&self, _offset: u64, _data: &[u8]) -> Result<()> {
        // WGPU doesn't have direct write, this would use queue.write_buffer
        // For now, return an error indicating this needs a queue reference
        Err(engine_graphics_traits::GraphicsError::InvalidOperation(
            "Direct buffer write not supported - use queue.write_buffer".to_string()
        ))
    }
    
    fn read(&self) -> Result<Vec<u8>> {
        // Buffer reading requires mapping which is async in WGPU
        Err(engine_graphics_traits::GraphicsError::InvalidOperation(
            "Sync buffer read not supported - use async mapping".to_string()
        ))
    }
    
    fn size(&self) -> u64 {
        self.size
    }
    
    fn map_write(&self) -> Result<BufferMappedRange> {
        // Mapping is async in WGPU
        Err(engine_graphics_traits::GraphicsError::InvalidOperation(
            "Sync buffer mapping not supported - use async mapping".to_string()
        ))
    }
    
    fn unmap(&self) {
        self.buffer.unmap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_graphics_traits::{BufferDescriptor, BufferUsage};
    
    #[allow(unused_imports)]
    use super::*;
    
    // Helper to create test device and queue
    async fn create_test_device() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to request adapter");
        
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Test Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to request device")
    }
    
    #[test]
    fn test_buffer_creation() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            
            let wgpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Test Buffer"),
                size: 1024,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            
            let buffer = WgpuBuffer::new(wgpu_buffer);
            assert_eq!(buffer.size(), 1024);
        });
    }
    
    #[test]
    fn test_buffer_size() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            
            let sizes = vec![256, 512, 1024, 2048];
            for size in sizes {
                let wgpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Test Buffer"),
                    size,
                    usage: wgpu::BufferUsages::UNIFORM,
                    mapped_at_creation: false,
                });
                
                let buffer = WgpuBuffer::new(wgpu_buffer);
                assert_eq!(buffer.size(), size);
            }
        });
    }
    
    #[test]
    fn test_buffer_write_returns_error() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            
            let wgpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Test Buffer"),
                size: 1024,
                usage: wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            
            let buffer = WgpuBuffer::new(wgpu_buffer);
            let data = vec![1, 2, 3, 4];
            
            // Direct write should return error
            let result = buffer.write(0, &data);
            assert!(result.is_err());
        });
    }
    
    #[test]
    fn test_buffer_read_returns_error() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            
            let wgpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Test Buffer"),
                size: 1024,
                usage: wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            
            let buffer = WgpuBuffer::new(wgpu_buffer);
            
            // Sync read should return error
            let result = buffer.read();
            assert!(result.is_err());
        });
    }
    
    #[test]
    fn test_buffer_raw_access() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            
            let wgpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Test Buffer"),
                size: 512,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            
            let buffer = WgpuBuffer::new(wgpu_buffer);
            let raw = buffer.raw();
            assert_eq!(raw.size(), 512);
        });
    }
}