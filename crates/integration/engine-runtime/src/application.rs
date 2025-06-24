//! Application trait and builder

use crate::RuntimeResult;

/// Application trait for game implementations
pub trait Application {
    /// Initialize the application
    fn initialize(&mut self) -> RuntimeResult<()>;

    /// Update the application logic
    fn update(&mut self, delta_time: f32) -> RuntimeResult<()>;

    /// Render the application
    fn render(&mut self) -> RuntimeResult<()>;

    /// Handle application events
    fn handle_event(&mut self, event: ApplicationEvent) -> RuntimeResult<()>;

    /// Check if the application should exit
    fn should_exit(&self) -> bool;
}

/// Application events
#[derive(Debug, Clone)]
pub enum ApplicationEvent {
    WindowResize { width: u32, height: u32 },
    WindowClose,
    Suspend,
    Resume,
}

/// Application builder for creating applications
pub struct ApplicationBuilder {
    title: String,
    width: u32,
    height: u32,
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationBuilder {
    /// Create a new application builder
    pub fn new() -> Self {
        Self {
            title: "Game Application".to_string(),
            width: 800,
            height: 600,
        }
    }

    /// Set application title
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set application window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Build a basic application
    pub fn build(self) -> BasicApplication {
        BasicApplication {
            title: self.title,
            width: self.width,
            height: self.height,
            should_exit: false,
        }
    }
}

/// Basic application implementation
pub struct BasicApplication {
    title: String,
    width: u32,
    height: u32,
    should_exit: bool,
}

impl Application for BasicApplication {
    fn initialize(&mut self) -> RuntimeResult<()> {
        log::info!("Initializing application: {}", self.title);
        Ok(())
    }

    fn update(&mut self, _delta_time: f32) -> RuntimeResult<()> {
        // Basic update implementation
        Ok(())
    }

    fn render(&mut self) -> RuntimeResult<()> {
        // Basic render implementation
        Ok(())
    }

    fn handle_event(&mut self, event: ApplicationEvent) -> RuntimeResult<()> {
        match event {
            ApplicationEvent::WindowClose => {
                self.should_exit = true;
            }
            ApplicationEvent::WindowResize { width, height } => {
                self.width = width;
                self.height = height;
                log::info!("Window resized to {}x{}", width, height);
            }
            _ => {}
        }
        Ok(())
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }
}
