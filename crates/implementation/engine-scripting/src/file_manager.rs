//! Script file management with validation and error handling

use crate::{ScriptError, ScriptResult};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

/// Manages script files and provides validation
#[derive(Debug)]
pub struct ScriptFileManager {
    /// Base directory for scripts
    script_directory: PathBuf,
    /// Cache of script file metadata
    file_cache: HashMap<String, ScriptFileInfo>,
}

/// Information about a script file
#[derive(Debug, Clone)]
pub struct ScriptFileInfo {
    /// Full path to the script file
    pub path: PathBuf,
    /// Whether the file exists and is valid
    pub exists: bool,
    /// File size in bytes
    pub size: u64,
    /// Last modification time
    pub modified: Option<std::time::SystemTime>,
    /// Validation error if any
    pub validation_error: Option<String>,
}

/// Script file validation result
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptValidation {
    /// File is valid and ready to use
    Valid,
    /// File doesn't exist
    NotFound,
    /// File exists but has syntax errors
    SyntaxError(String),
    /// File exists but is empty
    Empty,
    /// File has invalid extension
    InvalidExtension,
    /// File is too large
    TooLarge(u64),
}

impl ScriptFileManager {
    /// Create a new script file manager
    pub fn new(script_directory: PathBuf) -> ScriptResult<Self> {
        // Create script directory if it doesn't exist
        if !script_directory.exists() {
            fs::create_dir_all(&script_directory)
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to create script directory: {}", e)))?;
        }

        Ok(Self {
            script_directory,
            file_cache: HashMap::new(),
        })
    }

    /// Get the base script directory
    pub fn script_directory(&self) -> &Path {
        &self.script_directory
    }

    /// Validate a script file by relative path
    pub fn validate_script(&mut self, relative_path: &str) -> ScriptValidation {
        let full_path = self.script_directory.join(relative_path);
        self.validate_script_at_path(&full_path)
    }

    /// Validate a script file at an absolute path
    pub fn validate_script_at_path(&mut self, path: &Path) -> ScriptValidation {
        // Check extension
        if let Some(extension) = path.extension() {
            if extension != "lua" {
                return ScriptValidation::InvalidExtension;
            }
        } else {
            return ScriptValidation::InvalidExtension;
        }

        // Check if file exists
        if !path.exists() {
            return ScriptValidation::NotFound;
        }

        // Check file metadata
        let metadata = match fs::metadata(path) {
            Ok(meta) => meta,
            Err(_) => return ScriptValidation::NotFound,
        };

        // Check file size (limit to 1MB)
        const MAX_FILE_SIZE: u64 = 1024 * 1024;
        if metadata.len() > MAX_FILE_SIZE {
            return ScriptValidation::TooLarge(metadata.len());
        }

        // Check if file is empty
        if metadata.len() == 0 {
            return ScriptValidation::Empty;
        }

        // Read and validate syntax
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => return ScriptValidation::SyntaxError(format!("Failed to read file: {}", e)),
        };

        // Basic Lua syntax validation using mlua
        if let Err(e) = mlua::Lua::new().load(&content).exec() {
            return ScriptValidation::SyntaxError(format!("Lua syntax error: {}", e));
        }

        ScriptValidation::Valid
    }

    /// Get script file information
    pub fn get_file_info(&mut self, relative_path: &str) -> ScriptFileInfo {
        let full_path = self.script_directory.join(relative_path);
        
        // Check cache first
        if let Some(cached_info) = self.file_cache.get(relative_path) {
            // Check if file modification time has changed
            if let Ok(metadata) = fs::metadata(&full_path) {
                if let Ok(modified) = metadata.modified() {
                    if cached_info.modified == Some(modified) {
                        return cached_info.clone();
                    }
                }
            }
        }

        // Create fresh file info
        let info = self.create_file_info(&full_path);
        self.file_cache.insert(relative_path.to_string(), info.clone());
        info
    }

    /// Create a new script file with template content
    pub fn create_script_file(&self, relative_path: &str, template_name: Option<&str>) -> ScriptResult<PathBuf> {
        let full_path = self.script_directory.join(relative_path);
        
        // Check if file already exists
        if full_path.exists() {
            return Err(ScriptError::RuntimeError(format!("Script file already exists: {}", relative_path)));
        }

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to create directory: {}", e)))?;
        }

        // Get template content
        let content = match template_name {
            Some("entity") => include_str!("../lua/examples/entity_template.lua"),
            Some("game_manager") => include_str!("../lua/examples/game_manager.lua"),
            Some("player_controller") => include_str!("../lua/examples/player_controller.lua"),
            _ => include_str!("../lua/examples/basic_template.lua"),
        };

        // Write file
        fs::write(&full_path, content)
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to write script file: {}", e)))?;

        Ok(full_path)
    }

    /// List all script files in the directory
    pub fn list_script_files(&self) -> ScriptResult<Vec<String>> {
        let mut files = Vec::new();
        self.scan_directory(&self.script_directory, "", &mut files)?;
        Ok(files)
    }

    /// Clear the file cache
    pub fn clear_cache(&mut self) {
        self.file_cache.clear();
    }

    fn create_file_info(&self, path: &Path) -> ScriptFileInfo {
        let exists = path.exists();
        let mut size = 0;
        let mut modified = None;
        let mut validation_error = None;

        if exists {
            if let Ok(metadata) = fs::metadata(path) {
                size = metadata.len();
                modified = metadata.modified().ok();
            }

            // Validate the file
            let validation = self.validate_file_content(path);
            if validation != ScriptValidation::Valid {
                validation_error = Some(format!("{:?}", validation));
            }
        }

        ScriptFileInfo {
            path: path.to_path_buf(),
            exists,
            size,
            modified,
            validation_error,
        }
    }

    fn validate_file_content(&self, path: &Path) -> ScriptValidation {
        // This is a simplified version that doesn't modify self
        if !path.exists() {
            return ScriptValidation::NotFound;
        }

        if let Some(extension) = path.extension() {
            if extension != "lua" {
                return ScriptValidation::InvalidExtension;
            }
        } else {
            return ScriptValidation::InvalidExtension;
        }

        let metadata = match fs::metadata(path) {
            Ok(meta) => meta,
            Err(_) => return ScriptValidation::NotFound,
        };

        const MAX_FILE_SIZE: u64 = 1024 * 1024;
        if metadata.len() > MAX_FILE_SIZE {
            return ScriptValidation::TooLarge(metadata.len());
        }

        if metadata.len() == 0 {
            return ScriptValidation::Empty;
        }

        ScriptValidation::Valid
    }

    fn scan_directory(&self, dir: &Path, prefix: &str, files: &mut Vec<String>) -> ScriptResult<()> {
        let entries = fs::read_dir(dir)
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| ScriptError::RuntimeError(format!("Failed to read directory entry: {}", e)))?;
            
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            
            if path.is_dir() {
                // Recursively scan subdirectories
                let new_prefix = if prefix.is_empty() {
                    name
                } else {
                    format!("{}/{}", prefix, name)
                };
                self.scan_directory(&path, &new_prefix, files)?;
            } else if path.extension().map_or(false, |ext| ext == "lua") {
                // Add Lua files
                let relative_path = if prefix.is_empty() {
                    name
                } else {
                    format!("{}/{}", prefix, name)
                };
                files.push(relative_path);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_manager() -> (ScriptFileManager, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let manager = ScriptFileManager::new(temp_dir.path().to_path_buf())
            .expect("Failed to create script file manager");
        (manager, temp_dir)
    }

    #[test]
    fn test_script_file_manager_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let manager = ScriptFileManager::new(temp_dir.path().to_path_buf());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_validate_nonexistent_script() {
        let (mut manager, _temp_dir) = create_test_manager();
        let result = manager.validate_script("nonexistent.lua");
        assert_eq!(result, ScriptValidation::NotFound);
    }

    #[test]
    fn test_validate_invalid_extension() {
        let (mut manager, _temp_dir) = create_test_manager();
        let result = manager.validate_script("test.txt");
        assert_eq!(result, ScriptValidation::InvalidExtension);
    }

    #[test]
    fn test_validate_empty_script() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Create empty file
        let script_path = temp_dir.path().join("empty.lua");
        File::create(&script_path).expect("Failed to create file");
        
        let result = manager.validate_script("empty.lua");
        assert_eq!(result, ScriptValidation::Empty);
    }

    #[test]
    fn test_validate_valid_script() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Create valid Lua script
        let script_path = temp_dir.path().join("valid.lua");
        let mut file = File::create(&script_path).expect("Failed to create file");
        writeln!(file, "function update()\n  print('Hello, World!')\nend").expect("Failed to write file");
        
        let result = manager.validate_script("valid.lua");
        assert_eq!(result, ScriptValidation::Valid);
    }

    #[test]
    fn test_validate_syntax_error() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Create script with syntax error
        let script_path = temp_dir.path().join("invalid.lua");
        let mut file = File::create(&script_path).expect("Failed to create file");
        writeln!(file, "function update(\n  -- missing closing parenthesis").expect("Failed to write file");
        
        let result = manager.validate_script("invalid.lua");
        match result {
            ScriptValidation::SyntaxError(_) => (),
            _ => panic!("Expected syntax error, got: {:?}", result),
        }
    }

    #[test]
    fn test_get_file_info_existing() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Create test file
        let script_path = temp_dir.path().join("test.lua");
        let mut file = File::create(&script_path).expect("Failed to create file");
        writeln!(file, "print('test')").expect("Failed to write file");
        
        let info = manager.get_file_info("test.lua");
        assert!(info.exists);
        assert!(info.size > 0);
        assert!(info.modified.is_some());
        assert!(info.validation_error.is_none());
    }

    #[test]
    fn test_get_file_info_nonexistent() {
        let (mut manager, _temp_dir) = create_test_manager();
        
        let info = manager.get_file_info("nonexistent.lua");
        assert!(!info.exists);
        assert_eq!(info.size, 0);
        assert!(info.modified.is_none());
    }

    #[test]
    fn test_create_script_file() {
        let (manager, temp_dir) = create_test_manager();
        
        let result = manager.create_script_file("new_script.lua", Some("entity"));
        assert!(result.is_ok());
        
        let created_path = result.unwrap();
        assert!(created_path.exists());
        
        let content = fs::read_to_string(&created_path).expect("Failed to read created file");
        assert!(content.contains("function")); // Should contain template content
    }

    #[test]
    fn test_create_script_file_already_exists() {
        let (manager, temp_dir) = create_test_manager();
        
        // Create file first
        let script_path = temp_dir.path().join("existing.lua");
        File::create(&script_path).expect("Failed to create file");
        
        let result = manager.create_script_file("existing.lua", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_script_files() {
        let (manager, temp_dir) = create_test_manager();
        
        // Create some test files
        File::create(temp_dir.path().join("script1.lua")).expect("Failed to create file");
        File::create(temp_dir.path().join("script2.lua")).expect("Failed to create file");
        File::create(temp_dir.path().join("not_a_script.txt")).expect("Failed to create file");
        
        // Create subdirectory with script
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).expect("Failed to create subdirectory");
        File::create(sub_dir.join("script3.lua")).expect("Failed to create file");
        
        let files = manager.list_script_files().expect("Failed to list files");
        
        assert_eq!(files.len(), 3);
        assert!(files.contains(&"script1.lua".to_string()));
        assert!(files.contains(&"script2.lua".to_string()));
        assert!(files.contains(&"subdir/script3.lua".to_string()));
        assert!(!files.contains(&"not_a_script.txt".to_string()));
    }

    #[test]
    fn test_file_cache() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Create test file
        let script_path = temp_dir.path().join("cached.lua");
        let mut file = File::create(&script_path).expect("Failed to create file");
        writeln!(file, "print('cached')").expect("Failed to write file");
        
        // First call should cache the info
        let info1 = manager.get_file_info("cached.lua");
        
        // Second call should return cached info
        let info2 = manager.get_file_info("cached.lua");
        
        assert_eq!(info1.size, info2.size);
        assert_eq!(info1.modified, info2.modified);
    }

    #[test]
    fn test_clear_cache() {
        let (mut manager, temp_dir) = create_test_manager();
        
        // Create test file
        let script_path = temp_dir.path().join("cached.lua");
        let mut file = File::create(&script_path).expect("Failed to create file");
        writeln!(file, "print('cached')").expect("Failed to write file");
        
        // Cache the info
        manager.get_file_info("cached.lua");
        assert!(!manager.file_cache.is_empty());
        
        // Clear cache
        manager.clear_cache();
        assert!(manager.file_cache.is_empty());
    }
}