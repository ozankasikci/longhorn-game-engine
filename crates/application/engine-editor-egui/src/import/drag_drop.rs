use std::path::{Path, PathBuf};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Mesh,
    Texture,
    Audio,
    Unknown,
}

pub struct DragDropHandler {
    pending_imports: VecDeque<PathBuf>,
}

impl DragDropHandler {
    pub fn new() -> Self {
        Self {
            pending_imports: VecDeque::new(),
        }
    }
    
    pub fn handle_drop(&mut self, files: Vec<PathBuf>) {
        for file in files {
            if self.is_supported_file(&file) {
                self.pending_imports.push_back(file);
            }
        }
    }
    
    pub fn has_pending_imports(&self) -> bool {
        !self.pending_imports.is_empty()
    }
    
    pub fn get_pending_imports(&mut self) -> Vec<PathBuf> {
        self.pending_imports.drain(..).collect()
    }
    
    pub fn detect_file_type(&self, path: &Path) -> Option<FileType> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        
        match ext.as_str() {
            "obj" | "fbx" | "gltf" | "glb" | "dae" | "3ds" => Some(FileType::Mesh),
            "png" | "jpg" | "jpeg" | "tga" | "bmp" | "dds" => Some(FileType::Texture),
            "wav" | "mp3" | "ogg" | "flac" => Some(FileType::Audio),
            _ => Some(FileType::Unknown),
        }
    }
    
    fn is_supported_file(&self, path: &Path) -> bool {
        matches!(self.detect_file_type(path), Some(t) if t != FileType::Unknown)
    }
}