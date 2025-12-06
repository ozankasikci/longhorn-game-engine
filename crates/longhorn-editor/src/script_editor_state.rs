use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ScriptError {
    pub line: usize,
    pub message: String,
}

pub struct ScriptEditorState {
    /// Currently open script path (relative to project)
    pub open_file: Option<PathBuf>,
    /// Full absolute path to the file
    pub full_path: Option<PathBuf>,
    /// Current editor content
    pub content: String,
    /// Content at last save (for dirty detection)
    original_content: String,
    /// Parse errors with line numbers
    pub errors: Vec<ScriptError>,
}

impl ScriptEditorState {
    pub fn new() -> Self {
        Self {
            open_file: None,
            full_path: None,
            content: String::new(),
            original_content: String::new(),
            errors: Vec::new(),
        }
    }

    /// Check if the content has been modified since last save
    pub fn is_dirty(&self) -> bool {
        self.content != self.original_content
    }

    /// Check if a file is currently open
    pub fn is_open(&self) -> bool {
        self.open_file.is_some()
    }

    /// Open a script file for editing
    pub fn open(&mut self, relative_path: PathBuf, project_path: &Path) -> io::Result<()> {
        // Construct the full absolute path
        let full_path = project_path.join(&relative_path);

        // Read the file content
        let content = fs::read_to_string(&full_path)?;

        // Update state
        self.open_file = Some(relative_path);
        self.full_path = Some(full_path);
        self.content = content.clone();
        self.original_content = content;
        self.errors.clear();

        Ok(())
    }

    /// Save the current content to disk
    pub fn save(&mut self) -> io::Result<()> {
        if let Some(full_path) = &self.full_path {
            fs::write(full_path, &self.content)?;
            // Update original_content to match what was saved
            self.original_content = self.content.clone();
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No file is currently open",
            ))
        }
    }

    /// Close the currently open file
    pub fn close(&mut self) {
        self.open_file = None;
        self.full_path = None;
        self.content.clear();
        self.original_content.clear();
        self.errors.clear();
    }

    /// Set parse/validation errors
    pub fn set_errors(&mut self, errors: Vec<ScriptError>) {
        self.errors = errors;
    }

    /// Get just the filename for display purposes
    pub fn filename(&self) -> Option<&str> {
        self.open_file.as_ref().and_then(|path| {
            path.file_name().and_then(|name| name.to_str())
        })
    }
}

impl Default for ScriptEditorState {
    fn default() -> Self {
        Self::new()
    }
}
