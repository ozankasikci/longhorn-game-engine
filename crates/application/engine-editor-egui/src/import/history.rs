use crate::import::ImportSettings;
use engine_resource_core::ResourceId;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImportRecord {
    #[allow(dead_code)]
    pub timestamp: SystemTime,
    pub source_path: PathBuf,
    #[allow(dead_code)]
    pub imported_path: PathBuf,
    pub resource_id: ResourceId,
    #[allow(dead_code)]
    pub import_settings: ImportSettings,
    pub success: bool,
}

#[allow(dead_code)]
pub struct ImportHistory {
    records: Vec<ImportRecord>,
    max_records: usize,
}

impl Default for ImportHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
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
        self.records
            .iter()
            .rev() // Search from most recent
            .find(|r| r.source_path == path)
    }

    #[allow(dead_code)]
    pub fn find_by_resource_id(&self, id: &ResourceId) -> Option<&ImportRecord> {
        self.records.iter().rev().find(|r| r.resource_id == *id)
    }

    pub fn get_recent(&self, count: usize) -> Vec<&ImportRecord> {
        let start = self.records.len().saturating_sub(count);
        self.records[start..].iter().rev().collect()
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.records.clear();
    }
}
