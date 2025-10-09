use egui::{Context, ScrollArea};
use engine_resource_core::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

// ScriptLanguage enum for asset typing
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptLanguage {
    TypeScript,
    Lua,
}

#[cfg(test)]
mod typescript_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Mesh,
    Texture,
    Material,
    Audio,
    Animation,
    Prefab,
    Script,
    TypeScriptScript,
    LuaScript,
    Other,
}

#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub id: ResourceId,
    pub name: String,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub asset_type: AssetType,
    #[allow(dead_code)]
    pub size_bytes: u64,
    #[allow(dead_code)]
    pub import_time: SystemTime,
}

impl AssetInfo {
    /// Check if this asset is a TypeScript script
    pub fn is_typescript_script(&self) -> bool {
        matches!(self.asset_type, AssetType::TypeScriptScript)
    }

    /// Check if this asset is a Lua script
    pub fn is_lua_script(&self) -> bool {
        matches!(self.asset_type, AssetType::LuaScript)
    }

    /// Check if this asset is any kind of script
    pub fn is_script(&self) -> bool {
        matches!(
            self.asset_type,
            AssetType::Script | AssetType::TypeScriptScript | AssetType::LuaScript
        )
    }

    /// Get the file extension of this asset
    pub fn get_file_extension(&self) -> Option<&str> {
        self.path.extension().and_then(|ext| ext.to_str())
    }

    /// Get the script language if this is a script asset
    pub fn get_script_language(&self) -> Option<ScriptLanguage> {
        match self.asset_type {
            AssetType::TypeScriptScript => Some(ScriptLanguage::TypeScript),
            AssetType::LuaScript => Some(ScriptLanguage::Lua),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct AssetBrowserState {
    assets: HashMap<ResourceId, AssetInfo>,
    selected_asset: Option<ResourceId>,
    pub search_query: String,
}

impl AssetBrowserState {
    #[allow(dead_code)]
    pub fn add_asset(&mut self, asset: AssetInfo) {
        self.assets.insert(asset.id, asset);
    }

    /// Get script assets sorted by TypeScript first, then Lua
    pub fn get_script_assets_sorted(&self) -> impl Iterator<Item = &AssetInfo> {
        let mut scripts: Vec<_> = self.assets
            .values()
            .filter(|asset| asset.is_script())
            .collect();
        
        scripts.sort_by_key(|asset| match asset.asset_type {
            AssetType::TypeScriptScript => 0, // TypeScript first
            AssetType::LuaScript => 1,        // Lua second
            AssetType::Script => 2,           // Generic scripts last
            _ => 3,
        });
        
        scripts.into_iter()
    }

    /// Get assets filtered by type and sorted appropriately
    pub fn get_filtered_assets(&self) -> impl Iterator<Item = &AssetInfo> {
        self.assets.values().filter(move |asset| {
            if self.search_query.is_empty() {
                true
            } else {
                asset.name.to_lowercase().contains(&self.search_query.to_lowercase())
            }
        })
    }

    /// Get assets organized by folder
    pub fn get_assets_by_folder(&self) -> std::collections::HashMap<String, Vec<&AssetInfo>> {
        use std::collections::HashMap;
        
        let mut organized = HashMap::new();
        
        for asset in self.assets.values() {
            let folder = asset.path.parent()
                .and_then(|p| p.to_str())
                .unwrap_or("root")
                .to_string();
            
            organized.entry(folder).or_insert_with(Vec::new).push(asset);
        }
        
        organized
    }

    #[allow(dead_code)]
    pub fn remove_asset(&mut self, id: &ResourceId) {
        self.assets.remove(id);
        if self.selected_asset == Some(*id) {
            self.selected_asset = None;
        }
    }

    #[allow(dead_code)]
    pub fn has_asset(&self, id: &ResourceId) -> bool {
        self.assets.contains_key(id)
    }

    #[allow(dead_code)]
    pub fn get_asset(&self, id: &ResourceId) -> Option<&AssetInfo> {
        self.assets.get(id)
    }

    #[allow(dead_code)]
    pub fn get_assets(&self) -> impl Iterator<Item = &AssetInfo> {
        self.assets.values()
    }

    #[allow(dead_code)]
    pub fn select_asset(&mut self, id: ResourceId) {
        self.selected_asset = Some(id);
    }

    #[allow(dead_code)]
    pub fn get_selected_asset(&self) -> Option<&AssetInfo> {
        self.selected_asset.and_then(|id| self.assets.get(&id))
    }
}

pub struct AssetBrowser {
    #[allow(dead_code)]
    show_thumbnails: bool,
    #[allow(dead_code)]
    thumbnail_size: f32,
}

impl Default for AssetBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetBrowser {
    pub fn new() -> Self {
        Self {
            show_thumbnails: true,
            thumbnail_size: 64.0,
        }
    }

    #[allow(dead_code)]
    pub fn show(&mut self, ctx: &Context, state: &mut AssetBrowserState) {
        egui::SidePanel::right("asset_browser")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Asset Browser");

                // Search bar
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut state.search_query);
                });

                ui.separator();

                // View options
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.show_thumbnails, "Show Thumbnails");
                    if self.show_thumbnails {
                        ui.add(
                            egui::Slider::new(&mut self.thumbnail_size, 32.0..=128.0).text("Size"),
                        );
                    }
                });

                ui.separator();

                // Asset list
                ScrollArea::vertical().show(ui, |ui| {
                    let filtered_assets: Vec<_> = state
                        .assets
                        .values()
                        .filter(|asset| {
                            state.search_query.is_empty()
                                || asset
                                    .name
                                    .to_lowercase()
                                    .contains(&state.search_query.to_lowercase())
                        })
                        .cloned() // Clone to avoid borrow issues
                        .collect();

                    if self.show_thumbnails {
                        // Grid view with thumbnails
                        let items_per_row =
                            (ui.available_width() / (self.thumbnail_size + 10.0)) as usize;

                        let mut clicked_asset = None;

                        egui::Grid::new("asset_grid")
                            .num_columns(items_per_row.max(1))
                            .spacing([5.0, 5.0])
                            .show(ui, |ui| {
                                for (i, asset) in filtered_assets.iter().enumerate() {
                                    if i > 0 && i % items_per_row == 0 {
                                        ui.end_row();
                                    }

                                    let selected = state.selected_asset == Some(asset.id);

                                    ui.allocate_ui(
                                        egui::vec2(self.thumbnail_size, self.thumbnail_size + 20.0),
                                        |ui| {
                                            let response = ui.group(|ui| {
                                                // Thumbnail placeholder
                                                let (rect, _) = ui.allocate_exact_size(
                                                    egui::vec2(
                                                        self.thumbnail_size,
                                                        self.thumbnail_size,
                                                    ),
                                                    egui::Sense::click(),
                                                );

                                                ui.painter().rect_filled(
                                                    rect,
                                                    4.0,
                                                    if selected {
                                                        egui::Color32::from_rgb(100, 100, 150)
                                                    } else {
                                                        egui::Color32::from_rgb(60, 60, 60)
                                                    },
                                                );

                                                // Asset type icon
                                                let icon = match asset.asset_type {
                                                    AssetType::Mesh => "📐",
                                                    AssetType::Texture => "🖼",
                                                    AssetType::Material => "🎨",
                                                    AssetType::Audio => "🔊",
                                                    AssetType::Animation => "🎬",
                                                    AssetType::Prefab => "📦",
                                                    AssetType::Script => "📜",
                                                    AssetType::TypeScriptScript => "🔷",
                                                    AssetType::LuaScript => "🌙",
                                                    AssetType::Other => "📄",
                                                };

                                                ui.painter().text(
                                                    rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    icon,
                                                    egui::FontId::proportional(24.0),
                                                    egui::Color32::WHITE,
                                                );

                                                // Asset name
                                                ui.label(egui::RichText::new(&asset.name).small());
                                            });

                                            if response.response.clicked() {
                                                clicked_asset = Some(asset.id);
                                            }
                                        },
                                    );
                                }
                            });

                        if let Some(id) = clicked_asset {
                            state.select_asset(id);
                        }
                    } else {
                        // List view
                        let mut clicked_asset = None;

                        for asset in &filtered_assets {
                            let selected = state.selected_asset == Some(asset.id);

                            if ui.selectable_label(selected, &asset.name).clicked() {
                                clicked_asset = Some(asset.id);
                            }
                        }

                        if let Some(id) = clicked_asset {
                            state.select_asset(id);
                        }
                    }
                });

                ui.separator();

                // Selected asset info
                if let Some(asset) = state.get_selected_asset() {
                    ui.heading("Asset Details");
                    ui.label(format!("Name: {}", asset.name));
                    ui.label(format!("Type: {:?}", asset.asset_type));
                    ui.label(format!("Path: {}", asset.path.display()));
                    ui.label(format!("Size: {} bytes", asset.size_bytes));
                }
            });
    }
}
