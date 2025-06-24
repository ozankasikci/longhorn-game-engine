//! Bounding volume calculations and spatial bounds

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

/// Bounding sphere
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

/// Combined bounding information
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub aabb: BoundingBox,
    pub sphere: BoundingSphere,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::empty()
    }
}

impl BoundingBox {
    /// Create an empty bounding box
    pub fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    /// Create a bounding box from min and max points
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create a bounding box from a center point and size
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// Create a bounding box from a single point
    pub fn from_point(point: Vec3) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    /// Create a bounding box from a list of points
    pub fn from_points(points: &[Vec3]) -> Self {
        if points.is_empty() {
            return Self::empty();
        }

        let mut min = points[0];
        let mut max = points[0];

        for &point in points.iter().skip(1) {
            min = min.min(point);
            max = max.max(point);
        }

        Self { min, max }
    }

    /// Get the center of the bounding box
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the size of the bounding box
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Get the half-size (extents) of the bounding box
    pub fn extents(&self) -> Vec3 {
        self.size() * 0.5
    }

    /// Get the volume of the bounding box
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }

    /// Get the surface area of the bounding box
    pub fn surface_area(&self) -> f32 {
        let size = self.size();
        2.0 * (size.x * size.y + size.y * size.z + size.z * size.x)
    }

    /// Check if the bounding box is valid (not empty)
    pub fn is_valid(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y && self.min.z <= self.max.z
    }

    /// Check if the bounding box is empty
    pub fn is_empty(&self) -> bool {
        !self.is_valid()
    }

    /// Expand the bounding box to include a point
    pub fn expand_to_include(&mut self, point: Vec3) {
        if self.is_empty() {
            self.min = point;
            self.max = point;
        } else {
            self.min = self.min.min(point);
            self.max = self.max.max(point);
        }
    }

    /// Expand the bounding box to include another bounding box
    pub fn expand_to_include_box(&mut self, other: &BoundingBox) {
        if other.is_valid() {
            if self.is_empty() {
                *self = *other;
            } else {
                self.min = self.min.min(other.min);
                self.max = self.max.max(other.max);
            }
        }
    }

    /// Expand the bounding box by a margin
    pub fn expand_by_margin(&mut self, margin: f32) {
        if self.is_valid() {
            let margin_vec = Vec3::splat(margin);
            self.min -= margin_vec;
            self.max += margin_vec;
        }
    }

    /// Get a copy expanded by a margin
    pub fn expanded_by_margin(&self, margin: f32) -> Self {
        let mut result = *self;
        result.expand_by_margin(margin);
        result
    }

    /// Check if the bounding box contains a point
    pub fn contains_point(&self, point: Vec3) -> bool {
        self.is_valid()
            && point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Check if the bounding box fully contains another bounding box
    pub fn contains_box(&self, other: &BoundingBox) -> bool {
        self.is_valid()
            && other.is_valid()
            && self.min.x <= other.min.x
            && self.max.x >= other.max.x
            && self.min.y <= other.min.y
            && self.max.y >= other.max.y
            && self.min.z <= other.min.z
            && self.max.z >= other.max.z
    }

    /// Check if the bounding box intersects with another bounding box
    pub fn intersects_box(&self, other: &BoundingBox) -> bool {
        self.is_valid()
            && other.is_valid()
            && self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Check if the bounding box intersects with a sphere
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        if !self.is_valid() {
            return false;
        }

        let closest_point = self.closest_point_to(center);
        (closest_point - center).length_squared() <= radius * radius
    }

    /// Get the closest point on the bounding box to a given point
    pub fn closest_point_to(&self, point: Vec3) -> Vec3 {
        Vec3::new(
            point.x.clamp(self.min.x, self.max.x),
            point.y.clamp(self.min.y, self.max.y),
            point.z.clamp(self.min.z, self.max.z),
        )
    }

    /// Get the distance from a point to the bounding box (0 if inside)
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        let closest = self.closest_point_to(point);
        (closest - point).length()
    }

    /// Get the squared distance from a point to the bounding box
    pub fn distance_squared_to_point(&self, point: Vec3) -> f32 {
        let closest = self.closest_point_to(point);
        (closest - point).length_squared()
    }

    /// Get the intersection of two bounding boxes
    pub fn intersection(&self, other: &BoundingBox) -> Option<BoundingBox> {
        if !self.intersects_box(other) {
            return None;
        }

        Some(BoundingBox {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        })
    }

    /// Get the union of two bounding boxes
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }

        BoundingBox {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Transform the bounding box by a matrix
    pub fn transformed(&self, transform: &glam::Mat4) -> BoundingBox {
        if self.is_empty() {
            return *self;
        }

        // Transform all 8 corners of the bounding box
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        let transformed_corners: Vec<Vec3> = corners
            .iter()
            .map(|&corner| transform.transform_point3(corner))
            .collect();

        BoundingBox::from_points(&transformed_corners)
    }

    /// Get the 8 corner points of the bounding box
    pub fn corners(&self) -> [Vec3; 8] {
        [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ]
    }
}

impl Default for BoundingSphere {
    fn default() -> Self {
        Self::empty()
    }
}

impl BoundingSphere {
    /// Create an empty bounding sphere
    pub fn empty() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 0.0,
        }
    }

    /// Create a bounding sphere from center and radius
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    /// Create a bounding sphere from a list of points
    pub fn from_points(points: &[Vec3]) -> Self {
        if points.is_empty() {
            return Self::empty();
        }

        // Use centroid as initial center
        let center = points.iter().fold(Vec3::ZERO, |acc, &p| acc + p) / points.len() as f32;

        // Find the maximum distance from center
        let max_distance_squared = points
            .iter()
            .map(|&p| (p - center).length_squared())
            .fold(0.0, f32::max);

        Self {
            center,
            radius: max_distance_squared.sqrt(),
        }
    }

    /// Create a bounding sphere from a bounding box
    pub fn from_aabb(aabb: &BoundingBox) -> Self {
        if aabb.is_empty() {
            return Self::empty();
        }

        let center = aabb.center();
        let radius = (aabb.max - center).length();

        Self { center, radius }
    }

    /// Check if the sphere is valid (radius >= 0)
    pub fn is_valid(&self) -> bool {
        self.radius >= 0.0
    }

    /// Check if the sphere is empty
    pub fn is_empty(&self) -> bool {
        self.radius <= 0.0
    }

    /// Check if the sphere contains a point
    pub fn contains_point(&self, point: Vec3) -> bool {
        (point - self.center).length_squared() <= self.radius * self.radius
    }

    /// Check if the sphere fully contains another sphere
    pub fn contains_sphere(&self, other: &BoundingSphere) -> bool {
        let distance = (other.center - self.center).length();
        distance + other.radius <= self.radius
    }

    /// Check if the sphere intersects with another sphere
    pub fn intersects_sphere(&self, other: &BoundingSphere) -> bool {
        let distance = (other.center - self.center).length();
        distance <= self.radius + other.radius
    }

    /// Check if the sphere intersects with a bounding box
    pub fn intersects_aabb(&self, aabb: &BoundingBox) -> bool {
        aabb.intersects_sphere(self.center, self.radius)
    }

    /// Expand the sphere to include a point
    pub fn expand_to_include(&mut self, point: Vec3) {
        let distance = (point - self.center).length();
        if distance > self.radius {
            self.radius = distance;
        }
    }

    /// Expand the sphere to include another sphere
    pub fn expand_to_include_sphere(&mut self, other: &BoundingSphere) {
        if other.is_empty() {
            return;
        }

        if self.is_empty() {
            *self = *other;
            return;
        }

        let distance = (other.center - self.center).length();
        let new_radius = (distance + other.radius).max(self.radius);

        if new_radius > self.radius {
            // Need to adjust center and radius
            let t = (new_radius - self.radius) / (2.0 * new_radius);
            self.center = self.center.lerp(other.center, t);
            self.radius = new_radius;
        }
    }

    /// Get the distance from a point to the sphere surface (negative if inside)
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        (point - self.center).length() - self.radius
    }

    /// Transform the sphere by a matrix (assuming uniform scale)
    pub fn transformed(&self, transform: &glam::Mat4) -> BoundingSphere {
        let new_center = transform.transform_point3(self.center);

        // Calculate scale factor (assumes uniform scaling)
        let scale_x = transform.x_axis.length();
        let scale_y = transform.y_axis.length();
        let scale_z = transform.z_axis.length();
        let scale = scale_x.max(scale_y).max(scale_z);

        BoundingSphere {
            center: new_center,
            radius: self.radius * scale,
        }
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self::empty()
    }
}

impl Bounds {
    /// Create empty bounds
    pub fn empty() -> Self {
        Self {
            aabb: BoundingBox::empty(),
            sphere: BoundingSphere::empty(),
        }
    }

    /// Create bounds from points
    pub fn from_points(points: &[Vec3]) -> Self {
        Self {
            aabb: BoundingBox::from_points(points),
            sphere: BoundingSphere::from_points(points),
        }
    }

    /// Create bounds from AABB
    pub fn from_aabb(aabb: BoundingBox) -> Self {
        Self {
            sphere: BoundingSphere::from_aabb(&aabb),
            aabb,
        }
    }

    /// Check if bounds are valid
    pub fn is_valid(&self) -> bool {
        self.aabb.is_valid() && self.sphere.is_valid()
    }

    /// Check if bounds are empty
    pub fn is_empty(&self) -> bool {
        self.aabb.is_empty() || self.sphere.is_empty()
    }

    /// Transform bounds
    pub fn transformed(&self, transform: &glam::Mat4) -> Self {
        Self {
            aabb: self.aabb.transformed(transform),
            sphere: self.sphere.transformed(transform),
        }
    }
}
