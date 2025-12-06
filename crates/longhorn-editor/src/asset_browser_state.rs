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
}

impl AssetBrowserState {
    pub fn new() -> Self {
        Self {
            selected_folder: PathBuf::new(),
            expanded_folders: HashSet::new(),
            selected_file: None,
            renaming: None,
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
