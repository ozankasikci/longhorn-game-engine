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
}
