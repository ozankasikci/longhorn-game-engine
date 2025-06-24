//! Component storage and management for ECS V2
//!
//! This module provides the component storage infrastructure including:
//! - Type-erased component arrays
//! - Component registration system
//! - Component array factories

use crate::error::{EcsError, EcsResult};
use engine_component_traits::{Component, ComponentClone, ComponentTicks};
use once_cell::sync::Lazy;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Factory function for creating component arrays
pub type ComponentArrayFactory = Arc<dyn Fn() -> Box<dyn ComponentArrayTrait> + Send + Sync>;

/// Global component registry for dynamic component array creation
static COMPONENT_REGISTRY: Lazy<Mutex<HashMap<TypeId, ComponentArrayFactory>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Register a component type in the global registry
///
/// This must be called for each component type before it can be used in the ECS.
/// Typically called during application initialization.
///
/// # Example
/// ```rust
/// use engine_ecs_core::ecs_v2::register_component;
/// use engine_component_traits::Component;
/// 
/// #[derive(Clone, Debug)]
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
/// 
/// #[derive(Clone, Debug)]
/// struct Velocity { x: f32, y: f32 }
/// impl Component for Velocity {}
/// 
/// register_component::<Position>();
/// register_component::<Velocity>();
/// ```
pub fn register_component<T: Component>() {
    let type_id = TypeId::of::<T>();
    let factory: ComponentArrayFactory = Arc::new(|| Box::new(ComponentArray::<T>::new()));

    COMPONENT_REGISTRY.lock().unwrap().insert(type_id, factory);
}

/// Create a new component array for the given type
pub(crate) fn create_component_array(type_id: TypeId) -> Option<Box<dyn ComponentArrayTrait>> {
    COMPONENT_REGISTRY
        .lock()
        .unwrap()
        .get(&type_id)
        .map(|factory| factory())
}

/// Check if a component type is registered
pub fn is_component_registered(type_id: TypeId) -> bool {
    COMPONENT_REGISTRY.lock().unwrap().contains_key(&type_id)
}

/// Trait for type-erased component storage operations
///
/// This allows us to store different component types in a homogeneous collection
/// while maintaining type safety through downcasting when needed.
pub trait ComponentArrayTrait: Send + Sync {
    /// Remove element at index by swapping with last element
    fn swap_remove(&mut self, index: usize);

    /// Get the number of elements
    fn len(&self) -> usize;

    /// Check if the array is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get capacity of the underlying storage
    fn capacity(&self) -> usize;

    /// Get the TypeId of the stored component
    fn type_id(&self) -> TypeId;

    /// Downcast to Any for type-specific operations
    fn as_any(&self) -> &dyn Any;

    /// Downcast to Any for type-specific mutable operations
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Clone a component at the given index
    fn clone_component_at(&self, index: usize) -> Option<Box<dyn ComponentClone>>;

    /// Get component ticks at the given index
    fn get_ticks_at(&self, index: usize) -> Option<ComponentTicks>;

    /// Push a cloned component
    fn push_cloned(
        &mut self,
        component: Box<dyn ComponentClone>,
        ticks: ComponentTicks,
    ) -> EcsResult<()>;
}

/// Storage for a single component type within an archetype
///
/// Components are stored in contiguous arrays for cache efficiency.
/// Each component has associated change tracking ticks.
pub struct ComponentArray<T: Component> {
    data: Vec<T>,
    ticks: Vec<ComponentTicks>,
}

impl<T: Component> ComponentArray<T> {
    /// Create a new empty component array
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            ticks: Vec::new(),
        }
    }

    /// Create a new component array with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            ticks: Vec::with_capacity(capacity),
        }
    }

    /// Push a component with its ticks
    pub fn push(&mut self, component: T, ticks: ComponentTicks) {
        self.data.push(component);
        self.ticks.push(ticks);
    }

    /// Get a reference to a component at the given index
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Get a mutable reference to a component at the given index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
}

impl<T: Component> ComponentArrayTrait for ComponentArray<T> {
    fn swap_remove(&mut self, index: usize) {
        self.data.swap_remove(index);
        self.ticks.swap_remove(index);
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn capacity(&self) -> usize {
        self.data.capacity()
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_component_at(&self, index: usize) -> Option<Box<dyn ComponentClone>> {
        self.data.get(index).map(|comp| {
            // T: Component which implements ComponentClone
            comp.clone_boxed()
        })
    }

    fn get_ticks_at(&self, index: usize) -> Option<ComponentTicks> {
        self.ticks.get(index).copied()
    }

    fn push_cloned(
        &mut self,
        component: Box<dyn ComponentClone>,
        ticks: ComponentTicks,
    ) -> EcsResult<()> {
        // We need to use into_any to consume the box and downcast
        let any_box = component.into_any();
        if let Ok(typed_box) = any_box.downcast::<T>() {
            self.push(*typed_box, ticks);
            Ok(())
        } else {
            Err(EcsError::ComponentTypeMismatch)
        }
    }
}

/// Type-erased component array for dynamic component handling
pub struct ErasedComponentArray {
    inner: Box<dyn ComponentArrayTrait>,
}

impl ErasedComponentArray {
    /// Create a new erased component array
    pub fn new(inner: Box<dyn ComponentArrayTrait>) -> Self {
        Self { inner }
    }

    /// Get the inner component array trait object
    pub fn inner(&self) -> &dyn ComponentArrayTrait {
        &*self.inner
    }

    /// Get the inner component array trait object mutably
    pub fn inner_mut(&mut self) -> &mut dyn ComponentArrayTrait {
        &mut *self.inner
    }

    /// Try to downcast to a specific component array type
    pub fn downcast_ref<T: Component>(&self) -> Option<&ComponentArray<T>> {
        self.inner.as_any().downcast_ref()
    }

    /// Try to downcast to a specific component array type mutably
    pub fn downcast_mut<T: Component>(&mut self) -> Option<&mut ComponentArray<T>> {
        self.inner.as_any_mut().downcast_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_component_traits::Tick;

    #[derive(Clone, Debug, PartialEq)]
    struct TestComponent {
        value: i32,
    }

    impl Component for TestComponent {}

    #[derive(Clone, Debug, PartialEq)]
    struct AnotherComponent {
        data: String,
    }

    impl Component for AnotherComponent {}

    #[test]
    fn test_component_registration() {
        // Use a unique component type for this test to avoid conflicts
        #[derive(Clone, Debug, PartialEq)]
        struct UniqueTestComponent {
            value: i32,
        }
        impl Component for UniqueTestComponent {}

        let type_id = TypeId::of::<UniqueTestComponent>();

        // Initially not registered
        assert!(!is_component_registered(type_id));

        // Register component
        register_component::<UniqueTestComponent>();
        assert!(is_component_registered(type_id));

        // Re-registration should be idempotent
        register_component::<UniqueTestComponent>();
        assert!(is_component_registered(type_id));
    }

    #[test]
    fn test_component_array_creation() {
        register_component::<TestComponent>();

        let array = create_component_array(TypeId::of::<TestComponent>());
        assert!(array.is_some());

        let unregistered = create_component_array(TypeId::of::<AnotherComponent>());
        assert!(unregistered.is_none());
    }

    #[test]
    fn test_component_array_basic_operations() {
        let mut array = ComponentArray::<TestComponent>::new();
        let tick1 = ComponentTicks::new(Tick::new(1));
        let tick2 = ComponentTicks::new(Tick::new(2));

        // Push components
        array.push(TestComponent { value: 42 }, tick1);
        array.push(TestComponent { value: 84 }, tick2);

        assert_eq!(array.len(), 2);
        assert!(!array.is_empty());

        // Get components
        assert_eq!(array.get(0).unwrap().value, 42);
        assert_eq!(array.get(1).unwrap().value, 84);
        assert!(array.get(2).is_none());

        // Get mutable
        if let Some(comp) = array.get_mut(0) {
            comp.value = 100;
        }
        assert_eq!(array.get(0).unwrap().value, 100);
    }

    #[test]
    fn test_component_array_with_capacity() {
        let array = ComponentArray::<TestComponent>::with_capacity(100);
        assert!(array.capacity() >= 100);
        assert_eq!(array.len(), 0);
    }

    #[test]
    fn test_component_array_swap_remove() {
        let mut array = ComponentArray::<TestComponent>::new();
        let tick = ComponentTicks::new(Tick::new(1));

        array.push(TestComponent { value: 1 }, tick);
        array.push(TestComponent { value: 2 }, tick);
        array.push(TestComponent { value: 3 }, tick);

        // Remove middle element
        array.swap_remove(1);

        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0).unwrap().value, 1);
        assert_eq!(array.get(1).unwrap().value, 3); // Last element moved here
    }

    #[test]
    fn test_component_array_trait_object() {
        let mut array = ComponentArray::<TestComponent>::new();
        let tick = ComponentTicks::new(Tick::new(1));
        array.push(TestComponent { value: 42 }, tick);

        let trait_obj: &dyn ComponentArrayTrait = &array;
        assert_eq!(trait_obj.len(), 1);
        assert_eq!(trait_obj.type_id(), TypeId::of::<TestComponent>());

        // Test downcasting
        let downcast = trait_obj
            .as_any()
            .downcast_ref::<ComponentArray<TestComponent>>();
        assert!(downcast.is_some());
        assert_eq!(downcast.unwrap().get(0).unwrap().value, 42);
    }

    #[test]
    fn test_component_cloning() {
        let mut array = ComponentArray::<TestComponent>::new();
        let tick = ComponentTicks::new(Tick::new(1));
        array.push(TestComponent { value: 42 }, tick);

        let cloned = array.clone_component_at(0);
        assert!(cloned.is_some());

        let cloned_box = cloned.unwrap();
        let cloned_comp = cloned_box.as_any().downcast_ref::<TestComponent>();
        assert!(cloned_comp.is_some());
        assert_eq!(cloned_comp.unwrap().value, 42);
    }

    #[test]
    fn test_component_ticks() {
        let mut array = ComponentArray::<TestComponent>::new();
        let tick1 = ComponentTicks::new(Tick::new(10));
        let tick2 = ComponentTicks::new(Tick::new(20));

        array.push(TestComponent { value: 1 }, tick1);
        array.push(TestComponent { value: 2 }, tick2);

        let ticks = array.get_ticks_at(0);
        assert!(ticks.is_some());
        assert_eq!(ticks.unwrap().added.get(), 10);

        let ticks2 = array.get_ticks_at(1);
        assert_eq!(ticks2.unwrap().added.get(), 20);
    }

    #[test]
    fn test_push_cloned() {
        let mut array = ComponentArray::<TestComponent>::new();
        let component = TestComponent { value: 42 };
        let tick = ComponentTicks::new(Tick::new(1));

        let cloned_box: Box<dyn ComponentClone> = Box::new(component.clone());
        let result = array.push_cloned(cloned_box, tick);

        assert!(result.is_ok());
        assert_eq!(array.len(), 1);
        assert_eq!(array.get(0).unwrap().value, 42);
    }

    #[test]
    fn test_push_cloned_wrong_type() {
        let mut array = ComponentArray::<TestComponent>::new();
        let wrong_component = AnotherComponent {
            data: "wrong".to_string(),
        };
        let tick = ComponentTicks::new(Tick::new(1));

        let cloned_box: Box<dyn ComponentClone> = Box::new(wrong_component);
        let result = array.push_cloned(cloned_box, tick);

        assert!(result.is_err());
        match result {
            Err(EcsError::ComponentTypeMismatch) => (),
            _ => panic!("Expected ComponentTypeMismatch error"),
        }
    }

    #[test]
    fn test_erased_component_array() {
        let mut inner = ComponentArray::<TestComponent>::new();
        let tick = ComponentTicks::new(Tick::new(1));
        inner.push(TestComponent { value: 42 }, tick);

        let mut erased = ErasedComponentArray::new(Box::new(inner));

        // Test downcasting
        let downcast = erased.downcast_ref::<TestComponent>();
        assert!(downcast.is_some());
        assert_eq!(downcast.unwrap().get(0).unwrap().value, 42);

        // Test mutable downcasting
        let downcast_mut = erased.downcast_mut::<TestComponent>();
        assert!(downcast_mut.is_some());
        downcast_mut.unwrap().get_mut(0).unwrap().value = 100;

        // Verify change
        let downcast = erased.downcast_ref::<TestComponent>();
        assert_eq!(downcast.unwrap().get(0).unwrap().value, 100);
    }

    #[test]
    fn test_component_registry_thread_safety() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::thread;

        let registered = Arc::new(AtomicBool::new(false));
        let registered_clone = registered.clone();

        // Register from another thread
        let handle = thread::spawn(move || {
            register_component::<TestComponent>();
            registered_clone.store(true, Ordering::SeqCst);
        });

        handle.join().unwrap();

        assert!(registered.load(Ordering::SeqCst));
        assert!(is_component_registered(TypeId::of::<TestComponent>()));
    }
}
