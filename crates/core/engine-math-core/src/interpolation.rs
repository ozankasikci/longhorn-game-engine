//! Interpolation functions for smooth animations and transitions

use glam::{Quat, Vec2, Vec3, Vec4};

/// Linear interpolation between two values
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Linear interpolation between two Vec2 values
pub fn lerp_vec2(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t
}

/// Linear interpolation between two Vec3 values
pub fn lerp_vec3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t
}

/// Linear interpolation between two Vec4 values
pub fn lerp_vec4(a: Vec4, b: Vec4, t: f32) -> Vec4 {
    a + (b - a) * t
}

/// Spherical linear interpolation between two quaternions
pub fn slerp_quat(a: Quat, b: Quat, t: f32) -> Quat {
    a.slerp(b, t)
}

/// Inverse linear interpolation - find t given a, b, and value
pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if (b - a).abs() < f32::EPSILON {
        0.0
    } else {
        (value - a) / (b - a)
    }
}

/// Clamped linear interpolation (t is clamped to [0, 1])
pub fn lerp_clamped(a: f32, b: f32, t: f32) -> f32 {
    lerp(a, b, t.clamp(0.0, 1.0))
}

/// Smooth step interpolation (Hermite interpolation)
/// Returns smooth transition between 0 and 1 when t goes from 0 to 1
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smoother step interpolation (6th order polynomial)
/// Even smoother than smoothstep
pub fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Cubic interpolation between four points
/// Useful for smooth curves through multiple points
pub fn cubic_interpolate(y0: f32, y1: f32, y2: f32, y3: f32, t: f32) -> f32 {
    let a0 = y3 - y2 - y0 + y1;
    let a1 = y0 - y1 - a0;
    let a2 = y2 - y0;
    let a3 = y1;

    a0 * t * t * t + a1 * t * t + a2 * t + a3
}

/// Cosine interpolation - smoother than linear
pub fn cosine_interpolate(a: f32, b: f32, t: f32) -> f32 {
    let t2 = (1.0 - (t * std::f32::consts::PI).cos()) / 2.0;
    lerp(a, b, t2)
}

/// Exponential ease-in interpolation
pub fn ease_in_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else {
        2.0_f32.powf(10.0 * (t - 1.0))
    }
}

/// Exponential ease-out interpolation
pub fn ease_out_expo(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0_f32.powf(-10.0 * t)
    }
}

/// Exponential ease-in-out interpolation
pub fn ease_in_out_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        0.5 * 2.0_f32.powf(20.0 * t - 10.0)
    } else {
        0.5 * (2.0 - 2.0_f32.powf(-20.0 * t + 10.0))
    }
}

/// Quadratic ease-in interpolation
pub fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// Quadratic ease-out interpolation
pub fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Quadratic ease-in-out interpolation
pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Bounce ease-out interpolation
pub fn ease_out_bounce(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;

    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < f32::EPSILON);
        assert!((lerp(0.0, 10.0, 0.0) - 0.0).abs() < f32::EPSILON);
        assert!((lerp(0.0, 10.0, 1.0) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inverse_lerp() {
        assert!((inverse_lerp(0.0, 10.0, 5.0) - 0.5).abs() < f32::EPSILON);
        assert!((inverse_lerp(0.0, 10.0, 0.0) - 0.0).abs() < f32::EPSILON);
        assert!((inverse_lerp(0.0, 10.0, 10.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_smoothstep() {
        assert_eq!(smoothstep(0.0), 0.0);
        assert_eq!(smoothstep(1.0), 1.0);
        assert!(smoothstep(0.5) > 0.4 && smoothstep(0.5) < 0.6);
    }

    #[test]
    fn test_ease_functions() {
        // Test that easing functions return expected values at boundaries
        assert_eq!(ease_in_quad(0.0), 0.0);
        assert_eq!(ease_in_quad(1.0), 1.0);
        assert_eq!(ease_out_quad(0.0), 0.0);
        assert_eq!(ease_out_quad(1.0), 1.0);
    }
}
