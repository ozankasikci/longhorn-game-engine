use crate::ImportJobId;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ImportProgress {
    job_id: ImportJobId,
    stage: String,
    percentage: f32,
}

impl ImportProgress {
    pub fn new(job_id: ImportJobId) -> Self {
        Self {
            job_id,
            stage: "Initializing".to_string(),
            percentage: 0.0,
        }
    }

    pub fn job_id(&self) -> ImportJobId {
        self.job_id
    }

    pub fn stage(&self) -> &str {
        &self.stage
    }

    pub fn percentage(&self) -> f32 {
        self.percentage.clamp(0.0, 1.0)
    }

    pub fn set_stage(&mut self, stage: impl Into<String>, percentage: f32) {
        self.stage = stage.into();
        self.percentage = percentage.clamp(0.0, 1.0);
    }

    pub fn update_percentage(&mut self, percentage: f32) {
        self.percentage = percentage.clamp(0.0, 1.0);
    }
}

/// Thread-safe progress tracker
pub struct ProgressTracker {
    progress: Arc<Mutex<ImportProgress>>,
}

impl ProgressTracker {
    pub fn new(job_id: ImportJobId) -> Self {
        Self {
            progress: Arc::new(Mutex::new(ImportProgress::new(job_id))),
        }
    }

    pub fn set_stage(&self, stage: impl Into<String>, percentage: f32) {
        if let Ok(mut progress) = self.progress.lock() {
            progress.set_stage(stage, percentage);
        }
    }

    pub fn update_percentage(&self, percentage: f32) {
        if let Ok(mut progress) = self.progress.lock() {
            progress.update_percentage(percentage);
        }
    }

    pub fn get_progress(&self) -> ImportProgress {
        self.progress.lock().unwrap().clone()
    }

    pub fn clone_handle(&self) -> Arc<Mutex<ImportProgress>> {
        self.progress.clone()
    }
}
