use egui::Ui;
use longhorn_core::{World, Name, EntityHandle};
use crate::{EditorState, UiStateTracker};

pub struct SceneTreePanel;

impl SceneTreePanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        world: &World,
        state: &mut EditorState,
        ui_state: &mut UiStateTracker,
    ) {
        ui.heading("Scene Tree");
        ui.separator();

        let mut entities: Vec<_> = world.inner().iter().map(|entity_ref| entity_ref.entity()).collect();

        // Handle pending select by path
        if let Some(path) = ui_state.take_pending_tree_select() {
            let entity_name = path.split('/').last().unwrap_or(&path);
            if let Some(entity) = entities.iter().find(|e| {
                let handle = EntityHandle::new(**e);
                world.get::<Name>(handle)
                    .ok()
                    .map(|n| n.0 == entity_name)
                    .unwrap_or(false)
            }) {
                state.select(Some(*entity));
            }
        }

        if entities.is_empty() {
            ui.label("(No entities)");
        } else {
            entities.sort_by_key(|e| e.id());

            for entity in entities {
                let handle = EntityHandle::new(entity);
                let name = world.get::<Name>(handle)
                    .ok()
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|| format!("Entity {}", entity.id()));

                let is_selected = state.is_selected(entity);

                // Register as clickable element for remote control
                let element_id = format!("entity_{}", entity.id());
                ui_state.register_clickable(&element_id, &name, "selectable");

                // Check if this element should be triggered
                let should_trigger = ui_state.take_pending_trigger()
                    .map(|id| id == element_id)
                    .unwrap_or(false);

                if ui.selectable_label(is_selected, &name).clicked() || should_trigger {
                    state.select(Some(entity));
                }
            }
        }
    }
}

impl Default for SceneTreePanel {
    fn default() -> Self {
        Self::new()
    }
}
