use crate::{ImportSettings, ImportError};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;
use dashmap::DashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ImportJobId(Uuid);

impl ImportJobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

pub struct ImportJob {
    id: ImportJobId,
    source_path: PathBuf,
    settings: ImportSettings,
    status: ImportStatus,
    cancelled: Arc<AtomicBool>,
    error: Option<ImportError>,
}

impl ImportJob {
    pub fn new(source_path: PathBuf, settings: ImportSettings) -> Self {
        Self {
            id: ImportJobId::new(),
            source_path,
            settings,
            status: ImportStatus::Pending,
            cancelled: Arc::new(AtomicBool::new(false)),
            error: None,
        }
    }
    
    pub fn id(&self) -> ImportJobId {
        self.id
    }
    
    pub fn source_path(&self) -> &PathBuf {
        &self.source_path
    }
    
    pub fn settings(&self) -> &ImportSettings {
        &self.settings
    }
    
    pub fn status(&self) -> ImportStatus {
        self.status
    }
    
    pub fn set_status(&mut self, status: ImportStatus) {
        self.status = status;
    }
    
    pub fn cancel(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        self.status = ImportStatus::Cancelled;
    }
    
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
    
    pub fn set_error(&mut self, error: ImportError) {
        self.error = Some(error);
        self.status = ImportStatus::Failed;
    }
    
    pub fn error(&self) -> Option<&ImportError> {
        self.error.as_ref()
    }
    
    pub fn cancellation_token(&self) -> Arc<AtomicBool> {
        self.cancelled.clone()
    }
}

pub struct ImportQueue {
    pending_jobs: DashMap<ImportJobId, ImportJob>,
    completed_jobs: DashMap<ImportJobId, ImportJob>,
}

impl ImportQueue {
    pub fn new() -> Self {
        Self {
            pending_jobs: DashMap::new(),
            completed_jobs: DashMap::new(),
        }
    }
    
    pub fn add_job(&self, job: ImportJob) {
        let id = job.id();
        self.pending_jobs.insert(id, job);
    }
    
    pub fn take_next_job(&self) -> Option<ImportJob> {
        let entry = self.pending_jobs.iter().next()?;
        let (id, _) = entry.pair();
        let id = *id;
        drop(entry);
        
        self.pending_jobs.remove(&id).map(|(_, job)| job)
    }
    
    pub fn complete_job(&self, job: ImportJob) {
        let id = job.id();
        self.completed_jobs.insert(id, job);
    }
    
    pub fn pending_count(&self) -> usize {
        self.pending_jobs.len()
    }
    
    pub fn completed_count(&self) -> usize {
        self.completed_jobs.len()
    }
    
    pub fn job_status(&self, id: ImportJobId) -> Option<ImportStatus> {
        if let Some(job) = self.pending_jobs.get(&id) {
            Some(job.status())
        } else if let Some(job) = self.completed_jobs.get(&id) {
            Some(job.status())
        } else {
            None
        }
    }
    
    pub fn cancel_job(&self, id: ImportJobId) -> bool {
        if let Some(mut job) = self.pending_jobs.get_mut(&id) {
            job.cancel();
            true
        } else {
            false
        }
    }
    
    pub fn get_job(&self, id: ImportJobId) -> Option<dashmap::mapref::one::Ref<ImportJobId, ImportJob>> {
        self.pending_jobs.get(&id)
            .or_else(|| self.completed_jobs.get(&id))
    }
}