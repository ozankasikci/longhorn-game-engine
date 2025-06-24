use std::path::{Path, PathBuf};
use std::fs;
use engine_editor_assets::ProjectAsset;

#[derive(Debug)]
pub enum FolderOperationError {
    InvalidPath(String),
    AlreadyExists(String),
    IoError(std::io::Error),
    InvalidName(String),
}

impl std::fmt::Display for FolderOperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FolderOperationError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            FolderOperationError::AlreadyExists(name) => write!(f, "Folder already exists: {}", name),
            FolderOperationError::IoError(err) => write!(f, "IO error: {}", err),
            FolderOperationError::InvalidName(name) => write!(f, "Invalid folder name: {}", name),
        }
    }
}

impl From<std::io::Error> for FolderOperationError {
    fn from(err: std::io::Error) -> Self {
        FolderOperationError::IoError(err)
    }
}

pub struct FolderManager {
    pub project_root: PathBuf,
}

impl FolderManager {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    /// Create a new folder at the specified path
    pub fn create_folder(&self, parent_path: &Path, folder_name: &str) -> Result<(), FolderOperationError> {
        // Validate folder name
        if folder_name.is_empty() {
            return Err(FolderOperationError::InvalidName("Folder name cannot be empty".to_string()));
        }
        
        if folder_name.contains(|c: char| {
            c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|'
        }) {
            return Err(FolderOperationError::InvalidName("Folder name contains invalid characters".to_string()));
        }

        let full_path = self.project_root.join(parent_path).join(folder_name);
        
        if full_path.exists() {
            return Err(FolderOperationError::AlreadyExists(folder_name.to_string()));
        }

        fs::create_dir_all(&full_path)?;
        Ok(())
    }

    /// Delete a folder and all its contents
    pub fn delete_folder(&self, folder_path: &Path) -> Result<(), FolderOperationError> {
        let full_path = self.project_root.join(folder_path);
        
        if !full_path.exists() {
            return Err(FolderOperationError::InvalidPath(folder_path.display().to_string()));
        }

        fs::remove_dir_all(&full_path)?;
        Ok(())
    }

    /// Rename a folder
    pub fn rename_folder(&self, old_path: &Path, new_name: &str) -> Result<(), FolderOperationError> {
        // Validate new name
        if new_name.is_empty() {
            return Err(FolderOperationError::InvalidName("Folder name cannot be empty".to_string()));
        }
        
        if new_name.contains(|c: char| {
            c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|'
        }) {
            return Err(FolderOperationError::InvalidName("Folder name contains invalid characters".to_string()));
        }

        let full_old_path = self.project_root.join(old_path);
        
        if !full_old_path.exists() {
            return Err(FolderOperationError::InvalidPath(old_path.display().to_string()));
        }

        let parent = full_old_path.parent()
            .ok_or_else(|| FolderOperationError::InvalidPath("Cannot rename root folder".to_string()))?;
        
        let new_path = parent.join(new_name);
        
        if new_path.exists() {
            return Err(FolderOperationError::AlreadyExists(new_name.to_string()));
        }

        fs::rename(&full_old_path, &new_path)?;
        Ok(())
    }

    /// Move a folder to a new location
    pub fn move_folder(&self, source_path: &Path, target_parent: &Path) -> Result<(), FolderOperationError> {
        let full_source = self.project_root.join(source_path);
        let full_target_parent = self.project_root.join(target_parent);
        
        if !full_source.exists() {
            return Err(FolderOperationError::InvalidPath(source_path.display().to_string()));
        }
        
        if !full_target_parent.exists() {
            return Err(FolderOperationError::InvalidPath(target_parent.display().to_string()));
        }

        let folder_name = full_source.file_name()
            .ok_or_else(|| FolderOperationError::InvalidPath("Cannot get folder name".to_string()))?;
        
        let target_path = full_target_parent.join(folder_name);
        
        if target_path.exists() {
            return Err(FolderOperationError::AlreadyExists(
                folder_name.to_string_lossy().to_string()
            ));
        }

        // Check if trying to move folder into itself
        if target_path.starts_with(&full_source) {
            return Err(FolderOperationError::InvalidPath("Cannot move folder into itself".to_string()));
        }

        fs::rename(&full_source, &target_path)?;
        Ok(())
    }
    
    /// Move a file to a new location
    pub fn move_file(&self, source_path: &Path, target_folder: &Path) -> Result<(), FolderOperationError> {
        let full_source = self.project_root.join(source_path);
        let full_target_folder = self.project_root.join(target_folder);
        
        if !full_source.exists() {
            return Err(FolderOperationError::InvalidPath(source_path.display().to_string()));
        }
        
        if !full_target_folder.exists() || !full_target_folder.is_dir() {
            return Err(FolderOperationError::InvalidPath("Target must be a directory".to_string()));
        }

        let file_name = full_source.file_name()
            .ok_or_else(|| FolderOperationError::InvalidPath("Cannot get file name".to_string()))?;
        
        let target_path = full_target_folder.join(file_name);
        
        if target_path.exists() {
            return Err(FolderOperationError::AlreadyExists(
                file_name.to_string_lossy().to_string()
            ));
        }

        fs::rename(&full_source, &target_path)?;
        Ok(())
    }

    /// Load the file system structure into ProjectAsset tree
    pub fn load_project_structure(&self, path: &Path) -> Result<Vec<ProjectAsset>, FolderOperationError> {
        let full_path = self.project_root.join(path);
        self.load_directory(&full_path)
    }

    fn load_directory(&self, dir_path: &Path) -> Result<Vec<ProjectAsset>, FolderOperationError> {
        let mut assets = Vec::new();
        
        let entries = fs::read_dir(dir_path)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            // Skip hidden files and folders
            if file_name.starts_with('.') {
                continue;
            }
            
            if path.is_dir() {
                let children = self.load_directory(&path)?;
                assets.push(ProjectAsset::folder(&file_name, children));
            } else {
                assets.push(ProjectAsset::file(&file_name));
            }
        }
        
        // Sort folders first, then files
        assets.sort_by(|a, b| {
            match (&a.children, &b.children) {
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        Ok(assets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_folder() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FolderManager::new(temp_dir.path());
        
        // Create a folder
        manager.create_folder(Path::new(""), "test_folder").unwrap();
        assert!(temp_dir.path().join("test_folder").exists());
        
        // Try to create the same folder again
        let result = manager.create_folder(Path::new(""), "test_folder");
        assert!(matches!(result, Err(FolderOperationError::AlreadyExists(_))));
    }

    #[test]
    fn test_invalid_folder_names() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FolderManager::new(temp_dir.path());
        
        let invalid_names = vec!["", "folder/name", "folder\\name", "folder:name", "folder*name"];
        
        for name in invalid_names {
            let result = manager.create_folder(Path::new(""), name);
            assert!(matches!(result, Err(FolderOperationError::InvalidName(_))));
        }
    }

    #[test]
    fn test_rename_folder() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FolderManager::new(temp_dir.path());
        
        // Create a folder
        manager.create_folder(Path::new(""), "old_name").unwrap();
        
        // Rename it
        manager.rename_folder(Path::new("old_name"), "new_name").unwrap();
        
        assert!(!temp_dir.path().join("old_name").exists());
        assert!(temp_dir.path().join("new_name").exists());
    }
    
    #[test]
    fn test_move_folder() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FolderManager::new(temp_dir.path());
        
        // Create folders
        manager.create_folder(Path::new(""), "source").unwrap();
        manager.create_folder(Path::new(""), "target").unwrap();
        
        // Move source into target
        manager.move_folder(Path::new("source"), Path::new("target")).unwrap();
        
        assert!(!temp_dir.path().join("source").exists());
        assert!(temp_dir.path().join("target").join("source").exists());
    }
    
    #[test]
    fn test_move_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FolderManager::new(temp_dir.path());
        
        // Create a target folder
        manager.create_folder(Path::new(""), "target").unwrap();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();
        
        // Move file into target
        manager.move_file(Path::new("test.txt"), Path::new("target")).unwrap();
        
        assert!(!test_file.exists());
        assert!(temp_dir.path().join("target").join("test.txt").exists());
    }
    
    #[test]
    fn test_cannot_move_folder_into_itself() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FolderManager::new(temp_dir.path());
        
        // Create nested folders
        manager.create_folder(Path::new(""), "parent").unwrap();
        manager.create_folder(Path::new("parent"), "child").unwrap();
        
        // Try to move parent into child
        let result = manager.move_folder(Path::new("parent"), Path::new("parent/child"));
        assert!(matches!(result, Err(FolderOperationError::InvalidPath(_))));
    }
}