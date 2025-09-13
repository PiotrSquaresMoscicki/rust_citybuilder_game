use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::cell::{RefCell, Ref, RefMut};
use crate::diffing::{DebugTracker, diff_components};

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

/// Storage for a specific component type using RefCell for interior mutability
struct ComponentPool {
    components: HashMap<Entity, RefCell<Box<dyn Component>>>,
}

impl ComponentPool {
    fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
    
    fn insert(&mut self, entity: Entity, component: Box<dyn Component>) {
        self.components.insert(entity, RefCell::new(component));
    }
    
    fn get(&self, entity: Entity) -> Option<Ref<Box<dyn Component>>> {
        self.components.get(&entity).map(|c| c.borrow())
    }
    
    fn get_mut(&self, entity: Entity) -> Option<RefMut<Box<dyn Component>>> {
        self.components.get(&entity).map(|c| c.borrow_mut())
    }
    
    fn remove(&mut self, entity: Entity) -> Option<RefCell<Box<dyn Component>>> {
        self.components.remove(&entity)
    }
    
    fn contains(&self, entity: Entity) -> bool {
        self.components.contains_key(&entity)
    }
    
    fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.components.keys().copied()
    }
}

/// EntityIterator for entities with two component types
/// This implements the API: EntityIterator<ComponentType1, Mut<ComponentType2>>
/// where ComponentType1 is accessed immutably and Mut<ComponentType2> is accessed mutably
pub struct EntityIterator<A1, A2> {
    world: *const World,
    entities: Vec<Entity>,
    index: usize,
    _phantom: PhantomData<(A1, A2)>,
}

impl<A1, A2> EntityIterator<A1, A2>
where
    A1: AccessMode,
    A2: AccessMode,
{
    /// Create a new iterator for entities with components A1 and A2
    pub fn new(world: &World) -> Self {
        let type_ids = vec![A1::component_type_id(), A2::component_type_id()];
        let entities = world.entities_with_components(&type_ids);
        Self {
            world: world as *const World,
            entities,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

/// Trait to map access modes to their corresponding reference types
pub trait AccessModeToRef<T: Component + 'static> {
    type Ref: std::ops::Deref<Target = T>;
    
    fn get_from_pool(pool: &ComponentPool, entity: Entity) -> Option<Self::Ref>;
}

/// Immutable access maps to ComponentRef
impl<T: Component + 'static> AccessModeToRef<T> for T {
    type Ref = ComponentRef<T>;
    
    fn get_from_pool(pool: &ComponentPool, entity: Entity) -> Option<Self::Ref> {
        ComponentRef::from_pool(pool, entity)
    }
}

/// Mutable access maps to ComponentRefMut
impl<T: Component + 'static> AccessModeToRef<T> for Mut<T> {
    type Ref = ComponentRefMut<T>;
    
    fn get_from_pool(pool: &ComponentPool, entity: Entity) -> Option<Self::Ref> {
        ComponentRefMut::from_pool(pool, entity)
    }
}

impl<A1, A2> Iterator for EntityIterator<A1, A2>
where
    A1: AccessMode + AccessModeToRef<A1::Component>,
    A2: AccessMode + AccessModeToRef<A2::Component>,
{
    type Item = (A1::Ref, A2::Ref);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.entities.len() {
            let entity = self.entities[self.index];
            self.index += 1;
            
            // Safety: We ensure the world pointer is valid during iterator lifetime
            let world = unsafe { &*self.world };
            
            let type_id1 = A1::component_type_id();
            let type_id2 = A2::component_type_id();
            
            if let (Some(pool1), Some(pool2)) = (
                world.component_pools.get(&type_id1),
                world.component_pools.get(&type_id2),
            ) {
                if let (Some(comp1), Some(comp2)) = (
                    A1::get_from_pool(pool1, entity),
                    A2::get_from_pool(pool2, entity),
                ) {
                    return Some((comp1, comp2));
                }
            }
        }
        None
    }
}

/// Trait for component references that can be either mutable or immutable
pub trait ComponentAccess<T: Component + 'static> {
    fn from_pool(pool: &ComponentPool, entity: Entity) -> Option<Self>
    where
        Self: Sized;
}

/// Immutable component reference
pub struct ComponentRef<T> {
    _component: Ref<'static, Box<dyn Component>>,
    component_ref: *const T,
}

impl<T: Component + 'static> ComponentRef<T> {
    fn new(component: Ref<Box<dyn Component>>) -> Self {
        let component_ptr = component.as_any().downcast_ref::<T>().unwrap() as *const T;
        
        // Safety: We extend the lifetime to 'static for the wrapper
        // The actual lifetime is managed by the Ref<> which we keep
        let component_static: Ref<'static, Box<dyn Component>> = unsafe { std::mem::transmute(component) };
        
        Self {
            _component: component_static,
            component_ref: component_ptr,
        }
    }
}

impl<T: Component + 'static> ComponentAccess<T> for ComponentRef<T> {
    fn from_pool(pool: &ComponentPool, entity: Entity) -> Option<Self> {
        pool.get(entity).map(ComponentRef::new)
    }
}

impl<T> std::ops::Deref for ComponentRef<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.component_ref }
    }
}

/// Mutable component reference
pub struct ComponentRefMut<T> {
    _component: RefMut<'static, Box<dyn Component>>,
    component_ref: *mut T,
}

impl<T: Component + 'static> ComponentRefMut<T> {
    fn new(mut component: RefMut<Box<dyn Component>>) -> Self {
        let component_ptr = component.as_any_mut().downcast_mut::<T>().unwrap() as *mut T;
        
        // Safety: We extend the lifetime to 'static for the wrapper
        // The actual lifetime is managed by the RefMut<> which we keep
        let component_static: RefMut<'static, Box<dyn Component>> = unsafe { std::mem::transmute(component) };
        
        Self {
            _component: component_static,
            component_ref: component_ptr,
        }
    }
}

impl<T: Component + 'static> ComponentAccess<T> for ComponentRefMut<T> {
    fn from_pool(pool: &ComponentPool, entity: Entity) -> Option<Self> {
        pool.get_mut(entity).map(ComponentRefMut::new)
    }
}

impl<T> std::ops::Deref for ComponentRefMut<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.component_ref }
    }
}

impl<T> std::ops::DerefMut for ComponentRefMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.component_ref }
    }
}

/// World manages entities, component pools, and systems
pub struct World {
    next_entity_id: Entity,
    entities: Vec<Entity>,
    component_pools: HashMap<TypeId, ComponentPool>,
    systems: Vec<Box<dyn Fn(&World)>>,
    pub debug_tracker: DebugTracker,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            entities: Vec::new(),
            component_pools: HashMap::new(),
            systems: Vec::new(),
            debug_tracker: DebugTracker::new(),
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
        self.component_pools
            .get(&type_id)
            .map_or(false, |pool| pool.contains(entity))
    }
    
    /// Remove an entity and all its components
    pub fn remove_entity(&mut self, entity: Entity) {
        // Remove from entities list
        self.entities.retain(|&e| e != entity);
        
        // Remove from all component pools
        for pool in self.component_pools.values_mut() {
            pool.remove(entity);
        }
    }
    
    /// Get all entities that have a specific component
    pub fn entities_with_component<T: Component + 'static>(&self) -> Vec<Entity> {
        let type_id = TypeId::of::<T>();
        self.component_pools
            .get(&type_id)
            .map_or(Vec::new(), |pool| pool.entities().collect())
    }
    
    /// Get all entities that have all specified component types
    fn entities_with_components(&self, type_ids: &[TypeId]) -> Vec<Entity> {
        if type_ids.is_empty() {
            return self.entities.clone();
        }
        
        // Start with entities from the first component type
        let mut entities = if let Some(pool) = self.component_pools.get(&type_ids[0]) {
            pool.entities().collect::<Vec<_>>()
        } else {
            return Vec::new();
        };
        
        // Filter by remaining component types
        for &type_id in &type_ids[1..] {
            if let Some(pool) = self.component_pools.get(&type_id) {
                entities.retain(|&entity| pool.contains(entity));
            } else {
                return Vec::new();
            }
        }
        
        entities
    }
    
    /// Create an EntityIterator with the API: EntityIterator<ComponentType1, Mut<ComponentType2>>
    /// Plain types (T) are accessed immutably, Mut<T> types are accessed mutably
    pub fn iter_entities<A1, A2>(&self) -> EntityIterator<A1, A2>
    where
        A1: AccessMode + AccessModeToRef<A1::Component>,
        A2: AccessMode + AccessModeToRef<A2::Component>,
    {
        EntityIterator::new(self)
    }
    
    /// Add a system to the world
    pub fn add_system<F>(&mut self, system: F)
    where
        F: Fn(&World) + 'static,
    {
        self.systems.push(Box::new(system));
    }
    
    /// Run all systems
    pub fn run_systems(&self) {
        for system in &self.systems {
            system(self);
        }
    }
    
    /// Enable debug tracking for component state changes
    pub fn enable_debug_tracking(&mut self) {
        self.debug_tracker.enable();
    }
    
    /// Disable debug tracking
    pub fn disable_debug_tracking(&mut self) {
        self.debug_tracker.disable();
    }
    
    /// Advance to the next frame for debug tracking
    pub fn next_frame(&mut self) {
        self.debug_tracker.next_frame();
    }
    
    /// Get debug diff history in human-readable format
    pub fn get_debug_history(&self) -> String {
        self.debug_tracker.get_diff_history_formatted()
    }
    
    /// Clear debug history
    pub fn clear_debug_history(&mut self) {
        self.debug_tracker.clear_history();
    }
    
    /// Get a snapshot of a component for diffing (internal method)
    pub(crate) fn get_component_snapshot(&self, entity: Entity, type_id: TypeId) -> Option<Box<dyn Component>> {
        let pool = self.component_pools.get(&type_id)?;
        let component = pool.get(entity)?;
        Some(component.clone_box())
    }
    
    /// Run a single system with debug tracking
    pub fn run_system_with_debug<F>(&mut self, system_name: &str, system: F, mutable_types: &[TypeId]) 
    where 
        F: Fn(&World)
    {
        if self.debug_tracker.enabled {
            // Get all entities that have any of the mutable component types
            let mut tracked_entities = std::collections::HashSet::new();
            for &type_id in mutable_types {
                for entity in self.component_pools.get(&type_id).map_or(vec![], |pool| pool.entities().collect()) {
                    tracked_entities.insert(entity);
                }
            }
            let entities: Vec<Entity> = tracked_entities.into_iter().collect();
            
            // Take snapshot before system execution
            let mut snapshots = HashMap::new();
            for &entity in &entities {
                for &type_id in mutable_types {
                    if let Some(component) = self.get_component_snapshot(entity, type_id) {
                        snapshots.insert((entity, type_id), component);
                    }
                }
            }
            
            // Run the system
            system(self);
            
            // Record diffs after system execution
            let mut component_diffs = Vec::new();
            for &entity in &entities {
                for &type_id in mutable_types {
                    if let Some(old_component) = snapshots.get(&(entity, type_id)) {
                        if let Some(new_component) = self.get_component_snapshot(entity, type_id) {
                            // Use diffable trait to get actual differences
                            if let Some(mut diff) = diff_components(old_component.as_ref(), new_component.as_ref(), type_id) {
                                diff.entity_id = entity;
                                component_diffs.push(diff);
                            }
                        }
                    }
                }
            }
            
            if !component_diffs.is_empty() {
                let record = crate::diffing::SystemDiffRecord {
                    frame_number: self.debug_tracker.frame_number,
                    system_name: system_name.to_string(),
                    component_diffs,
                };
                self.debug_tracker.diff_history.push(record);
            }
        } else {
            // Just run the system without tracking
            system(self);
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}