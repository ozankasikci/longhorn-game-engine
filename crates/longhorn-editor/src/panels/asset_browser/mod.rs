mod tree_view;
mod grid_view;
mod file_ops;

pub use tree_view::*;
pub use grid_view::*;
pub use file_ops::*;

use egui::{Ui, Vec2, Sense, CursorIcon};
use crate::asset_browser_state::{AssetBrowserState, DirectoryNode, MIN_TREE_WIDTH, MAX_TREE_WIDTH};
use crate::styling::Colors;
use crate::ui_state::UiStateTracker;

/// Width of the resize handle
const SPLITTER_WIDTH: f32 = 4.0;

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
        ui_state: &mut UiStateTracker,
    ) -> Option<AssetBrowserAction> {
        if root.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("No project loaded");
            });
            return None;
        }

        let root = root.unwrap();
        let mut action = None;

        // Get total available space
        let available_rect = ui.available_rect_before_wrap();
        let available_width = available_rect.width();
        let available_height = available_rect.height();

        // Clamp tree width to available space
        let max_tree = (available_width - 100.0).min(MAX_TREE_WIDTH);
        state.tree_width = state.tree_width.clamp(MIN_TREE_WIDTH, max_tree);

        // Calculate widths
        let tree_width = state.tree_width;
        let content_width = available_width - tree_width - SPLITTER_WIDTH;

        // Left panel: Tree view
        let tree_rect = egui::Rect::from_min_size(
            available_rect.min,
            Vec2::new(tree_width, available_height),
        );

        let mut tree_ui = ui.new_child(egui::UiBuilder::new().max_rect(tree_rect));
        tree_ui.painter().rect_filled(tree_rect, 0.0, Colors::BG_EXTREME);

        egui::ScrollArea::vertical()
            .id_salt("asset_tree_scroll")
            .show(&mut tree_ui, |ui| {
                if let Some(tree_action) = show_tree_view(ui, state, root) {
                    action = Some(tree_action);
                }
            });

        // Splitter (resize handle)
        let splitter_rect = egui::Rect::from_min_size(
            egui::pos2(available_rect.min.x + tree_width, available_rect.min.y),
            Vec2::new(SPLITTER_WIDTH, available_height),
        );

        let splitter_response = ui.allocate_rect(splitter_rect, Sense::drag());

        let splitter_color = if splitter_response.hovered() || splitter_response.dragged() {
            Colors::ACCENT
        } else {
            Colors::STROKE_DEFAULT
        };
        ui.painter().rect_filled(splitter_rect, 0.0, splitter_color);

        if splitter_response.hovered() || splitter_response.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
        }

        if splitter_response.dragged() {
            let delta = splitter_response.drag_delta().x;
            state.tree_width = (state.tree_width + delta).clamp(MIN_TREE_WIDTH, max_tree);
        }

        // Right panel: Content view
        let content_rect = egui::Rect::from_min_size(
            egui::pos2(available_rect.min.x + tree_width + SPLITTER_WIDTH, available_rect.min.y),
            Vec2::new(content_width, available_height),
        );

        let mut content_ui = ui.new_child(egui::UiBuilder::new().max_rect(content_rect));

        egui::ScrollArea::vertical()
            .id_salt("asset_grid_scroll")
            .show(&mut content_ui, |ui| {
                if let Some(grid_action) = show_grid_view(ui, state, root, ui_state) {
                    action = Some(grid_action);
                }
            });

        // Advance the cursor past all the content we've drawn
        ui.allocate_rect(available_rect, Sense::hover());

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
