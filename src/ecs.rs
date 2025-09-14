use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};
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

/// Unique identifier for system types
pub type SystemTypeId = &'static str;

/// Error type for dependency-related issues
#[derive(Debug, PartialEq)]
pub enum DependencyError {
    CircularDependency(Vec<SystemTypeId>),
    UnknownSystemDependency(SystemTypeId),
}

/// Trait for systems that are objects with an update method
pub trait System {
    /// Get the unique type identifier for this system
    fn system_type_id(&self) -> SystemTypeId;
    
    /// Get the dependencies of this system
    /// These systems will be executed before this system
    fn get_dependencies(&self) -> Vec<SystemTypeId>;
    
    /// Update method that takes entity iterators as needed
    /// The implementation should call world.iter_entities() to get iterators
    fn update(&self, world: &World);
    
    /// Get a description of this system for debugging purposes
    fn get_system_name(&self) -> &str;
    
    /// Get the component types that this system accesses mutably (for debugging)
    fn get_mutable_component_types(&self) -> Vec<TypeId>;
}

/// Trait for systems that can be called with different iterator combinations
pub trait SystemCall {
    /// Execute the system with iterators created from the world
    fn call(&self, world: &World);
    
    /// Get the component types that this system accesses mutably (for debugging)
    fn get_mutable_component_types(&self) -> Vec<TypeId>;
    
    /// Get a description of this system for debugging purposes
    fn get_system_name(&self) -> &str;
    
    /// Get the system type ID for dependency tracking
    fn get_system_type_id(&self) -> SystemTypeId;
    
    /// Get the dependencies of this system
    fn get_dependencies(&self) -> Vec<SystemTypeId>;
}

/// Implementation for systems that take a single iterator
pub struct SingleIteratorSystem<A1, A2, F> {
    function: F,
    system_name: String,
    system_type_id: SystemTypeId,
    dependencies: Vec<SystemTypeId>,
    _phantom: PhantomData<(A1, A2)>,
}

impl<A1, A2, F> SingleIteratorSystem<A1, A2, F>
where
    A1: AccessMode + AccessModeToRef<A1::Component>,
    A2: AccessMode + AccessModeToRef<A2::Component>,
    F: Fn(EntityIterator<A1, A2>),
{
    pub fn new(function: F, system_name: String) -> Self {
        Self {
            function,
            system_type_id: "",
            system_name,
            dependencies: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_dependencies(function: F, system_name: String, system_type_id: SystemTypeId, dependencies: Vec<SystemTypeId>) -> Self {
        Self {
            function,
            system_name,
            system_type_id,
            dependencies,
            _phantom: PhantomData,
        }
    }
}

impl<A1, A2, F> SystemCall for SingleIteratorSystem<A1, A2, F>
where
    A1: AccessMode + AccessModeToRef<A1::Component>,
    A2: AccessMode + AccessModeToRef<A2::Component>,
    F: Fn(EntityIterator<A1, A2>),
{
    fn call(&self, world: &World) {
        let iterator = world.iter_entities::<A1, A2>();
        (self.function)(iterator);
    }
    
    fn get_mutable_component_types(&self) -> Vec<TypeId> {
        let mut types = Vec::new();
        if A1::is_mutable() {
            types.push(A1::component_type_id());
        }
        if A2::is_mutable() {
            types.push(A2::component_type_id());
        }
        types
    }
    
    fn get_system_name(&self) -> &str {
        &self.system_name
    }
    
    fn get_system_type_id(&self) -> SystemTypeId {
        self.system_type_id
    }
    
    fn get_dependencies(&self) -> Vec<SystemTypeId> {
        self.dependencies.clone()
    }
}

/// Implementation for systems that take two iterators
pub struct DualIteratorSystem<A1, A2, B1, B2, F> {
    function: F,
    system_name: String,
    system_type_id: SystemTypeId,
    dependencies: Vec<SystemTypeId>,
    _phantom: PhantomData<(A1, A2, B1, B2)>,
}

impl<A1, A2, B1, B2, F> DualIteratorSystem<A1, A2, B1, B2, F>
where
    A1: AccessMode + AccessModeToRef<A1::Component>,
    A2: AccessMode + AccessModeToRef<A2::Component>,
    B1: AccessMode + AccessModeToRef<B1::Component>,
    B2: AccessMode + AccessModeToRef<B2::Component>,
    F: Fn(EntityIterator<A1, A2>, EntityIterator<B1, B2>),
{
    pub fn new(function: F, system_name: String) -> Self {
        Self {
            function,
            system_name,
            system_type_id: "",
            dependencies: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_dependencies(function: F, system_name: String, system_type_id: SystemTypeId, dependencies: Vec<SystemTypeId>) -> Self {
        Self {
            function,
            system_name,
            system_type_id,
            dependencies,
            _phantom: PhantomData,
        }
    }
}

impl<A1, A2, B1, B2, F> SystemCall for DualIteratorSystem<A1, A2, B1, B2, F>
where
    A1: AccessMode + AccessModeToRef<A1::Component>,
    A2: AccessMode + AccessModeToRef<A2::Component>,
    B1: AccessMode + AccessModeToRef<B1::Component>,
    B2: AccessMode + AccessModeToRef<B2::Component>,
    F: Fn(EntityIterator<A1, A2>, EntityIterator<B1, B2>),
{
    fn call(&self, world: &World) {
        let iterator1 = world.iter_entities::<A1, A2>();
        let iterator2 = world.iter_entities::<B1, B2>();
        (self.function)(iterator1, iterator2);
    }
    
    fn get_mutable_component_types(&self) -> Vec<TypeId> {
        let mut types = Vec::new();
        if A1::is_mutable() {
            types.push(A1::component_type_id());
        }
        if A2::is_mutable() {
            types.push(A2::component_type_id());
        }
        if B1::is_mutable() {
            types.push(B1::component_type_id());
        }
        if B2::is_mutable() {
            types.push(B2::component_type_id());
        }
        types
    }
    
    fn get_system_name(&self) -> &str {
        &self.system_name
    }
    
    fn get_system_type_id(&self) -> SystemTypeId {
        self.system_type_id
    }
    
    fn get_dependencies(&self) -> Vec<SystemTypeId> {
        self.dependencies.clone()
    }
}

/// Implementation for systems that take three iterators
pub struct TripleIteratorSystem<A1, A2, B1, B2, C1, C2, F> {
    function: F,
    system_name: String,
    system_type_id: SystemTypeId,
    dependencies: Vec<SystemTypeId>,
    _phantom: PhantomData<(A1, A2, B1, B2, C1, C2)>,
}

impl<A1, A2, B1, B2, C1, C2, F> TripleIteratorSystem<A1, A2, B1, B2, C1, C2, F>
where
    A1: AccessMode + AccessModeToRef<A1::Component>,
    A2: AccessMode + AccessModeToRef<A2::Component>,
    B1: AccessMode + AccessModeToRef<B1::Component>,
    B2: AccessMode + AccessModeToRef<B2::Component>,
    C1: AccessMode + AccessModeToRef<C1::Component>,
    C2: AccessMode + AccessModeToRef<C2::Component>,
    F: Fn(EntityIterator<A1, A2>, EntityIterator<B1, B2>, EntityIterator<C1, C2>),
{
    pub fn new(function: F, system_name: String) -> Self {
        Self {
            function,
            system_name,
            system_type_id: "",
            dependencies: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_dependencies(function: F, system_name: String, system_type_id: SystemTypeId, dependencies: Vec<SystemTypeId>) -> Self {
        Self {
            function,
            system_name,
            system_type_id,
            dependencies,
            _phantom: PhantomData,
        }
    }
}

impl<A1, A2, B1, B2, C1, C2, F> SystemCall for TripleIteratorSystem<A1, A2, B1, B2, C1, C2, F>
where
    A1: AccessMode + AccessModeToRef<A1::Component>,
    A2: AccessMode + AccessModeToRef<A2::Component>,
    B1: AccessMode + AccessModeToRef<B1::Component>,
    B2: AccessMode + AccessModeToRef<B2::Component>,
    C1: AccessMode + AccessModeToRef<C1::Component>,
    C2: AccessMode + AccessModeToRef<C2::Component>,
    F: Fn(EntityIterator<A1, A2>, EntityIterator<B1, B2>, EntityIterator<C1, C2>),
{
    fn call(&self, world: &World) {
        let iterator1 = world.iter_entities::<A1, A2>();
        let iterator2 = world.iter_entities::<B1, B2>();
        let iterator3 = world.iter_entities::<C1, C2>();
        (self.function)(iterator1, iterator2, iterator3);
    }
    
    fn get_mutable_component_types(&self) -> Vec<TypeId> {
        let mut types = Vec::new();
        if A1::is_mutable() {
            types.push(A1::component_type_id());
        }
        if A2::is_mutable() {
            types.push(A2::component_type_id());
        }
        if B1::is_mutable() {
            types.push(B1::component_type_id());
        }
        if B2::is_mutable() {
            types.push(B2::component_type_id());
        }
        if C1::is_mutable() {
            types.push(C1::component_type_id());
        }
        if C2::is_mutable() {
            types.push(C2::component_type_id());
        }
        types
    }
    
    fn get_system_name(&self) -> &str {
        &self.system_name
    }
    
    fn get_system_type_id(&self) -> SystemTypeId {
        self.system_type_id
    }
    
    fn get_dependencies(&self) -> Vec<SystemTypeId> {
        self.dependencies.clone()
    }
}

/// World manages entities, component pools, and systems
pub struct World {
    next_entity_id: Entity,
    entities: Vec<Entity>,
    component_pools: HashMap<TypeId, ComponentPool>,
    systems: Vec<Box<dyn System>>,
    /// Ordered systems after dependency resolution
    ordered_systems: Vec<usize>, // Indices into systems vec
    /// Map from system type ID to index in systems vec
    system_type_map: HashMap<SystemTypeId, usize>,
    legacy_systems: Vec<Box<dyn Fn(&World)>>, // Keep legacy systems for backward compatibility
    legacy_system_calls: Vec<Box<dyn SystemCall>>, // Keep old SystemCall systems for backward compatibility
    /// Map from system type ID to index in legacy_system_calls vec
    legacy_system_type_map: HashMap<SystemTypeId, usize>,
    /// Ordered legacy systems after dependency resolution
    ordered_legacy_systems: Vec<usize>, // Indices into legacy_system_calls vec
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
            ordered_systems: Vec::new(),
            system_type_map: HashMap::new(),
            legacy_systems: Vec::new(),
            legacy_system_calls: Vec::new(),
            legacy_system_type_map: HashMap::new(),
            ordered_legacy_systems: Vec::new(),
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
    
    /// Get the total number of entities in the world
    pub fn entity_count(&self) -> usize {
        self.entities.len()
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
    
    /// Add a system object to the world
    pub fn add_system_object<S: System + 'static>(&mut self, system: S) -> Result<(), DependencyError> {
        let system_type_id = system.system_type_id();
        let index = self.systems.len();
        
        self.systems.push(Box::new(system));
        self.system_type_map.insert(system_type_id, index);
        
        // Try to resolve dependencies
        self.try_resolve_dependencies()
    }
    
    /// Add a legacy system to the world (for backward compatibility)
    pub fn add_system<F>(&mut self, system: F)
    where
        F: Fn(&World) + 'static,
    {
        self.legacy_systems.push(Box::new(system));
    }
    
    /// Add a system that takes a single entity iterator
    pub fn add_single_iterator_system<A1, A2, F>(&mut self, system: F, system_name: &str)
    where
        A1: AccessMode + AccessModeToRef<A1::Component> + 'static,
        A2: AccessMode + AccessModeToRef<A2::Component> + 'static,
        F: Fn(EntityIterator<A1, A2>) + 'static,
    {
        let system_impl = SingleIteratorSystem::new(system, system_name.to_string());
        self.legacy_system_calls.push(Box::new(system_impl));
        self.invalidate_system_order();
    }

    /// Add a system that takes a single entity iterator with dependencies
    pub fn add_single_iterator_system_with_dependencies<A1, A2, F>(
        &mut self, 
        system: F, 
        system_name: &str,
        system_type_id: SystemTypeId,
        dependencies: Vec<SystemTypeId>
    ) -> Result<(), DependencyError>
    where
        A1: AccessMode + AccessModeToRef<A1::Component> + 'static,
        A2: AccessMode + AccessModeToRef<A2::Component> + 'static,
        F: Fn(EntityIterator<A1, A2>) + 'static,
    {
        let system_impl = SingleIteratorSystem::with_dependencies(system, system_name.to_string(), system_type_id, dependencies);
        let index = self.legacy_system_calls.len();
        self.legacy_system_calls.push(Box::new(system_impl));
        self.legacy_system_type_map.insert(system_type_id, index);
        
        // Try to resolve legacy system dependencies
        self.try_resolve_legacy_dependencies()
    }
    
    /// Add a system that takes two entity iterators
    pub fn add_dual_iterator_system<A1, A2, B1, B2, F>(&mut self, system: F, system_name: &str)
    where
        A1: AccessMode + AccessModeToRef<A1::Component> + 'static,
        A2: AccessMode + AccessModeToRef<A2::Component> + 'static,
        B1: AccessMode + AccessModeToRef<B1::Component> + 'static,
        B2: AccessMode + AccessModeToRef<B2::Component> + 'static,
        F: Fn(EntityIterator<A1, A2>, EntityIterator<B1, B2>) + 'static,
    {
        let system_impl = DualIteratorSystem::new(system, system_name.to_string());
        self.legacy_system_calls.push(Box::new(system_impl));
        self.invalidate_system_order();
    }

    /// Add a system that takes two entity iterators with dependencies
    pub fn add_dual_iterator_system_with_dependencies<A1, A2, B1, B2, F>(
        &mut self, 
        system: F, 
        system_name: &str,
        system_type_id: SystemTypeId,
        dependencies: Vec<SystemTypeId>
    ) -> Result<(), DependencyError>
    where
        A1: AccessMode + AccessModeToRef<A1::Component> + 'static,
        A2: AccessMode + AccessModeToRef<A2::Component> + 'static,
        B1: AccessMode + AccessModeToRef<B1::Component> + 'static,
        B2: AccessMode + AccessModeToRef<B2::Component> + 'static,
        F: Fn(EntityIterator<A1, A2>, EntityIterator<B1, B2>) + 'static,
    {
        let system_impl = DualIteratorSystem::with_dependencies(system, system_name.to_string(), system_type_id, dependencies);
        let index = self.legacy_system_calls.len();
        self.legacy_system_calls.push(Box::new(system_impl));
        self.legacy_system_type_map.insert(system_type_id, index);
        self.try_resolve_legacy_dependencies()
    }
    
    /// Add a system that takes three entity iterators
    pub fn add_triple_iterator_system<A1, A2, B1, B2, C1, C2, F>(&mut self, system: F, system_name: &str)
    where
        A1: AccessMode + AccessModeToRef<A1::Component> + 'static,
        A2: AccessMode + AccessModeToRef<A2::Component> + 'static,
        B1: AccessMode + AccessModeToRef<B1::Component> + 'static,
        B2: AccessMode + AccessModeToRef<B2::Component> + 'static,
        C1: AccessMode + AccessModeToRef<C1::Component> + 'static,
        C2: AccessMode + AccessModeToRef<C2::Component> + 'static,
        F: Fn(EntityIterator<A1, A2>, EntityIterator<B1, B2>, EntityIterator<C1, C2>) + 'static,
    {
        let system_impl = TripleIteratorSystem::new(system, system_name.to_string());
        self.legacy_system_calls.push(Box::new(system_impl));
        self.invalidate_system_order();
    }

    /// Add a system that takes three entity iterators with dependencies
    pub fn add_triple_iterator_system_with_dependencies<A1, A2, B1, B2, C1, C2, F>(
        &mut self, 
        system: F, 
        system_name: &str,
        system_type_id: SystemTypeId,
        dependencies: Vec<SystemTypeId>
    ) -> Result<(), DependencyError>
    where
        A1: AccessMode + AccessModeToRef<A1::Component> + 'static,
        A2: AccessMode + AccessModeToRef<A2::Component> + 'static,
        B1: AccessMode + AccessModeToRef<B1::Component> + 'static,
        B2: AccessMode + AccessModeToRef<B2::Component> + 'static,
        C1: AccessMode + AccessModeToRef<C1::Component> + 'static,
        C2: AccessMode + AccessModeToRef<C2::Component> + 'static,
        F: Fn(EntityIterator<A1, A2>, EntityIterator<B1, B2>, EntityIterator<C1, C2>) + 'static,
    {
        let system_impl = TripleIteratorSystem::with_dependencies(system, system_name.to_string(), system_type_id, dependencies);
        let index = self.legacy_system_calls.len();
        self.legacy_system_calls.push(Box::new(system_impl));
        self.legacy_system_type_map.insert(system_type_id, index);
        self.try_resolve_legacy_dependencies()
    }
    
    /// Run all systems (new System objects, legacy SystemCall, and legacy functions) in dependency order
    pub fn run_systems(&mut self) {
        // Get execution order for new systems
        let ordered_systems = self.ordered_systems.clone();
        
        // Run new System objects in dependency order
        for &system_index in &ordered_systems {
            if system_index < self.systems.len() {
                // For now, just call update without debug tracking
                // We'll need to make systems have an update method that takes &World
                // and make the systems manage their own state mutations if needed
                self.systems[system_index].update(self);
            }
        }
        
        // Run legacy SystemCall systems in dependency order
        let ordered_legacy_systems = self.ordered_legacy_systems.clone();
        for &system_index in &ordered_legacy_systems {
            if system_index < self.legacy_system_calls.len() {
                self.legacy_system_calls[system_index].call(self);
            }
        }
        
        // Run legacy function systems for backward compatibility
        for system in &self.legacy_systems {
            system(self);
        }
    }
    
    /// Force resolution of dependencies before running systems
    pub fn finalize_systems(&mut self) -> Result<(), DependencyError> {
        // Resolve dependencies for both new and legacy systems
        self.resolve_dependencies()?;
        self.resolve_legacy_dependencies()?;
        Ok(())
    }
    
    /// Run all new System objects with debug tracking
    pub fn run_systems_with_debug(&mut self) {
        // Get execution order for new systems
        let ordered_systems = self.ordered_systems.clone();
        
        for &system_index in &ordered_systems {
            if system_index < self.systems.len() {
                let mutable_types = self.systems[system_index].get_mutable_component_types();
                let system_name = self.systems[system_index].get_system_name();
                
                if self.debug_tracker.enabled {
                    // Get all entities that have any of the mutable component types
                    let mut tracked_entities = std::collections::HashSet::new();
                    for &type_id in &mutable_types {
                        for entity in self.component_pools.get(&type_id).map_or(vec![], |pool| pool.entities().collect()) {
                            tracked_entities.insert(entity);
                        }
                    }
                    let entities: Vec<Entity> = tracked_entities.into_iter().collect();
                    
                    // Take snapshot before system execution
                    let mut snapshots = HashMap::new();
                    for &entity in &entities {
                        for &type_id in &mutable_types {
                            if let Some(component) = self.get_component_snapshot(entity, type_id) {
                                snapshots.insert((entity, type_id), component);
                            }
                        }
                    }
                    
                    // Run the system
                    self.systems[system_index].update(self);
                    
                    // Record diffs after system execution
                    let mut component_diffs = Vec::new();
                    for &entity in &entities {
                        for &type_id in &mutable_types {
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
                    self.systems[system_index].update(self);
                }
            }
        }
        
        // Also run legacy systems with debug tracking
        for system in &self.legacy_system_calls {
            let mutable_types = system.get_mutable_component_types();
            let system_name = system.get_system_name();
            
            if self.debug_tracker.enabled {
                // Same debug tracking logic as above
                let mut tracked_entities = std::collections::HashSet::new();
                for &type_id in &mutable_types {
                    for entity in self.component_pools.get(&type_id).map_or(vec![], |pool| pool.entities().collect()) {
                        tracked_entities.insert(entity);
                    }
                }
                let entities: Vec<Entity> = tracked_entities.into_iter().collect();
                
                let mut snapshots = HashMap::new();
                for &entity in &entities {
                    for &type_id in &mutable_types {
                        if let Some(component) = self.get_component_snapshot(entity, type_id) {
                            snapshots.insert((entity, type_id), component);
                        }
                    }
                }
                
                system.call(self);
                
                let mut component_diffs = Vec::new();
                for &entity in &entities {
                    for &type_id in &mutable_types {
                        if let Some(old_component) = snapshots.get(&(entity, type_id)) {
                            if let Some(new_component) = self.get_component_snapshot(entity, type_id) {
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
                system.call(self);
            }
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
    
    /// Get the number of registered System objects (for testing)
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }
    
    /// Get the number of registered legacy SystemCall systems (for testing)
    pub fn legacy_system_call_count(&self) -> usize {
        self.legacy_system_calls.len()
    }
    
    /// Get the number of registered legacy function systems (for testing)
    pub fn legacy_system_count(&self) -> usize {
        self.legacy_systems.len()
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

    /// Try to resolve dependencies for legacy systems only
    fn try_resolve_legacy_dependencies(&mut self) -> Result<(), DependencyError> {
        // Check if all dependencies are registered for legacy SystemCall systems
        let mut missing_deps = Vec::new();
        for system in &self.legacy_system_calls {
            for dep in system.get_dependencies() {
                if !dep.is_empty() && !self.legacy_system_type_map.contains_key(dep) {
                    missing_deps.push(dep);
                }
            }
        }
        
        // If there are missing dependencies, just update ordered_legacy_systems with current systems
        if !missing_deps.is_empty() {
            // Add new legacy systems to the end for now
            while self.ordered_legacy_systems.len() < self.legacy_system_calls.len() {
                self.ordered_legacy_systems.push(self.ordered_legacy_systems.len());
            }
            return Ok(()); // Don't fail on missing deps, they might be added later
        }
        
        // All dependencies are available, do full resolution for legacy systems
        self.resolve_legacy_dependencies()
    }

    /// Try to resolve dependencies, allowing for forward references
    fn try_resolve_dependencies(&mut self) -> Result<(), DependencyError> {
        // Check if all dependencies are registered for new System objects
        let mut missing_deps = Vec::new();
        for system in &self.systems {
            for dep in system.get_dependencies() {
                if !dep.is_empty() && !self.system_type_map.contains_key(dep) {
                    missing_deps.push(dep);
                }
            }
        }
        
        // Check dependencies for legacy SystemCall objects too
        for system in &self.legacy_system_calls {
            for dep in system.get_dependencies() {
                if !dep.is_empty() && !self.system_type_map.contains_key(dep) {
                    missing_deps.push(dep);
                }
            }
        }
        
        // If there are missing dependencies, just update ordered_systems with current systems
        if !missing_deps.is_empty() {
            // Add new systems to the end for now
            while self.ordered_systems.len() < self.systems.len() {
                self.ordered_systems.push(self.ordered_systems.len());
            }
            return Ok(()); // Don't fail on missing deps, they might be added later
        }
        
        // All dependencies are available, do full resolution
        self.resolve_dependencies()
    }

    /// Resolve system dependencies and create execution order
    /// Returns error if circular dependencies are detected
    fn resolve_dependencies(&mut self) -> Result<(), DependencyError> {
        // Build dependency graph for new System objects only for now
        // Legacy systems will be handled separately
        let mut dependency_graph: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut dependents: HashMap<usize, HashSet<usize>> = HashMap::new();
        
        // Initialize graph for new System objects
        for i in 0..self.systems.len() {
            dependency_graph.insert(i, Vec::new());
            dependents.insert(i, HashSet::new());
        }
        
        // Populate graph with dependencies for new System objects
        for (i, system) in self.systems.iter().enumerate() {
            let deps = system.get_dependencies();
            for dep_type_id in deps {
                if let Some(&dep_index) = self.system_type_map.get(dep_type_id) {
                    if dep_index < 1000 { // Only new systems (not legacy with offset)
                        // This system depends on dep_index
                        dependency_graph.get_mut(&i).unwrap().push(dep_index);
                        dependents.get_mut(&dep_index).unwrap().insert(i);
                    }
                } else if !dep_type_id.is_empty() {
                    // Unknown dependency
                    return Err(DependencyError::UnknownSystemDependency(dep_type_id));
                }
            }
        }
        
        // Topological sort using Kahn's algorithm
        let mut ordered = Vec::new();
        let mut in_degree = vec![0; self.systems.len()];
        
        // Calculate in-degrees
        for i in 0..self.systems.len() {
            in_degree[i] = dependency_graph[&i].len();
        }
        
        // Queue nodes with no dependencies
        let mut queue = VecDeque::new();
        for i in 0..self.systems.len() {
            if in_degree[i] == 0 {
                queue.push_back(i);
            }
        }
        
        // Process queue
        while let Some(current) = queue.pop_front() {
            ordered.push(current);
            
            // Reduce in-degree for all dependents
            for &dependent in &dependents[&current] {
                in_degree[dependent] -= 1;
                if in_degree[dependent] == 0 {
                    queue.push_back(dependent);
                }
            }
        }
        
        // Check for circular dependencies
        if ordered.len() != self.systems.len() {
            // Find systems involved in circular dependency
            let mut cycle_systems = Vec::new();
            for i in 0..self.systems.len() {
                if in_degree[i] > 0 {
                    cycle_systems.push(self.systems[i].system_type_id());
                }
            }
            return Err(DependencyError::CircularDependency(cycle_systems));
        }
        
        self.ordered_systems = ordered;
        Ok(())
    }
    
    /// Resolve legacy system dependencies and create execution order
    /// Returns error if circular dependencies are detected
    fn resolve_legacy_dependencies(&mut self) -> Result<(), DependencyError> {
        // Build dependency graph for legacy SystemCall systems
        let mut dependency_graph: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut dependents: HashMap<usize, HashSet<usize>> = HashMap::new();
        
        // Initialize graph for legacy SystemCall systems
        for i in 0..self.legacy_system_calls.len() {
            dependency_graph.insert(i, Vec::new());
            dependents.insert(i, HashSet::new());
        }
        
        // Populate graph with dependencies for legacy SystemCall systems
        for (i, system) in self.legacy_system_calls.iter().enumerate() {
            let deps = system.get_dependencies();
            for dep_type_id in deps {
                if let Some(&dep_index) = self.legacy_system_type_map.get(dep_type_id) {
                    // This system depends on dep_index
                    dependency_graph.get_mut(&i).unwrap().push(dep_index);
                    dependents.get_mut(&dep_index).unwrap().insert(i);
                } else if !dep_type_id.is_empty() {
                    // Unknown dependency
                    return Err(DependencyError::UnknownSystemDependency(dep_type_id));
                }
            }
        }
        
        // Topological sort using Kahn's algorithm
        let mut ordered = Vec::new();
        let mut in_degree = vec![0; self.legacy_system_calls.len()];
        
        // Calculate in-degrees
        for i in 0..self.legacy_system_calls.len() {
            in_degree[i] = dependency_graph[&i].len();
        }
        
        // Queue nodes with no dependencies
        let mut queue = VecDeque::new();
        for i in 0..self.legacy_system_calls.len() {
            if in_degree[i] == 0 {
                queue.push_back(i);
            }
        }
        
        // Process queue
        while let Some(current) = queue.pop_front() {
            ordered.push(current);
            
            // Reduce in-degree for all dependents
            for &dependent in &dependents[&current] {
                in_degree[dependent] -= 1;
                if in_degree[dependent] == 0 {
                    queue.push_back(dependent);
                }
            }
        }
        
        // Check for circular dependencies
        if ordered.len() != self.legacy_system_calls.len() {
            // Find systems involved in circular dependency
            let mut cycle_systems = Vec::new();
            for i in 0..self.legacy_system_calls.len() {
                if in_degree[i] > 0 {
                    cycle_systems.push(self.legacy_system_calls[i].get_system_type_id());
                }
            }
            return Err(DependencyError::CircularDependency(cycle_systems));
        }
        
        self.ordered_legacy_systems = ordered;
        Ok(())
    }
    
    /// Invalidate the current system order (forces re-resolution on next run)
    fn invalidate_system_order(&mut self) {
        // For systems without dependencies, just add them to the order
        if self.ordered_systems.len() < self.systems.len() {
            // Add new systems to the end if they have no dependencies
            let new_index = self.systems.len() - 1;
            self.ordered_systems.push(new_index);
        }
    }
    
    /// Get the execution order of systems (both new and legacy)
    pub fn get_system_execution_order(&self) -> Vec<&str> {
        let mut order = Vec::new();
        
        // Add new System objects execution order first
        for &i in &self.ordered_systems {
            if i < self.systems.len() {
                order.push(self.systems[i].get_system_name());
            }
        }
        
        // Add legacy SystemCall systems execution order
        for &i in &self.ordered_legacy_systems {
            if i < self.legacy_system_calls.len() {
                order.push(self.legacy_system_calls[i].get_system_name());
            }
        }
        
        order
    }
    
    /// Get system dependencies for both new System objects and legacy SystemCall systems
    pub fn get_system_dependencies(&self, system_name: &str) -> Option<Vec<SystemTypeId>> {
        // First check new System objects
        if let Some(deps) = self.systems
            .iter()
            .find(|s| s.get_system_name() == system_name)
            .map(|s| s.get_dependencies()) {
            return Some(deps);
        }
        
        // Then check legacy SystemCall systems
        self.legacy_system_calls
            .iter()
            .find(|s| s.get_system_name() == system_name)
            .map(|s| s.get_dependencies())
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}