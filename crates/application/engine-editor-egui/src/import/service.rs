use engine_asset_import::{ImportContext, ImportPipeline, ImportSettings as AssetImportSettings};
use engine_mesh_import::obj::ObjImporter;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
// use engine_texture_import::TextureImporter;
// use engine_audio_import::AudioImporter;
use super::wrappers::{
    ObjImporterWrapper, StandardAudioImporterWrapper, StandardTextureImporterWrapper,
};
use super::{CollisionType, ImportSettings};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum ImportStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportError {
    FileNotFound,
    UnsupportedFormat,
    ProcessingError(String),
    IoError(String),
}

#[derive(Clone, Debug)]
pub struct ImportHandle {
    path: PathBuf,
    status: Arc<Mutex<ImportStatus>>,
    progress: Arc<Mutex<f32>>,
    error: Arc<Mutex<Option<ImportError>>>,
    result: Arc<Mutex<Option<Result<Vec<Uuid>, ImportError>>>>,
}

impl ImportHandle {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            status: Arc::new(Mutex::new(ImportStatus::Pending)),
            progress: Arc::new(Mutex::new(0.0)),
            error: Arc::new(Mutex::new(None)),
            result: Arc::new(Mutex::new(None)),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn status(&self) -> ImportStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn progress(&self) -> f32 {
        *self.progress.lock().unwrap()
    }

    pub fn error(&self) -> Option<ImportError> {
        self.error.lock().unwrap().clone()
    }

    pub fn update_progress(&self, progress: f32) {
        *self.progress.lock().unwrap() = progress;
        if progress > 0.0 && progress < 1.0 {
            *self.status.lock().unwrap() = ImportStatus::InProgress;
        }
    }

    pub fn complete(&self, result: Result<Vec<Uuid>, ImportError>) {
        match &result {
            Ok(_) => {
                *self.status.lock().unwrap() = ImportStatus::Completed;
                *self.progress.lock().unwrap() = 1.0;
            }
            Err(e) => {
                *self.status.lock().unwrap() = ImportStatus::Failed;
                *self.error.lock().unwrap() = Some(e.clone());
            }
        }
        *self.result.lock().unwrap() = Some(result);
    }
}

pub struct ImportService {
    pipeline: ImportPipeline,
    notification_sender: Option<mpsc::Sender<ImportNotification>>,
}

impl ImportService {
    pub fn new() -> Self {
        Self {
            pipeline: ImportPipeline::new(),
            notification_sender: None,
        }
    }

    pub fn pipeline(&self) -> &ImportPipeline {
        &self.pipeline
    }

    pub fn register_mesh_importers(&mut self) {
        // Register wrapped mesh importers that convert MeshData to Vec<u8>
        self.pipeline
            .register_importer(Box::new(ObjImporterWrapper::create()));
        // TODO: Add GLTF and FBX importers when available
        // self.pipeline.register_importer(Box::new(GltfImporterWrapper::create()));
        // self.pipeline.register_importer(Box::new(FbxImporterWrapper::create()));
    }

    pub fn register_texture_importers(&mut self) {
        // Register wrapped texture importers that convert TextureData to Vec<u8>
        // TODO: Uncomment when TextureImporter is available
        // self.pipeline.register_importer(Box::new(StandardTextureImporterWrapper::create()));
    }

    pub fn register_audio_importers(&mut self) {
        // Register wrapped audio importers that convert AudioData to Vec<u8>
        // TODO: Uncomment when AudioImporter is available
        // self.pipeline.register_importer(Box::new(StandardAudioImporterWrapper::create()));
    }

    pub fn set_notification_sender(&mut self, sender: mpsc::Sender<ImportNotification>) {
        self.notification_sender = Some(sender);
    }

    pub fn start_import(&mut self, path: PathBuf, settings: ImportSettings) -> ImportHandle {
        let handle = ImportHandle::new(path.clone());

        // Send notification
        if let Some(sender) = &self.notification_sender {
            let _ = sender.send(ImportNotification::Started {
                path: path.clone(),
                handle: handle.clone(),
            });
        }

        // TODO: Actually start async import here
        // For now, just return the handle

        handle
    }

    pub fn process_queue(&mut self, queue: &ImportQueue) -> Vec<ImportHandle> {
        let mut handles = Vec::new();

        while let Some((path, settings)) = queue.next() {
            let handle = self.start_import(path, settings);
            handles.push(handle);
        }

        handles
    }
}

#[derive(Debug, Clone)]
pub enum ImportNotification {
    Started { path: PathBuf, handle: ImportHandle },
    Progress { path: PathBuf, progress: f32 },
    Completed { path: PathBuf, assets: Vec<Uuid> },
    Failed { path: PathBuf, error: ImportError },
}

pub struct ImportQueue {
    items: Arc<Mutex<Vec<(PathBuf, ImportSettings)>>>,
}

impl ImportQueue {
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add(&self, path: PathBuf, settings: ImportSettings) {
        self.items.lock().unwrap().push((path, settings));
    }

    pub fn len(&self) -> usize {
        self.items.lock().unwrap().len()
    }

    pub fn pending_count(&self) -> usize {
        self.len()
    }

    pub fn next(&self) -> Option<(PathBuf, ImportSettings)> {
        self.items.lock().unwrap().pop()
    }
}

pub struct ImportSettingsConverter;

impl ImportSettingsConverter {
    pub fn new() -> Self {
        Self
    }

    pub fn convert(&self, ui_settings: &ImportSettings) -> ConvertedImportSettings {
        ConvertedImportSettings {
            scale_factor: ui_settings.scale,
            generate_lods: ui_settings.generate_lods,
            optimize_mesh: ui_settings.optimize_meshes,
        }
    }
}

// Temporary struct for converted settings
pub struct ConvertedImportSettings {
    scale_factor: f32,
    generate_lods: bool,
    optimize_mesh: bool,
}

impl ConvertedImportSettings {
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub fn generate_lods(&self) -> bool {
        self.generate_lods
    }

    pub fn optimize_mesh(&self) -> bool {
        self.optimize_mesh
    }
}

pub struct ImportUIState {
    active_imports: Vec<ImportHandle>,
    completed_imports: Vec<ImportHandle>,
}

impl ImportUIState {
    pub fn new() -> Self {
        Self {
            active_imports: Vec::new(),
            completed_imports: Vec::new(),
        }
    }

    pub fn active_imports(&self) -> &[ImportHandle] {
        &self.active_imports
    }

    pub fn completed_imports(&self) -> &[ImportHandle] {
        &self.completed_imports
    }

    pub fn is_importing(&self) -> bool {
        !self.active_imports.is_empty()
    }

    pub fn add_import(&mut self, handle: ImportHandle) {
        self.active_imports.push(handle);
    }

    pub fn update(&mut self) {
        // Move completed imports to completed list
        let mut i = 0;
        while i < self.active_imports.len() {
            let status = self.active_imports[i].status();
            if status == ImportStatus::Completed || status == ImportStatus::Failed {
                let handle = self.active_imports.remove(i);
                self.completed_imports.push(handle);
            } else {
                i += 1;
            }
        }
    }
}
