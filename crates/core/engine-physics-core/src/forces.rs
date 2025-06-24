//! Force application abstractions

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Force application modes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ForceMode {
    /// Apply force over time (F = ma)
    Force,
    /// Apply instantaneous impulse (change in momentum)
    Impulse,
    /// Apply acceleration directly (ignores mass)
    Acceleration,
    /// Apply velocity change directly (ignores mass)
    VelocityChange,
}

/// Force application point
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ForceApplication {
    /// Apply at center of mass
    CenterOfMass,
    /// Apply at specific local point
    LocalPoint(Vec3),
    /// Apply at specific world point
    WorldPoint(Vec3),
}

/// Force descriptor for applying forces to rigid bodies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Force {
    /// Force vector
    pub force: Vec3,
    /// How to apply the force
    pub mode: ForceMode,
    /// Where to apply the force
    pub application: ForceApplication,
    /// Duration (for continuous forces)
    pub duration: Option<f32>,
}

/// Torque descriptor for applying rotational forces
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Torque {
    /// Torque vector (axis * magnitude)
    pub torque: Vec3,
    /// How to apply the torque
    pub mode: ForceMode,
    /// Duration (for continuous torques)
    pub duration: Option<f32>,
}

impl Force {
    /// Create a force applied at center of mass
    pub fn at_center_of_mass(force: Vec3) -> Self {
        Self {
            force,
            mode: ForceMode::Force,
            application: ForceApplication::CenterOfMass,
            duration: None,
        }
    }

    /// Create an impulse applied at center of mass
    pub fn impulse_at_center_of_mass(impulse: Vec3) -> Self {
        Self {
            force: impulse,
            mode: ForceMode::Impulse,
            application: ForceApplication::CenterOfMass,
            duration: None,
        }
    }

    /// Create a force applied at a world point
    pub fn at_world_point(force: Vec3, point: Vec3) -> Self {
        Self {
            force,
            mode: ForceMode::Force,
            application: ForceApplication::WorldPoint(point),
            duration: None,
        }
    }

    /// Set duration for continuous force
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set force mode
    pub fn with_mode(mut self, mode: ForceMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Torque {
    /// Create a torque
    pub fn new(torque: Vec3) -> Self {
        Self {
            torque,
            mode: ForceMode::Force,
            duration: None,
        }
    }

    /// Create a torque impulse
    pub fn impulse(impulse: Vec3) -> Self {
        Self {
            torque: impulse,
            mode: ForceMode::Impulse,
            duration: None,
        }
    }

    /// Set duration for continuous torque
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set torque mode
    pub fn with_mode(mut self, mode: ForceMode) -> Self {
        self.mode = mode;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_mode_enum() {
        let modes = [
            ForceMode::Force,
            ForceMode::Impulse,
            ForceMode::Acceleration,
            ForceMode::VelocityChange,
        ];

        for mode in &modes {
            // Test that all variants can be matched
            match mode {
                ForceMode::Force => {}
                ForceMode::Impulse => {}
                ForceMode::Acceleration => {}
                ForceMode::VelocityChange => {}
            }
        }

        assert_eq!(ForceMode::Force, ForceMode::Force);
        assert_ne!(ForceMode::Force, ForceMode::Impulse);
    }

    #[test]
    fn test_force_application_enum() {
        let center_of_mass = ForceApplication::CenterOfMass;
        let local_point = ForceApplication::LocalPoint(Vec3::new(1.0, 2.0, 3.0));
        let world_point = ForceApplication::WorldPoint(Vec3::new(4.0, 5.0, 6.0));

        match center_of_mass {
            ForceApplication::CenterOfMass => {}
            _ => panic!("Should be center of mass"),
        }

        match local_point {
            ForceApplication::LocalPoint(point) => {
                assert_eq!(point, Vec3::new(1.0, 2.0, 3.0));
            }
            _ => panic!("Should be local point"),
        }

        match world_point {
            ForceApplication::WorldPoint(point) => {
                assert_eq!(point, Vec3::new(4.0, 5.0, 6.0));
            }
            _ => panic!("Should be world point"),
        }

        assert_eq!(
            ForceApplication::CenterOfMass,
            ForceApplication::CenterOfMass
        );
        assert_ne!(ForceApplication::CenterOfMass, local_point);
    }

    #[test]
    fn test_force_at_center_of_mass() {
        let force_vector = Vec3::new(10.0, 20.0, 30.0);
        let force = Force::at_center_of_mass(force_vector);

        assert_eq!(force.force, force_vector);
        assert_eq!(force.mode, ForceMode::Force);
        assert_eq!(force.application, ForceApplication::CenterOfMass);
        assert!(force.duration.is_none());
    }

    #[test]
    fn test_impulse_at_center_of_mass() {
        let impulse_vector = Vec3::new(5.0, 10.0, 15.0);
        let impulse = Force::impulse_at_center_of_mass(impulse_vector);

        assert_eq!(impulse.force, impulse_vector);
        assert_eq!(impulse.mode, ForceMode::Impulse);
        assert_eq!(impulse.application, ForceApplication::CenterOfMass);
        assert!(impulse.duration.is_none());
    }

    #[test]
    fn test_force_at_world_point() {
        let force_vector = Vec3::new(1.0, 2.0, 3.0);
        let world_point = Vec3::new(4.0, 5.0, 6.0);
        let force = Force::at_world_point(force_vector, world_point);

        assert_eq!(force.force, force_vector);
        assert_eq!(force.mode, ForceMode::Force);
        assert_eq!(force.application, ForceApplication::WorldPoint(world_point));
        assert!(force.duration.is_none());
    }

    #[test]
    fn test_force_builder_pattern() {
        let force = Force::at_center_of_mass(Vec3::new(1.0, 2.0, 3.0))
            .with_duration(5.0)
            .with_mode(ForceMode::Acceleration);

        assert_eq!(force.force, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(force.mode, ForceMode::Acceleration);
        assert_eq!(force.application, ForceApplication::CenterOfMass);
        assert_eq!(force.duration, Some(5.0));
    }

    #[test]
    fn test_force_with_duration() {
        let force = Force::at_center_of_mass(Vec3::ZERO).with_duration(2.5);
        assert_eq!(force.duration, Some(2.5));

        let force_no_duration = Force::at_center_of_mass(Vec3::ZERO);
        assert!(force_no_duration.duration.is_none());
    }

    #[test]
    fn test_force_with_mode() {
        let force = Force::at_center_of_mass(Vec3::ZERO).with_mode(ForceMode::VelocityChange);
        assert_eq!(force.mode, ForceMode::VelocityChange);

        let impulse = Force::impulse_at_center_of_mass(Vec3::ZERO).with_mode(ForceMode::Force);
        assert_eq!(impulse.mode, ForceMode::Force);
    }

    #[test]
    fn test_torque_new() {
        let torque_vector = Vec3::new(0.1, 0.2, 0.3);
        let torque = Torque::new(torque_vector);

        assert_eq!(torque.torque, torque_vector);
        assert_eq!(torque.mode, ForceMode::Force);
        assert!(torque.duration.is_none());
    }

    #[test]
    fn test_torque_impulse() {
        let impulse_vector = Vec3::new(1.0, 2.0, 3.0);
        let torque = Torque::impulse(impulse_vector);

        assert_eq!(torque.torque, impulse_vector);
        assert_eq!(torque.mode, ForceMode::Impulse);
        assert!(torque.duration.is_none());
    }

    #[test]
    fn test_torque_builder_pattern() {
        let torque = Torque::new(Vec3::new(0.5, 1.0, 1.5))
            .with_duration(3.0)
            .with_mode(ForceMode::Acceleration);

        assert_eq!(torque.torque, Vec3::new(0.5, 1.0, 1.5));
        assert_eq!(torque.mode, ForceMode::Acceleration);
        assert_eq!(torque.duration, Some(3.0));
    }

    #[test]
    fn test_torque_with_duration() {
        let torque = Torque::new(Vec3::ZERO).with_duration(1.5);
        assert_eq!(torque.duration, Some(1.5));

        let torque_no_duration = Torque::new(Vec3::ZERO);
        assert!(torque_no_duration.duration.is_none());
    }

    #[test]
    fn test_torque_with_mode() {
        let torque = Torque::new(Vec3::ZERO).with_mode(ForceMode::VelocityChange);
        assert_eq!(torque.mode, ForceMode::VelocityChange);

        let impulse_torque = Torque::impulse(Vec3::ZERO).with_mode(ForceMode::Force);
        assert_eq!(impulse_torque.mode, ForceMode::Force);
    }

    #[test]
    fn test_force_struct_creation() {
        let force = Force {
            force: Vec3::new(10.0, 0.0, 0.0),
            mode: ForceMode::Impulse,
            application: ForceApplication::LocalPoint(Vec3::new(0.0, 1.0, 0.0)),
            duration: Some(2.0),
        };

        assert_eq!(force.force, Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(force.mode, ForceMode::Impulse);
        match force.application {
            ForceApplication::LocalPoint(point) => {
                assert_eq!(point, Vec3::new(0.0, 1.0, 0.0));
            }
            _ => panic!("Should be local point"),
        }
        assert_eq!(force.duration, Some(2.0));
    }

    #[test]
    fn test_torque_struct_creation() {
        let torque = Torque {
            torque: Vec3::new(0.0, 0.0, 5.0),
            mode: ForceMode::Acceleration,
            duration: Some(1.0),
        };

        assert_eq!(torque.torque, Vec3::new(0.0, 0.0, 5.0));
        assert_eq!(torque.mode, ForceMode::Acceleration);
        assert_eq!(torque.duration, Some(1.0));
    }

    #[test]
    fn test_force_mode_usage_patterns() {
        // Test typical usage patterns for different force modes

        // Continuous force (like gravity)
        let gravity =
            Force::at_center_of_mass(Vec3::new(0.0, -9.81, 0.0)).with_mode(ForceMode::Force);
        assert_eq!(gravity.mode, ForceMode::Force);
        assert!(gravity.duration.is_none());

        // Instantaneous impulse (like explosion)
        let explosion = Force::at_world_point(Vec3::new(100.0, 100.0, 0.0), Vec3::ZERO)
            .with_mode(ForceMode::Impulse);
        assert_eq!(explosion.mode, ForceMode::Impulse);

        // Direct acceleration (like wind)
        let wind = Force::at_center_of_mass(Vec3::new(2.0, 0.0, 0.0))
            .with_mode(ForceMode::Acceleration)
            .with_duration(5.0);
        assert_eq!(wind.mode, ForceMode::Acceleration);
        assert_eq!(wind.duration, Some(5.0));

        // Direct velocity change (like telekinesis)
        let telekinesis = Force::at_center_of_mass(Vec3::new(0.0, 10.0, 0.0))
            .with_mode(ForceMode::VelocityChange);
        assert_eq!(telekinesis.mode, ForceMode::VelocityChange);
    }

    #[test]
    fn test_torque_usage_patterns() {
        // Continuous torque (like motor)
        let motor_torque = Torque::new(Vec3::new(0.0, 0.0, 10.0));
        assert_eq!(motor_torque.mode, ForceMode::Force);

        // Impulse torque (like collision)
        let collision_torque = Torque::impulse(Vec3::new(5.0, 0.0, 0.0));
        assert_eq!(collision_torque.mode, ForceMode::Impulse);

        // Timed torque (like temporary spin boost)
        let spin_boost = Torque::new(Vec3::new(0.0, 20.0, 0.0)).with_duration(2.0);
        assert_eq!(spin_boost.duration, Some(2.0));
    }

    #[test]
    fn test_zero_vectors() {
        let zero_force = Force::at_center_of_mass(Vec3::ZERO);
        assert_eq!(zero_force.force, Vec3::ZERO);

        let zero_torque = Torque::new(Vec3::ZERO);
        assert_eq!(zero_torque.torque, Vec3::ZERO);

        let zero_point_force = Force::at_world_point(Vec3::ZERO, Vec3::ZERO);
        assert_eq!(zero_point_force.force, Vec3::ZERO);
        match zero_point_force.application {
            ForceApplication::WorldPoint(point) => assert_eq!(point, Vec3::ZERO),
            _ => panic!("Should be world point"),
        }
    }
}
