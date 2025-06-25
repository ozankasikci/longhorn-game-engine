pub mod command;
pub mod pipeline;
pub mod resource;
pub mod surface;
pub mod traits;

pub use command::*;
pub use pipeline::*;
pub use resource::*;
pub use surface::*;
pub use traits::*;

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Device creation failed: {0}")]
    DeviceCreation(String),
    #[error("Surface configuration failed: {0}")]
    SurfaceConfiguration(String),
    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),
    #[error("Resource binding failed: {0}")]
    ResourceBinding(String),
}

pub type RendererResult<T> = Result<T, RendererError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_error_display() {
        let errors = [
            RendererError::DeviceCreation("GPU not found".to_string()),
            RendererError::SurfaceConfiguration("invalid format".to_string()),
            RendererError::ShaderCompilation("syntax error at line 42".to_string()),
            RendererError::ResourceBinding("texture not found".to_string()),
        ];

        for error in &errors {
            // Ensure all errors implement Display properly
            let display_string = format!("{}", error);
            assert!(!display_string.is_empty());

            // Ensure all errors implement Debug properly
            let debug_string = format!("{:?}", error);
            assert!(!debug_string.is_empty());
        }
    }

    #[test]
    fn test_renderer_result_type() {
        // Test that our Result type alias works
        let success: RendererResult<i32> = Ok(42);
        let failure: RendererResult<i32> = Err(RendererError::DeviceCreation("test".to_string()));

        match success {
            Ok(value) => assert_eq!(value, 42),
            Err(_) => panic!("Should be success"),
        }

        match failure {
            Ok(_) => panic!("Should be error"),
            Err(error) => match error {
                RendererError::DeviceCreation(msg) => assert_eq!(msg, "test"),
                _ => panic!("Wrong error type"),
            },
        }
    }

    #[test]
    fn test_renderer_error_variants() {
        // Test specific error message formatting
        let device_error =
            RendererError::DeviceCreation("Vulkan initialization failed".to_string());
        assert!(device_error.to_string().contains("Device creation failed"));
        assert!(device_error
            .to_string()
            .contains("Vulkan initialization failed"));

        let surface_error = RendererError::SurfaceConfiguration("invalid swap chain".to_string());
        assert!(surface_error
            .to_string()
            .contains("Surface configuration failed"));
        assert!(surface_error.to_string().contains("invalid swap chain"));

        let shader_error = RendererError::ShaderCompilation("vertex shader error".to_string());
        assert!(shader_error
            .to_string()
            .contains("Shader compilation failed"));
        assert!(shader_error.to_string().contains("vertex shader error"));

        let binding_error = RendererError::ResourceBinding("uniform buffer missing".to_string());
        assert!(binding_error
            .to_string()
            .contains("Resource binding failed"));
        assert!(binding_error.to_string().contains("uniform buffer missing"));
    }
}
