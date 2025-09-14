use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::ecs::{Entity, Component, World, ComponentPool};
use std::any::TypeId;

/// Represents a single property change in a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDiff {
    pub property_name: String,
    pub new_value: String, // RON serialized value
}

/// Record of all changes to a component for a specific entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDiff {
    pub entity_id: Entity,
    pub component_type: String,
    pub changes: Vec<PropertyDiff>,
}

/// Complete record of changes during a system execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDiffRecord {
    pub frame_number: u64,
    pub system_name: String,
    pub component_diffs: Vec<ComponentDiff>,
}

/// Trait for types that can be diffed and restored
pub trait Diffable {
    /// Create a diff representing changes from self to other
    /// Returns None if there are no changes
    fn diff(&self, other: &Self) -> Option<Vec<PropertyDiff>>;
    
    /// Apply changes from a diff to restore state
    /// Returns true if successful, false if diff couldn't be applied
    fn apply_diff(&mut self, changes: &[PropertyDiff]) -> bool;
    
    /// Get the type name for debugging purposes
    fn type_name() -> &'static str where Self: Sized;
}

/// Implement Diffable for basic types
impl Diffable for i32 {
    fn diff(&self, other: &Self) -> Option<Vec<PropertyDiff>> {
        if self != other {
            Some(vec![PropertyDiff {
                property_name: "value".to_string(),
                new_value: ron::to_string(other).unwrap_or_default(),
            }])
        } else {
            None
        }
    }
    
    fn apply_diff(&mut self, changes: &[PropertyDiff]) -> bool {
        for change in changes {
            if change.property_name == "value" {
                if let Ok(new_value) = ron::from_str::<i32>(&change.new_value) {
                    *self = new_value;
                    return true;
                }
            }
        }
        false
    }
    
    fn type_name() -> &'static str {
        "i32"
    }
}

impl Diffable for f32 {
    fn diff(&self, other: &Self) -> Option<Vec<PropertyDiff>> {
        if (self - other).abs() > f32::EPSILON {
            Some(vec![PropertyDiff {
                property_name: "value".to_string(),
                new_value: ron::to_string(other).unwrap_or_default(),
            }])
        } else {
            None
        }
    }
    
    fn apply_diff(&mut self, changes: &[PropertyDiff]) -> bool {
        for change in changes {
            if change.property_name == "value" {
                if let Ok(new_value) = ron::from_str::<f32>(&change.new_value) {
                    *self = new_value;
                    return true;
                }
            }
        }
        false
    }
    
    fn type_name() -> &'static str {
        "f32"
    }
}

impl Diffable for String {
    fn diff(&self, other: &Self) -> Option<Vec<PropertyDiff>> {
        if self != other {
            Some(vec![PropertyDiff {
                property_name: "value".to_string(),
                new_value: ron::to_string(other).unwrap_or_default(),
            }])
        } else {
            None
        }
    }
    
    fn apply_diff(&mut self, changes: &[PropertyDiff]) -> bool {
        for change in changes {
            if change.property_name == "value" {
                if let Ok(new_value) = ron::from_str::<String>(&change.new_value) {
                    *self = new_value;
                    return true;
                }
            }
        }
        false
    }
    
    fn type_name() -> &'static str {
        "String"
    }
}

impl Diffable for f64 {
    fn diff(&self, other: &Self) -> Option<Vec<PropertyDiff>> {
        if (self - other).abs() > f64::EPSILON {
            Some(vec![PropertyDiff {
                property_name: "value".to_string(),
                new_value: ron::to_string(other).unwrap_or_default(),
            }])
        } else {
            None
        }
    }
    
    fn apply_diff(&mut self, changes: &[PropertyDiff]) -> bool {
        for change in changes {
            if change.property_name == "value" {
                if let Ok(new_value) = ron::from_str::<f64>(&change.new_value) {
                    *self = new_value;
                    return true;
                }
            }
        }
        false
    }
    
    fn type_name() -> &'static str {
        "f64"
    }
}

impl Diffable for u64 {
    fn diff(&self, other: &Self) -> Option<Vec<PropertyDiff>> {
        if self != other {
            Some(vec![PropertyDiff {
                property_name: "value".to_string(),
                new_value: ron::to_string(other).unwrap_or_default(),
            }])
        } else {
            None
        }
    }
    
    fn apply_diff(&mut self, changes: &[PropertyDiff]) -> bool {
        for change in changes {
            if change.property_name == "value" {
                if let Ok(new_value) = ron::from_str::<u64>(&change.new_value) {
                    *self = new_value;
                    return true;
                }
            }
        }
        false
    }
    
    fn type_name() -> &'static str {
        "u64"
    }
}

impl Diffable for bool {
    fn diff(&self, other: &Self) -> Option<Vec<PropertyDiff>> {
        if self != other {
            Some(vec![PropertyDiff {
                property_name: "value".to_string(),
                new_value: ron::to_string(other).unwrap_or_default(),
            }])
        } else {
            None
        }
    }
    
    fn apply_diff(&mut self, changes: &[PropertyDiff]) -> bool {
        for change in changes {
            if change.property_name == "value" {
                if let Ok(new_value) = ron::from_str::<bool>(&change.new_value) {
                    *self = new_value;
                    return true;
                }
            }
        }
        false
    }
    
    fn type_name() -> &'static str {
        "bool"
    }
}

impl<T: Diffable + Clone> Diffable for Vec<T> {
    fn diff(&self, other: &Self) -> Option<Vec<PropertyDiff>> {
        if self.len() != other.len() {
            // Size changed, record the entire new vector
            Some(vec![PropertyDiff {
                property_name: "value".to_string(),
                new_value: format!("Vec with {} elements", other.len()),
            }])
        } else {
            // Check element-wise differences
            let mut changes = Vec::new();
            for (i, (old, new)) in self.iter().zip(other.iter()).enumerate() {
                if let Some(element_diffs) = old.diff(new) {
                    for diff in element_diffs {
                        changes.push(PropertyDiff {
                            property_name: format!("[{}].{}", i, diff.property_name),
                            new_value: diff.new_value,
                        });
                    }
                }
            }
            
            if changes.is_empty() {
                None
            } else {
                Some(changes)
            }
        }
    }
    
    fn apply_diff(&mut self, changes: &[PropertyDiff]) -> bool {
        // Basic implementation - full replacement for size changes
        // Element-wise updates could be implemented more sophisticatedly
        for change in changes {
            if change.property_name == "value" {
                // For now, handle size changes by clearing the vector
                // A more complete implementation would parse the serialized data
                self.clear();
                return true;
            }
        }
        false
    }
    
    fn type_name() -> &'static str {
        "Vec"
    }
}

impl<K: std::fmt::Debug + Clone + Eq + std::hash::Hash, V: Diffable + Clone> Diffable for HashMap<K, V> {
    fn diff(&self, other: &Self) -> Option<Vec<PropertyDiff>> {
        let mut changes = Vec::new();
        
        // Check for added/modified entries
        for (key, new_value) in other.iter() {
            match self.get(key) {
                Some(old_value) => {
                    if let Some(value_diffs) = old_value.diff(new_value) {
                        for diff in value_diffs {
                            changes.push(PropertyDiff {
                                property_name: format!("[{:?}].{}", key, diff.property_name),
                                new_value: diff.new_value,
                            });
                        }
                    }
                }
                None => {
                    changes.push(PropertyDiff {
                        property_name: format!("[{:?}]", key),
                        new_value: "added".to_string(),
                    });
                }
            }
        }
        
        // Check for removed entries
        for key in self.keys() {
            if !other.contains_key(key) {
                changes.push(PropertyDiff {
                    property_name: format!("[{:?}]", key),
                    new_value: "removed".to_string(),
                });
            }
        }
        
        if changes.is_empty() {
            None
        } else {
            Some(changes)
        }
    }
    
    fn apply_diff(&mut self, _changes: &[PropertyDiff]) -> bool {
        // Basic implementation - HashMap diff application is complex
        // For now, just return false to indicate not implemented
        false
    }
    
    fn type_name() -> &'static str {
        "HashMap"
    }
}

/// Debug tracker for recording component state changes
pub struct DebugTracker {
    pub enabled: bool,
    pub frame_number: u64,
    pub diff_history: Vec<SystemDiffRecord>,
    component_snapshots: HashMap<(Entity, TypeId), Box<dyn Component>>,
    /// Stored world states for replay functionality
    pub world_states: Vec<WorldState>,
}

/// Represents a complete snapshot of the world state at a specific frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub frame_number: u64,
    pub entities: Vec<Entity>,
    pub component_data: HashMap<Entity, HashMap<String, String>>, // RON serialized components
}

impl DebugTracker {
    pub fn new() -> Self {
        Self {
            enabled: false,
            frame_number: 0,
            diff_history: Vec::new(),
            component_snapshots: HashMap::new(),
            world_states: Vec::new(),
        }
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    pub fn next_frame(&mut self) {
        self.frame_number += 1;
    }
    
    /// Take a snapshot of all mutable components before system execution
    pub fn snapshot_components(&mut self, world: &crate::ecs::World, entities: &[Entity], mutable_types: &[TypeId]) {
        if !self.enabled {
            return;
        }
        
        self.component_snapshots.clear();
        
        for &entity in entities {
            for &type_id in mutable_types {
                if let Some(component) = world.get_component_snapshot(entity, type_id) {
                    self.component_snapshots.insert((entity, type_id), component);
                }
            }
        }
    }
    
    /// Compare current state with snapshots and record diffs
    pub fn record_diffs(&mut self, world: &crate::ecs::World, system_name: &str, entities: &[Entity], mutable_types: &[TypeId]) {
        if !self.enabled {
            return;
        }
        
        let mut component_diffs = Vec::new();
        
        for &entity in entities {
            for &type_id in mutable_types {
                if let Some(old_component) = self.component_snapshots.get(&(entity, type_id)) {
                    if let Some(new_component) = world.get_component_snapshot(entity, type_id) {
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
            let record = SystemDiffRecord {
                frame_number: self.frame_number,
                system_name: system_name.to_string(),
                component_diffs,
            };
            self.diff_history.push(record);
        }
    }
    
    /// Get all diff records in human-readable format
    pub fn get_diff_history_formatted(&self) -> String {
        let mut output = String::new();
        
        for record in &self.diff_history {
            output.push_str(&format!(
                "Frame {}: System '{}'\n",
                record.frame_number,
                record.system_name
            ));
            
            for diff in &record.component_diffs {
                output.push_str(&format!(
                    "  Entity {}: {} changed\n",
                    diff.entity_id,
                    diff.component_type
                ));
                
                for change in &diff.changes {
                    output.push_str(&format!(
                        "    {} -> {}\n",
                        change.property_name,
                        change.new_value
                    ));
                }
            }
            output.push('\n');
        }
        
        output
    }
    
    /// Clear all recorded diffs
    pub fn clear_history(&mut self) {
        self.diff_history.clear();
    }
    
    /// Capture the current world state for later restoration
    pub fn capture_world_state(&mut self, entities: &[Entity], component_pools: &HashMap<TypeId, ComponentPool>) {
        if !self.enabled {
            return;
        }
        
        let mut component_data = HashMap::new();
        
        // For each entity, serialize all its components
        for &entity in entities {
            let mut entity_components = HashMap::new();
            
            // Check each component pool for this entity
            for (type_id, pool) in component_pools {
                if pool.contains(entity) {
                    if let Some(component_ref) = pool.get(entity) {
                        let type_name = get_component_type_name(*type_id);
                        if let Ok(serialized) = serialize_component(component_ref.as_ref(), *type_id) {
                            entity_components.insert(type_name, serialized);
                        }
                    }
                }
            }
            
            if !entity_components.is_empty() {
                component_data.insert(entity, entity_components);
            }
        }
        
        let state = WorldState {
            frame_number: self.frame_number,
            entities: entities.to_vec(),
            component_data,
        };
        
        self.world_states.push(state);
    }
    
    /// Restore world state to a specific frame
    pub fn restore_world_state(&self, world: &mut World, target_frame: u64) -> bool {
        if let Some(state) = self.world_states.iter().find(|s| s.frame_number == target_frame) {
            return self.apply_world_state(world, state);
        }
        false
    }
    
    /// Apply a world state to the current world
    pub fn apply_world_state(&self, world: &mut World, state: &WorldState) -> bool {
        // Clear current world state
        world.clear_world();
        
        // Restore entities
        world.set_entities(state.entities.clone());
        
        // Restore components
        for (&entity, components) in &state.component_data {
            for (type_name, serialized_data) in components {
                if let Some(component) = deserialize_component(type_name, serialized_data) {
                    let type_id = get_type_id_for_name(type_name);
                    let pool = world.get_component_pools_mut()
                        .entry(type_id)
                        .or_insert_with(|| crate::ecs::ComponentPool::new());
                    pool.insert(entity, component);
                }
            }
        }
        
        true
    }
    
    /// Replay changes from recorded diffs up to a specific frame
    pub fn replay_to_frame(&self, world: &mut World, target_frame: u64) -> bool {
        // Find the latest world state before or at target frame
        let base_state = self.world_states.iter()
            .filter(|s| s.frame_number <= target_frame)
            .max_by_key(|s| s.frame_number);
            
        if let Some(state) = base_state {
            // Restore to base state
            if !self.apply_world_state(world, state) {
                return false;
            }
            
            // Apply diffs from base state to target frame
            for record in &self.diff_history {
                if record.frame_number > state.frame_number && record.frame_number <= target_frame {
                    self.apply_system_diff_record(world, record);
                }
            }
            
            return true;
        }
        
        false
    }
    
    /// Apply a single system diff record to the world
    fn apply_system_diff_record(&self, world: &mut World, record: &SystemDiffRecord) {
        for component_diff in &record.component_diffs {
            self.apply_component_diff(world, component_diff);
        }
    }
    
    /// Apply a component diff to the world
    fn apply_component_diff(&self, world: &mut World, diff: &ComponentDiff) {
        let type_id = get_type_id_for_name(&diff.component_type);
        
        // Get the component and apply the diff
        if let Some(pool) = world.get_component_pools().get(&type_id) {
            if let Some(mut component_ref) = pool.get_mut(diff.entity_id) {
                apply_diff_to_component(component_ref.as_mut(), &diff.changes, type_id);
            }
        }
    }
}

/// Helper function to diff two components using their type information
pub fn diff_components(_old: &dyn Component, _new: &dyn Component, _type_id: TypeId) -> Option<ComponentDiff> {
    // Temporarily disabled to avoid dependency on examples module
    // TODO: Implement this with a proper registry system
    None
}

/// Macro to implement Diffable trait for structs
/// This is a basic implementation - a full proc macro would be better
#[macro_export]
macro_rules! diffable {
    ($struct_name:ident { $($field:ident),* }) => {
        impl $crate::diffing::Diffable for $struct_name {
            fn diff(&self, other: &Self) -> Option<Vec<$crate::diffing::PropertyDiff>> {
                let mut changes = Vec::new();
                
                $(
                    if let Some(field_diffs) = self.$field.diff(&other.$field) {
                        for diff in field_diffs {
                            changes.push($crate::diffing::PropertyDiff {
                                property_name: if diff.property_name == "value" {
                                    stringify!($field).to_string()
                                } else {
                                    format!("{}.{}", stringify!($field), diff.property_name)
                                },
                                new_value: diff.new_value,
                            });
                        }
                    }
                )*
                
                if changes.is_empty() {
                    None
                } else {
                    Some(changes)
                }
            }
            
            fn apply_diff(&mut self, changes: &[$crate::diffing::PropertyDiff]) -> bool {
                let mut applied = false;
                
                for change in changes {
                    $(
                        if change.property_name == stringify!($field) || 
                           change.property_name.starts_with(&format!("{}.", stringify!($field))) {
                            
                            if change.property_name == stringify!($field) {
                                // Direct field change
                                if let Ok(new_value) = ron::from_str(&change.new_value) {
                                    self.$field = new_value;
                                    applied = true;
                                }
                            } else {
                                // Nested change
                                let sub_changes = vec![$crate::diffing::PropertyDiff {
                                    property_name: change.property_name[stringify!($field).len() + 1..].to_string(),
                                    new_value: change.new_value.clone(),
                                }];
                                if self.$field.apply_diff(&sub_changes) {
                                    applied = true;
                                }
                            }
                        }
                    )*
                }
                
                applied
            }
            
            fn type_name() -> &'static str {
                stringify!($struct_name)
            }
        }
    };
}

/// Helper function to get component type name from TypeId
pub fn get_component_type_name(type_id: TypeId) -> String {
    use std::any::TypeId;
    
    if type_id == TypeId::of::<crate::examples::Position>() {
        "Position".to_string()
    } else if type_id == TypeId::of::<crate::examples::Velocity>() {
        "Velocity".to_string()
    } else if type_id == TypeId::of::<crate::examples::Health>() {
        "Health".to_string()
    } else {
        format!("Unknown_{:?}", type_id)
    }
}

/// Helper function to get TypeId from component type name
pub fn get_type_id_for_name(type_name: &str) -> TypeId {
    use std::any::TypeId;
    
    match type_name {
        "Position" => TypeId::of::<crate::examples::Position>(),
        "Velocity" => TypeId::of::<crate::examples::Velocity>(),
        "Health" => TypeId::of::<crate::examples::Health>(),
        _ => TypeId::of::<()>(), // Fallback
    }
}

/// Serialize a component to RON string
pub fn serialize_component(component: &dyn Component, type_id: TypeId) -> Result<String, String> {
    use std::any::TypeId;
    
    if type_id == TypeId::of::<crate::examples::Position>() {
        if let Some(pos) = component.as_any().downcast_ref::<crate::examples::Position>() {
            return ron::to_string(pos).map_err(|e| e.to_string());
        }
    } else if type_id == TypeId::of::<crate::examples::Velocity>() {
        if let Some(vel) = component.as_any().downcast_ref::<crate::examples::Velocity>() {
            return ron::to_string(vel).map_err(|e| e.to_string());
        }
    } else if type_id == TypeId::of::<crate::examples::Health>() {
        if let Some(health) = component.as_any().downcast_ref::<crate::examples::Health>() {
            return ron::to_string(health).map_err(|e| e.to_string());
        }
    }
    
    Err("Unknown component type".to_string())
}

/// Deserialize a component from RON string
pub fn deserialize_component(type_name: &str, data: &str) -> Option<Box<dyn Component>> {
    match type_name {
        "Position" => {
            if let Ok(pos) = ron::from_str::<crate::examples::Position>(data) {
                Some(Box::new(pos))
            } else {
                None
            }
        }
        "Velocity" => {
            if let Ok(vel) = ron::from_str::<crate::examples::Velocity>(data) {
                Some(Box::new(vel))
            } else {
                None
            }
        }
        "Health" => {
            if let Ok(health) = ron::from_str::<crate::examples::Health>(data) {
                Some(Box::new(health))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Apply diff changes to a component
pub fn apply_diff_to_component(component: &mut dyn Component, changes: &[PropertyDiff], type_id: TypeId) {
    use std::any::TypeId;
    
    if type_id == TypeId::of::<crate::examples::Position>() {
        if let Some(pos) = component.as_any_mut().downcast_mut::<crate::examples::Position>() {
            pos.apply_diff(changes);
        }
    } else if type_id == TypeId::of::<crate::examples::Velocity>() {
        if let Some(vel) = component.as_any_mut().downcast_mut::<crate::examples::Velocity>() {
            vel.apply_diff(changes);
        }
    } else if type_id == TypeId::of::<crate::examples::Health>() {
        if let Some(health) = component.as_any_mut().downcast_mut::<crate::examples::Health>() {
            health.apply_diff(changes);
        }
    }
}