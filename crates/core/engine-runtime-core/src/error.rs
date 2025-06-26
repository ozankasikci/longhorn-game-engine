use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Window creation failed: {0}")]
    WindowCreation(String),
    
    #[error("Event loop error: {0}")]
    EventLoop(String),
    
    #[error("Timing error: {0}")]
    Timing(String),
    
    #[error("Application error: {0}")]
    Application(String),
}