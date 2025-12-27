mod tree_view;
mod grid_view;
mod file_ops;

pub use tree_view::*;
pub use grid_view::*;
pub use file_ops::*;

use egui::{Ui, Vec2, Sense, CursorIcon};
use crate::project_panel_state::{ProjectPanelState, DirectoryNode, MIN_TREE_WIDTH, MAX_TREE_WIDTH};
use crate::styling::Colors;
use crate::ui_state::UiStateTracker;

/// Width of the resize handle
const SPLITTER_WIDTH: f32 = 4.0;

/// Project panel with tree and grid views
pub struct ProjectPanel;

impl ProjectPanel {
    pub fn new() -> Self {
        Self
    }

    /// Show the project panel
    /// Returns an action if the user triggered one (e.g., open file)
    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut ProjectPanelState,
        root: Option<&DirectoryNode>,
        ui_state: &mut UiStateTracker,
    ) -> Option<ProjectPanelAction> {
        if root.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("No project loaded");
            });
            return None;
        }

        let root = root.unwrap();
        let mut action = None;

        // Remove default padding/spacing for tight layout
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ui.spacing_mut().window_margin = egui::Margin::ZERO;

        // Get total available space
        let available_rect = ui.available_rect_before_wrap();
        let available_width = available_rect.width();
        let available_height = available_rect.height();

        // Get the full clip rect (includes the tab's inner margin area)
        // We'll paint backgrounds to this larger area so they extend edge-to-edge
        let full_rect = ui.clip_rect();

        // Clamp tree width to available space (ensure max >= min to avoid panic)
        let max_tree = (available_width - 100.0).min(MAX_TREE_WIDTH).max(MIN_TREE_WIDTH);
        state.tree_width = state.tree_width.clamp(MIN_TREE_WIDTH, max_tree);

        // Calculate widths
        let tree_width = state.tree_width;
        let content_width = available_width - tree_width - SPLITTER_WIDTH;

        // Calculate margin offsets (difference between full rect and available rect)
        let margin_left = available_rect.min.x - full_rect.min.x;
        let margin_top = available_rect.min.y - full_rect.min.y;
        let margin_right = full_rect.max.x - available_rect.max.x;
        let margin_bottom = full_rect.max.y - available_rect.max.y;

        // Left panel: Tree view (with clipping)
        let tree_rect = egui::Rect::from_min_size(
            available_rect.min,
            Vec2::new(tree_width, available_height),
        );

        // Expanded tree rect for background painting (extends into margin area)
        let tree_bg_rect = egui::Rect::from_min_max(
            egui::pos2(tree_rect.min.x - margin_left, tree_rect.min.y - margin_top),
            egui::pos2(tree_rect.max.x, tree_rect.max.y + margin_bottom),
        );

        let mut tree_ui = ui.new_child(egui::UiBuilder::new().max_rect(tree_rect));
        tree_ui.set_clip_rect(tree_rect);
        // Paint background to the expanded rect (fills margin area)
        ui.painter().rect_filled(tree_bg_rect, 0.0, Colors::BG_EXTREME);

        egui::ScrollArea::both()
            .id_salt("project_tree_scroll")
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
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

        // Expanded splitter rect for painting (extends into margin area)
        let splitter_bg_rect = egui::Rect::from_min_max(
            egui::pos2(splitter_rect.min.x, splitter_rect.min.y - margin_top),
            egui::pos2(splitter_rect.max.x, splitter_rect.max.y + margin_bottom),
        );

        let splitter_response = ui.allocate_rect(splitter_rect, Sense::drag());

        let splitter_color = if splitter_response.hovered() || splitter_response.dragged() {
            Colors::ACCENT
        } else {
            Colors::STROKE_DEFAULT
        };
        ui.painter().rect_filled(splitter_bg_rect, 0.0, splitter_color);

        if splitter_response.hovered() || splitter_response.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
        }

        if splitter_response.dragged() {
            let delta = splitter_response.drag_delta().x;
            state.tree_width = (state.tree_width + delta).clamp(MIN_TREE_WIDTH, max_tree);
        }

        // Right panel: Content view (with clipping)
        let content_rect = egui::Rect::from_min_size(
            egui::pos2(available_rect.min.x + tree_width + SPLITTER_WIDTH, available_rect.min.y),
            Vec2::new(content_width.max(0.0), available_height),
        );

        // Expanded content rect for background painting (extends into margin area)
        let content_bg_rect = egui::Rect::from_min_max(
            egui::pos2(content_rect.min.x, content_rect.min.y - margin_top),
            egui::pos2(content_rect.max.x + margin_right, content_rect.max.y + margin_bottom),
        );

        let mut content_ui = ui.new_child(egui::UiBuilder::new().max_rect(content_rect));
        content_ui.set_clip_rect(content_rect);
        // Paint background to the expanded rect (fills margin area)
        ui.painter().rect_filled(content_bg_rect, 0.0, Colors::BG_PANEL);

        egui::ScrollArea::both()
            .id_salt("project_grid_scroll")
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
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

impl Default for ProjectPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Context menu actions
#[derive(Debug, Clone)]
pub enum ContextAction {
    CreateFolder(std::path::PathBuf),
    CreateScene(std::path::PathBuf),
    CreateScript(std::path::PathBuf),
    Rename(std::path::PathBuf),
    Delete(std::path::PathBuf),
    Refresh,
    ImportAsset(std::path::PathBuf),
}

/// Actions that can be triggered from the project panel
#[derive(Debug, Clone)]
pub enum ProjectPanelAction {
    OpenScript(std::path::PathBuf),
    OpenScene(std::path::PathBuf),
    OpenImage(std::path::PathBuf),
    OpenExternal(std::path::PathBuf),
    Context(ContextAction),
}
