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

/// EntityIterator for entities with two component types (immutable first, mutable second)
/// This implements the API: EntityIterator<ComponentType1, mut ComponentType2>
pub struct EntityIterator<T1, T2> {
    world: *const World,
    entities: Vec<Entity>,
    index: usize,
    _phantom: PhantomData<(T1, T2)>,
}

impl<T1, T2> EntityIterator<T1, T2>
where
    T1: Component + 'static,
    T2: Component + 'static,
{
    /// Create a new iterator for entities with components T1 (immutable) and T2 (mutable)
    pub fn new(world: &World) -> Self {
        let type_ids = vec![TypeId::of::<T1>(), TypeId::of::<T2>()];
        let entities = world.entities_with_components(&type_ids);
        Self {
            world: world as *const World,
            entities,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<T1, T2> Iterator for EntityIterator<T1, T2>
where
    T1: Component + 'static,
    T2: Component + 'static,
{
    type Item = (ComponentRef<T1>, ComponentRefMut<T2>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.entities.len() {
            let entity = self.entities[self.index];
            self.index += 1;
            
            // Safety: We ensure the world pointer is valid during iterator lifetime
            let world = unsafe { &*self.world };
            
            let type_id1 = TypeId::of::<T1>();
            let type_id2 = TypeId::of::<T2>();
            
            if let (Some(pool1), Some(pool2)) = (
                world.component_pools.get(&type_id1),
                world.component_pools.get(&type_id2),
            ) {
                if let (Some(comp1), Some(comp2)) = (
                    pool1.get(entity),
                    pool2.get_mut(entity),
                ) {
                    let comp1_ref = ComponentRef::new(comp1);
                    let comp2_ref = ComponentRefMut::new(comp2);
                    return Some((comp1_ref, comp2_ref));
                }
            }
        }
        None
    }
}

/// Wrapper for immutable component reference
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

impl<T> std::ops::Deref for ComponentRef<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.component_ref }
    }
}

/// Wrapper for mutable component reference
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
    
    /// Create an EntityIterator with the API: EntityIterator<ComponentType1, mut ComponentType2>
    /// T1 is accessed immutably, T2 is accessed mutably
    pub fn iter_entities<T1: Component + 'static, T2: Component + 'static>(&self) -> EntityIterator<T1, T2> {
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
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}