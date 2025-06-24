use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FolderOperation {
    Create {
        parent: PathBuf,
        name: String,
    },
    Delete {
        path: PathBuf,
        // Store contents for undo
        contents: Vec<u8>,
    },
    Rename {
        old_path: PathBuf,
        new_name: String,
    },
    Move {
        source: PathBuf,
        target_parent: PathBuf,
    },
}

#[derive(Default)]
pub struct UndoRedoStack {
    undo_stack: Vec<FolderOperation>,
    redo_stack: Vec<FolderOperation>,
    max_size: usize,
}

impl UndoRedoStack {
    pub fn new() -> Self {
        Self::with_max_size(100)
    }
    
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }
    
    /// Push a new operation, clearing the redo stack
    pub fn push(&mut self, operation: FolderOperation) {
        self.undo_stack.push(operation);
        self.redo_stack.clear();
        
        // Limit stack size
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }
    
    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    
    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    
    /// Pop an operation from the undo stack and push it to redo
    pub fn undo(&mut self) -> Option<FolderOperation> {
        if let Some(operation) = self.undo_stack.pop() {
            self.redo_stack.push(operation.clone());
            Some(operation)
        } else {
            None
        }
    }
    
    /// Pop an operation from the redo stack and push it to undo
    pub fn redo(&mut self) -> Option<FolderOperation> {
        if let Some(operation) = self.redo_stack.pop() {
            self.undo_stack.push(operation.clone());
            Some(operation)
        } else {
            None
        }
    }
    
    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
    
    /// Get the number of operations in the undo stack
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }
    
    /// Get the number of operations in the redo stack
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_push_and_undo() {
        let mut stack = UndoRedoStack::new();
        
        let op = FolderOperation::Create {
            parent: PathBuf::from("parent"),
            name: "new_folder".to_string(),
        };
        
        stack.push(op.clone());
        assert_eq!(stack.undo_count(), 1);
        assert_eq!(stack.redo_count(), 0);
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
        
        let undone = stack.undo().unwrap();
        match undone {
            FolderOperation::Create { parent, name } => {
                assert_eq!(parent, PathBuf::from("parent"));
                assert_eq!(name, "new_folder");
            }
            _ => panic!("Wrong operation type"),
        }
        
        assert_eq!(stack.undo_count(), 0);
        assert_eq!(stack.redo_count(), 1);
        assert!(!stack.can_undo());
        assert!(stack.can_redo());
    }
    
    #[test]
    fn test_redo() {
        let mut stack = UndoRedoStack::new();
        
        let op = FolderOperation::Rename {
            old_path: PathBuf::from("old_name"),
            new_name: "new_name".to_string(),
        };
        
        stack.push(op);
        stack.undo();
        
        let redone = stack.redo().unwrap();
        match redone {
            FolderOperation::Rename { old_path, new_name } => {
                assert_eq!(old_path, PathBuf::from("old_name"));
                assert_eq!(new_name, "new_name");
            }
            _ => panic!("Wrong operation type"),
        }
        
        assert_eq!(stack.undo_count(), 1);
        assert_eq!(stack.redo_count(), 0);
    }
    
    #[test]
    fn test_push_clears_redo() {
        let mut stack = UndoRedoStack::new();
        
        stack.push(FolderOperation::Create {
            parent: PathBuf::from("parent1"),
            name: "folder1".to_string(),
        });
        
        stack.push(FolderOperation::Create {
            parent: PathBuf::from("parent2"),
            name: "folder2".to_string(),
        });
        
        stack.undo();
        assert_eq!(stack.redo_count(), 1);
        
        // Push new operation should clear redo stack
        stack.push(FolderOperation::Create {
            parent: PathBuf::from("parent3"),
            name: "folder3".to_string(),
        });
        
        assert_eq!(stack.redo_count(), 0);
        assert_eq!(stack.undo_count(), 2);
    }
    
    #[test]
    fn test_max_size() {
        let mut stack = UndoRedoStack::with_max_size(3);
        
        for i in 0..5 {
            stack.push(FolderOperation::Create {
                parent: PathBuf::from("parent"),
                name: format!("folder{}", i),
            });
        }
        
        // Should only have 3 operations (2, 3, 4)
        assert_eq!(stack.undo_count(), 3);
        
        // Check that oldest operations were removed
        let op1 = stack.undo().unwrap();
        match op1 {
            FolderOperation::Create { name, .. } => assert_eq!(name, "folder4"),
            _ => panic!("Wrong operation"),
        }
        
        let op2 = stack.undo().unwrap();
        match op2 {
            FolderOperation::Create { name, .. } => assert_eq!(name, "folder3"),
            _ => panic!("Wrong operation"),
        }
        
        let op3 = stack.undo().unwrap();
        match op3 {
            FolderOperation::Create { name, .. } => assert_eq!(name, "folder2"),
            _ => panic!("Wrong operation"),
        }
        
        assert!(stack.undo().is_none());
    }
    
    #[test]
    fn test_clear() {
        let mut stack = UndoRedoStack::new();
        
        stack.push(FolderOperation::Create {
            parent: PathBuf::from("parent"),
            name: "folder".to_string(),
        });
        
        stack.undo();
        
        assert_eq!(stack.undo_count(), 0);
        assert_eq!(stack.redo_count(), 1);
        
        stack.clear();
        
        assert_eq!(stack.undo_count(), 0);
        assert_eq!(stack.redo_count(), 0);
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
    }
}