use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Default content for new scenes (RON format)
const DEFAULT_SCENE_CONTENT: &str = r#"(
    name: "New Scene",
    entities: [],
)
"#;

/// Default content for new scripts (TypeScript)
const DEFAULT_SCRIPT_CONTENT: &str = "export default {};\n";

/// Generate a unique filename by appending a number if necessary
fn generate_unique_path(parent: &Path, base_name: &str, extension: &str) -> PathBuf {
    let mut path = parent.join(format!("{}{}", base_name, extension));
    let mut counter = 1;

    while path.exists() {
        path = parent.join(format!("{} {}{}", base_name, counter, extension));
        counter += 1;
    }

    path
}

/// Create a new folder with a unique name, returns the path
pub fn create_new_folder(parent: &Path) -> io::Result<PathBuf> {
    let path = generate_unique_path(parent, "New Folder", "");
    fs::create_dir(&path)?;
    Ok(path)
}

/// Create a new scene file with default RON content, returns the path
pub fn create_scene(parent: &Path) -> io::Result<PathBuf> {
    let path = generate_unique_path(parent, "New Scene", ".scn.ron");
    fs::write(&path, DEFAULT_SCENE_CONTENT)?;
    Ok(path)
}

/// Create a new script file with default TypeScript content, returns the path
pub fn create_script(parent: &Path) -> io::Result<PathBuf> {
    let path = generate_unique_path(parent, "NewScript", ".ts");
    fs::write(&path, DEFAULT_SCRIPT_CONTENT)?;
    Ok(path)
}

/// Create a new folder
pub fn create_folder(parent: &Path, name: &str) -> io::Result<()> {
    let path = parent.join(name);
    fs::create_dir(&path)
}

/// Rename a file or folder
pub fn rename(path: &Path, new_name: &str) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "No parent directory")
    })?;
    let new_path = parent.join(new_name);
    fs::rename(path, new_path)
}

/// Delete a file or folder
pub fn delete(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

/// Supported file extensions for import
const SUPPORTED_EXTENSIONS: &[&str] = &[
    // Images
    "png", "jpg", "jpeg", "gif", "bmp", "webp",
    // Audio
    "wav", "mp3", "ogg",
    // Fonts
    "ttf", "otf",
    // Data
    "json", "toml", "ron",
];

/// Check if a file extension is supported for import
fn is_supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Generate a unique path for imported file (preserves original filename structure)
fn unique_import_path(target_folder: &Path, filename: &str) -> PathBuf {
    let path = target_folder.join(filename);
    if !path.exists() {
        return path;
    }

    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str());

    for i in 1..1000 {
        let new_name = match ext {
            Some(e) => format!("{} ({}).{}", stem, i, e),
            None => format!("{} ({})", stem, i),
        };
        let new_path = target_folder.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
    }
    path // fallback (unlikely)
}

/// Import external files to the target folder
/// Returns the list of successfully imported file paths
pub fn import_files(files: &[PathBuf], target_folder: &Path) -> Vec<PathBuf> {
    let mut imported = Vec::new();

    for file in files {
        if !is_supported_extension(file) {
            continue;
        }

        let filename = match file.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        let dest = unique_import_path(target_folder, filename);
        if fs::copy(file, &dest).is_ok() {
            log::info!("Imported file: {:?} -> {:?}", file, dest);
            imported.push(dest);
        } else {
            log::warn!("Failed to import file: {:?}", file);
        }
    }

    imported
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_folder() {
        let temp = tempdir().unwrap();
        create_folder(temp.path(), "new_folder").unwrap();
        assert!(temp.path().join("new_folder").exists());
    }

    #[test]
    fn test_rename() {
        let temp = tempdir().unwrap();
        let original = temp.path().join("original.txt");
        fs::write(&original, "test").unwrap();

        rename(&original, "renamed.txt").unwrap();

        assert!(!original.exists());
        assert!(temp.path().join("renamed.txt").exists());
    }

    #[test]
    fn test_delete_file() {
        let temp = tempdir().unwrap();
        let file = temp.path().join("to_delete.txt");
        fs::write(&file, "test").unwrap();

        delete(&file).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn test_delete_folder() {
        let temp = tempdir().unwrap();
        let folder = temp.path().join("to_delete");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("file.txt"), "test").unwrap();

        delete(&folder).unwrap();
        assert!(!folder.exists());
    }

    #[test]
    fn test_create_scene() {
        let temp = tempdir().unwrap();
        let path = create_scene(temp.path()).unwrap();

        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with(".scn.ron"));

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("name:"));
        assert!(content.contains("entities:"));
    }

    #[test]
    fn test_create_scene_unique_names() {
        let temp = tempdir().unwrap();

        let path1 = create_scene(temp.path()).unwrap();
        let path2 = create_scene(temp.path()).unwrap();
        let path3 = create_scene(temp.path()).unwrap();

        assert!(path1.file_name().unwrap().to_string_lossy().contains("New Scene.scn.ron"));
        assert!(path2.file_name().unwrap().to_string_lossy().contains("New Scene 1.scn.ron"));
        assert!(path3.file_name().unwrap().to_string_lossy().contains("New Scene 2.scn.ron"));
    }

    #[test]
    fn test_create_script() {
        let temp = tempdir().unwrap();
        let path = create_script(temp.path()).unwrap();

        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with(".ts"));

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("export default"));
    }

    #[test]
    fn test_create_script_unique_names() {
        let temp = tempdir().unwrap();

        let path1 = create_script(temp.path()).unwrap();
        let path2 = create_script(temp.path()).unwrap();

        assert!(path1.file_name().unwrap().to_string_lossy().contains("NewScript.ts"));
        assert!(path2.file_name().unwrap().to_string_lossy().contains("NewScript 1.ts"));
    }

    #[test]
    fn test_create_new_folder() {
        let temp = tempdir().unwrap();

        let path1 = create_new_folder(temp.path()).unwrap();
        let path2 = create_new_folder(temp.path()).unwrap();

        assert!(path1.exists());
        assert!(path2.exists());
        assert!(path1.is_dir());
        assert!(path2.is_dir());
        assert!(path1.file_name().unwrap().to_string_lossy().contains("New Folder"));
        assert!(path2.file_name().unwrap().to_string_lossy().contains("New Folder 1"));
    }

    #[test]
    fn test_import_files_supported() {
        let temp = tempdir().unwrap();
        let source = tempdir().unwrap();

        // Create a test PNG file
        let png_path = source.path().join("test.png");
        fs::write(&png_path, b"fake png data").unwrap();

        let imported = import_files(&[png_path.clone()], temp.path());

        assert_eq!(imported.len(), 1);
        assert!(imported[0].exists());
        assert_eq!(imported[0].file_name().unwrap(), "test.png");
    }

    #[test]
    fn test_import_files_unsupported() {
        let temp = tempdir().unwrap();
        let source = tempdir().unwrap();

        // Create a file with unsupported extension
        let exe_path = source.path().join("test.exe");
        fs::write(&exe_path, b"fake exe data").unwrap();

        let imported = import_files(&[exe_path], temp.path());

        assert!(imported.is_empty());
    }

    #[test]
    fn test_import_files_auto_rename() {
        let temp = tempdir().unwrap();
        let source = tempdir().unwrap();

        // Create source files
        let png1 = source.path().join("image.png");
        let png2 = source.path().join("image2.png");
        fs::write(&png1, b"data1").unwrap();
        fs::write(&png2, b"data2").unwrap();

        // Import first file
        let imported1 = import_files(&[png1.clone()], temp.path());
        assert_eq!(imported1.len(), 1);
        assert_eq!(imported1[0].file_name().unwrap(), "image.png");

        // Import same filename again - should auto-rename
        fs::write(&png1, b"data1 again").unwrap();
        let imported2 = import_files(&[png1], temp.path());
        assert_eq!(imported2.len(), 1);
        assert_eq!(imported2[0].file_name().unwrap(), "image (1).png");
    }
}
