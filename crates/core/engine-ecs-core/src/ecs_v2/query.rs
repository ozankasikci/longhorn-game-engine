//! Query system for ECS V2
//!
//! Provides type-safe queries for accessing components across entities.

use crate::{Entity, World};
use engine_component_traits::Component;
use std::marker::PhantomData;

/// Marker trait for types that can be queried
pub trait QueryData: 'static {
    type Item<'a>;
    fn fetch<'a>(entity: Entity, world: &'a World) -> Option<Self::Item<'a>>;
}

/// Read-only component access
pub struct Read<T: Component>(PhantomData<T>);

impl<T: Component> QueryData for Read<T> {
    type Item<'a> = &'a T;

    fn fetch<'a>(entity: Entity, world: &'a World) -> Option<Self::Item<'a>> {
        world.get_component::<T>(entity)
    }
}

/// Mutable component access
pub struct Write<T: Component>(PhantomData<T>);

impl<T: Component> QueryData for Write<T> {
    type Item<'a> = &'a mut T;

    fn fetch<'a>(_entity: Entity, _world: &'a World) -> Option<Self::Item<'a>> {
        // For now, return None - proper implementation would need unsafe code
        None
    }
}

/// Changed component filter
pub struct Changed<T: Component>(PhantomData<T>);

impl<T: Component> QueryData for Changed<T> {
    type Item<'a> = &'a T;

    fn fetch<'a>(entity: Entity, world: &'a World) -> Option<Self::Item<'a>> {
        // For now, just return the component - proper implementation would check change ticks
        world.get_component::<T>(entity)
    }
}

/// Query builder for iterating over entities with specific components
pub struct Query<Q: QueryData> {
    _phantom: PhantomData<Q>,
}

impl<Q: QueryData> Query<Q> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<Q: QueryData> Default for Query<Q> {
    fn default() -> Self {
        Self::new()
    }
}

/// Mutable query builder
pub struct QueryMut<Q: QueryData> {
    _phantom: PhantomData<Q>,
}

impl<Q: QueryData> QueryMut<Q> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<Q: QueryData> Default for QueryMut<Q> {
    fn default() -> Self {
        Self::new()
    }
}

// For now, we'll use marker structs for tuple queries instead of implementing on references
pub struct Query1<T1: Component>(PhantomData<T1>);

impl<T1: Component> QueryData for Query1<T1> {
    type Item<'a> = &'a T1;

    fn fetch<'a>(entity: Entity, world: &'a World) -> Option<Self::Item<'a>> {
        world.get_component::<T1>(entity)
    }
}

pub struct Query2<T1: Component, T2: Component>(PhantomData<(T1, T2)>);

impl<T1: Component, T2: Component> QueryData for Query2<T1, T2> {
    type Item<'a> = (&'a T1, &'a T2);

    fn fetch<'a>(entity: Entity, world: &'a World) -> Option<Self::Item<'a>> {
        Some((
            world.get_component::<T1>(entity)?,
            world.get_component::<T2>(entity)?,
        ))
    }
}

pub struct Query3<T1: Component, T2: Component, T3: Component>(PhantomData<(T1, T2, T3)>);

impl<T1: Component, T2: Component, T3: Component> QueryData for Query3<T1, T2, T3> {
    type Item<'a> = (&'a T1, &'a T2, &'a T3);

    fn fetch<'a>(entity: Entity, world: &'a World) -> Option<Self::Item<'a>> {
        Some((
            world.get_component::<T1>(entity)?,
            world.get_component::<T2>(entity)?,
            world.get_component::<T3>(entity)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs_v2::test_utils::*;

    #[test]
    fn test_query_creation() {
        let _query = Query::<Read<Position>>::new();
        let _query_mut = QueryMut::<Write<Position>>::new();
    }

    #[test]
    fn test_read_query_data() {
        register_test_components();

        let mut world = World::new();
        let entity = world.spawn();
        world
            .add_component(entity, Position::new(1.0, 2.0, 3.0))
            .unwrap();

        let pos = Read::<Position>::fetch(entity, &world);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().x, 1.0);
    }

    #[test]
    fn test_tuple_query_data() {
        register_test_components();

        let mut world = World::new();
        let entity = world.spawn();
        world
            .add_component(entity, Position::new(1.0, 2.0, 3.0))
            .unwrap();
        world
            .add_component(entity, Velocity::new(4.0, 5.0, 6.0))
            .unwrap();

        let result = Query2::<Position, Velocity>::fetch(entity, &world);
        assert!(result.is_some());

        let (pos, vel) = result.unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(vel.x, 4.0);
    }

    #[test]
    fn test_changed_query_data() {
        register_test_components();

        let mut world = World::new();
        let entity = world.spawn();
        world
            .add_component(entity, Position::new(1.0, 2.0, 3.0))
            .unwrap();

        let pos = Changed::<Position>::fetch(entity, &world);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().x, 1.0);
    }

    #[test]
    fn test_missing_component() {
        register_test_components();

        let mut world = World::new();
        let entity = world.spawn();
        world
            .add_component(entity, Position::new(1.0, 2.0, 3.0))
            .unwrap();

        // Try to fetch Position and Velocity, but entity only has Position
        let result = Query2::<Position, Velocity>::fetch(entity, &world);
        assert!(result.is_none());
    }
}
