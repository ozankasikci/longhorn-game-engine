//! Scene management and organization

use crate::{NodeHierarchy, NodeId, SceneNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Handle for scene resources
pub type SceneHandle = u64;

/// Scene containing a hierarchy of nodes
#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub handle: SceneHandle,
    #[serde(skip)]
    pub hierarchy: NodeHierarchy,
    pub metadata: SceneMetadata,
}

/// Scene metadata and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMetadata {
    pub author: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub version: String,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub custom_properties: HashMap<String, serde_json::Value>,
}

/// Scene loading and management
#[derive(Debug)]
pub struct SceneManager {
    scenes: HashMap<SceneHandle, Scene>,
    active_scene: Option<SceneHandle>,
    next_handle: SceneHandle,
}

impl Default for SceneMetadata {
    fn default() -> Self {
        Self {
            author: None,
            description: None,
            tags: Vec::new(),
            version: "1.0.0".to_string(),
            created: None,
            modified: None,
            custom_properties: HashMap::new(),
        }
    }
}

impl Scene {
    /// Create a new empty scene
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            handle: 0, // Will be set by scene manager
            hierarchy: NodeHierarchy::new(),
            metadata: SceneMetadata::default(),
        }
    }

    /// Create a scene with metadata
    pub fn with_metadata(name: &str, metadata: SceneMetadata) -> Self {
        Self {
            name: name.to_string(),
            handle: 0,
            hierarchy: NodeHierarchy::new(),
            metadata,
        }
    }

    /// Add a node to the scene
    pub fn add_node(&mut self, node: SceneNode) -> NodeId {
        self.hierarchy.add_node(node)
    }

    /// Remove a node from the scene
    pub fn remove_node(&mut self, node_id: NodeId) -> bool {
        self.hierarchy.remove_node(node_id)
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: NodeId) -> Option<&SceneNode> {
        self.hierarchy.get_node(node_id)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut SceneNode> {
        self.hierarchy.get_node_mut(node_id)
    }

    /// Set parent-child relationship
    pub fn set_parent(
        &mut self,
        child_id: NodeId,
        parent_id: Option<NodeId>,
    ) -> Result<(), crate::SceneError> {
        self.hierarchy.set_parent(child_id, parent_id)
    }

    /// Get all root nodes
    pub fn root_nodes(&self) -> impl Iterator<Item = &SceneNode> {
        self.hierarchy.root_nodes()
    }

    /// Find nodes by name
    pub fn find_by_name(&self, name: &str) -> Vec<&SceneNode> {
        self.hierarchy.find_by_name(name)
    }

    /// Get all renderable nodes
    pub fn renderable_nodes(&self) -> Vec<&SceneNode> {
        self.hierarchy.find_renderable_nodes()
    }

    /// Get all camera nodes
    pub fn camera_nodes(&self) -> Vec<&SceneNode> {
        self.hierarchy.find_camera_nodes()
    }

    /// Get all light nodes
    pub fn light_nodes(&self) -> Vec<&SceneNode> {
        self.hierarchy.find_light_nodes()
    }

    /// Get primary camera (first camera found)
    pub fn primary_camera(&self) -> Option<&SceneNode> {
        self.camera_nodes().into_iter().next()
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.hierarchy.node_count()
    }

    /// Clear all nodes
    pub fn clear(&mut self) {
        self.hierarchy = NodeHierarchy::new();
    }

    /// Clone scene data (without hierarchy for serialization)
    pub fn clone_data(&self) -> Scene {
        Scene {
            name: self.name.clone(),
            handle: self.handle,
            hierarchy: NodeHierarchy::new(), // Empty hierarchy
            metadata: self.metadata.clone(),
        }
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneManager {
    /// Create a new scene manager
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
            active_scene: None,
            next_handle: 1,
        }
    }

    /// Create and add a new scene
    pub fn create_scene(&mut self, name: &str) -> SceneHandle {
        let handle = self.next_handle;
        self.next_handle += 1;

        let mut scene = Scene::new(name);
        scene.handle = handle;

        self.scenes.insert(handle, scene);
        handle
    }

    /// Add an existing scene
    pub fn add_scene(&mut self, mut scene: Scene) -> SceneHandle {
        let handle = self.next_handle;
        self.next_handle += 1;

        scene.handle = handle;
        self.scenes.insert(handle, scene);
        handle
    }

    /// Remove a scene
    pub fn remove_scene(&mut self, handle: SceneHandle) -> bool {
        if self.active_scene == Some(handle) {
            self.active_scene = None;
        }
        self.scenes.remove(&handle).is_some()
    }

    /// Get a scene by handle
    pub fn get_scene(&self, handle: SceneHandle) -> Option<&Scene> {
        self.scenes.get(&handle)
    }

    /// Get a mutable scene by handle
    pub fn get_scene_mut(&mut self, handle: SceneHandle) -> Option<&mut Scene> {
        self.scenes.get_mut(&handle)
    }

    /// Set active scene
    pub fn set_active_scene(&mut self, handle: SceneHandle) -> Result<(), crate::SceneError> {
        if self.scenes.contains_key(&handle) {
            self.active_scene = Some(handle);
            Ok(())
        } else {
            Err(crate::SceneError::SceneNotLoaded(handle))
        }
    }

    /// Get active scene
    pub fn active_scene(&self) -> Option<&Scene> {
        self.active_scene.and_then(|h| self.scenes.get(&h))
    }

    /// Get active scene (mutable)
    pub fn active_scene_mut(&mut self) -> Option<&mut Scene> {
        self.active_scene.and_then(|h| self.scenes.get_mut(&h))
    }

    /// Get active scene handle
    pub fn active_scene_handle(&self) -> Option<SceneHandle> {
        self.active_scene
    }

    /// List all scenes
    pub fn list_scenes(&self) -> Vec<(SceneHandle, &str)> {
        self.scenes
            .iter()
            .map(|(&handle, scene)| (handle, scene.name.as_str()))
            .collect()
    }

    /// Get scene count
    pub fn scene_count(&self) -> usize {
        self.scenes.len()
    }

    /// Find scene by name
    pub fn find_by_name(&self, name: &str) -> Option<SceneHandle> {
        self.scenes
            .iter()
            .find(|(_, scene)| scene.name == name)
            .map(|(&handle, _)| handle)
    }
}
