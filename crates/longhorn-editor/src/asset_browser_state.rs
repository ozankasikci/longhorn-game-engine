use std::collections::HashSet;
use std::path::PathBuf;

/// File type categorization for asset browser display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Script,
    Image,
    Audio,
    Scene,
    Unknown,
}

impl FileType {
    pub fn from_extension(ext: Option<&str>) -> Self {
        match ext {
            Some("ts") => FileType::Script,
            Some("png") | Some("jpg") | Some("jpeg") | Some("webp") => FileType::Image,
            Some("wav") | Some("mp3") | Some("ogg") => FileType::Audio,
            Some("scene.json") => FileType::Scene,
            _ => FileType::Unknown,
        }
    }
}

/// Default tree panel width
pub const DEFAULT_TREE_WIDTH: f32 = 180.0;
/// Minimum tree panel width
pub const MIN_TREE_WIDTH: f32 = 100.0;
/// Maximum tree panel width
pub const MAX_TREE_WIDTH: f32 = 400.0;

/// State for the asset browser panel
#[derive(Debug)]
pub struct AssetBrowserState {
    /// Currently selected folder (shown in grid view)
    pub selected_folder: PathBuf,
    /// Expanded folders in tree view
    pub expanded_folders: HashSet<PathBuf>,
    /// Currently selected file
    pub selected_file: Option<PathBuf>,
    /// Active rename operation
    pub renaming: Option<PathBuf>,
    /// Width of the tree panel (user-resizable)
    pub tree_width: f32,
}

impl AssetBrowserState {
    pub fn new() -> Self {
        Self {
            selected_folder: PathBuf::new(),
            expanded_folders: HashSet::new(),
            selected_file: None,
            renaming: None,
            tree_width: DEFAULT_TREE_WIDTH,
        }
    }
}

impl Default for AssetBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

/// A file entry in the asset browser
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub file_type: FileType,
}

impl FileEntry {
    pub fn new(path: PathBuf, name: String) -> Self {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        let file_type = FileType::from_extension(extension.as_deref());
        Self {
            path,
            name,
            extension,
            file_type,
        }
    }
}

/// A directory node in the asset browser tree
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_type_from_extension() {
        assert_eq!(FileType::from_extension(Some("ts")), FileType::Script);
        assert_eq!(FileType::from_extension(Some("png")), FileType::Image);
        assert_eq!(FileType::from_extension(Some("jpg")), FileType::Image);
        assert_eq!(FileType::from_extension(Some("wav")), FileType::Audio);
        assert_eq!(FileType::from_extension(Some("xyz")), FileType::Unknown);
        assert_eq!(FileType::from_extension(None), FileType::Unknown);
    }

    #[test]
    fn test_asset_browser_state_new() {
        let state = AssetBrowserState::new();
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
