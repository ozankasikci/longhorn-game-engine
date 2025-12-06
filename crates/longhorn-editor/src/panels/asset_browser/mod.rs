mod tree_view;
mod grid_view;
mod file_ops;

pub use tree_view::*;
pub use grid_view::*;
pub use file_ops::*;

use egui::Ui;
use crate::asset_browser_state::{AssetBrowserState, DirectoryNode};

/// Asset browser panel with tree and grid views
pub struct AssetBrowserPanel;

impl AssetBrowserPanel {
    pub fn new() -> Self {
        Self
    }

    /// Show the asset browser panel
    /// Returns an action if the user triggered one (e.g., open file)
    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut AssetBrowserState,
        root: Option<&DirectoryNode>,
    ) -> Option<AssetBrowserAction> {
        if root.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("No project loaded");
            });
            return None;
        }

        let root = root.unwrap();
        let mut action = None;

        // Two-pane layout: tree on left, grid on right
        ui.columns(2, |columns| {
            // Left pane: Tree view
            columns[0].push_id("asset_tree", |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(tree_action) = show_tree_view(ui, state, root) {
                        action = Some(tree_action);
                    }
                });
            });

            // Right pane: Grid view
            columns[1].push_id("asset_grid", |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(grid_action) = show_grid_view(ui, state, root) {
                        action = Some(grid_action);
                    }
                });
            });
        });

        action
    }
}

impl Default for AssetBrowserPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Context menu actions
#[derive(Debug, Clone)]
pub enum ContextAction {
    CreateFolder,
    Rename(std::path::PathBuf),
    Delete(std::path::PathBuf),
    Refresh,
}

/// Actions that can be triggered from the asset browser
#[derive(Debug, Clone)]
pub enum AssetBrowserAction {
    OpenScript(std::path::PathBuf),
    OpenImage(std::path::PathBuf),
    OpenExternal(std::path::PathBuf),
    Context(ContextAction),
}
