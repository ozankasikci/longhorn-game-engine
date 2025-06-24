use std::path::PathBuf;
use eframe::egui;

#[derive(Clone, Debug)]
pub enum DragItem {
    Folder(PathBuf),
    File(PathBuf),
}

#[derive(Default)]
pub struct DragDropState {
    pub dragging_item: Option<DragItem>,
    pub drag_start_pos: Option<egui::Pos2>,
    pub is_dragging: bool,
    pub hover_target: Option<PathBuf>,
    pub drop_allowed: bool,
}

impl DragDropState {
    pub fn start_drag(&mut self, item: DragItem, pos: egui::Pos2) {
        self.dragging_item = Some(item);
        self.drag_start_pos = Some(pos);
        self.is_dragging = true;
    }
    
    pub fn update_drag(&mut self, pos: egui::Pos2) {
        if let Some(start_pos) = self.drag_start_pos {
            // Only consider it dragging if moved more than threshold
            let distance = (pos - start_pos).length();
            if distance > 5.0 {
                self.is_dragging = true;
            }
        }
    }
    
    pub fn end_drag(&mut self) -> Option<(DragItem, PathBuf)> {
        if self.is_dragging && self.drop_allowed {
            if let (Some(item), Some(target)) = (self.dragging_item.take(), self.hover_target.take()) {
                self.reset();
                return Some((item, target));
            }
        }
        self.reset();
        None
    }
    
    pub fn cancel_drag(&mut self) {
        self.reset();
    }
    
    fn reset(&mut self) {
        self.dragging_item = None;
        self.drag_start_pos = None;
        self.is_dragging = false;
        self.hover_target = None;
        self.drop_allowed = false;
    }
    
    pub fn is_valid_drop_target(&self, source: &DragItem, target: &PathBuf) -> bool {
        match source {
            DragItem::Folder(source_path) => {
                // Can't drop folder into itself or its children
                !target.starts_with(source_path) && source_path != target
            }
            DragItem::File(_) => {
                // Files can be dropped into any folder
                true
            }
        }
    }
    
    pub fn render_drag_preview(&self, ui: &mut egui::Ui) {
        if !self.is_dragging {
            return;
        }
        
        if let Some(item) = &self.dragging_item {
            let cursor_pos = ui.ctx().pointer_hover_pos().unwrap_or_default();
            
            // Draw semi-transparent preview at cursor
            let preview_rect = egui::Rect::from_min_size(
                cursor_pos + egui::vec2(10.0, 10.0),
                egui::vec2(200.0, 20.0)
            );
            
            ui.painter().rect_filled(
                preview_rect,
                4.0,
                egui::Color32::from_rgba_premultiplied(50, 50, 50, 200)
            );
            
            let text = match item {
                DragItem::Folder(path) => {
                    format!("📁 {}", path.file_name().unwrap_or_default().to_string_lossy())
                }
                DragItem::File(path) => {
                    format!("📄 {}", path.file_name().unwrap_or_default().to_string_lossy())
                }
            };
            
            ui.painter().text(
                preview_rect.min + egui::vec2(5.0, 10.0),
                egui::Align2::LEFT_CENTER,
                text,
                egui::FontId::default(),
                egui::Color32::WHITE
            );
        }
    }
}

pub fn handle_drag_source(
    ui: &mut egui::Ui,
    drag_state: &mut DragDropState,
    item: DragItem,
    response: &egui::Response,
) {
    if response.drag_started_by(egui::PointerButton::Primary) {
        if let Some(pos) = ui.ctx().pointer_interact_pos() {
            drag_state.start_drag(item, pos);
        }
    }
    
    if response.dragged_by(egui::PointerButton::Primary) {
        if let Some(pos) = ui.ctx().pointer_latest_pos() {
            drag_state.update_drag(pos);
        }
    }
}

pub fn handle_drop_target(
    ui: &mut egui::Ui,
    drag_state: &mut DragDropState,
    target_path: PathBuf,
    response: &egui::Response,
) -> Option<(DragItem, PathBuf)> {
    if !drag_state.is_dragging {
        return None;
    }
    
    let is_hovering = response.hovered();
    
    if is_hovering {
        if let Some(item) = &drag_state.dragging_item {
            let is_valid = drag_state.is_valid_drop_target(item, &target_path);
            
            drag_state.hover_target = Some(target_path.clone());
            drag_state.drop_allowed = is_valid;
            
            // Visual feedback
            let color = if is_valid {
                egui::Color32::from_rgba_premultiplied(0, 255, 0, 30)
            } else {
                egui::Color32::from_rgba_premultiplied(255, 0, 0, 30)
            };
            
            ui.painter().rect_filled(
                response.rect,
                4.0,
                color
            );
        }
    } else if drag_state.hover_target.as_ref() == Some(&target_path) {
        drag_state.hover_target = None;
        drag_state.drop_allowed = false;
    }
    
    // Handle drop
    if response.drag_stopped_by(egui::PointerButton::Primary) && is_hovering {
        return drag_state.end_drag();
    }
    
    None
}