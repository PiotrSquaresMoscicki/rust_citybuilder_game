use std::any::Any;
use crate::ecs::{Component, Entity};

/// Component that manages parent-child relationships between entities
#[derive(Debug, Clone, PartialEq)]
pub struct HierarchyComponent {
    parent: Option<Entity>,
    children: Vec<Entity>,
}

impl HierarchyComponent {
    /// Creates a new hierarchy component with no parent or children
    pub fn new() -> Self {
        Self {
            parent: None,
            children: Vec::new(),
        }
    }

    /// Creates a hierarchy component with a parent
    pub fn with_parent(parent: Entity) -> Self {
        Self {
            parent: Some(parent),
            children: Vec::new(),
        }
    }

    /// Gets the parent entity if it exists
    pub fn parent(&self) -> Option<Entity> {
        self.parent
    }

    /// Sets the parent entity
    pub fn set_parent(&mut self, parent: Option<Entity>) {
        self.parent = parent;
    }

    /// Gets all child entities
    pub fn children(&self) -> &[Entity] {
        &self.children
    }

    /// Adds a child entity
    pub fn add_child(&mut self, child: Entity) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    /// Removes a child entity
    pub fn remove_child(&mut self, child: Entity) -> bool {
        if let Some(pos) = self.children.iter().position(|&c| c == child) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    /// Removes all children
    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    /// Returns true if this entity has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Returns true if this entity has a parent
    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    /// Returns the number of children
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Checks if the given entity is a child of this entity
    pub fn is_child(&self, entity: Entity) -> bool {
        self.children.contains(&entity)
    }

    /// Checks if the given entity is the parent of this entity
    pub fn is_parent(&self, entity: Entity) -> bool {
        self.parent == Some(entity)
    }
}

impl Component for HierarchyComponent {
    fn validate(&self) -> bool {
        // Ensure no entity is both a parent and a child (direct circular reference)
        if let Some(parent) = self.parent {
            !self.children.contains(&parent)
        } else {
            true
        }
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
        assert!(hierarchy.parent().is_none());
        assert!(hierarchy.children().is_empty());
        assert!(!hierarchy.has_children());
        assert!(!hierarchy.has_parent());
        assert_eq!(hierarchy.child_count(), 0);
    }

    #[test]
    fn test_hierarchy_with_parent() {
        let parent_entity = 42;
        let hierarchy = HierarchyComponent::with_parent(parent_entity);
        assert_eq!(hierarchy.parent(), Some(parent_entity));
        assert!(hierarchy.has_parent());
        assert!(hierarchy.is_parent(parent_entity));
        assert!(!hierarchy.is_parent(999));
    }

    #[test]
    fn test_parent_management() {
        let mut hierarchy = HierarchyComponent::new();
        let parent_entity = 100;

        hierarchy.set_parent(Some(parent_entity));
        assert_eq!(hierarchy.parent(), Some(parent_entity));
        assert!(hierarchy.has_parent());

        hierarchy.set_parent(None);
        assert!(hierarchy.parent().is_none());
        assert!(!hierarchy.has_parent());
    }

    #[test]
    fn test_child_management() {
        let mut hierarchy = HierarchyComponent::new();
        let child1 = 10;
        let child2 = 20;
        let child3 = 30;

        // Add children
        hierarchy.add_child(child1);
        hierarchy.add_child(child2);
        hierarchy.add_child(child3);

        assert_eq!(hierarchy.child_count(), 3);
        assert!(hierarchy.has_children());
        assert!(hierarchy.is_child(child1));
        assert!(hierarchy.is_child(child2));
        assert!(hierarchy.is_child(child3));
        assert!(!hierarchy.is_child(999));

        // Check children list
        let children = hierarchy.children();
        assert_eq!(children.len(), 3);
        assert!(children.contains(&child1));
        assert!(children.contains(&child2));
        assert!(children.contains(&child3));

        // Remove a child
        assert!(hierarchy.remove_child(child2));
        assert_eq!(hierarchy.child_count(), 2);
        assert!(!hierarchy.is_child(child2));
        assert!(hierarchy.is_child(child1));
        assert!(hierarchy.is_child(child3));

        // Try to remove non-existent child
        assert!(!hierarchy.remove_child(999));
        assert_eq!(hierarchy.child_count(), 2);

        // Clear all children
        hierarchy.clear_children();
        assert_eq!(hierarchy.child_count(), 0);
        assert!(!hierarchy.has_children());
        assert!(hierarchy.children().is_empty());
    }

    #[test]
    fn test_duplicate_child_prevention() {
        let mut hierarchy = HierarchyComponent::new();
        let child = 42;

        hierarchy.add_child(child);
        hierarchy.add_child(child); // Try to add again
        
        assert_eq!(hierarchy.child_count(), 1);
        assert!(hierarchy.is_child(child));
    }

    #[test]
    fn test_validation() {
        // Valid hierarchy - no circular reference
        let mut hierarchy = HierarchyComponent::new();
        hierarchy.set_parent(Some(100));
        hierarchy.add_child(200);
        hierarchy.add_child(300);
        assert!(hierarchy.validate());

        // Invalid hierarchy - parent is also a child (circular reference)
        let mut invalid_hierarchy = HierarchyComponent::new();
        invalid_hierarchy.set_parent(Some(100));
        invalid_hierarchy.add_child(100); // Same entity as parent
        assert!(!invalid_hierarchy.validate());

        // Valid hierarchy with no parent
        let no_parent_hierarchy = HierarchyComponent::new();
        assert!(no_parent_hierarchy.validate());
    }

    #[test]
    fn test_component_trait() {
        let hierarchy = HierarchyComponent::new();
        
        // Test as_any
        let as_any = hierarchy.as_any();
        assert!(as_any.downcast_ref::<HierarchyComponent>().is_some());

        // Test clone_box
        let cloned = hierarchy.clone_box();
        let downcast = cloned.as_any().downcast_ref::<HierarchyComponent>().unwrap();
        assert_eq!(downcast, &hierarchy);
    }
}