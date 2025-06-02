//! Scene node hierarchy management

use engine_components_core::{Transform, Light};
use engine_camera_core::Camera;
use engine_geometry_core::MeshHandle;
use engine_materials_core::MaterialHandle;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

/// Unique identifier for scene nodes
pub type NodeId = u64;

/// Scene node representing an object in the scene hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: NodeId,
    pub name: String,
    pub transform: Transform,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub components: NodeComponents,
    pub visible: bool,
    pub enabled: bool,
}

/// Components that can be attached to scene nodes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeComponents {
    pub mesh: Option<MeshHandle>,
    pub material: Option<MaterialHandle>,
    pub camera: Option<Camera>,
    pub light: Option<Light>,
    pub custom: HashMap<String, serde_json::Value>,
}

/// Scene hierarchy manager
#[derive(Debug, Default)]
pub struct NodeHierarchy {
    nodes: HashMap<NodeId, SceneNode>,
    root_nodes: HashSet<NodeId>,
    next_id: NodeId,
}

impl SceneNode {
    /// Create a new scene node
    pub fn new(name: &str) -> Self {
        Self {
            id: 0, // Will be set by hierarchy
            name: name.to_string(),
            transform: Transform::default(),
            parent: None,
            children: Vec::new(),
            components: NodeComponents::default(),
            visible: true,
            enabled: true,
        }
    }
    
    /// Add a child node ID
    pub fn add_child(&mut self, child_id: NodeId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }
    
    /// Remove a child node ID
    pub fn remove_child(&mut self, child_id: NodeId) {
        self.children.retain(|&id| id != child_id);
    }
    
    /// Check if this node has a specific component type
    pub fn has_mesh(&self) -> bool {
        self.components.mesh.is_some()
    }
    
    pub fn has_material(&self) -> bool {
        self.components.material.is_some()
    }
    
    pub fn has_camera(&self) -> bool {
        self.components.camera.is_some()
    }
    
    pub fn has_light(&self) -> bool {
        self.components.light.is_some()
    }
    
    /// Check if node is renderable (has both mesh and material)
    pub fn is_renderable(&self) -> bool {
        self.visible && self.enabled && 
        self.components.mesh.is_some() && 
        self.components.material.is_some()
    }
}

impl NodeHierarchy {
    /// Create a new node hierarchy
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_nodes: HashSet::new(),
            next_id: 1,
        }
    }
    
    /// Add a new node to the hierarchy
    pub fn add_node(&mut self, mut node: SceneNode) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        
        node.id = id;
        
        if node.parent.is_none() {
            self.root_nodes.insert(id);
        }
        
        self.nodes.insert(id, node);
        id
    }
    
    /// Get a node by ID
    pub fn get_node(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(&id)
    }
    
    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(&id)
    }
    
    /// Remove a node and all its children
    pub fn remove_node(&mut self, id: NodeId) -> bool {
        if let Some(node) = self.nodes.get(&id) {
            // Collect children to remove
            let children = node.children.clone();
            let parent = node.parent;
            
            // Remove from parent's children list
            if let Some(parent_id) = parent {
                if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
                    parent_node.remove_child(id);
                }
            } else {
                self.root_nodes.remove(&id);
            }
            
            // Remove the node
            self.nodes.remove(&id);
            
            // Recursively remove children
            for child_id in children {
                self.remove_node(child_id);
            }
            
            true
        } else {
            false
        }
    }
    
    /// Set parent-child relationship
    pub fn set_parent(&mut self, child_id: NodeId, parent_id: Option<NodeId>) -> Result<(), crate::SceneError> {
        // Check for circular dependency
        if let Some(pid) = parent_id {
            if self.would_create_cycle(child_id, pid) {
                return Err(crate::SceneError::CircularDependency);
            }
        }
        
        // Remove from current parent
        if let Some(child) = self.nodes.get(&child_id) {
            if let Some(old_parent_id) = child.parent {
                if let Some(old_parent) = self.nodes.get_mut(&old_parent_id) {
                    old_parent.remove_child(child_id);
                }
            } else {
                self.root_nodes.remove(&child_id);
            }
        } else {
            return Err(crate::SceneError::NodeNotFound(child_id));
        }
        
        // Set new parent
        if let Some(child) = self.nodes.get_mut(&child_id) {
            child.parent = parent_id;
            
            if let Some(pid) = parent_id {
                if let Some(parent) = self.nodes.get_mut(&pid) {
                    parent.add_child(child_id);
                } else {
                    return Err(crate::SceneError::NodeNotFound(pid));
                }
            } else {
                self.root_nodes.insert(child_id);
            }
        }
        
        Ok(())
    }
    
    /// Check if setting parent would create a cycle
    fn would_create_cycle(&self, child_id: NodeId, parent_id: NodeId) -> bool {
        let mut current = Some(parent_id);
        while let Some(node_id) = current {
            if node_id == child_id {
                return true;
            }
            current = self.nodes.get(&node_id).and_then(|n| n.parent);
        }
        false
    }
    
    /// Get all root nodes
    pub fn root_nodes(&self) -> impl Iterator<Item = &SceneNode> {
        self.root_nodes.iter().filter_map(|&id| self.nodes.get(&id))
    }
    
    /// Get all children of a node
    pub fn get_children(&self, parent_id: NodeId) -> Vec<&SceneNode> {
        if let Some(parent) = self.nodes.get(&parent_id) {
            parent.children.iter()
                .filter_map(|&child_id| self.nodes.get(&child_id))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Traverse the hierarchy depth-first
    pub fn traverse_depth_first<F>(&self, node_id: NodeId, mut visitor: F) 
    where 
        F: FnMut(&SceneNode),
    {
        if let Some(node) = self.nodes.get(&node_id) {
            visitor(node);
            for &child_id in &node.children {
                self.traverse_depth_first(child_id, &mut visitor);
            }
        }
    }
    
    /// Find nodes by name
    pub fn find_by_name(&self, name: &str) -> Vec<&SceneNode> {
        self.nodes.values()
            .filter(|node| node.name == name)
            .collect()
    }
    
    /// Find nodes with specific component types
    pub fn find_renderable_nodes(&self) -> Vec<&SceneNode> {
        self.nodes.values()
            .filter(|node| node.is_renderable())
            .collect()
    }
    
    pub fn find_camera_nodes(&self) -> Vec<&SceneNode> {
        self.nodes.values()
            .filter(|node| node.has_camera())
            .collect()
    }
    
    pub fn find_light_nodes(&self) -> Vec<&SceneNode> {
        self.nodes.values()
            .filter(|node| node.has_light())
            .collect()
    }
    
    /// Get total node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}