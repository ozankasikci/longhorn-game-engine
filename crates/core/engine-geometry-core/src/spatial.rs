//! Spatial queries and geometric operations

use crate::BoundingBox;
use glam::Vec3;
// use serde::{Serialize, Deserialize};

/// Ray for raycasting operations
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

/// Plane for clipping and intersection tests
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

/// Frustum for camera culling (6 planes: left, right, top, bottom, near, far)
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

/// Spatial query operations
pub struct SpatialQuery;

/// Ray intersection result
#[derive(Debug, Clone, Copy)]
pub struct RayIntersection {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

impl Ray {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get a point along the ray at distance t
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Test intersection with a plane
    pub fn intersect_plane(&self, plane: &Plane) -> Option<f32> {
        let denom = self.direction.dot(plane.normal);
        if denom.abs() < f32::EPSILON {
            return None; // Ray is parallel to plane
        }

        let t = (plane.distance - self.origin.dot(plane.normal)) / denom;
        if t >= 0.0 {
            Some(t)
        } else {
            None
        }
    }

    /// Test intersection with a sphere
    pub fn intersect_sphere(&self, center: Vec3, radius: f32) -> Option<f32> {
        let oc = self.origin - center;
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - radius * radius;

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

    /// Test intersection with an AABB
    pub fn intersect_aabb(&self, aabb: &BoundingBox) -> Option<f32> {
        if !aabb.is_valid() {
            return None;
        }

        let inv_dir = Vec3::new(
            1.0 / self.direction.x,
            1.0 / self.direction.y,
            1.0 / self.direction.z,
        );

        let t1 = (aabb.min - self.origin) * inv_dir;
        let t2 = (aabb.max - self.origin) * inv_dir;

        let tmin = t1.min(t2);
        let tmax = t1.max(t2);

        let t_near = tmin.x.max(tmin.y.max(tmin.z));
        let t_far = tmax.x.min(tmax.y.min(tmax.z));

        if t_near <= t_far && t_far >= 0.0 {
            Some(if t_near >= 0.0 { t_near } else { t_far })
        } else {
            None
        }
    }
}

impl Plane {
    /// Create a plane from a normal and distance
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self {
            normal: normal.normalize(),
            distance,
        }
    }

    /// Create a plane from three points
    pub fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let normal = (b - a).cross(c - a).normalize();
        let distance = normal.dot(a);
        Self { normal, distance }
    }

    /// Create a plane from a point and normal
    pub fn from_point_normal(point: Vec3, normal: Vec3) -> Self {
        let normal = normal.normalize();
        Self {
            distance: normal.dot(point),
            normal,
        }
    }

    /// Get the signed distance from a point to the plane
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }

    /// Check which side of the plane a point is on
    pub fn side(&self, point: Vec3) -> f32 {
        self.distance_to_point(point)
    }

    /// Project a point onto the plane
    pub fn project_point(&self, point: Vec3) -> Vec3 {
        point - self.normal * self.distance_to_point(point)
    }
}

impl Frustum {
    /// Create a frustum from a view-projection matrix
    pub fn from_matrix(view_proj: glam::Mat4) -> Self {
        let m = view_proj.to_cols_array_2d();

        // Extract frustum planes from the view-projection matrix
        let planes = [
            // Left plane
            Plane::new(
                Vec3::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0]),
                m[3][3] + m[3][0],
            ),
            // Right plane
            Plane::new(
                Vec3::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0]),
                m[3][3] - m[3][0],
            ),
            // Top plane
            Plane::new(
                Vec3::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1]),
                m[3][3] - m[3][1],
            ),
            // Bottom plane
            Plane::new(
                Vec3::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1]),
                m[3][3] + m[3][1],
            ),
            // Near plane
            Plane::new(
                Vec3::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2]),
                m[3][3] + m[3][2],
            ),
            // Far plane
            Plane::new(
                Vec3::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2]),
                m[3][3] - m[3][2],
            ),
        ];

        Self { planes }
    }

    /// Test if a point is inside the frustum
    pub fn contains_point(&self, point: Vec3) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(point) < 0.0 {
                return false;
            }
        }
        true
    }

    /// Test if a sphere intersects the frustum
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(center) < -radius {
                return false;
            }
        }
        true
    }

    /// Test if a bounding box intersects the frustum
    pub fn intersects_aabb(&self, aabb: &BoundingBox) -> bool {
        if !aabb.is_valid() {
            return false;
        }

        for plane in &self.planes {
            // Find the positive vertex (farthest in the direction of the normal)
            let positive_vertex = Vec3::new(
                if plane.normal.x >= 0.0 {
                    aabb.max.x
                } else {
                    aabb.min.x
                },
                if plane.normal.y >= 0.0 {
                    aabb.max.y
                } else {
                    aabb.min.y
                },
                if plane.normal.z >= 0.0 {
                    aabb.max.z
                } else {
                    aabb.min.z
                },
            );

            if plane.distance_to_point(positive_vertex) < 0.0 {
                return false;
            }
        }
        true
    }

    /// Get the 8 corner points of the frustum
    pub fn corners(&self) -> Option<[Vec3; 8]> {
        // This is complex to implement correctly, returning None for now
        // In a full implementation, this would compute the intersection
        // points of the frustum planes to get the 8 corners
        None
    }
}

impl SpatialQuery {
    /// Find the closest point on a line segment to a point
    pub fn closest_point_on_segment(start: Vec3, end: Vec3, point: Vec3) -> Vec3 {
        let segment = end - start;
        let to_point = point - start;
        let segment_length_sq = segment.length_squared();

        if segment_length_sq < f32::EPSILON {
            return start; // Degenerate segment
        }

        let t = (to_point.dot(segment) / segment_length_sq).clamp(0.0, 1.0);
        start + segment * t
    }

    /// Calculate the distance between a point and a line segment
    pub fn distance_point_to_segment(start: Vec3, end: Vec3, point: Vec3) -> f32 {
        let closest = Self::closest_point_on_segment(start, end, point);
        (point - closest).length()
    }

    /// Test if two line segments intersect (2D, ignoring Z)
    pub fn segments_intersect_2d(a1: Vec3, a2: Vec3, b1: Vec3, b2: Vec3) -> Option<Vec3> {
        let s1 = Vec3::new(a2.x - a1.x, a2.y - a1.y, 0.0);
        let s2 = Vec3::new(b2.x - b1.x, b2.y - b1.y, 0.0);

        let s = (-s1.y * (a1.x - b1.x) + s1.x * (a1.y - b1.y)) / (-s2.x * s1.y + s1.x * s2.y);
        let t = (s2.x * (a1.y - b1.y) - s2.y * (a1.x - b1.x)) / (-s2.x * s1.y + s1.x * s2.y);

        if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
            // Intersection detected
            Some(Vec3::new(a1.x + (t * s1.x), a1.y + (t * s1.y), 0.0))
        } else {
            None
        }
    }

    /// Calculate barycentric coordinates of a point with respect to a triangle
    pub fn barycentric_coordinates(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
        let v0 = c - a;
        let v1 = b - a;
        let v2 = p - a;

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

    /// Test if a point is inside a triangle using barycentric coordinates
    pub fn point_in_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> bool {
        let bary = Self::barycentric_coordinates(p, a, b, c);
        bary.x >= 0.0 && bary.y >= 0.0 && bary.z >= 0.0
    }
}
