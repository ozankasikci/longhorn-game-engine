//! Common types for the editor framework

use serde::{Deserialize, Serialize};

/// Scene object for editor state
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SceneObject {
    pub name: String,
}

/// Hierarchy object representation
#[derive(Clone, Debug)]
pub struct HierarchyObject {
    pub name: String,
    pub object_type: ObjectType,
    pub children: Option<Vec<HierarchyObject>>,
}

impl HierarchyObject {
    pub fn new(name: &str, object_type: ObjectType) -> Self {
        Self {
            name: name.to_string(),
            object_type,
            children: None,
        }
    }

    pub fn parent(name: &str, children: Vec<HierarchyObject>) -> Self {
        Self {
            name: name.to_string(),
            object_type: ObjectType::GameObject,
            children: Some(children),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ObjectType {
    GameObject,
    Camera,
    Light,
}
