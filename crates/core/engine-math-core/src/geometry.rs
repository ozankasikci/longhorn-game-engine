//! Geometric calculations and collision detection utilities

use glam::{Vec2, Vec3};

/// 2D Ray structure
#[derive(Debug, Clone, Copy)]
pub struct Ray2D {
    pub origin: Vec2,
    pub direction: Vec2,
}

/// 3D Ray structure
#[derive(Debug, Clone, Copy)]
pub struct Ray3D {
    pub origin: Vec3,
    pub direction: Vec3,
}

/// 2D Line segment
#[derive(Debug, Clone, Copy)]
pub struct LineSegment2D {
    pub start: Vec2,
    pub end: Vec2,
}

/// 3D Line segment
#[derive(Debug, Clone, Copy)]
pub struct LineSegment3D {
    pub start: Vec3,
    pub end: Vec3,
}

/// 2D Axis-Aligned Bounding Box
#[derive(Debug, Clone, Copy)]
pub struct AABB2D {
    pub min: Vec2,
    pub max: Vec2,
}

/// 3D Axis-Aligned Bounding Box
#[derive(Debug, Clone, Copy)]
pub struct AABB3D {
    pub min: Vec3,
    pub max: Vec3,
}

/// 2D Circle
#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

/// 3D Sphere
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

/// 3D Plane defined by normal and distance from origin
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Ray2D {
    pub fn new(origin: Vec2, direction: Vec2) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
    
    pub fn point_at(&self, t: f32) -> Vec2 {
        self.origin + self.direction * t
    }
}

impl Ray3D {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
    
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

impl AABB2D {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }
    
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }
    
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }
    
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
    
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y
    }
    
    pub fn intersects(&self, other: &AABB2D) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y
    }
    
    pub fn expand(&mut self, point: Vec2) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }
}

impl AABB3D {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
    
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }
    
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
    
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
    
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
    
    pub fn intersects(&self, other: &AABB3D) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }
}

impl Circle {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }
    
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.center.distance_squared(point) <= self.radius * self.radius
    }
    
    pub fn intersects_circle(&self, other: &Circle) -> bool {
        let distance = self.center.distance(other.center);
        distance <= (self.radius + other.radius)
    }
    
    pub fn intersects_aabb(&self, aabb: &AABB2D) -> bool {
        let closest_point = Vec2::new(
            self.center.x.clamp(aabb.min.x, aabb.max.x),
            self.center.y.clamp(aabb.min.y, aabb.max.y),
        );
        self.contains_point(closest_point)
    }
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
    
    pub fn contains_point(&self, point: Vec3) -> bool {
        self.center.distance_squared(point) <= self.radius * self.radius
    }
    
    pub fn intersects_sphere(&self, other: &Sphere) -> bool {
        let distance = self.center.distance(other.center);
        distance <= (self.radius + other.radius)
    }
    
    pub fn intersects_aabb(&self, aabb: &AABB3D) -> bool {
        let closest_point = Vec3::new(
            self.center.x.clamp(aabb.min.x, aabb.max.x),
            self.center.y.clamp(aabb.min.y, aabb.max.y),
            self.center.z.clamp(aabb.min.z, aabb.max.z),
        );
        self.contains_point(closest_point)
    }
}

impl Plane {
    pub fn from_point_normal(point: Vec3, normal: Vec3) -> Self {
        let normal = normal.normalize();
        Self {
            normal,
            distance: normal.dot(point),
        }
    }
    
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }
    
    pub fn closest_point(&self, point: Vec3) -> Vec3 {
        point - self.normal * self.distance_to_point(point)
    }
}

// Intersection functions

/// Test if two line segments intersect in 2D
pub fn line_segments_intersect_2d(seg1: LineSegment2D, seg2: LineSegment2D) -> Option<Vec2> {
    let d1 = seg1.end - seg1.start;
    let d2 = seg2.end - seg2.start;
    let d3 = seg1.start - seg2.start;
    
    let cross = d1.x * d2.y - d1.y * d2.x;
    
    if cross.abs() < f32::EPSILON {
        return None; // Lines are parallel
    }
    
    let t1 = (d3.x * d2.y - d3.y * d2.x) / cross;
    let t2 = (d3.x * d1.y - d3.y * d1.x) / cross;
    
    if t1 >= 0.0 && t1 <= 1.0 && t2 >= 0.0 && t2 <= 1.0 {
        Some(seg1.start + d1 * t1)
    } else {
        None
    }
}

/// Ray-plane intersection
pub fn ray_plane_intersection(ray: Ray3D, plane: Plane) -> Option<f32> {
    let denom = plane.normal.dot(ray.direction);
    
    if denom.abs() < f32::EPSILON {
        return None; // Ray is parallel to plane
    }
    
    let t = (plane.distance - plane.normal.dot(ray.origin)) / denom;
    
    if t >= 0.0 {
        Some(t)
    } else {
        None
    }
}

/// Ray-sphere intersection
pub fn ray_sphere_intersection(ray: Ray3D, sphere: Sphere) -> Option<f32> {
    let oc = ray.origin - sphere.center;
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.dot(oc) - sphere.radius * sphere.radius;
    
    let discriminant = b * b - 4.0 * a * c;
    
    if discriminant < 0.0 {
        return None;
    }
    
    let sqrt_discriminant = discriminant.sqrt();
    let t1 = (-b - sqrt_discriminant) / (2.0 * a);
    let t2 = (-b + sqrt_discriminant) / (2.0 * a);
    
    if t1 >= 0.0 {
        Some(t1)
    } else if t2 >= 0.0 {
        Some(t2)
    } else {
        None
    }
}

/// Ray-AABB intersection (3D)
pub fn ray_aabb_intersection(ray: Ray3D, aabb: AABB3D) -> Option<f32> {
    let inv_dir = Vec3::new(
        if ray.direction.x.abs() < f32::EPSILON { f32::INFINITY } else { 1.0 / ray.direction.x },
        if ray.direction.y.abs() < f32::EPSILON { f32::INFINITY } else { 1.0 / ray.direction.y },
        if ray.direction.z.abs() < f32::EPSILON { f32::INFINITY } else { 1.0 / ray.direction.z },
    );
    
    let t1 = (aabb.min - ray.origin) * inv_dir;
    let t2 = (aabb.max - ray.origin) * inv_dir;
    
    let tmin = t1.min(t2);
    let tmax = t1.max(t2);
    
    let tmin_val = tmin.x.max(tmin.y).max(tmin.z);
    let tmax_val = tmax.x.min(tmax.y).min(tmax.z);
    
    if tmax_val < 0.0 || tmin_val > tmax_val {
        None
    } else {
        Some(if tmin_val >= 0.0 { tmin_val } else { tmax_val })
    }
}

/// Point to line distance in 2D
pub fn point_to_line_distance_2d(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;
    
    let line_length_squared = line_vec.length_squared();
    
    if line_length_squared < f32::EPSILON {
        return point_vec.length(); // Line is a point
    }
    
    let t = point_vec.dot(line_vec) / line_length_squared;
    let projection = line_start + line_vec * t.clamp(0.0, 1.0);
    
    point.distance(projection)
}

/// Barycentric coordinates for a point in a triangle
pub fn barycentric_coordinates(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> Vec3 {
    let v0 = c - a;
    let v1 = b - a;
    let v2 = point - a;
    
    let dot00 = v0.dot(v0);
    let dot01 = v0.dot(v1);
    let dot02 = v0.dot(v2);
    let dot11 = v1.dot(v1);
    let dot12 = v1.dot(v2);
    
    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
    
    Vec3::new(1.0 - u - v, v, u)
}

/// Check if point is inside triangle using barycentric coordinates
pub fn point_in_triangle(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let bary = barycentric_coordinates(point, a, b, c);
    bary.x >= 0.0 && bary.y >= 0.0 && bary.z >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb2d() {
        let aabb = AABB2D::new(Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0));
        
        assert!(aabb.contains_point(Vec2::new(1.0, 1.0)));
        assert!(!aabb.contains_point(Vec2::new(3.0, 3.0)));
        
        let center = aabb.center();
        assert!((center - Vec2::new(1.0, 1.0)).length() < f32::EPSILON);
    }

    #[test]
    fn test_circle() {
        let circle = Circle::new(Vec2::new(0.0, 0.0), 1.0);
        
        assert!(circle.contains_point(Vec2::new(0.5, 0.5)));
        assert!(!circle.contains_point(Vec2::new(2.0, 0.0)));
        
        let other_circle = Circle::new(Vec2::new(1.5, 0.0), 1.0);
        assert!(circle.intersects_circle(&other_circle));
    }

    #[test]
    fn test_ray_sphere_intersection() {
        let ray = Ray3D::new(Vec3::new(-2.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
        
        let intersection = ray_sphere_intersection(ray, sphere);
        assert!(intersection.is_some());
        assert!((intersection.unwrap() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_point_in_triangle() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(1.0, 0.0);
        let c = Vec2::new(0.5, 1.0);
        
        assert!(point_in_triangle(Vec2::new(0.5, 0.3), a, b, c));
        assert!(!point_in_triangle(Vec2::new(0.5, -0.1), a, b, c));
    }
}