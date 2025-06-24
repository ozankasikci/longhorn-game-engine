use std::path::{Path, PathBuf};
use engine_editor_assets::ProjectAsset;

#[derive(Debug, Clone)]
pub struct SearchFilter {
    query: String,
    case_sensitive: bool,
}

impl SearchFilter {
    pub fn new(query: String) -> Self {
        Self {
            query,
            case_sensitive: false,
        }
    }
    
    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }
    
    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        if self.case_sensitive {
            path_str.contains(&self.query)
        } else {
            path_str.to_lowercase().contains(&self.query.to_lowercase())
        }
    }
    
    pub fn filter_assets(&self, assets: &[ProjectAsset], current_path: &Path) -> Vec<PathBuf> {
        let mut results = Vec::new();
        self.filter_assets_recursive(assets, current_path, &mut results);
        results
    }
    
    fn filter_assets_recursive(&self, assets: &[ProjectAsset], current_path: &Path, results: &mut Vec<PathBuf>) {
        for asset in assets {
            let asset_path = current_path.join(&asset.name);
            
            if self.matches(&asset_path) {
                results.push(asset_path.clone());
            }
            
            if let Some(children) = &asset.children {
                self.filter_assets_recursive(children, &asset_path, results);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_case_insensitive_search() {
        let filter = SearchFilter::new("test".to_string());
        
        assert!(filter.matches(Path::new("test_file.rs")));
        assert!(filter.matches(Path::new("TEST_FILE.rs")));
        assert!(filter.matches(Path::new("folder/test/file.rs")));
        assert!(!filter.matches(Path::new("file.rs")));
    }
    
    #[test]
    fn test_case_sensitive_search() {
        let filter = SearchFilter::new("Test".to_string()).with_case_sensitive(true);
        
        assert!(filter.matches(Path::new("Test_file.rs")));
        assert!(!filter.matches(Path::new("test_file.rs")));
        assert!(!filter.matches(Path::new("TEST_FILE.rs")));
    }
    
    #[test]
    fn test_filter_assets() {
        let assets = vec![
            ProjectAsset::file("test_file.rs"),
            ProjectAsset::folder("src", vec![
                ProjectAsset::file("main.rs"),
                ProjectAsset::file("test_module.rs"),
            ]),
            ProjectAsset::file("readme.md"),
        ];
        
        let filter = SearchFilter::new("test".to_string());
        let results = filter.filter_assets(&assets, Path::new(""));
        
        assert_eq!(results.len(), 2);
        assert!(results.contains(&PathBuf::from("test_file.rs")));
        assert!(results.contains(&PathBuf::from("src/test_module.rs")));
    }
    
    #[test]
    fn test_partial_match() {
        let filter = SearchFilter::new("mod".to_string());
        
        assert!(filter.matches(Path::new("module.rs")));
        assert!(filter.matches(Path::new("test_mod.rs")));
        assert!(filter.matches(Path::new("src/models/user.rs")));
    }
    
    #[test]
    fn test_empty_query() {
        let filter = SearchFilter::new("".to_string());
        
        // Empty query should match everything
        assert!(filter.matches(Path::new("any_file.rs")));
        assert!(filter.matches(Path::new("folder/file.txt")));
    }
}