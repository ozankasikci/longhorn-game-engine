use egui::{Context, ScrollArea};
use engine_resource_core::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Mesh,
    Texture,
    Material,
    Audio,
    Animation,
    Prefab,
    Script,
    Other,
}

#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub id: ResourceId,
    pub name: String,
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub size_bytes: u64,
    pub import_time: SystemTime,
}

#[derive(Default)]
pub struct AssetBrowserState {
    assets: HashMap<ResourceId, AssetInfo>,
    selected_asset: Option<ResourceId>,
    search_query: String,
}

impl AssetBrowserState {
    pub fn add_asset(&mut self, asset: AssetInfo) {
        self.assets.insert(asset.id, asset);
    }

    pub fn remove_asset(&mut self, id: &ResourceId) {
        self.assets.remove(id);
        if self.selected_asset == Some(*id) {
            self.selected_asset = None;
        }
    }

    pub fn has_asset(&self, id: &ResourceId) -> bool {
        self.assets.contains_key(id)
    }

    pub fn get_asset(&self, id: &ResourceId) -> Option<&AssetInfo> {
        self.assets.get(id)
    }

    pub fn get_assets(&self) -> impl Iterator<Item = &AssetInfo> {
        self.assets.values()
    }

    pub fn select_asset(&mut self, id: ResourceId) {
        self.selected_asset = Some(id);
    }

    pub fn get_selected_asset(&self) -> Option<&AssetInfo> {
        self.selected_asset.and_then(|id| self.assets.get(&id))
    }
}

pub struct AssetBrowser {
    show_thumbnails: bool,
    thumbnail_size: f32,
}

impl AssetBrowser {
    pub fn new() -> Self {
        Self {
            show_thumbnails: true,
            thumbnail_size: 64.0,
        }
    }

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
