use crate::ImportJobId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMetadata {
    pub job_id: ImportJobId,
    pub source_path: PathBuf,
    pub import_time: SystemTime,
    pub file_size: u64,
    pub import_duration_ms: u64,
    pub importer_name: String,
}

impl ImportMetadata {
    pub fn new(
        job_id: ImportJobId,
        source_path: PathBuf,
        file_size: u64,
        importer_name: String,
    ) -> Self {
        Self {
            job_id,
            source_path,
            import_time: SystemTime::now(),
            file_size,
            import_duration_ms: 0,
            importer_name,
        }
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.import_duration_ms = duration_ms;
        self
    }
}
