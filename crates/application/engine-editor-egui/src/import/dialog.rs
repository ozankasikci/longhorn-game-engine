use std::path::PathBuf;
use egui::{Context, Window, ScrollArea};
use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum ImportResult {
    Import(PathBuf, ImportSettings),
    ImportBatch(Vec<PathBuf>, ImportSettings),
    Cancel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSettings {
    pub scale: f32,
    pub generate_lods: bool,
    pub optimize_meshes: bool,
    pub auto_generate_collision: bool,
    pub collision_type: CollisionType,
    pub lod_levels: Vec<f32>,
}

impl Default for ImportSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            generate_lods: false,
            optimize_meshes: true,
            auto_generate_collision: false,
            collision_type: CollisionType::None,
            lod_levels: vec![],
        }
    }
}

impl ImportSettings {
    pub fn is_default(&self) -> bool {
        self.scale == 1.0 
            && !self.generate_lods 
            && self.optimize_meshes 
            && !self.auto_generate_collision
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CollisionType {
    None,
    Box,
    Sphere,
    Capsule,
    ConvexHull,
    TriangleMesh,
}

pub struct ImportDialog {
    visible: bool,
    selected_files: Vec<PathBuf>,
    selected_files_set: HashSet<PathBuf>,
    settings: ImportSettings,
    file_dialog_result: Option<Vec<PathBuf>>,
}

impl ImportDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            selected_files: Vec::new(),
            selected_files_set: HashSet::new(),
            settings: ImportSettings::default(),
            file_dialog_result: None,
        }
    }
    
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    pub fn selected_files(&self) -> &[PathBuf] {
        &self.selected_files
    }
    
    pub fn import_settings(&self) -> &ImportSettings {
        &self.settings
    }
    
    pub fn show_with_files(&mut self, files: Vec<PathBuf>) {
        self.selected_files = files;
        self.visible = true;
    }
    
    pub fn hide(&mut self) {
        self.visible = false;
    }
    
    pub fn open(&mut self) {
        self.visible = true;
    }
    
    pub fn show(&mut self, ctx: &Context) -> Option<ImportResult> {
        if !self.visible {
            return None;
        }
        
        let mut result = None;
        let mut should_close = false;
        
        Window::new("Import Assets")
            .open(&mut self.visible)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Selected Files");
                    
                    if ui.button("Browse...").clicked() {
                        // Open native file dialog
                        if let Some(files) = rfd::FileDialog::new()
                            .add_filter("3D Models", &["obj", "gltf", "glb", "fbx"])
                            .add_filter("Textures", &["png", "jpg", "jpeg", "tga", "dds"])
                            .add_filter("Audio", &["wav", "mp3", "ogg"])
                            .add_filter("All Files", &["*"])
                            .set_directory(".")
                            .pick_files()
                        {
                            // Add only new files (avoid duplicates)
                            for file in files {
                                if self.selected_files_set.insert(file.clone()) {
                                    self.selected_files.push(file);
                                }
                            }
                        }
                    }
                    
                    if ui.button("Clear All").clicked() {
                        self.selected_files.clear();
                        self.selected_files_set.clear();
                    }
                    
                    // Show selected files with remove buttons
                    let mut to_remove = None;
                    for (idx, file) in self.selected_files.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(file.display().to_string());
                            if ui.small_button("×").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    }
                    
                    // Remove file if requested
                    if let Some(idx) = to_remove {
                        let removed_file = self.selected_files.remove(idx);
                        self.selected_files_set.remove(&removed_file);
                    }
                    
                    ui.separator();
                    
                    ui.heading("Import Settings");
                    
                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        ui.add(egui::DragValue::new(&mut self.settings.scale)
                            .speed(0.01)
                            .range(0.001..=1000.0));
                    });
                    
                    ui.checkbox(&mut self.settings.optimize_meshes, "Optimize Meshes");
                    ui.checkbox(&mut self.settings.generate_lods, "Generate LODs");
                    
                    if self.settings.generate_lods {
                        ui.indent("lod_settings", |ui| {
                            ui.label("LOD Distances:");
                            if ui.button("Add LOD Level").clicked() {
                                self.settings.lod_levels.push(50.0);
                            }
                            
                            let mut to_remove = None;
                            for (i, distance) in self.settings.lod_levels.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("LOD {}", i + 1));
                                    ui.add(egui::DragValue::new(distance)
                                        .speed(1.0)
                                        .range(1.0..=1000.0));
                                    if ui.button("X").clicked() {
                                        to_remove = Some(i);
                                    }
                                });
                            }
                            
                            if let Some(idx) = to_remove {
                                self.settings.lod_levels.remove(idx);
                            }
                        });
                    }
                    
                    ui.checkbox(&mut self.settings.auto_generate_collision, "Auto Generate Collision");
                    
                    if self.settings.auto_generate_collision {
                        ui.indent("collision_settings", |ui| {
                            ui.label("Collision Type:");
                            ui.radio_value(&mut self.settings.collision_type, CollisionType::Box, "Box");
                            ui.radio_value(&mut self.settings.collision_type, CollisionType::Sphere, "Sphere");
                            ui.radio_value(&mut self.settings.collision_type, CollisionType::Capsule, "Capsule");
                            ui.radio_value(&mut self.settings.collision_type, CollisionType::ConvexHull, "Convex Hull");
                            ui.radio_value(&mut self.settings.collision_type, CollisionType::TriangleMesh, "Triangle Mesh");
                        });
                    }
                });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            result = Some(ImportResult::Cancel);
                            should_close = true;
                        }
                        
                        ui.add_enabled_ui(!self.selected_files.is_empty(), |ui| {
                            if ui.button("Import").clicked() {
                                // Import all selected files with the same settings
                                if self.selected_files.len() == 1 {
                                    result = Some(ImportResult::Import(
                                        self.selected_files[0].clone(), 
                                        self.settings.clone()
                                    ));
                                } else {
                                    result = Some(ImportResult::ImportBatch(
                                        self.selected_files.clone(), 
                                        self.settings.clone()
                                    ));
                                }
                                should_close = true;
                            }
                        });
                    });
                });
            });
        
        if should_close {
            self.visible = false;
            self.selected_files.clear();
            self.selected_files_set.clear();
        }
        
        result
    }
}