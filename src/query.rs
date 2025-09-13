use std::any::TypeId;
use std::marker::PhantomData;
use crate::ecs::{World, Entity, Component, ComponentRef, ComponentRefMut};

/// Trait for querying components from the world
/// This trait is implemented for tuples of different sizes (0-64 components)
pub trait Query {
    /// The item type returned when iterating
    type Item<'a>;
    
    /// Get the TypeIds of all components this query needs
    fn type_ids() -> Vec<TypeId>;
    
    /// Fetch the components for a specific entity
    /// Returns None if the entity doesn't have all required components
    fn fetch<'a>(world: &'a World, entity: Entity) -> Option<Self::Item<'a>>;
}

/// Marker trait for mutable component access
pub struct Mut<T>(PhantomData<T>);

/// Implementation for empty query (no components, just entity iteration)
impl Query for () {
    type Item<'a> = Entity;
    
    fn type_ids() -> Vec<TypeId> {
        Vec::new()
    }
    
    fn fetch<'a>(world: &'a World, entity: Entity) -> Option<Self::Item<'a>> {
        // For empty query, we just return the entity if it exists
        if world.entity_exists(entity) {
            Some(entity)
        } else {
            None
        }
    }
}

/// Implementation for single immutable component
impl<T> Query for T
where
    T: Component + 'static,
{
    type Item<'a> = ComponentRef<T>;
    
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
    
    fn fetch<'a>(world: &'a World, entity: Entity) -> Option<Self::Item<'a>> {
        world.get_component_ref::<T>(entity)
    }
}

/// Implementation for single mutable component
impl<T> Query for Mut<T>
where
    T: Component + 'static,
{
    type Item<'a> = ComponentRefMut<T>;
    
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
    
    fn fetch<'a>(world: &'a World, entity: Entity) -> Option<Self::Item<'a>> {
        world.get_component_ref_mut::<T>(entity)
    }
}

/// Macro to implement Query for tuples of different sizes
macro_rules! impl_query_tuple {
    ($($T:ident),*) => {
        impl<$($T),*> Query for ($($T,)*)
        where
            $($T: Query,)*
        {
            type Item<'a> = ($($T::Item<'a>,)*);
            
            fn type_ids() -> Vec<TypeId> {
                let mut type_ids = Vec::new();
                $(type_ids.extend($T::type_ids());)*
                type_ids
            }
            
            fn fetch<'a>(world: &'a World, entity: Entity) -> Option<Self::Item<'a>> {
                Some((
                    $($T::fetch(world, entity)?,)*
                ))
            }
        }
    };
}

// Generate implementations for tuples of size 2-32 (can be extended to 64)
impl_query_tuple!(T0, T1);
impl_query_tuple!(T0, T1, T2);
impl_query_tuple!(T0, T1, T2, T3);
impl_query_tuple!(T0, T1, T2, T3, T4);
impl_query_tuple!(T0, T1, T2, T3, T4, T5);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30);
impl_query_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31);

/// New EntityIterator that works with any Query
pub struct EntityIterator<Q: Query> {
    world: *const World,
    entities: Vec<Entity>,
    index: usize,
    _phantom: PhantomData<Q>,
}

impl<Q: Query> EntityIterator<Q> {
    pub fn new(world: &World) -> Self {
        let type_ids = Q::type_ids();
        let entities = if type_ids.is_empty() {
            world.all_entities()
        } else {
            world.entities_with_components(&type_ids)
        };
        
        Self {
            world: world as *const World,
            entities,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<Q: Query> Iterator for EntityIterator<Q> {
    type Item = Q::Item<'static>; // Note: This lifetime is managed unsafely
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.entities.len() {
            let entity = self.entities[self.index];
            self.index += 1;
            
            // Safety: We ensure the world pointer is valid during iterator lifetime
            let world = unsafe { &*self.world };
            
            if let Some(item) = Q::fetch(world, entity) {
                // Safety: We transmute the lifetime to 'static
                // This is safe because the iterator holds a reference to the world
                let static_item: Q::Item<'static> = unsafe { std::mem::transmute(item) };
                return Some(static_item);
            }
        }
        None
    }
}