use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Instant, SystemTime};
use serde::{Deserialize, Serialize};

/// File type categorization for project panel display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Script,
    Text,
    Image,
    Audio,
    Scene,
    Unknown,
}

impl FileType {
    /// Determine file type from filename, checking compound extensions first
    pub fn from_filename(filename: &str, ext: Option<&str>) -> Self {
        let filename_lower = filename.to_lowercase();

        // Check compound extensions first
        if filename_lower.ends_with(".scn.ron") || filename_lower.ends_with(".scn.json") {
            return FileType::Scene;
        }

        // Fall back to simple extension matching
        Self::from_extension(ext)
    }

    pub fn from_extension(ext: Option<&str>) -> Self {
        match ext {
            Some("ts") | Some("js") => FileType::Script,
            Some("json") | Some("txt") | Some("md") | Some("html") | Some("css") | Some("toml") | Some("yaml") | Some("yml") => FileType::Text,
            Some("png") | Some("jpg") | Some("jpeg") | Some("webp") | Some("gif") | Some("bmp") => FileType::Image,
            Some("wav") | Some("mp3") | Some("ogg") | Some("flac") => FileType::Audio,
            Some("scene") => FileType::Scene,
            _ => FileType::Unknown,
        }
    }

    /// Returns true if this file type should be opened in the script editor
    pub fn is_text_editable(&self) -> bool {
        matches!(self, FileType::Script | FileType::Text | FileType::Scene)
    }

    /// Get the color for this file type
    pub fn icon_color(&self) -> [u8; 3] {
        match self {
            FileType::Script => [100, 150, 255],   // Blue
            FileType::Text => [150, 150, 150],     // Gray
            FileType::Image => [100, 200, 100],    // Green
            FileType::Audio => [200, 100, 200],    // Purple
            FileType::Scene => [255, 150, 50],     // Orange
            FileType::Unknown => [128, 128, 128],  // Dark gray
        }
    }

    /// Get the icon character for this file type
    pub fn icon_char(&self) -> &'static str {
        match self {
            FileType::Script => "📜",
            FileType::Text => "📄",
            FileType::Image => "🖼",
            FileType::Audio => "🎵",
            FileType::Scene => "🎬",
            FileType::Unknown => "📦",
        }
    }
}

/// Default tree panel width
pub const DEFAULT_TREE_WIDTH: f32 = 180.0;
/// Minimum tree panel width
pub const MIN_TREE_WIDTH: f32 = 100.0;
/// Maximum tree panel width
pub const MAX_TREE_WIDTH: f32 = 400.0;

/// Double-click threshold in milliseconds
const DOUBLE_CLICK_MS: u128 = 400;

/// State for the project panel
pub struct ProjectPanelState {
    /// Currently selected folder (shown in grid view)
    pub selected_folder: PathBuf,
    /// Current folder being viewed in content pane
    pub current_folder: PathBuf,
    /// Expanded folders in tree view
    pub expanded_folders: HashSet<PathBuf>,
    /// Currently selected file
    pub selected_file: Option<PathBuf>,
    /// Active rename operation
    pub renaming: Option<PathBuf>,
    /// Width of the tree panel (user-resizable)
    pub tree_width: f32,
    /// Last click time and path for manual double-click detection
    last_click: Option<(Instant, PathBuf)>,
    /// Drop target folder for external file drag-drop
    pub drop_target: Option<PathBuf>,
}

impl std::fmt::Debug for ProjectPanelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectPanelState")
            .field("selected_folder", &self.selected_folder)
            .field("current_folder", &self.current_folder)
            .field("expanded_folders", &self.expanded_folders)
            .field("selected_file", &self.selected_file)
            .field("renaming", &self.renaming)
            .field("tree_width", &self.tree_width)
            .field("drop_target", &self.drop_target)
            .finish()
    }
}

impl ProjectPanelState {
    pub fn new() -> Self {
        Self {
            selected_folder: PathBuf::new(),
            current_folder: PathBuf::new(),
            expanded_folders: HashSet::new(),
            selected_file: None,
            renaming: None,
            tree_width: DEFAULT_TREE_WIDTH,
            last_click: None,
            drop_target: None,
        }
    }

    /// Navigate to a folder (for breadcrumbs)
    pub fn navigate_to(&mut self, folder: PathBuf) {
        self.current_folder = folder;
    }

    /// Save the panel state to a file
    pub fn save_to_file(&self, project_root: &std::path::Path) -> std::io::Result<()> {
        let state_file = project_root.join(".longhorn").join("project_panel_state.json");

        // Create .longhorn directory if it doesn't exist
        if let Some(parent) = state_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize only the persistent parts
        let persistent_state = PersistentPanelState {
            expanded_folders: self.expanded_folders.iter().cloned().collect(),
            selected_folder: self.selected_folder.clone(),
            tree_width: self.tree_width,
        };

        let json = serde_json::to_string_pretty(&persistent_state)?;
        std::fs::write(state_file, json)?;
        Ok(())
    }

    /// Load the panel state from a file
    pub fn load_from_file(&mut self, project_root: &std::path::Path) -> std::io::Result<()> {
        let state_file = project_root.join(".longhorn").join("project_panel_state.json");

        if !state_file.exists() {
            return Ok(()); // No saved state, use defaults
        }

        let json = std::fs::read_to_string(state_file)?;
        let persistent_state: PersistentPanelState = serde_json::from_str(&json)?;

        self.expanded_folders = persistent_state.expanded_folders.into_iter().collect();
        self.selected_folder = persistent_state.selected_folder;
        self.tree_width = persistent_state.tree_width;

        Ok(())
    }

    /// Check if this click is a double-click on the same file.
    /// Returns true if it's a double-click, and updates the last click time.
    pub fn check_double_click(&mut self, path: &PathBuf) -> bool {
        let now = Instant::now();
        let is_double_click = if let Some((last_time, last_path)) = &self.last_click {
            last_path == path && now.duration_since(*last_time).as_millis() < DOUBLE_CLICK_MS
        } else {
            false
        };

        if is_double_click {
            // Reset after double-click so triple-click doesn't trigger again
            self.last_click = None;
        } else {
            // Record this click
            self.last_click = Some((now, path.clone()));
        }

        is_double_click
    }
}

impl Default for ProjectPanelState {
    fn default() -> Self {
        Self::new()
    }
}

/// A file entry in the project panel
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub file_type: FileType,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
}

impl FileEntry {
    pub fn new(path: PathBuf, name: String) -> Self {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        // Check for compound extensions like .scn.ron
        let file_type = FileType::from_filename(&name, extension.as_deref());

        // Get file metadata
        let metadata = std::fs::metadata(&path).ok();
        let size = metadata.as_ref().and_then(|m| Some(m.len()));
        let modified = metadata.as_ref().and_then(|m| m.modified().ok());

        Self {
            path,
            name,
            extension,
            file_type,
            size,
            modified,
        }
    }

    /// Format file size for display
    pub fn format_size(&self) -> String {
        match self.size {
            Some(bytes) => {
                if bytes < 1024 {
                    format!("{} B", bytes)
                } else if bytes < 1024 * 1024 {
                    format!("{:.1} KB", bytes as f64 / 1024.0)
                } else if bytes < 1024 * 1024 * 1024 {
                    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                } else {
                    format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
                }
            }
            None => "Unknown".to_string(),
        }
    }
}

/// A directory node in the project panel tree
#[derive(Debug, Clone)]
pub struct DirectoryNode {
    pub path: PathBuf,
    pub name: String,
    pub children: Vec<DirectoryNode>,
    pub files: Vec<FileEntry>,
}

impl DirectoryNode {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self {
            path,
            name,
            children: Vec::new(),
            files: Vec::new(),
        }
    }

    /// Scan a directory and build the tree structure
    pub fn scan(path: &std::path::Path) -> std::io::Result<Self> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let mut node = Self::new(path.to_path_buf(), name);

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_name = entry
                .file_name()
                .to_str()
                .unwrap_or("")
                .to_string();

            // Skip hidden files
            if entry_name.starts_with('.') {
                continue;
            }

            if entry_path.is_dir() {
                if let Ok(child) = Self::scan(&entry_path) {
                    node.children.push(child);
                }
            } else {
                node.files.push(FileEntry::new(entry_path, entry_name));
            }
        }

        // Sort children and files alphabetically
        node.children.sort_by(|a, b| a.name.cmp(&b.name));
        node.files.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(node)
    }
}

/// Persistent state that gets saved to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistentPanelState {
    expanded_folders: Vec<PathBuf>,
    selected_folder: PathBuf,
    tree_width: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_type_from_extension() {
        assert_eq!(FileType::from_extension(Some("ts")), FileType::Script);
        assert_eq!(FileType::from_extension(Some("js")), FileType::Script);
        assert_eq!(FileType::from_extension(Some("json")), FileType::Text);
        assert_eq!(FileType::from_extension(Some("txt")), FileType::Text);
        assert_eq!(FileType::from_extension(Some("md")), FileType::Text);
        assert_eq!(FileType::from_extension(Some("png")), FileType::Image);
        assert_eq!(FileType::from_extension(Some("jpg")), FileType::Image);
        assert_eq!(FileType::from_extension(Some("wav")), FileType::Audio);
        assert_eq!(FileType::from_extension(Some("xyz")), FileType::Unknown);
        assert_eq!(FileType::from_extension(None), FileType::Unknown);
    }

    #[test]
    fn test_file_type_is_text_editable() {
        assert!(FileType::Script.is_text_editable());
        assert!(FileType::Text.is_text_editable());
        assert!(FileType::Scene.is_text_editable());
        assert!(!FileType::Image.is_text_editable());
        assert!(!FileType::Audio.is_text_editable());
        assert!(!FileType::Unknown.is_text_editable());
    }

    #[test]
    fn test_project_panel_state_new() {
        let state = ProjectPanelState::new();
        assert!(state.selected_folder.as_os_str().is_empty());
        assert!(state.expanded_folders.is_empty());
        assert!(state.selected_file.is_none());
    }

    #[test]
    fn test_file_entry_creation() {
        let entry = FileEntry::new(
            PathBuf::from("assets/scripts/player.ts"),
            "player.ts".to_string(),
        );
        assert_eq!(entry.name, "player.ts");
        assert_eq!(entry.file_type, FileType::Script);
        assert_eq!(entry.extension, Some("ts".to_string()));
    }

    #[test]
    fn test_directory_node_creation() {
        let node = DirectoryNode::new(
            PathBuf::from("assets/scripts"),
            "scripts".to_string(),
        );
        assert_eq!(node.name, "scripts");
        assert!(node.children.is_empty());
        assert!(node.files.is_empty());
    }

    #[test]
    fn test_scan_directory() {
        // Create a temp directory structure
        let temp = tempdir().unwrap();
        let assets = temp.path().join("assets");
        fs::create_dir_all(assets.join("scripts")).unwrap();
        fs::write(assets.join("scripts/test.ts"), "// test").unwrap();
        fs::write(assets.join("image.png"), &[0u8; 10]).unwrap();

        let node = DirectoryNode::scan(&assets).unwrap();

        assert_eq!(node.name, "assets");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].name, "scripts");
        assert_eq!(node.files.len(), 1);
        assert_eq!(node.files[0].name, "image.png");
    }

    #[test]
    fn test_scan_skips_hidden_files() {
        // Create a temp directory with hidden files
        let temp = tempdir().unwrap();
        let assets = temp.path().join("assets");
        fs::create_dir_all(&assets).unwrap();
        fs::write(assets.join("visible.ts"), "// visible").unwrap();
        fs::write(assets.join(".hidden"), "hidden").unwrap();
        fs::create_dir_all(assets.join(".hidden_dir")).unwrap();

        let node = DirectoryNode::scan(&assets).unwrap();

        // Should only have visible file, no hidden files or directories
        assert_eq!(node.files.len(), 1);
        assert_eq!(node.files[0].name, "visible.ts");
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn test_scan_sorts_alphabetically() {
        // Create a temp directory with multiple files and folders
        let temp = tempdir().unwrap();
        let assets = temp.path().join("assets");
        fs::create_dir_all(assets.join("zebra")).unwrap();
        fs::create_dir_all(assets.join("alpha")).unwrap();
        fs::write(assets.join("z_file.txt"), "z").unwrap();
        fs::write(assets.join("a_file.txt"), "a").unwrap();

        let node = DirectoryNode::scan(&assets).unwrap();

        // Children and files should be sorted alphabetically
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].name, "alpha");
        assert_eq!(node.children[1].name, "zebra");

        assert_eq!(node.files.len(), 2);
        assert_eq!(node.files[0].name, "a_file.txt");
        assert_eq!(node.files[1].name, "z_file.txt");
    }
}
