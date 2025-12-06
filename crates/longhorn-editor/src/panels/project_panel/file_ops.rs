use std::fs;
use std::io;
use std::path::Path;

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
}
