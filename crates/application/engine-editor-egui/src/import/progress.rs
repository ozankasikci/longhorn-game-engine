use std::collections::HashMap;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImportTask {
    pub id: u64,
    #[allow(dead_code)]
    pub file_path: PathBuf,
    pub total_bytes: usize,
    pub processed_bytes: usize,
    pub status: ImportStatus,
}

#[allow(dead_code)]
pub struct ImportProgress {
    tasks: HashMap<u64, ImportTask>,
    next_id: u64,
}

#[allow(dead_code)]
impl Default for ImportProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ImportProgress {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_task(&mut self, mut task: ImportTask) {
        task.id = self.next_id;
        self.next_id += 1;
        self.tasks.insert(task.id, task);
    }

    pub fn update_task(&mut self, id: u64, processed_bytes: usize, status: ImportStatus) {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.processed_bytes = processed_bytes;
            task.status = status;

            // Remove completed tasks
            if status == ImportStatus::Completed || status == ImportStatus::Failed {
                self.tasks.remove(&id);
            }
        }
    }

    pub fn get_task(&self, id: u64) -> Option<&ImportTask> {
        self.tasks.get(&id)
    }

    pub fn active_tasks(&self) -> usize {
        self.tasks
            .iter()
            .filter(|(_, t)| {
                t.status == ImportStatus::Processing || t.status == ImportStatus::Pending
            })
            .count()
    }

    pub fn overall_progress(&self) -> f32 {
        let total_bytes: usize = self.tasks.values().map(|t| t.total_bytes).sum();
        let processed_bytes: usize = self.tasks.values().map(|t| t.processed_bytes).sum();

        if total_bytes == 0 {
            0.0
        } else {
            processed_bytes as f32 / total_bytes as f32
        }
    }

    #[allow(dead_code)]
    pub fn show(&self, ctx: &egui::Context) {
        if self.tasks.is_empty() {
            return;
        }

        egui::Window::new("Import Progress")
            .resizable(false)
            .show(ctx, |ui| {
                for task in self.tasks.values() {
                    ui.horizontal(|ui| {
                        ui.label(task.file_path.display().to_string());

                        let progress = if task.total_bytes > 0 {
                            task.processed_bytes as f32 / task.total_bytes as f32
                        } else {
                            0.0
                        };

                        ui.add(egui::ProgressBar::new(progress));

                        match task.status {
                            ImportStatus::Pending => ui.label("Pending"),
                            ImportStatus::Processing => ui.label("Processing"),
                            ImportStatus::Completed => ui.label("Completed"),
                            ImportStatus::Failed => ui.colored_label(egui::Color32::RED, "Failed"),
                        };
                    });
                }

                ui.separator();
                ui.label(format!(
                    "Overall Progress: {:.1}%",
                    self.overall_progress() * 100.0
                ));
            });
    }
}
