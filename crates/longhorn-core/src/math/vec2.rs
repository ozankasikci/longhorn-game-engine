use glam::Vec2;

/// Extension trait for Vec2 with game-specific utilities
pub trait Vec2Ext {
    /// Create a Vec2 from an angle (in radians)
    fn from_angle(angle: f32) -> Vec2;

    /// Get the angle of this vector (in radians)
    fn angle(&self) -> f32;

    /// Rotate this vector by an angle (in radians)
    fn rotate_by(&self, angle: f32) -> Vec2;
}

impl Vec2Ext for Vec2 {
    fn from_angle(angle: f32) -> Vec2 {
        Vec2::new(angle.cos(), angle.sin())
    }

    fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    fn rotate_by(&self, angle: f32) -> Vec2 {
        let cos = angle.cos();
        let sin = angle.sin();
        Vec2::new(
            self.x * cos - self.y * sin,
            self.x * sin + self.y * cos,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_from_angle() {
        let v = Vec2::from_angle(0.0);
        assert!((v.x - 1.0).abs() < 0.0001);
        assert!(v.y.abs() < 0.0001);

        let v = Vec2::from_angle(PI / 2.0);
        assert!(v.x.abs() < 0.0001);
        assert!((v.y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_angle() {
        let v = Vec2::new(1.0, 0.0);
        assert!((v.angle() - 0.0).abs() < 0.0001);

        let v = Vec2::new(0.0, 1.0);
        assert!((v.angle() - PI / 2.0).abs() < 0.0001);

        let v = Vec2::new(-1.0, 0.0);
        assert!((v.angle() - PI).abs() < 0.0001);
    }

    #[test]
    fn test_rotate_by() {
        let v = Vec2::new(1.0, 0.0);
        let rotated = v.rotate_by(PI / 2.0);
        assert!(rotated.x.abs() < 0.0001);
        assert!((rotated.y - 1.0).abs() < 0.0001);

        let v = Vec2::new(1.0, 1.0);
        let rotated = v.rotate_by(PI);
        assert!((rotated.x + 1.0).abs() < 0.0001);
        assert!((rotated.y + 1.0).abs() < 0.0001);
    }
}
