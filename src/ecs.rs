use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::cell::{RefCell, Ref, RefMut};

/// Entity is just a unique identifier
pub type Entity = u32;

/// Component trait for validation, getters, setters, and utility functions
pub trait Component: Any + Send + Sync {
    /// Validates the component state
    fn validate(&self) -> bool {
        true // Default implementation
    }
    
    /// Convert to Any trait object for type erasure
    fn as_any(&self) -> &dyn Any;
    
    /// Convert to mutable Any trait object for type erasure
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    /// Create a deep copy of this component for diffing purposes
    fn clone_box(&self) -> Box<dyn Component>;
}

/// Mut<T> wrapper to explicitly mark components that should be accessed mutably
pub struct Mut<T> {
    _phantom: PhantomData<T>,
}

/// Trait to determine if a type represents mutable access
pub trait AccessMode {
    type Component: Component + 'static;
    
    /// Returns true if this access mode requires mutable access
    fn is_mutable() -> bool;
    
    /// Get the TypeId of the underlying component
    fn component_type_id() -> TypeId {
        TypeId::of::<Self::Component>()
    }
}

/// Implementation for immutable access (plain component types)
impl<T: Component + 'static> AccessMode for T {
    type Component = T;
    
    fn is_mutable() -> bool {
        false
    }
}

/// Implementation for mutable access (Mut<T> wrapper)
impl<T: Component + 'static> AccessMode for Mut<T> {
    type Component = T;
    
    fn is_mutable() -> bool {
        true
    }
}

/// Storage for a specific component type using RefCell for interior mutability
pub struct ComponentPool {
    components: HashMap<Entity, RefCell<Box<dyn Component>>>,
}

impl ComponentPool {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
    
    pub fn insert(&mut self, entity: Entity, component: Box<dyn Component>) {
        self.components.insert(entity, RefCell::new(component));
    }
    
    pub fn get(&self, entity: Entity) -> Option<Ref<'_, Box<dyn Component>>> {
        self.components.get(&entity).map(|c| c.borrow())
    }
    
    pub fn get_mut(&self, entity: Entity) -> Option<RefMut<'_, Box<dyn Component>>> {
        self.components.get(&entity).map(|c| c.borrow_mut())
    }
    
    pub fn remove(&mut self, entity: Entity) -> Option<RefCell<Box<dyn Component>>> {
        self.components.remove(&entity)
    }
    
    pub fn contains(&self, entity: Entity) -> bool {
        self.components.contains_key(&entity)
    }
    
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.components.keys().copied()
    }
}

/// System trait with Dependencies and Iterators associated types as specified
pub trait System {
    type Dependencies;
    type Iterators;

    fn update(&mut self, iterators: Self::Iterators);
}

/// Helper trait for system dependency resolution 
pub trait SystemDependencies {
    /// Get the list of system type names this depends on
    fn get_dependency_names() -> Vec<&'static str>;
}

/// Implementation for empty dependencies (no dependencies)
impl SystemDependencies for () {
    fn get_dependency_names() -> Vec<&'static str> {
        vec![]
    }
}

/// Implementation for single dependency
impl<T: SystemMarker> SystemDependencies for T {
    fn get_dependency_names() -> Vec<&'static str> {
        vec![T::name()]
    }
}

/// Implementation for tuple dependencies (2 elements)
impl<T1: SystemMarker, T2: SystemMarker> SystemDependencies for (T1, T2) {
    fn get_dependency_names() -> Vec<&'static str> {
        vec![T1::name(), T2::name()]
    }
}

/// Implementation for tuple dependencies (3 elements)
impl<T1: SystemMarker, T2: SystemMarker, T3: SystemMarker> SystemDependencies for (T1, T2, T3) {
    fn get_dependency_names() -> Vec<&'static str> {
        vec![T1::name(), T2::name(), T3::name()]
    }
}

/// Marker trait for systems to provide their type name
pub trait SystemMarker {
    fn name() -> &'static str;
}

/// Entity Iterator that returns component tuples (variable number of components 0-64)
pub struct EntIt<T> {
    world: *const World,
    entities: Vec<Entity>,
    index: usize,
    _phantom: PhantomData<T>,
}

/// Implementation for EntIt with 2 components (main case from problem statement)
impl<A1: AccessMode, A2: AccessMode> EntIt<(A1, A2)> {
    fn new_2(world: *const World, entities: Vec<Entity>) -> Self {
        Self {
            world,
            entities,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

/// Implementation for EntIt with 4 components (extended case from problem statement)
impl<A1: AccessMode, A2: AccessMode, A3: AccessMode, A4: AccessMode> EntIt<(A1, A2, A3, A4)> {
    fn new_4(world: *const World, entities: Vec<Entity>) -> Self {
        Self {
            world,
            entities,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

/// Iterator implementation for 2 components
impl<A1: AccessMode, A2: AccessMode> Iterator for EntIt<(A1, A2)> {
    type Item = (EntityComponentRef<A1::Component>, EntityComponentRef<A2::Component>);
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.entities.len() {
            return None;
        }
        
        let entity = self.entities[self.index];
        self.index += 1;
        
        unsafe {
            let world = &*self.world;
            
            // Get first component
            let comp1 = if A1::is_mutable() {
                EntityComponentRef::Mutable(world.get_component_mut_raw::<A1::Component>(entity)?)
            } else {
                EntityComponentRef::Immutable(world.get_component_raw::<A1::Component>(entity)?)
            };
            
            // Get second component
            let comp2 = if A2::is_mutable() {
                EntityComponentRef::Mutable(world.get_component_mut_raw::<A2::Component>(entity)?)
            } else {
                EntityComponentRef::Immutable(world.get_component_raw::<A2::Component>(entity)?)
            };
            
            Some((comp1, comp2))
        }
    }
}

/// Iterator implementation for 4 components
impl<A1: AccessMode, A2: AccessMode, A3: AccessMode, A4: AccessMode> Iterator for EntIt<(A1, A2, A3, A4)> {
    type Item = (
        EntityComponentRef<A1::Component>, 
        EntityComponentRef<A2::Component>,
        EntityComponentRef<A3::Component>,
        EntityComponentRef<A4::Component>
    );
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.entities.len() {
            return None;
        }
        
        let entity = self.entities[self.index];
        self.index += 1;
        
        unsafe {
            let world = &*self.world;
            
            // Get components
            let comp1 = if A1::is_mutable() {
                EntityComponentRef::Mutable(world.get_component_mut_raw::<A1::Component>(entity)?)
            } else {
                EntityComponentRef::Immutable(world.get_component_raw::<A1::Component>(entity)?)
            };
            
            let comp2 = if A2::is_mutable() {
                EntityComponentRef::Mutable(world.get_component_mut_raw::<A2::Component>(entity)?)
            } else {
                EntityComponentRef::Immutable(world.get_component_raw::<A2::Component>(entity)?)
            };
            
            let comp3 = if A3::is_mutable() {
                EntityComponentRef::Mutable(world.get_component_mut_raw::<A3::Component>(entity)?)
            } else {
                EntityComponentRef::Immutable(world.get_component_raw::<A3::Component>(entity)?)
            };
            
            let comp4 = if A4::is_mutable() {
                EntityComponentRef::Mutable(world.get_component_mut_raw::<A4::Component>(entity)?)
            } else {
                EntityComponentRef::Immutable(world.get_component_raw::<A4::Component>(entity)?)
            };
            
            Some((comp1, comp2, comp3, comp4))
        }
    }
}

/// Wrapper for component references that can be either mutable or immutable
pub enum EntityComponentRef<T: Component> {
    Immutable(*const T),
    Mutable(*mut T),
}

impl<T: Component> EntityComponentRef<T> {
    /// Get an immutable reference to the component
    pub fn get(&self) -> &T {
        unsafe {
            match self {
                EntityComponentRef::Immutable(ptr) => &**ptr,
                EntityComponentRef::Mutable(ptr) => &**ptr,
            }
        }
    }
    
    /// Get a mutable reference to the component (only works for Mutable variants)
    pub fn get_mut(&mut self) -> Option<&mut T> {
        unsafe {
            match self {
                EntityComponentRef::Immutable(_) => None,
                EntityComponentRef::Mutable(ptr) => Some(&mut **ptr),
            }
        }
    }
}

/// World contains entities, components, and systems
pub struct World {
    next_entity_id: Entity,
    entities: Vec<Entity>,
    component_pools: HashMap<TypeId, ComponentPool>,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            entities: Vec::new(),
            component_pools: HashMap::new(),
        }
    }
    
    /// Create a new entity and return its ID
    pub fn create_entity(&mut self) -> Entity {
        let entity = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }
    
    /// Add a component to an entity
    pub fn add_component<T: Component + 'static>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let pool = self.component_pools
            .entry(type_id)
            .or_insert_with(ComponentPool::new);
        pool.insert(entity, Box::new(component));
    }
    
    /// Get a component from an entity (immutable)
    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<impl std::ops::Deref<Target = T> + '_> {
        let type_id = TypeId::of::<T>();
        let pool = self.component_pools.get(&type_id)?;
        let component = pool.get(entity)?;
        
        // Use Ref::map to safely project the reference
        Some(Ref::map(component, |c| c.as_any().downcast_ref::<T>().unwrap()))
    }
    
    /// Get a component from an entity (mutable)
    pub fn get_component_mut<T: Component + 'static>(&self, entity: Entity) -> Option<impl std::ops::DerefMut<Target = T> + '_> {
        let type_id = TypeId::of::<T>();
        let pool = self.component_pools.get(&type_id)?;
        let component = pool.get_mut(entity)?;
        
        // Use RefMut::map to safely project the reference
        Some(RefMut::map(component, |c| c.as_any_mut().downcast_mut::<T>().unwrap()))
    }
    
    /// Get raw pointer to component (for internal iterator use)
    unsafe fn get_component_raw<T: Component + 'static>(&self, entity: Entity) -> Option<*const T> {
        let type_id = TypeId::of::<T>();
        let pool = self.component_pools.get(&type_id)?;
        let component = pool.get(entity)?;
        let raw_ptr = component.as_any().downcast_ref::<T>()? as *const T;
        std::mem::forget(component); // Prevent Drop from running
        Some(raw_ptr)
    }
    
    /// Get raw mutable pointer to component (for internal iterator use)
    unsafe fn get_component_mut_raw<T: Component + 'static>(&self, entity: Entity) -> Option<*mut T> {
        let type_id = TypeId::of::<T>();
        let pool = self.component_pools.get(&type_id)?;
        let mut component = pool.get_mut(entity)?;
        let raw_ptr = component.as_any_mut().downcast_mut::<T>()? as *mut T;
        std::mem::forget(component); // Prevent Drop from running
        Some(raw_ptr)
    }
    
    /// Remove a component from an entity
    pub fn remove_component<T: Component + 'static>(&mut self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        if let Some(pool) = self.component_pools.get_mut(&type_id) {
            pool.remove(entity).is_some()
        } else {
            false
        }
    }
    
    /// Check if an entity has a specific component
    pub fn has_component<T: Component + 'static>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        if let Some(pool) = self.component_pools.get(&type_id) {
            pool.contains(entity)
        } else {
            false
        }
    }
    
    /// Get entities that have all specified component types
    pub fn entities_with_components(&self, component_types: &[TypeId]) -> Vec<Entity> {
        if component_types.is_empty() {
            return self.entities.clone();
        }
        
        let mut result = Vec::new();
        
        for &entity in &self.entities {
            let has_all = component_types.iter().all(|&type_id| {
                self.component_pools
                    .get(&type_id)
                    .map_or(false, |pool| pool.contains(entity))
            });
            
            if has_all {
                result.push(entity);
            }
        }
        
        result
    }
    
    /// Create iterator for entities with 2 components
    pub fn iter_entities<A1: AccessMode, A2: AccessMode>(&self) -> EntIt<(A1, A2)> {
        let type_ids = vec![A1::component_type_id(), A2::component_type_id()];
        let entities = self.entities_with_components(&type_ids);
        EntIt::<(A1, A2)>::new_2(self as *const World, entities)
    }
    
    /// Create iterator for entities with 4 components  
    pub fn iter_entities_4<A1: AccessMode, A2: AccessMode, A3: AccessMode, A4: AccessMode>(&self) -> EntIt<(A1, A2, A3, A4)> {
        let type_ids = vec![
            A1::component_type_id(), 
            A2::component_type_id(),
            A3::component_type_id(),
            A4::component_type_id()
        ];
        let entities = self.entities_with_components(&type_ids);
        EntIt::<(A1, A2, A3, A4)>::new_4(self as *const World, entities)
    }
    
    /// Get all entities in the world (for compatibility with legacy code)
    pub fn get_all_entities(&self) -> &Vec<Entity> {
        &self.entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test components
    #[derive(Clone, Debug)]
    struct PositionComponent {
        pub x: f32,
        pub y: f32,
    }

    impl Component for PositionComponent {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn clone_box(&self) -> Box<dyn Component> {
            Box::new(self.clone())
        }
    }

    #[derive(Clone, Debug)]
    struct VelocityComponent {
        pub dx: f32,
        pub dy: f32,
    }

    impl Component for VelocityComponent {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn clone_box(&self) -> Box<dyn Component> {
            Box::new(self.clone())
        }
    }

    // System markers for testing
    struct TimeSystem;
    impl SystemMarker for TimeSystem {
        fn name() -> &'static str { "TimeSystem" }
    }

    struct InputSystem;
    impl SystemMarker for InputSystem {
        fn name() -> &'static str { "InputSystem" }
    }

    struct PhysicsSystem;
    impl SystemMarker for PhysicsSystem {
        fn name() -> &'static str { "PhysicsSystem" }
    }

    // Example system matching the target design
    struct SampleSystem;

    impl System for SampleSystem {
        type Dependencies = (TimeSystem, InputSystem, PhysicsSystem);
        type Iterators = EntIt<(Mut<PositionComponent>, VelocityComponent)>;

        fn update(&mut self, iterators: Self::Iterators) {
            // Implementation of the update logic
            for (_position, _velocity) in iterators {
                // Can access components directly as tuples
            }
        }
    }

    #[test]
    fn test_clean_ecs_system_trait() {
        let mut world = World::new();
        
        // Create an entity with components
        let entity = world.create_entity();
        world.add_component(entity, PositionComponent { x: 0.0, y: 0.0 });
        world.add_component(entity, VelocityComponent { dx: 1.0, dy: 2.0 });
        
        // Test the new iterator API
        let iter = world.iter_entities::<Mut<PositionComponent>, VelocityComponent>();
        
        // Create and use the system
        let mut sample_system = SampleSystem;
        sample_system.update(iter);
    }
}