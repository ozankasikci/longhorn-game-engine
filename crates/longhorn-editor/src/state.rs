use longhorn_core::EntityHandle;

pub struct EditorState {
    pub selected_entity: Option<EntityHandle>,
    pub game_path: Option<String>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            selected_entity: None,
            game_path: None,
        }
    }

    pub fn select(&mut self, entity: EntityHandle) {
        self.selected_entity = Some(entity);
    }

    pub fn is_selected(&self, entity: EntityHandle) -> bool {
        self.selected_entity == Some(entity)
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}
