//! Platform abstraction layer for the mobile game engine
//! 
//! This crate provides platform-specific functionality and abstractions
//! for different operating systems and devices.

pub mod window;
pub mod filesystem;
pub mod threading;
pub mod time;
pub mod system;
pub mod mobile;

pub use window::{Window, WindowBuilder, WindowEvent};
pub use filesystem::{FileSystem, Path};
pub use time::{Timer, Clock};
pub use system::{SystemInfo, Platform};

/// Platform system errors
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),
    #[error("Window creation failed: {0}")]
    WindowError(String),
    #[error("File system error: {0}")]
    FileSystemError(String),
    #[error("Platform API error: {0}")]
    ApiError(String),
}

/// Platform system result type
pub type PlatformResult<T> = Result<T, PlatformError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        // Placeholder test
        assert!(true);
    }
}