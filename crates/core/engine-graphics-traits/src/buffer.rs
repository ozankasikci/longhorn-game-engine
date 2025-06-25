use crate::Result;

/// Trait for graphics buffers (vertex, index, uniform, etc.)
pub trait GraphicsBuffer: Send + Sync {
    /// Write data to the buffer at the specified offset
    fn write(&self, offset: u64, data: &[u8]) -> Result<()>;

    /// Read data from the buffer (may not be supported by all backends)
    fn read(&self) -> Result<Vec<u8>>;

    /// Get the size of the buffer in bytes
    fn size(&self) -> u64;

    /// Map the buffer for writing (returns a writable slice)
    fn map_write(&self) -> Result<BufferMappedRange>;

    /// Unmap the buffer after mapping
    fn unmap(&self);
}

/// A mapped range of a buffer that can be written to
pub struct BufferMappedRange<'a> {
    data: &'a mut [u8],
}

impl<'a> BufferMappedRange<'a> {
    /// Create a new mapped range
    pub fn new(data: &'a mut [u8]) -> Self {
        Self { data }
    }

    /// Get the mapped data as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GraphicsError;

    // Mock implementation for testing
    struct MockBuffer {
        data: std::sync::Mutex<Vec<u8>>,
        size: u64,
    }

    impl MockBuffer {
        fn new(size: u64) -> Self {
            Self {
                data: std::sync::Mutex::new(vec![0; size as usize]),
                size,
            }
        }
    }

    impl GraphicsBuffer for MockBuffer {
        fn write(&self, offset: u64, data: &[u8]) -> Result<()> {
            let mut buffer = self.data.lock().unwrap();
            if offset + data.len() as u64 > self.size {
                return Err(GraphicsError::InvalidOperation(
                    "Write exceeds buffer bounds".to_string(),
                ));
            }
            buffer[offset as usize..offset as usize + data.len()].copy_from_slice(data);
            Ok(())
        }

        fn read(&self) -> Result<Vec<u8>> {
            Ok(self.data.lock().unwrap().clone())
        }

        fn size(&self) -> u64 {
            self.size
        }

        fn map_write(&self) -> Result<BufferMappedRange> {
            // In a real implementation, this would return a mapped range
            Err(GraphicsError::InvalidOperation(
                "Mapping not supported in mock".to_string(),
            ))
        }

        fn unmap(&self) {
            // No-op for mock
        }
    }

    #[test]
    fn test_buffer_write_and_read() {
        let buffer = MockBuffer::new(64);
        let data = vec![1, 2, 3, 4];

        // Write data
        buffer.write(0, &data).expect("Failed to write");

        // Read back
        let read_data = buffer.read().expect("Failed to read");
        assert_eq!(&read_data[0..4], &data[..]);
    }

    #[test]
    fn test_buffer_write_with_offset() {
        let buffer = MockBuffer::new(64);
        let data = vec![5, 6, 7, 8];

        // Write with offset
        buffer.write(10, &data).expect("Failed to write");

        // Read back and verify
        let read_data = buffer.read().expect("Failed to read");
        assert_eq!(&read_data[10..14], &data[..]);
    }

    #[test]
    fn test_buffer_write_out_of_bounds() {
        let buffer = MockBuffer::new(10);
        let data = vec![1, 2, 3, 4, 5, 6];

        // Try to write beyond buffer size
        let result = buffer.write(8, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_buffer_size() {
        let buffer = MockBuffer::new(128);
        assert_eq!(buffer.size(), 128);
    }
}
