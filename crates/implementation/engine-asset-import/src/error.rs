use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Processing error: {0}")]
    ProcessingError(String),
    
    #[error("Invalid asset: {0}")]
    InvalidAsset(String),
    
    #[error("Import cancelled")]
    Cancelled,
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}