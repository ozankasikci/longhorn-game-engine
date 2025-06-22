use crate::ImportJob;
use std::collections::VecDeque;

pub struct ImportBatch {
    jobs: VecDeque<ImportJob>,
}

impl ImportBatch {
    pub fn new() -> Self {
        Self {
            jobs: VecDeque::new(),
        }
    }
    
    pub fn add_job(&mut self, job: ImportJob) {
        self.jobs.push_back(job);
    }
    
    pub fn job_count(&self) -> usize {
        self.jobs.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }
    
    pub fn take_next_job(&mut self) -> Option<ImportJob> {
        self.jobs.pop_front()
    }
    
    pub fn take_all_jobs(&mut self) -> Vec<ImportJob> {
        self.jobs.drain(..).collect()
    }
    
    pub fn clear(&mut self) {
        self.jobs.clear();
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &ImportJob> {
        self.jobs.iter()
    }
}