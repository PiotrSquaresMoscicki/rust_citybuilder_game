use std::any::Any;
use crate::ecs::{Component, Entity};
use serde::{Serialize, Deserialize};

/// Hierarchy component that allows entities to have parent-child relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyComponent {
    /// Parent entity ID, if any
    pub parent: Option<Entity>,
    /// List of child entity IDs
    pub children: Vec<Entity>,
}

impl HierarchyComponent {
    /// Create a new hierarchy component with no parent or children
    pub fn new() -> Self {
        Self {
            parent: None,
            children: Vec::new(),
        }
    }
    
    /// Create a hierarchy component with a parent
    pub fn with_parent(parent: Entity) -> Self {
        Self {
            parent: Some(parent),
            children: Vec::new(),
        }
    }
    
    /// Get the parent entity, if any
    pub fn get_parent(&self) -> Option<Entity> {
        self.parent
    }
    
    /// Set the parent entity
    pub fn set_parent(&mut self, parent: Option<Entity>) {
        self.parent = parent;
    }
    
    /// Get all child entities
    pub fn get_children(&self) -> &[Entity] {
        &self.children
    }
    
    /// Add a child entity
    pub fn add_child(&mut self, child: Entity) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }
    
    /// Remove a child entity
    pub fn remove_child(&mut self, child: Entity) -> bool {
        if let Some(pos) = self.children.iter().position(|&x| x == child) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Check if this entity has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
    
    /// Check if this entity has a parent
    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
    
    /// Get the number of children
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
    
    /// Clear all children
    pub fn clear_children(&mut self) {
        self.children.clear();
    }
    
    /// Check if an entity is a child of this entity
    pub fn is_child(&self, entity: Entity) -> bool {
        self.children.contains(&entity)
    }
}

impl Component for HierarchyComponent {
    fn validate(&self) -> bool {
        // Check that parent is not in children (prevents self-referencing)
        if let Some(parent) = self.parent {
            if self.children.contains(&parent) {
                return false;
            }
        }
        
        // Check for duplicate children
        let mut sorted_children = self.children.clone();
        sorted_children.sort();
        for i in 1..sorted_children.len() {
            if sorted_children[i] == sorted_children[i - 1] {
                return false;
            }
        }
        
        true
    }
    
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

impl Default for HierarchyComponent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_creation() {
        let hierarchy = HierarchyComponent::new();
        assert!(hierarchy.get_parent().is_none());
        assert!(hierarchy.get_children().is_empty());
        assert!(!hierarchy.has_parent());
        assert!(!hierarchy.has_children());
        assert_eq!(hierarchy.child_count(), 0);
    }
    
    #[test]
    fn test_hierarchy_with_parent() {
        let parent_id = 42;
        let hierarchy = HierarchyComponent::with_parent(parent_id);
        assert_eq!(hierarchy.get_parent(), Some(parent_id));
        assert!(hierarchy.has_parent());
        assert!(!hierarchy.has_children());
    }
    
    #[test]
    fn test_add_remove_children() {
        let mut hierarchy = HierarchyComponent::new();
        let child1 = 10;
        let child2 = 20;
        
        // Add children
        hierarchy.add_child(child1);
        hierarchy.add_child(child2);
        
        assert!(hierarchy.has_children());
        assert_eq!(hierarchy.child_count(), 2);
        assert!(hierarchy.is_child(child1));
        assert!(hierarchy.is_child(child2));
        assert!(!hierarchy.is_child(99)); // Non-existent child
        
        // Remove a child
        assert!(hierarchy.remove_child(child1));
        assert!(!hierarchy.is_child(child1));
        assert!(hierarchy.is_child(child2));
        assert_eq!(hierarchy.child_count(), 1);
        
        // Try to remove non-existent child
        assert!(!hierarchy.remove_child(99));
        assert_eq!(hierarchy.child_count(), 1);
        
        // Clear all children
        hierarchy.clear_children();
        assert!(!hierarchy.has_children());
        assert_eq!(hierarchy.child_count(), 0);
    }
    
    #[test]
    fn test_prevent_duplicate_children() {
        let mut hierarchy = HierarchyComponent::new();
        let child = 10;
        
        hierarchy.add_child(child);
        hierarchy.add_child(child); // Try to add the same child again
        
        assert_eq!(hierarchy.child_count(), 1);
        assert!(hierarchy.is_child(child));
    }
    
    #[test]
    fn test_validation_prevents_self_referencing() {
        let mut hierarchy = HierarchyComponent::new();
        let entity_id = 5;
        
        hierarchy.set_parent(Some(entity_id));
        hierarchy.add_child(entity_id); // Same as parent - invalid
        
        assert!(!hierarchy.validate());
    }
    
    #[test]
    fn test_validation_valid_hierarchy() {
        let mut hierarchy = HierarchyComponent::new();
        
        hierarchy.set_parent(Some(1));
        hierarchy.add_child(2);
        hierarchy.add_child(3);
        
        assert!(hierarchy.validate());
    }
    
    #[test]
    fn test_component_trait() {
        let hierarchy = HierarchyComponent::new();
        
        // Test that it implements Component trait properly
        assert!(hierarchy.validate());
        
        // Test cloning
        let cloned = hierarchy.clone_box();
        let downcast = cloned.as_any().downcast_ref::<HierarchyComponent>().unwrap();
        assert_eq!(downcast.parent, hierarchy.parent);
        assert_eq!(downcast.children, hierarchy.children);
    }
}