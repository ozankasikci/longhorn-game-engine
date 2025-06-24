//! Mathematical constants commonly used in game development

use std::f32::consts;

// Re-export standard mathematical constants
pub use consts::*;

// Game-specific mathematical constants

/// Golden ratio (φ) - useful for aesthetically pleasing proportions
pub const GOLDEN_RATIO: f32 = 1.618_034;

/// Square root of 2 - useful for diagonal calculations
pub const SQRT_2: f32 = consts::SQRT_2;

/// Square root of 3 - useful for hexagonal and triangular calculations
pub const SQRT_3: f32 = 1.732_050_8;

/// Natural logarithm of 2
pub const LN_2: f32 = consts::LN_2;

/// Natural logarithm of 10
pub const LN_10: f32 = consts::LN_10;

/// Inverse of PI (1/π)
pub const INV_PI: f32 = consts::FRAC_1_PI;

/// Twice PI (2π) - full circle in radians
pub const TWO_PI: f32 = 2.0 * PI;

/// Half PI (π/2) - quarter circle in radians  
pub const HALF_PI: f32 = PI * 0.5;

/// Quarter PI (π/4) - eighth circle in radians
pub const QUARTER_PI: f32 = PI * 0.25;

/// Three quarters PI (3π/4)
pub const THREE_QUARTER_PI: f32 = PI * 0.75;

/// Conversion factor from degrees to radians
pub const DEG_TO_RAD: f32 = PI / 180.0;

/// Conversion factor from radians to degrees
pub const RAD_TO_DEG: f32 = 180.0 / PI;

/// Small epsilon value for floating point comparisons
pub const EPSILON: f32 = f32::EPSILON;

/// Larger epsilon for more lenient floating point comparisons
pub const LARGE_EPSILON: f32 = 1e-6;

/// Very small epsilon for high-precision comparisons
pub const SMALL_EPSILON: f32 = 1e-9;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!((GOLDEN_RATIO - 1.618_034).abs() < EPSILON);
        assert!((TWO_PI - (2.0 * PI)).abs() < EPSILON);
        assert!((DEG_TO_RAD * 180.0 - PI).abs() < EPSILON);
        assert!((RAD_TO_DEG * PI - 180.0).abs() < EPSILON);
    }
}
