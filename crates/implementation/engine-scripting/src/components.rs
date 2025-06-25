//! Common components for scripting tests and examples

use engine_component_traits::impl_component;

/// Transform component for position, rotation, and scale
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl_component!(Transform);

/// Health component
#[derive(Debug, Clone, PartialEq)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl_component!(Health);