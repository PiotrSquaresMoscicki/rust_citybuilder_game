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

/// Entity Iterator that returns entities for now (simplified approach)
/// Later can be extended to return component tuples directly
pub struct EntIt<T> {
    entities: Vec<Entity>,
    index: usize,
    _phantom: PhantomData<T>,
}

impl<T> EntIt<T> {
    pub fn new(entities: Vec<Entity>) -> Self {
        Self {
            entities,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<T> Iterator for EntIt<T> {
    type Item = Entity;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entities.len() {
            let entity = self.entities[self.index];
            self.index += 1;
            Some(entity)
        } else {
            None
        }
    }
}

/// World contains entities, components, and systems
pub struct World {
    next_entity_id: Entity,
    pub entities: Vec<Entity>,  // Make public for access
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
    
    /// Create iterator for entities with specified components (simplified API)
    pub fn iter_entities<A1: AccessMode, A2: AccessMode>(&self) -> EntIt<(A1, A2)> {
        let type_ids = vec![A1::component_type_id(), A2::component_type_id()];
        let entities = self.entities_with_components(&type_ids);
        EntIt::new(entities)
    }
    
    /// Create iterator for entities with 1 component
    pub fn iter_entities_1<A1: AccessMode>(&self) -> EntIt<(A1,)> {
        let type_ids = vec![A1::component_type_id()];
        let entities = self.entities_with_components(&type_ids);
        EntIt::new(entities)
    }
    
    /// Create iterator for entities with 3 components
    pub fn iter_entities_3<A1: AccessMode, A2: AccessMode, A3: AccessMode>(&self) -> EntIt<(A1, A2, A3)> {
        let type_ids = vec![A1::component_type_id(), A2::component_type_id(), A3::component_type_id()];
        let entities = self.entities_with_components(&type_ids);
        EntIt::new(entities)
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
        EntIt::new(entities)
    }
}

/// System registry for managing system execution order based on dependencies
pub struct SystemRegistry {
    systems: Vec<(String, Box<dyn FnMut(&World)>)>,
    dependencies: HashMap<String, Vec<String>>,
}

impl SystemRegistry {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            dependencies: HashMap::new(),
        }
    }
    
    /// Register a system with its dependencies
    pub fn register_system<S>(&mut self, name: &str, _system: S) 
    where
        S: System + 'static,
        S::Dependencies: SystemDependencies,
    {
        let dependencies = S::Dependencies::get_dependency_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        self.dependencies.insert(name.to_string(), dependencies);
        
        // Store the system as a closure that calls the system function
        let system_name = name.to_string();
        let closure = Box::new(move |_world: &World| {
            // For simplified approach, we'll just call update with placeholder
            // In a full implementation, we'd create the appropriate iterators
            println!("Executing system: {}", system_name);
        });
        
        self.systems.push((name.to_string(), closure));
    }
    
    /// Execute all systems in dependency order
    pub fn execute_systems(&mut self, world: &World) -> Result<(), String> {
        // Simple execution order for now - in full implementation would do topological sort
        for (name, system_fn) in &mut self.systems {
            println!("Executing system: {}", name);
            system_fn(world);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test components
    #[derive(Clone, Debug)]
    pub struct PositionComponent {
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
    pub struct VelocityComponent {
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
    pub struct TimeSystem;
    impl SystemMarker for TimeSystem {
        fn name() -> &'static str { "TimeSystem" }
    }

    pub struct InputSystem;
    impl SystemMarker for InputSystem {
        fn name() -> &'static str { "InputSystem" }
    }

    pub struct PhysicsSystem;
    impl SystemMarker for PhysicsSystem {
        fn name() -> &'static str { "PhysicsSystem" }
    }

    // Example system matching the target design
    pub struct SampleSystem;

    impl System for SampleSystem {
        type Dependencies = (TimeSystem, InputSystem, PhysicsSystem);
        type Iterators = EntIt<(Mut<PositionComponent>, VelocityComponent)>;

        fn update(&mut self, _iterators: Self::Iterators) {
            // Implementation of the update logic
            println!("SampleSystem executing...");
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
        
        // Test system registry
        let mut registry = SystemRegistry::new();
        registry.register_system("SampleSystem", SampleSystem);
        
        let _ = registry.execute_systems(&world);
    }
    
    #[test]
    fn test_component_access() {
        let mut world = World::new();
        
        let entity = world.create_entity();
        world.add_component(entity, PositionComponent { x: 10.0, y: 20.0 });
        
        // Test immutable access
        {
            let pos = world.get_component::<PositionComponent>(entity).unwrap();
            assert_eq!(pos.x, 10.0);
            assert_eq!(pos.y, 20.0);
        }
        
        // Test mutable access
        {
            let mut pos = world.get_component_mut::<PositionComponent>(entity).unwrap();
            pos.x = 15.0;
            pos.y = 25.0;
        }
        
        // Verify changes
        {
            let pos = world.get_component::<PositionComponent>(entity).unwrap();
            assert_eq!(pos.x, 15.0);
            assert_eq!(pos.y, 25.0);
        }
    }
}