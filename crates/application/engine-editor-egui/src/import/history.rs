use std::path::{Path, PathBuf};
use std::time::SystemTime;
use engine_resource_core::ResourceId;
use crate::import::ImportSettings;

#[derive(Debug, Clone)]
pub struct ImportRecord {
    pub timestamp: SystemTime,
    pub source_path: PathBuf,
    pub imported_path: PathBuf,
    pub resource_id: ResourceId,
    pub import_settings: ImportSettings,
    pub success: bool,
}

pub struct ImportHistory {
    records: Vec<ImportRecord>,
    max_records: usize,
}

impl ImportHistory {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            max_records: 1000,
        }
    }
    
    pub fn add_record(&mut self, record: ImportRecord) {
        self.records.push(record);
        
        // Keep only the most recent records
        if self.records.len() > self.max_records {
            self.records.remove(0);
        }
    }
    
    pub fn total_imports(&self) -> usize {
        self.records.len()
    }
    
    pub fn successful_imports(&self) -> usize {
        self.records.iter().filter(|r| r.success).count()
    }
    
    pub fn failed_imports(&self) -> usize {
        self.records.iter().filter(|r| !r.success).count()
    }
    
    pub fn find_by_source(&self, path: &Path) -> Option<&ImportRecord> {
        self.records.iter()
            .rev() // Search from most recent
            .find(|r| r.source_path == path)
    }
    
    pub fn find_by_resource_id(&self, id: &ResourceId) -> Option<&ImportRecord> {
        self.records.iter()
            .rev()
            .find(|r| r.resource_id == *id)
    }
    
    pub fn get_recent(&self, count: usize) -> Vec<&ImportRecord> {
        let start = self.records.len().saturating_sub(count);
        self.records[start..].iter().rev().collect()
    }
    
    pub fn clear(&mut self) {
        self.records.clear();
    }
}