use engine_ecs_core::{Entity, World};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use super::SnapshotError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMemento {
    pub entity_id: u32,
    pub components: HashMap<String, Vec<u8>>, // Component type name -> serialized data
}

impl EntityMemento {
    pub fn capture(world: &World, entity: Entity) -> Self {
        let mut components = HashMap::new();
        
        // Capture Transform component
        if let Some(transform) = world.get_component::<engine_components_3d::Transform>(entity) {
            if let Ok(serialized) = bincode::serialize(&*transform) {
                components.insert("Transform".to_string(), serialized);
            }
        }
        
        // Capture Name component
        if let Some(name) = world.get_component::<engine_components_ui::Name>(entity) {
            if let Ok(serialized) = bincode::serialize(&*name) {
                components.insert("Name".to_string(), serialized);
            }
        }
        
        // Capture TypeScriptScript component
        if let Some(script) = world.get_component::<engine_scripting::components::TypeScriptScript>(entity) {
            if let Ok(serialized) = bincode::serialize(&*script) {
                components.insert("TypeScriptScript".to_string(), serialized);
            }
        }
        
        Self {
            entity_id: entity.id(),
            components,
        }
    }
    
    pub fn restore(&self, world: &mut World, entity: Entity) -> Result<(), SnapshotError> {
        // Restore Transform component
        if let Some(transform_data) = self.components.get("Transform") {
            let transform: engine_components_3d::Transform = bincode::deserialize(transform_data)
                .map_err(|e| SnapshotError::DeserializationError(e.to_string()))?;
            
            if let Some(mut existing_transform) = world.get_component_mut::<engine_components_3d::Transform>(entity) {
                *existing_transform = transform;
            } else {
                world.add_component(entity, transform)
                    .map_err(|e| SnapshotError::SerializationError(format!("Failed to add Transform: {:?}", e)))?;
            }
        }
        
        // Restore Name component
        if let Some(name_data) = self.components.get("Name") {
            let name: engine_components_ui::Name = bincode::deserialize(name_data)
                .map_err(|e| SnapshotError::DeserializationError(e.to_string()))?;
            
            if let Some(mut existing_name) = world.get_component_mut::<engine_components_ui::Name>(entity) {
                *existing_name = name;
            } else {
                world.add_component(entity, name)
                    .map_err(|e| SnapshotError::SerializationError(format!("Failed to add Name: {:?}", e)))?;
            }
        }
        
        // Restore TypeScriptScript component
        if let Some(script_data) = self.components.get("TypeScriptScript") {
            let script: engine_scripting::components::TypeScriptScript = bincode::deserialize(script_data)
                .map_err(|e| SnapshotError::DeserializationError(e.to_string()))?;
            
            if let Some(mut existing_script) = world.get_component_mut::<engine_scripting::components::TypeScriptScript>(entity) {
                *existing_script = script;
            } else {
                world.add_component(entity, script)
                    .map_err(|e| SnapshotError::SerializationError(format!("Failed to add TypeScriptScript: {:?}", e)))?;
            }
        }
        
        Ok(())
    }
}