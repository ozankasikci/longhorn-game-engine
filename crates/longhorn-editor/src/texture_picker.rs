use longhorn_core::AssetId;
use std::path::PathBuf;

/// State for the texture picker popup
#[derive(Debug, Clone)]
pub struct TexturePickerState {
    /// Whether the picker is currently open
    pub is_open: bool,
    /// The entity that will receive the selected texture
    pub target_entity: Option<hecs::Entity>,
}

impl TexturePickerState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            target_entity: None,
        }
    }

    /// Open the texture picker for a specific entity
    pub fn open_for_entity(&mut self, entity: hecs::Entity) {
        self.is_open = true;
        self.target_entity = Some(entity);
    }

    /// Close the texture picker
    pub fn close(&mut self) {
        self.is_open = false;
        self.target_entity = None;
    }
}

impl Default for TexturePickerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of texture picker interaction
#[derive(Debug, Clone)]
pub enum TexturePickerAction {
    None,
    SelectTexture {
        entity: hecs::Entity,
        asset_id: AssetId,
        path: String,
    },
}

/// Show the texture picker popup window
///
/// Returns an action if a texture was selected
pub fn show_texture_picker(
    ctx: &egui::Context,
    state: &mut TexturePickerState,
    image_files: &[PathBuf],
    asset_manager: &mut longhorn_assets::AssetManager<longhorn_assets::FilesystemSource>,
    project_root: Option<&std::path::Path>,
) -> TexturePickerAction {
    if !state.is_open {
        return TexturePickerAction::None;
    }

    let Some(target_entity) = state.target_entity else {
        return TexturePickerAction::None;
    };

    let mut action = TexturePickerAction::None;
    let mut should_close = false;

    egui::Window::new("Select Texture")
        .collapsible(false)
        .resizable(true)
        .default_width(600.0)
        .default_height(400.0)
        .show(ctx, |ui| {
            ui.heading("Choose a texture for the Sprite");
            ui.separator();

            if image_files.is_empty() {
                ui.label("No image files found in the project.");
                ui.label("Import PNG or JPEG files to use as textures.");
                ui.separator();
                if ui.button("Close").clicked() {
                    should_close = true;
                }
                return;
            }

            // Show images in a scrollable grid
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    for image_path in image_files {
                        // Get the file name to display
                        let file_name = image_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown");

                        // Create a selectable button for each image
                        // For now, just show the filename with an icon
                        let label = format!("🖼 {}", file_name);

                        if ui.button(&label).clicked() {
                            // Convert absolute path to relative path for registry lookup
                            // The registry stores paths relative to project root
                            let Some(root) = project_root else {
                                log::error!("Cannot select texture: No project root set");
                                continue;
                            };

                            let Ok(relative_path) = image_path.strip_prefix(root) else {
                                log::error!("Image path {:?} is not under project root {:?}, skipping", image_path, root);
                                continue;
                            };

                            let Some(path_str) = relative_path.to_str() else {
                                log::error!("Failed to convert path {:?} to string, skipping", relative_path);
                                continue;
                            };

                            // Try to get existing asset ID from registry, or import the asset
                            let asset_id = if let Some(id) = asset_manager.registry().get_id(path_str) {
                                id
                            } else {
                                // If not in registry, auto-import it
                                log::info!("Image {} not in asset registry, importing...", path_str);
                                match asset_manager.import_asset(image_path, path_str) {
                                    Ok(id) => {
                                        log::info!("Successfully imported {} with ID {}", path_str, id.0);
                                        id
                                    }
                                    Err(e) => {
                                        log::error!("Failed to import asset {}: {}", path_str, e);
                                        // Skip this image if import fails
                                        continue;
                                    }
                                }
                            };

                            action = TexturePickerAction::SelectTexture {
                                entity: target_entity,
                                asset_id,
                                path: path_str.to_string(),
                            };
                            should_close = true;
                        }
                    }
                });
            });

            ui.separator();

            // Cancel button at bottom
            if ui.button("Cancel").clicked() {
                should_close = true;
            }
        });

    if should_close {
        state.close();
    }

    action
}

/// Helper function to collect all image files from a directory tree
pub(crate) fn collect_image_files(root: &crate::DirectoryNode) -> Vec<PathBuf> {
    let mut images = Vec::new();
    collect_images_recursive(root, &mut images);
    images
}

fn collect_images_recursive(node: &crate::DirectoryNode, images: &mut Vec<PathBuf>) {
    // Add image files from this directory
    for file in &node.files {
        if file.file_type == crate::FileType::Image {
            images.push(file.path.clone());
        }
    }

    // Recurse into subdirectories
    for child in &node.children {
        collect_images_recursive(child, images);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_picker_state_new() {
        let state = TexturePickerState::new();
        assert!(!state.is_open);
        assert!(state.target_entity.is_none());
    }

    #[test]
    fn test_texture_picker_open_close() {
        let mut state = TexturePickerState::new();
        // Create a real entity by spawning into a hecs World
        let mut world = hecs::World::new();
        let entity = world.spawn(());

        state.open_for_entity(entity);
        assert!(state.is_open);
        assert_eq!(state.target_entity, Some(entity));

        state.close();
        assert!(!state.is_open);
        assert!(state.target_entity.is_none());
    }
}
