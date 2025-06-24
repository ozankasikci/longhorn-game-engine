//! Common test utilities for ECS V2 testing

use engine_component_traits::Component;

/// Test position component
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Component for Position {}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

/// Test velocity component
#[derive(Clone, Debug, PartialEq)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Component for Velocity {}

impl Velocity {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

/// Test health component
#[derive(Clone, Debug, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Component for Health {}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn damaged(current: f32, max: f32) -> Self {
        Self { current, max }
    }
}

/// Test name component
#[derive(Clone, Debug, PartialEq)]
pub struct Name {
    pub name: String,
}

impl Component for Name {}

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Test tag component (zero-sized)
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Player;
impl Component for Player {}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Enemy;
impl Component for Enemy {}

/// Register all test components
pub fn register_test_components() {
    use crate::ecs_v2::component::register_component;

    register_component::<Position>();
    register_component::<Velocity>();
    register_component::<Health>();
    register_component::<Name>();
    register_component::<Player>();
    register_component::<Enemy>();
}

/// Create a test world with some entities
pub fn create_test_world() -> crate::ecs_v2::world::World {
    use crate::ecs_v2::world::World;

    register_test_components();
    World::new()
}

/// Assert that two floats are approximately equal
pub fn assert_float_eq(a: f32, b: f32, epsilon: f32) {
    assert!(
        (a - b).abs() < epsilon,
        "Float values not equal: {} != {} (epsilon: {})",
        a,
        b,
        epsilon
    );
}

/// Measure execution time of a closure
pub fn measure_time<F, R>(f: F) -> (R, std::time::Duration)
where
    F: FnOnce() -> R,
{
    let start = std::time::Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_creation() {
        let pos = Position::new(1.0, 2.0, 3.0);
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 3.0);

        let vel = Velocity::zero();
        assert_eq!(vel.x, 0.0);
        assert_eq!(vel.y, 0.0);
        assert_eq!(vel.z, 0.0);

        let health = Health::new(100.0);
        assert_eq!(health.current, 100.0);
        assert_eq!(health.max, 100.0);

        let name = Name::new("Test Entity");
        assert_eq!(name.name, "Test Entity");
    }

    #[test]
    fn test_float_equality() {
        assert_float_eq(1.0, 1.0, 0.001);
        assert_float_eq(1.0, 1.0001, 0.001);
    }

    #[test]
    #[should_panic]
    fn test_float_inequality() {
        assert_float_eq(1.0, 2.0, 0.001);
    }
}
