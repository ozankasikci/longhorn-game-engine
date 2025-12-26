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
}
