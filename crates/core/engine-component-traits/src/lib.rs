// Component trait definitions extracted to break circular dependencies

use std::any::{Any, TypeId};

/// Component trait - marker for types that can be stored as components
pub trait Component: 'static + Send + Sync + ComponentClone {
    fn type_id() -> TypeId
    where
        Self: Sized,
    {
        TypeId::of::<Self>()
    }
}

/// Helper trait for component cloning with type erasure
pub trait ComponentClone {
    /// Clone this component as a type-erased box
    fn clone_boxed(&self) -> Box<dyn ComponentClone>;

    /// Get as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Convert boxed self to boxed Any
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

/// Blanket implementation for cloneable types
impl<T: Clone + Component + 'static> ComponentClone for T {
    fn clone_boxed(&self) -> Box<dyn ComponentClone> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// Macro to implement Component trait for types that already implement Clone
#[macro_export]
macro_rules! impl_component {
    ($type:ty) => {
        impl $crate::Component for $type {}
    };
}

// Legacy ECS Component trait (from ecs.rs)
/// Basic component trait for the legacy ECS system
pub trait ComponentV1: 'static + Send + Sync {}

/// Change tracking for components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentTicks {
    pub added: Tick,
    pub changed: Tick,
}

impl ComponentTicks {
    pub fn new(tick: Tick) -> Self {
        Self {
            added: tick,
            changed: tick,
        }
    }

    pub fn mark_changed(&mut self, tick: Tick) {
        self.changed = tick;
    }

    pub fn is_added(&self, last_run: Tick) -> bool {
        self.added.is_newer_than(last_run)
    }

    pub fn is_changed(&self, last_run: Tick) -> bool {
        self.changed.is_newer_than(last_run)
    }
}

/// Global tick counter for change detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tick(pub u32);

impl Tick {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(&self) -> u32 {
        self.0
    }

    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    pub fn is_newer_than(&self, other: Tick) -> bool {
        // Handle wrap-around for u32 tick values
        self.0.wrapping_sub(other.0) < u32::MAX / 2
    }
}

// Bundle trait for inserting multiple components at once
pub trait Bundle: Send + Sync + 'static {
    /// Get the type IDs of all components in this bundle
    fn component_ids() -> Vec<TypeId>
    where
        Self: Sized;

    /// Convert this bundle into its component parts
    fn into_components(self) -> Vec<(TypeId, Box<dyn ComponentClone>)>;
}
