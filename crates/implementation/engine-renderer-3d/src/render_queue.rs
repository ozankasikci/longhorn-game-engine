//! Render Queue and Object Sorting
//!
//! This module provides efficient sorting and batching of render objects
//! to minimize state changes and optimize GPU performance.

use crate::{Camera, RenderObject};
use glam::Vec4Swizzles;
use std::cmp::Ordering;

/// Sorting strategy for render objects
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    /// No sorting - render in the order objects were added
    None,
    /// Front-to-back sorting (good for opaque objects, enables early-z rejection)
    FrontToBack,
    /// Back-to-front sorting (required for alpha blending)
    BackToFront,
    /// Sort by material first, then by distance (minimizes state changes)
    MaterialThenDistance,
    /// Sort by material only (maximizes batching)
    Material,
}

/// Information about a render object used for sorting
#[derive(Debug, Clone)]
pub struct RenderItem {
    /// The render object to draw
    pub object: RenderObject,
    /// Distance from camera (for distance-based sorting)
    pub camera_distance: f32,
    /// Estimated GPU cost (for optimization)
    pub draw_cost: u32,
}

impl RenderItem {
    /// Create a new render item from a render object and camera
    pub fn new(object: RenderObject, camera: &Camera) -> Self {
        let camera_distance = Self::calculate_camera_distance(&object, camera);
        let draw_cost = Self::estimate_draw_cost(&object);

        Self {
            object,
            camera_distance,
            draw_cost,
        }
    }

    /// Calculate distance from camera to object
    fn calculate_camera_distance(object: &RenderObject, camera: &Camera) -> f32 {
        let object_position = object.transform.col(3).xyz();
        camera.position.distance(object_position)
    }

    /// Estimate the GPU cost of drawing this object
    fn estimate_draw_cost(object: &RenderObject) -> u32 {
        // Simple heuristic: base cost + material switches
        // In a real engine this would be more sophisticated
        let base_cost = 100;
        let material_cost = object.material_id * 10; // Material switches are expensive
        base_cost + material_cost
    }
}

/// Render queue that sorts and batches objects for efficient rendering
pub struct RenderQueue {
    /// Render items to be drawn
    items: Vec<RenderItem>,
    /// Current sorting mode
    sort_mode: SortMode,
    /// Whether the queue needs to be re-sorted
    needs_sorting: bool,
}

impl RenderQueue {
    /// Create a new empty render queue
    pub fn new(sort_mode: SortMode) -> Self {
        Self {
            items: Vec::new(),
            sort_mode,
            needs_sorting: false,
        }
    }

    /// Add a render object to the queue
    pub fn add_object(&mut self, object: RenderObject, camera: &Camera) {
        let item = RenderItem::new(object, camera);
        self.items.push(item);
        self.needs_sorting = true;
    }

    /// Add multiple render objects to the queue
    pub fn add_objects(&mut self, objects: Vec<RenderObject>, camera: &Camera) {
        for object in objects {
            let item = RenderItem::new(object, camera);
            self.items.push(item);
        }
        self.needs_sorting = true;
    }

    /// Clear all items from the queue
    pub fn clear(&mut self) {
        self.items.clear();
        self.needs_sorting = false;
    }

    /// Set the sorting mode
    pub fn set_sort_mode(&mut self, sort_mode: SortMode) {
        if self.sort_mode != sort_mode {
            self.sort_mode = sort_mode;
            self.needs_sorting = true;
        }
    }

    /// Get the current sorting mode
    pub fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    /// Sort the render queue according to the current sort mode
    pub fn sort(&mut self) {
        if !self.needs_sorting {
            return;
        }

        match self.sort_mode {
            SortMode::None => {
                // No sorting needed
            }
            SortMode::FrontToBack => {
                self.items.sort_by(|a, b| {
                    a.camera_distance
                        .partial_cmp(&b.camera_distance)
                        .unwrap_or(Ordering::Equal)
                });
            }
            SortMode::BackToFront => {
                self.items.sort_by(|a, b| {
                    b.camera_distance
                        .partial_cmp(&a.camera_distance)
                        .unwrap_or(Ordering::Equal)
                });
            }
            SortMode::MaterialThenDistance => {
                self.items.sort_by(|a, b| {
                    // First sort by material ID
                    match a.object.material_id.cmp(&b.object.material_id) {
                        Ordering::Equal => {
                            // Then by distance (front to back)
                            a.camera_distance
                                .partial_cmp(&b.camera_distance)
                                .unwrap_or(Ordering::Equal)
                        }
                        other => other,
                    }
                });
            }
            SortMode::Material => {
                self.items
                    .sort_by(|a, b| a.object.material_id.cmp(&b.object.material_id));
            }
        }

        self.needs_sorting = false;
    }

    /// Get the sorted render items
    pub fn get_sorted_items(&mut self) -> &[RenderItem] {
        self.sort();
        &self.items
    }

    /// Get render statistics
    pub fn get_stats(&self) -> RenderQueueStats {
        let mut material_groups = std::collections::HashMap::new();
        let mut total_draw_cost = 0;

        for item in &self.items {
            *material_groups.entry(item.object.material_id).or_insert(0) += 1;
            total_draw_cost += item.draw_cost;
        }

        RenderQueueStats {
            total_objects: self.items.len(),
            material_groups: material_groups.len(),
            estimated_draw_calls: self.estimate_draw_calls(),
            total_draw_cost,
            needs_sorting: self.needs_sorting,
        }
    }

    /// Estimate the number of draw calls needed
    fn estimate_draw_calls(&self) -> usize {
        if self.items.is_empty() {
            return 0;
        }

        match self.sort_mode {
            SortMode::Material | SortMode::MaterialThenDistance => {
                // Count unique materials
                let mut materials = std::collections::HashSet::new();
                for item in &self.items {
                    materials.insert(item.object.material_id);
                }
                materials.len()
            }
            _ => {
                // Assume each object is a separate draw call
                self.items.len()
            }
        }
    }

    /// Get items grouped by material (for batched rendering)
    pub fn get_material_groups(&mut self) -> Vec<MaterialGroup> {
        self.sort();

        let mut groups = Vec::new();
        let mut current_material_id = None;
        let mut current_group_start = 0;

        for (i, item) in self.items.iter().enumerate() {
            match current_material_id {
                Some(material_id) if material_id == item.object.material_id => {
                    // Continue current group
                }
                _ => {
                    // Start new group
                    if let Some(material_id) = current_material_id {
                        groups.push(MaterialGroup {
                            material_id,
                            start_index: current_group_start,
                            count: i - current_group_start,
                        });
                    }
                    current_material_id = Some(item.object.material_id);
                    current_group_start = i;
                }
            }
        }

        // Add the last group
        if let Some(material_id) = current_material_id {
            groups.push(MaterialGroup {
                material_id,
                start_index: current_group_start,
                count: self.items.len() - current_group_start,
            });
        }

        groups
    }
}

/// Statistics about the render queue
#[derive(Debug, Clone)]
pub struct RenderQueueStats {
    /// Total number of objects in the queue
    pub total_objects: usize,
    /// Number of different materials
    pub material_groups: usize,
    /// Estimated number of draw calls
    pub estimated_draw_calls: usize,
    /// Total estimated draw cost
    pub total_draw_cost: u32,
    /// Whether the queue needs sorting
    pub needs_sorting: bool,
}

impl std::fmt::Display for RenderQueueStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "RenderQueueStats {{ objects: {}, materials: {}, draw_calls: {}, cost: {}, needs_sort: {} }}",
            self.total_objects,
            self.material_groups,
            self.estimated_draw_calls,
            self.total_draw_cost,
            self.needs_sorting
        )
    }
}

/// A group of objects that share the same material
#[derive(Debug, Clone)]
pub struct MaterialGroup {
    /// Material ID for this group
    pub material_id: u32,
    /// Starting index in the render queue
    pub start_index: usize,
    /// Number of objects in this group
    pub count: usize,
}

impl MaterialGroup {
    /// Get the range of indices for this group
    pub fn range(&self) -> std::ops::Range<usize> {
        self.start_index..(self.start_index + self.count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3};

    fn create_test_object(material_id: u32, position: Vec3) -> RenderObject {
        let transform = Mat4::from_translation(position);
        RenderObject::new(transform, 0, material_id)
    }

    fn create_test_camera() -> Camera {
        Camera::new(16.0 / 9.0)
    }

    #[test]
    fn test_render_queue_creation() {
        let queue = RenderQueue::new(SortMode::FrontToBack);
        assert_eq!(queue.sort_mode(), SortMode::FrontToBack);
        assert_eq!(queue.items.len(), 0);
    }

    #[test]
    fn test_add_object() {
        let mut queue = RenderQueue::new(SortMode::None);
        let camera = create_test_camera();
        let object = create_test_object(1, Vec3::new(0.0, 0.0, -5.0));

        queue.add_object(object, &camera);
        assert_eq!(queue.items.len(), 1);
        assert!(queue.needs_sorting);
    }

    #[test]
    fn test_front_to_back_sorting() {
        let mut queue = RenderQueue::new(SortMode::FrontToBack);
        let camera = create_test_camera();

        // Add objects at different distances
        queue.add_object(create_test_object(1, Vec3::new(0.0, 0.0, -10.0)), &camera);
        queue.add_object(create_test_object(1, Vec3::new(0.0, 0.0, -5.0)), &camera);
        queue.add_object(create_test_object(1, Vec3::new(0.0, 0.0, -15.0)), &camera);

        let items = queue.get_sorted_items();

        // Should be sorted front-to-back (smallest distance first)
        assert!(items[0].camera_distance < items[1].camera_distance);
        assert!(items[1].camera_distance < items[2].camera_distance);
    }

    #[test]
    fn test_back_to_front_sorting() {
        let mut queue = RenderQueue::new(SortMode::BackToFront);
        let camera = create_test_camera();

        // Add objects at different distances
        queue.add_object(create_test_object(1, Vec3::new(0.0, 0.0, -10.0)), &camera);
        queue.add_object(create_test_object(1, Vec3::new(0.0, 0.0, -5.0)), &camera);
        queue.add_object(create_test_object(1, Vec3::new(0.0, 0.0, -15.0)), &camera);

        let items = queue.get_sorted_items();

        // Should be sorted back-to-front (largest distance first)
        assert!(items[0].camera_distance > items[1].camera_distance);
        assert!(items[1].camera_distance > items[2].camera_distance);
    }

    #[test]
    fn test_material_sorting() {
        let mut queue = RenderQueue::new(SortMode::Material);
        let camera = create_test_camera();

        // Add objects with different materials
        queue.add_object(create_test_object(3, Vec3::new(0.0, 0.0, -5.0)), &camera);
        queue.add_object(create_test_object(1, Vec3::new(0.0, 0.0, -5.0)), &camera);
        queue.add_object(create_test_object(2, Vec3::new(0.0, 0.0, -5.0)), &camera);

        let items = queue.get_sorted_items();

        // Should be sorted by material ID
        assert_eq!(items[0].object.material_id, 1);
        assert_eq!(items[1].object.material_id, 2);
        assert_eq!(items[2].object.material_id, 3);
    }

    #[test]
    fn test_material_groups() {
        let mut queue = RenderQueue::new(SortMode::Material);
        let camera = create_test_camera();

        // Add objects with different materials
        queue.add_object(create_test_object(1, Vec3::new(0.0, 0.0, -5.0)), &camera);
        queue.add_object(create_test_object(1, Vec3::new(1.0, 0.0, -5.0)), &camera);
        queue.add_object(create_test_object(2, Vec3::new(2.0, 0.0, -5.0)), &camera);
        queue.add_object(create_test_object(2, Vec3::new(3.0, 0.0, -5.0)), &camera);
        queue.add_object(create_test_object(2, Vec3::new(4.0, 0.0, -5.0)), &camera);

        let groups = queue.get_material_groups();

        // Should have 2 groups
        assert_eq!(groups.len(), 2);

        // First group: material 1, 2 objects
        assert_eq!(groups[0].material_id, 1);
        assert_eq!(groups[0].count, 2);

        // Second group: material 2, 3 objects
        assert_eq!(groups[1].material_id, 2);
        assert_eq!(groups[1].count, 3);
    }

    #[test]
    fn test_queue_stats() {
        let mut queue = RenderQueue::new(SortMode::Material);
        let camera = create_test_camera();

        // Add objects with different materials
        queue.add_object(create_test_object(1, Vec3::new(0.0, 0.0, -5.0)), &camera);
        queue.add_object(create_test_object(2, Vec3::new(1.0, 0.0, -5.0)), &camera);
        queue.add_object(create_test_object(1, Vec3::new(2.0, 0.0, -5.0)), &camera);

        let stats = queue.get_stats();

        assert_eq!(stats.total_objects, 3);
        assert_eq!(stats.material_groups, 2);
        assert!(stats.needs_sorting);
    }
}
