use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::cell::{RefCell, Ref, RefMut};

/// Entity is just a unique identifier
pub type Entity = u32;

/// Component trait for validation, getters, setters, and utility functions
pub trait Component: Any {
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

impl<T> Mut<T> {
    /// Extract the inner type from Mut<T>
    pub fn inner_type() -> PhantomData<T> {
        PhantomData
    }
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
    
    fn remove(&mut self, entity: Entity) -> Option<RefCell<Box<dyn Component>>> {
        self.components.remove(&entity)
    }
    
    pub fn contains(&self, entity: Entity) -> bool {
        self.components.contains_key(&entity)
    }
    
    fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.components.keys().copied()
    }
}

/// New System trait with Dependencies and Iterators associated types
pub trait System {
    type Dependencies;
    type Iterators;

    fn update(&mut self, iterators: Self::Iterators);
}

/// Helper trait to convert system dependencies into iterator types
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

/// Entity Iterator trait for working with variable number of components
pub trait EntIt {
    type Item;
    
    fn next(&mut self) -> Option<Self::Item>;
}

/// Component reference wrapper for immutable access
pub struct ComponentRef<T> {
    _ref: Ref<'static, Box<dyn Component>>,
    _phantom: PhantomData<T>,
}

/// Component reference wrapper for mutable access
pub struct ComponentRefMut<T> {
    _ref: RefMut<'static, Box<dyn Component>>,
    _phantom: PhantomData<T>,
}

/// Entity iterator for 1 component
pub struct EntIt1<A1> {
    world: *const World,
    entities: Vec<Entity>,
    index: usize,
    _phantom: PhantomData<A1>,
}

/// Entity iterator for 2 components
pub struct EntIt2<A1, A2> {
    world: *const World,
    entities: Vec<Entity>,
    index: usize,
    _phantom: PhantomData<(A1, A2)>,
}

/// Entity iterator for 3 components
pub struct EntIt3<A1, A2, A3> {
    world: *const World,
    entities: Vec<Entity>,
    index: usize,
    _phantom: PhantomData<(A1, A2, A3)>,
}

/// Implementation for tuple iterators (1 element)
impl<A1: SystemMarker> SystemDependencies for (EntIt1<A1>,) {
    fn get_dependency_names() -> Vec<&'static str> {
        vec![]
    }
}

/// Implementation for tuple iterators (2 elements)
impl<A1: SystemMarker, A2: SystemMarker> SystemDependencies for (EntIt1<A1>, EntIt1<A2>) {
    fn get_dependency_names() -> Vec<&'static str> {
        vec![]
    }
}

/// World contains entities, components, and systems
pub struct World {
    next_entity_id: Entity,
    entities: Vec<Entity>,
    component_pools: HashMap<TypeId, ComponentPool>,
    systems: Vec<Box<dyn System<Dependencies = (), Iterators = ()>>>,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            entities: Vec::new(),
            component_pools: HashMap::new(),
            systems: Vec::new(),
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
    
    /// Create iterator for entities with specific component type
    pub fn iter_entities_1<A1: AccessMode>(&self) -> EntIt1<A1> {
        let type_ids = vec![A1::component_type_id()];
        let entities = self.entities_with_components(&type_ids);
        EntIt1 {
            world: self as *const World,
            entities,
            index: 0,
            _phantom: PhantomData,
        }
    }
    
    /// Create iterator for entities with two specific component types
    pub fn iter_entities_2<A1: AccessMode, A2: AccessMode>(&self) -> EntIt2<A1, A2> {
        let type_ids = vec![A1::component_type_id(), A2::component_type_id()];
        let entities = self.entities_with_components(&type_ids);
        EntIt2 {
            world: self as *const World,
            entities,
            index: 0,
            _phantom: PhantomData,
        }
    }
    
    /// Create iterator for entities with three specific component types
    pub fn iter_entities_3<A1: AccessMode, A2: AccessMode, A3: AccessMode>(&self) -> EntIt3<A1, A2, A3> {
        let type_ids = vec![A1::component_type_id(), A2::component_type_id(), A3::component_type_id()];
        let entities = self.entities_with_components(&type_ids);
        EntIt3 {
            world: self as *const World,
            entities,
            index: 0,
            _phantom: PhantomData,
        }
    }
    
    /// Run all systems
    pub fn update(&mut self) {
        // For now, just run systems in order they were added
        // Later we'll implement dependency-based ordering
        for system in &mut self.systems {
            system.update(());
        }
    }
}

// Basic implementations for the iterators would go here
// These are simplified placeholder implementations

impl<A1: AccessMode> Iterator for EntIt1<A1> {
    type Item = (); // Placeholder - would return actual component references
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entities.len() {
            self.index += 1;
            Some(())
        } else {
            None
        }
    }
}

impl<A1: AccessMode, A2: AccessMode> Iterator for EntIt2<A1, A2> {
    type Item = (); // Placeholder - would return actual component references
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entities.len() {
            self.index += 1;
            Some(())
        } else {
            None
        }
    }
}

impl<A1: AccessMode, A2: AccessMode, A3: AccessMode> Iterator for EntIt3<A1, A2, A3> {
    type Item = (); // Placeholder - would return actual component references
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entities.len() {
            self.index += 1;
            Some(())
        } else {
            None
        }
    }
}