use egui::Ui;
use longhorn_core::{World, Name, EntityHandle};
use crate::EditorState;

pub struct SceneTreePanel;

impl SceneTreePanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, world: &World, state: &mut EditorState) {
        ui.heading("Scene Tree");
        ui.separator();

        let mut entities: Vec<_> = world.inner().iter().map(|entity_ref| entity_ref.entity()).collect();

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

                if ui.selectable_label(is_selected, &name).clicked() {
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
