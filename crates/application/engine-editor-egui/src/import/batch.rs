use std::path::PathBuf;
use std::collections::HashMap;
use crate::import::ImportSettings;

#[derive(Debug, Clone)]
pub struct BatchImportOptions {
    pub use_same_settings: bool,
    pub base_settings: ImportSettings,
    pub output_directory: PathBuf,
}

#[derive(Debug)]
struct BatchImportState {
    files: Vec<PathBuf>,
    completed: usize,
    failed: usize,
    active: bool,
}

pub struct BatchImporter {
    batches: HashMap<u64, BatchImportState>,
    next_batch_id: u64,
}

impl BatchImporter {
    pub fn new() -> Self {
        Self {
            batches: HashMap::new(),
            next_batch_id: 1,
        }
    }
    
    pub fn start_batch(&mut self, files: Vec<PathBuf>, _options: BatchImportOptions) -> u64 {
        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;
        
        let state = BatchImportState {
            files,
            completed: 0,
            failed: 0,
            active: true,
        };
        
        self.batches.insert(batch_id, state);
        batch_id
    }
    
    pub fn is_batch_active(&self, batch_id: u64) -> bool {
        self.batches.get(&batch_id)
            .map(|s| s.active)
            .unwrap_or(false)
    }
    
    pub fn batch_total_files(&self, batch_id: u64) -> usize {
        self.batches.get(&batch_id)
            .map(|s| s.files.len())
            .unwrap_or(0)
    }
    
    pub fn batch_completed_files(&self, batch_id: u64) -> usize {
        self.batches.get(&batch_id)
            .map(|s| s.completed)
            .unwrap_or(0)
    }
    
    pub fn batch_failed_files(&self, batch_id: u64) -> usize {
        self.batches.get(&batch_id)
            .map(|s| s.failed)
            .unwrap_or(0)
    }
    
    pub fn update_batch_progress(&mut self, batch_id: u64, completed: usize, failed: usize) {
        if let Some(state) = self.batches.get_mut(&batch_id) {
            state.completed = completed;
            state.failed = failed;
            
            // Check if batch is complete
            if state.completed + state.failed >= state.files.len() {
                state.active = false;
            }
        }
    }
    
    pub fn cancel_batch(&mut self, batch_id: u64) {
        if let Some(state) = self.batches.get_mut(&batch_id) {
            state.active = false;
        }
    }
    
    pub fn remove_completed_batches(&mut self) {
        self.batches.retain(|_, state| state.active);
    }
}