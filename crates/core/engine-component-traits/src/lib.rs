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
        // A tick is not newer than itself
        if self.0 == other.0 {
            return false;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    // Test components for testing
    #[derive(Debug, Clone, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Debug, Clone)]
    struct Health(#[allow(dead_code)] pub u32);

    // Implement Component for test types
    impl_component!(Position);
    impl_component!(Velocity);
    impl_component!(Health);

    // Implement legacy trait for testing (moved outside function to avoid warnings)
    impl ComponentV1 for Position {}
    impl ComponentV1 for Velocity {}

    #[test]
    fn test_component_type_id() {
        let pos_id = <Position as Component>::type_id();
        let vel_id = <Velocity as Component>::type_id();
        let health_id = <Health as Component>::type_id();

        // Each component should have a unique TypeId
        assert_ne!(pos_id, vel_id);
        assert_ne!(pos_id, health_id);
        assert_ne!(vel_id, health_id);

        // Same component type should have same TypeId
        assert_eq!(pos_id, <Position as Component>::type_id());
        assert_eq!(TypeId::of::<Position>(), <Position as Component>::type_id());
    }

    #[test]
    fn test_component_clone_trait() {
        let pos = Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        // Test clone_boxed
        let cloned_box = pos.clone_boxed();
        let cloned_any = cloned_box.as_any();
        let cloned_pos = cloned_any.downcast_ref::<Position>().unwrap();
        assert_eq!(&pos, cloned_pos);

        // Test as_any
        let pos_any = pos.as_any();
        let pos_downcast = pos_any.downcast_ref::<Position>().unwrap();
        assert_eq!(&pos, pos_downcast);
    }

    #[test]
    fn test_component_clone_mut() {
        let mut pos = Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        // Test as_any_mut
        let pos_any_mut = pos.as_any_mut();
        let pos_mut = pos_any_mut.downcast_mut::<Position>().unwrap();
        pos_mut.x = 10.0;

        assert_eq!(pos.x, 10.0);
    }

    #[test]
    fn test_component_into_any() {
        let pos = Position {
            x: 5.0,
            y: 6.0,
            z: 7.0,
        };
        let boxed_pos = Box::new(pos.clone());

        // Test into_any
        let any_box = boxed_pos.into_any();
        let recovered_pos = any_box.downcast::<Position>().unwrap();
        assert_eq!(*recovered_pos, pos);
    }

    #[test]
    fn test_tick_creation_and_operations() {
        let tick1 = Tick::new(10);
        let tick2 = Tick::new(20);

        assert_eq!(tick1.get(), 10);
        assert_eq!(tick2.get(), 20);

        // Test ordering
        assert!(tick1 < tick2);
        assert!(tick2 > tick1);
        assert_eq!(tick1, Tick::new(10));
    }

    #[test]
    fn test_tick_increment() {
        let mut tick = Tick::new(5);
        assert_eq!(tick.get(), 5);

        tick.increment();
        assert_eq!(tick.get(), 6);

        tick.increment();
        assert_eq!(tick.get(), 7);
    }

    #[test]
    fn test_tick_wrapping() {
        let mut tick = Tick::new(u32::MAX);
        tick.increment();
        assert_eq!(tick.get(), 0);
    }

    #[test]
    fn test_tick_is_newer_than() {
        let tick1 = Tick::new(10);
        let tick2 = Tick::new(15);
        let tick3 = Tick::new(5);

        assert!(tick2.is_newer_than(tick1));
        assert!(!tick1.is_newer_than(tick2));
        assert!(tick1.is_newer_than(tick3));

        // Same tick should not be newer than itself
        assert!(!tick1.is_newer_than(tick1));
    }

    #[test]
    fn test_tick_wraparound_comparison() {
        // Test wraparound behavior
        let old_tick = Tick::new(u32::MAX - 5);
        let new_tick = Tick::new(5); // Wrapped around

        assert!(new_tick.is_newer_than(old_tick));
        assert!(!old_tick.is_newer_than(new_tick));
    }

    #[test]
    fn test_component_ticks_creation() {
        let tick = Tick::new(100);
        let comp_ticks = ComponentTicks::new(tick);

        assert_eq!(comp_ticks.added, tick);
        assert_eq!(comp_ticks.changed, tick);
    }

    #[test]
    fn test_component_ticks_mark_changed() {
        let initial_tick = Tick::new(100);
        let mut comp_ticks = ComponentTicks::new(initial_tick);

        let new_tick = Tick::new(150);
        comp_ticks.mark_changed(new_tick);

        assert_eq!(comp_ticks.added, initial_tick); // Should not change
        assert_eq!(comp_ticks.changed, new_tick);
    }

    #[test]
    fn test_component_ticks_is_added() {
        let add_tick = Tick::new(100);
        let comp_ticks = ComponentTicks::new(add_tick);

        let last_run_before = Tick::new(90);
        let last_run_after = Tick::new(110);

        assert!(comp_ticks.is_added(last_run_before));
        assert!(!comp_ticks.is_added(last_run_after));
    }

    #[test]
    fn test_component_ticks_is_changed() {
        let initial_tick = Tick::new(100);
        let mut comp_ticks = ComponentTicks::new(initial_tick);

        let change_tick = Tick::new(150);
        comp_ticks.mark_changed(change_tick);

        let last_run_before = Tick::new(120);
        let last_run_after = Tick::new(160);

        assert!(comp_ticks.is_changed(last_run_before));
        assert!(!comp_ticks.is_changed(last_run_after));
    }

    #[test]
    fn test_component_v1_trait() {
        // Just verify the trait exists and can be implemented
        // The trait is mostly a marker trait
        // (impls moved outside function to avoid warnings)
    }

    #[test]
    fn test_impl_component_macro() {
        // Test that the macro works by using it in the test components above
        // If the macro didn't work, the component implementations would fail
        let pos = Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let vel = Velocity {
            x: 0.1,
            y: 0.2,
            z: 0.3,
        };
        let health = Health(100);

        // Test that they implement Component trait
        let _pos_id = <Position as Component>::type_id();
        let _vel_id = <Velocity as Component>::type_id();
        let _health_id = <Health as Component>::type_id();

        // Test that they can be cloned via the trait
        let _pos_clone = pos.clone_boxed();
        let _vel_clone = vel.clone_boxed();
        let _health_clone = health.clone_boxed();
    }
}
