//! Curve and spline functions for smooth paths and animations

use glam::{Vec2, Vec3};

/// Quadratic Bezier curve evaluation
/// Returns point on curve at parameter t (0.0 to 1.0)
pub fn quadratic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    u * u * p0 + 2.0 * u * t * p1 + t * t * p2
}

/// Cubic Bezier curve evaluation
/// Returns point on curve at parameter t (0.0 to 1.0)
pub fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    let u2 = u * u;
    let u3 = u2 * u;
    let t2 = t * t;
    let t3 = t2 * t;
    
    u3 * p0 + 3.0 * u2 * t * p1 + 3.0 * u * t2 * p2 + t3 * p3
}

/// 3D Cubic Bezier curve evaluation
pub fn cubic_bezier_3d(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let u = 1.0 - t;
    let u2 = u * u;
    let u3 = u2 * u;
    let t2 = t * t;
    let t3 = t2 * t;
    
    u3 * p0 + 3.0 * u2 * t * p1 + 3.0 * u * t2 * p2 + t3 * p3
}

/// Cubic Bezier derivative (tangent) at parameter t
pub fn cubic_bezier_derivative(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    3.0 * u * u * (p1 - p0) + 6.0 * u * t * (p2 - p1) + 3.0 * t * t * (p3 - p2)
}

/// Catmull-Rom spline interpolation through four points
/// Returns smooth curve that passes through p1 and p2
pub fn catmull_rom_spline(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    
    let a = -0.5 * p0 + 1.5 * p1 - 1.5 * p2 + 0.5 * p3;
    let b = p0 - 2.5 * p1 + 2.0 * p2 - 0.5 * p3;
    let c = -0.5 * p0 + 0.5 * p2;
    let d = p1;
    
    a * t3 + b * t2 + c * t + d
}

/// 3D Catmull-Rom spline interpolation
pub fn catmull_rom_spline_3d(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;
    
    let a = -0.5 * p0 + 1.5 * p1 - 1.5 * p2 + 0.5 * p3;
    let b = p0 - 2.5 * p1 + 2.0 * p2 - 0.5 * p3;
    let c = -0.5 * p0 + 0.5 * p2;
    let d = p1;
    
    a * t3 + b * t2 + c * t + d
}

/// Hermite spline interpolation with explicit tangents
pub fn hermite_spline(p0: Vec2, p1: Vec2, m0: Vec2, m1: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    
    h00 * p0 + h10 * m0 + h01 * p1 + h11 * m1
}

/// B-spline basis function (cubic)
pub fn bspline_basis(t: f32, i: i32) -> f32 {
    match i {
        0 => {
            let u = 1.0 - t;
            u * u * u / 6.0
        }
        1 => {
            (3.0 * t * t * t - 6.0 * t * t + 4.0) / 6.0
        }
        2 => {
            (-3.0 * t * t * t + 3.0 * t * t + 3.0 * t + 1.0) / 6.0
        }
        3 => {
            t * t * t / 6.0
        }
        _ => 0.0,
    }
}

/// Cubic B-spline curve evaluation
pub fn cubic_bspline(points: &[Vec2], t: f32) -> Vec2 {
    if points.len() < 4 {
        return Vec2::ZERO;
    }
    
    let segment = (t * (points.len() - 3) as f32).floor() as usize;
    let local_t = t * (points.len() - 3) as f32 - segment as f32;
    
    let segment = segment.min(points.len() - 4);
    
    let mut result = Vec2::ZERO;
    for i in 0..4 {
        result += bspline_basis(local_t, i) * points[segment + i as usize];
    }
    
    result
}

/// Arc length parameterization helper - estimate curve length
pub fn estimate_bezier_length(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, segments: u32) -> f32 {
    let mut length = 0.0;
    let mut prev_point = p0;
    
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let point = cubic_bezier(p0, p1, p2, p3, t);
        length += prev_point.distance(point);
        prev_point = point;
    }
    
    length
}

/// Find parameter t for a given arc length along a Bezier curve
pub fn bezier_parameter_from_arc_length(
    p0: Vec2, 
    p1: Vec2, 
    p2: Vec2, 
    p3: Vec2, 
    target_length: f32,
    total_length: f32,
    tolerance: f32
) -> f32 {
    if target_length <= 0.0 {
        return 0.0;
    }
    if target_length >= total_length {
        return 1.0;
    }
    
    let mut low = 0.0;
    let mut high = 1.0;
    let mut t = target_length / total_length; // Initial guess
    
    // Binary search for the correct parameter
    for _ in 0..20 { // Max iterations
        let current_length = estimate_bezier_length_to_t(p0, p1, p2, p3, t, 20);
        let diff = current_length - target_length;
        
        if diff.abs() < tolerance {
            break;
        }
        
        if diff > 0.0 {
            high = t;
        } else {
            low = t;
        }
        
        t = (low + high) * 0.5;
    }
    
    t
}

/// Helper function to estimate arc length up to parameter t
fn estimate_bezier_length_to_t(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32, segments: u32) -> f32 {
    let mut length = 0.0;
    let mut prev_point = p0;
    
    let step = t / segments as f32;
    for i in 1..=segments {
        let param = (i as f32 * step).min(t);
        let point = cubic_bezier(p0, p1, p2, p3, param);
        length += prev_point.distance(point);
        prev_point = point;
    }
    
    length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadratic_bezier() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(0.5, 1.0);
        let p2 = Vec2::new(1.0, 0.0);
        
        let start = quadratic_bezier(p0, p1, p2, 0.0);
        let end = quadratic_bezier(p0, p1, p2, 1.0);
        let mid = quadratic_bezier(p0, p1, p2, 0.5);
        
        assert!((start - p0).length() < f32::EPSILON);
        assert!((end - p2).length() < f32::EPSILON);
        assert!(mid.y > 0.0); // Should be above the line connecting p0 and p2
    }

    #[test]
    fn test_cubic_bezier() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(0.33, 1.0);
        let p2 = Vec2::new(0.67, 1.0);
        let p3 = Vec2::new(1.0, 0.0);
        
        let start = cubic_bezier(p0, p1, p2, p3, 0.0);
        let end = cubic_bezier(p0, p1, p2, p3, 1.0);
        
        assert!((start - p0).length() < f32::EPSILON);
        assert!((end - p3).length() < f32::EPSILON);
    }

    #[test]
    fn test_catmull_rom() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(1.0, 0.0);
        let p2 = Vec2::new(2.0, 0.0);
        let p3 = Vec2::new(3.0, 0.0);
        
        // Should pass through p1 and p2 at t=0 and t=1
        let start = catmull_rom_spline(p0, p1, p2, p3, 0.0);
        let end = catmull_rom_spline(p0, p1, p2, p3, 1.0);
        
        assert!((start - p1).length() < f32::EPSILON);
        assert!((end - p2).length() < f32::EPSILON);
    }

    #[test]
    fn test_bspline_basis() {
        // Test that basis functions sum to 1
        let t = 0.5;
        let sum = bspline_basis(t, 0) + bspline_basis(t, 1) + bspline_basis(t, 2) + bspline_basis(t, 3);
        assert!((sum - 1.0).abs() < f32::EPSILON);
    }
}