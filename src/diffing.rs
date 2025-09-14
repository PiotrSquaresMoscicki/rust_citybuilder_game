use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::ecs::{Entity, Component};
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

/// Trait for types that can be diffed
pub trait Diffable {
    /// Create a diff representing changes from self to other
    /// Returns None if there are no changes
    fn diff(&self, other: &Self) -> Option<Vec<PropertyDiff>>;
    
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
}

impl DebugTracker {
    pub fn new() -> Self {
        Self {
            enabled: false,
            frame_number: 0,
            diff_history: Vec::new(),
            component_snapshots: HashMap::new(),
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
}

/// Helper function to diff two components using their type information
pub fn diff_components(old: &dyn Component, new: &dyn Component, type_id: TypeId) -> Option<ComponentDiff> {
    use std::any::TypeId;
    
    // This is a basic implementation - in a full system, you'd use a registry
    // to map TypeIds to specific diffing functions
    if type_id == TypeId::of::<crate::examples::Position>() {
        if let (Some(old_pos), Some(new_pos)) = (
            old.as_any().downcast_ref::<crate::examples::Position>(),
            new.as_any().downcast_ref::<crate::examples::Position>()
        ) {
            if let Some(changes) = old_pos.diff(new_pos) {
                return Some(ComponentDiff {
                    entity_id: 0, // Will be set by caller
                    component_type: "Position".to_string(),
                    changes,
                });
            }
        }
    } else if type_id == TypeId::of::<crate::examples::Velocity>() {
        if let (Some(old_vel), Some(new_vel)) = (
            old.as_any().downcast_ref::<crate::examples::Velocity>(),
            new.as_any().downcast_ref::<crate::examples::Velocity>()
        ) {
            if let Some(changes) = old_vel.diff(new_vel) {
                return Some(ComponentDiff {
                    entity_id: 0, // Will be set by caller
                    component_type: "Velocity".to_string(),
                    changes,
                });
            }
        }
    } else if type_id == TypeId::of::<crate::examples::Health>() {
        if let (Some(old_health), Some(new_health)) = (
            old.as_any().downcast_ref::<crate::examples::Health>(),
            new.as_any().downcast_ref::<crate::examples::Health>()
        ) {
            if let Some(changes) = old_health.diff(new_health) {
                return Some(ComponentDiff {
                    entity_id: 0, // Will be set by caller
                    component_type: "Health".to_string(),
                    changes,
                });
            }
        }
    }
    
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
            
            fn type_name() -> &'static str {
                stringify!($struct_name)
            }
        }
    };
}