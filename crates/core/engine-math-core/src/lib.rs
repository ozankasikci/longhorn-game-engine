// Engine Math Core - Centralized mathematical utilities for the mobile game engine

// Re-export all glam types and functions for convenience
pub use glam::*;

/// Convert degrees to radians
pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}

/// Convert radians to degrees
pub fn radians_to_degrees(radians: f32) -> f32 {
    radians * 180.0 / std::f32::consts::PI
}

/// Linear interpolation between two values
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamp a value between min and max
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degrees_to_radians() {
        assert!((degrees_to_radians(180.0) - std::f32::consts::PI).abs() < f32::EPSILON);
        assert!((degrees_to_radians(90.0) - std::f32::consts::FRAC_PI_2).abs() < f32::EPSILON);
        assert!((degrees_to_radians(0.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_radians_to_degrees() {
        assert!((radians_to_degrees(std::f32::consts::PI) - 180.0).abs() < f32::EPSILON);
        assert!((radians_to_degrees(std::f32::consts::FRAC_PI_2) - 90.0).abs() < f32::EPSILON);
        assert!((radians_to_degrees(0.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lerp() {
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < f32::EPSILON);
        assert!((lerp(0.0, 10.0, 0.0) - 0.0).abs() < f32::EPSILON);
        assert!((lerp(0.0, 10.0, 1.0) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }
}