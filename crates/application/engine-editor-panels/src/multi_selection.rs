use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct MultiSelection {
    selected_paths: HashSet<PathBuf>,
    last_selected: Option<PathBuf>,
    anchor_path: Option<PathBuf>,
}

impl MultiSelection {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Clear all selections
    pub fn clear(&mut self) {
        self.selected_paths.clear();
        self.last_selected = None;
        self.anchor_path = None;
    }
    
    /// Select a single item, clearing previous selections
    pub fn select_single(&mut self, path: PathBuf) {
        self.clear();
        self.selected_paths.insert(path.clone());
        self.last_selected = Some(path.clone());
        self.anchor_path = Some(path);
    }
    
    /// Toggle selection of an item (for Ctrl+Click)
    pub fn toggle_selection(&mut self, path: PathBuf) {
        if self.selected_paths.contains(&path) {
            self.selected_paths.remove(&path);
            if self.last_selected.as_ref() == Some(&path) {
                self.last_selected = self.selected_paths.iter().next().cloned();
            }
        } else {
            self.selected_paths.insert(path.clone());
            self.last_selected = Some(path);
        }
    }
    
    /// Select a range of items (for Shift+Click)
    pub fn select_range(&mut self, from: &Path, to: &Path, all_items: &[PathBuf]) {
        // Find indices of from and to
        let from_idx = all_items.iter().position(|p| p == from);
        let to_idx = all_items.iter().position(|p| p == to);
        
        if let (Some(start), Some(end)) = (from_idx, to_idx) {
            let (start, end) = if start <= end { (start, end) } else { (end, start) };
            
            // Clear current selection and select range
            self.clear();
            for i in start..=end {
                if let Some(path) = all_items.get(i) {
                    self.selected_paths.insert(path.clone());
                }
            }
            self.last_selected = Some(to.to_path_buf());
        }
    }
    
    /// Add to selection (for Ctrl+Shift+Click)
    pub fn add_range(&mut self, from: &Path, to: &Path, all_items: &[PathBuf]) {
        let from_idx = all_items.iter().position(|p| p == from);
        let to_idx = all_items.iter().position(|p| p == to);
        
        if let (Some(start), Some(end)) = (from_idx, to_idx) {
            let (start, end) = if start <= end { (start, end) } else { (end, start) };
            
            for i in start..=end {
                if let Some(path) = all_items.get(i) {
                    self.selected_paths.insert(path.clone());
                }
            }
            self.last_selected = Some(to.to_path_buf());
        }
    }
    
    /// Check if a path is selected
    pub fn is_selected(&self, path: &Path) -> bool {
        self.selected_paths.contains(path)
    }
    
    /// Get all selected paths
    pub fn selected_paths(&self) -> Vec<PathBuf> {
        self.selected_paths.iter().cloned().collect()
    }
    
    /// Get the count of selected items
    pub fn count(&self) -> usize {
        self.selected_paths.len()
    }
    
    /// Get the last selected item
    pub fn last_selected(&self) -> Option<&PathBuf> {
        self.last_selected.as_ref()
    }
    
    /// Set anchor for range selection
    pub fn set_anchor(&mut self, path: PathBuf) {
        self.anchor_path = Some(path);
    }
    
    /// Get anchor for range selection
    pub fn anchor(&self) -> Option<&PathBuf> {
        self.anchor_path.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_select_single() {
        let mut selection = MultiSelection::new();
        let path1 = PathBuf::from("folder1");
        let path2 = PathBuf::from("folder2");
        
        selection.select_single(path1.clone());
        assert_eq!(selection.count(), 1);
        assert!(selection.is_selected(&path1));
        assert!(!selection.is_selected(&path2));
        
        selection.select_single(path2.clone());
        assert_eq!(selection.count(), 1);
        assert!(!selection.is_selected(&path1));
        assert!(selection.is_selected(&path2));
    }
    
    #[test]
    fn test_toggle_selection() {
        let mut selection = MultiSelection::new();
        let path1 = PathBuf::from("folder1");
        let path2 = PathBuf::from("folder2");
        
        selection.toggle_selection(path1.clone());
        assert!(selection.is_selected(&path1));
        assert_eq!(selection.count(), 1);
        
        selection.toggle_selection(path2.clone());
        assert!(selection.is_selected(&path1));
        assert!(selection.is_selected(&path2));
        assert_eq!(selection.count(), 2);
        
        selection.toggle_selection(path1.clone());
        assert!(!selection.is_selected(&path1));
        assert!(selection.is_selected(&path2));
        assert_eq!(selection.count(), 1);
    }
    
    #[test]
    fn test_select_range() {
        let mut selection = MultiSelection::new();
        let all_items = vec![
            PathBuf::from("file1"),
            PathBuf::from("file2"),
            PathBuf::from("file3"),
            PathBuf::from("file4"),
            PathBuf::from("file5"),
        ];
        
        selection.select_range(&all_items[1], &all_items[3], &all_items);
        assert_eq!(selection.count(), 3);
        assert!(!selection.is_selected(&all_items[0]));
        assert!(selection.is_selected(&all_items[1]));
        assert!(selection.is_selected(&all_items[2]));
        assert!(selection.is_selected(&all_items[3]));
        assert!(!selection.is_selected(&all_items[4]));
    }
    
    #[test]
    fn test_select_range_reverse() {
        let mut selection = MultiSelection::new();
        let all_items = vec![
            PathBuf::from("file1"),
            PathBuf::from("file2"),
            PathBuf::from("file3"),
            PathBuf::from("file4"),
        ];
        
        // Select from index 3 to 1 (reverse order)
        selection.select_range(&all_items[3], &all_items[1], &all_items);
        assert_eq!(selection.count(), 3);
        assert!(selection.is_selected(&all_items[1]));
        assert!(selection.is_selected(&all_items[2]));
        assert!(selection.is_selected(&all_items[3]));
    }
    
    #[test]
    fn test_add_range() {
        let mut selection = MultiSelection::new();
        let all_items = vec![
            PathBuf::from("file1"),
            PathBuf::from("file2"),
            PathBuf::from("file3"),
            PathBuf::from("file4"),
            PathBuf::from("file5"),
        ];
        
        // First select one item
        selection.select_single(all_items[0].clone());
        assert_eq!(selection.count(), 1);
        
        // Add a range
        selection.add_range(&all_items[2], &all_items[4], &all_items);
        assert_eq!(selection.count(), 4);
        assert!(selection.is_selected(&all_items[0]));
        assert!(!selection.is_selected(&all_items[1]));
        assert!(selection.is_selected(&all_items[2]));
        assert!(selection.is_selected(&all_items[3]));
        assert!(selection.is_selected(&all_items[4]));
    }
    
    #[test]
    fn test_clear() {
        let mut selection = MultiSelection::new();
        let path1 = PathBuf::from("folder1");
        let path2 = PathBuf::from("folder2");
        
        selection.toggle_selection(path1);
        selection.toggle_selection(path2);
        assert_eq!(selection.count(), 2);
        
        selection.clear();
        assert_eq!(selection.count(), 0);
        assert!(selection.last_selected().is_none());
        assert!(selection.anchor().is_none());
    }
    
    #[test]
    fn test_last_selected() {
        let mut selection = MultiSelection::new();
        let path1 = PathBuf::from("folder1");
        let path2 = PathBuf::from("folder2");
        
        selection.toggle_selection(path1.clone());
        assert_eq!(selection.last_selected(), Some(&path1));
        
        selection.toggle_selection(path2.clone());
        assert_eq!(selection.last_selected(), Some(&path2));
        
        selection.toggle_selection(path2);
        assert_eq!(selection.last_selected(), Some(&path1));
    }
}