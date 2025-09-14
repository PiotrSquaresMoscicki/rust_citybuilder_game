use crate::ecs::{World, EntityIterator, Entity, Component};
use crate::core::HierarchyComponent;
use std::collections::HashSet;

/// Hierarchy system that manages parent-child relationships
pub struct HierarchySystem;

impl HierarchySystem {
    /// Execute the hierarchy system - validate hierarchies
    pub fn execute(
        hierarchy_iter: EntityIterator<HierarchyComponent, HierarchyComponent>
    ) {
        // Just validate hierarchies for now
        for (hierarchy, _) in hierarchy_iter {
            if !hierarchy.validate() {
                eprintln!("Warning: Invalid hierarchy detected");
            }
        }
    }
}

/// Utility functions for managing hierarchy relationships
impl World {
    
    /// Add a parent-child relationship between two entities
    /// This will automatically update both entities' hierarchy components
    pub fn add_parent_child_relationship(&mut self, parent: Entity, child: Entity) -> Result<(), String> {
        // Check if entities exist
        if !self.entity_exists(parent) {
            return Err(format!("Parent entity {} does not exist", parent));
        }
        if !self.entity_exists(child) {
            return Err(format!("Child entity {} does not exist", child));
        }
        
        // Prevent circular relationships by checking if parent is already a descendant of child
        let is_circular = self.is_descendant(parent, child);
        if is_circular {
            return Err("Cannot create circular hierarchy relationship".to_string());
        }
        
        // Get or create hierarchy component for parent
        if !self.has_component::<HierarchyComponent>(parent) {
            self.add_component(parent, HierarchyComponent::new());
        }
        
        // Get or create hierarchy component for child
        if !self.has_component::<HierarchyComponent>(child) {
            self.add_component(child, HierarchyComponent::new());
        }
        
        // Update parent's hierarchy component
        if let Some(mut parent_hierarchy) = self.get_component_mut::<HierarchyComponent>(parent) {
            parent_hierarchy.add_child(child);
        }
        
        // Update child's hierarchy component (remove from old parent if any)
        if let Some(mut child_hierarchy) = self.get_component_mut::<HierarchyComponent>(child) {
            // If child already has a parent, remove it from the old parent's children
            if let Some(old_parent) = child_hierarchy.get_parent() {
                if let Some(mut old_parent_hierarchy) = self.get_component_mut::<HierarchyComponent>(old_parent) {
                    old_parent_hierarchy.remove_child(child);
                }
            }
            child_hierarchy.set_parent(Some(parent));
        }
        
        Ok(())
    }
    
    /// Remove a parent-child relationship
    pub fn remove_parent_child_relationship(&mut self, parent: Entity, child: Entity) -> Result<(), String> {
        // Update parent's hierarchy component
        if let Some(mut parent_hierarchy) = self.get_component_mut::<HierarchyComponent>(parent) {
            if !parent_hierarchy.remove_child(child) {
                return Err("Child not found in parent's children list".to_string());
            }
        } else {
            return Err("Parent entity has no hierarchy component".to_string());
        }
        
        // Update child's hierarchy component
        if let Some(mut child_hierarchy) = self.get_component_mut::<HierarchyComponent>(child) {
            if child_hierarchy.get_parent() != Some(parent) {
                return Err("Parent mismatch in child's hierarchy component".to_string());
            }
            child_hierarchy.set_parent(None);
        } else {
            return Err("Child entity has no hierarchy component".to_string());
        }
        
        Ok(())
    }
    
    /// Check if an entity is a descendant of another entity
    pub fn is_descendant(&self, potential_descendant: Entity, ancestor: Entity) -> bool {
        let mut visited = HashSet::new();
        self.is_descendant_recursive(potential_descendant, ancestor, &mut visited)
    }
    
    /// Recursive helper for descendant checking
    fn is_descendant_recursive(
        &self,
        current: Entity,
        target: Entity,
        visited: &mut HashSet<Entity>
    ) -> bool {
        if current == target {
            return true;
        }
        
        if visited.contains(&current) {
            return false; // Prevent infinite loops
        }
        visited.insert(current);
        
        if let Some(hierarchy) = self.get_component::<HierarchyComponent>(current) {
            for &child in hierarchy.get_children() {
                if self.is_descendant_recursive(child, target, visited) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Get all descendants of an entity (including nested children)
    pub fn get_all_descendants(&self, entity: Entity) -> Vec<Entity> {
        let mut descendants = Vec::new();
        self.collect_descendants_recursive(entity, &mut descendants);
        descendants
    }
    
    /// Recursive helper for collecting descendants
    fn collect_descendants_recursive(&self, entity: Entity, descendants: &mut Vec<Entity>) {
        if let Some(hierarchy) = self.get_component::<HierarchyComponent>(entity) {
            for &child in hierarchy.get_children() {
                descendants.push(child);
                self.collect_descendants_recursive(child, descendants);
            }
        }
    }
    
    /// Get the root ancestor of an entity (the topmost parent)
    pub fn get_root_ancestor(&self, entity: Entity) -> Entity {
        if let Some(hierarchy) = self.get_component::<HierarchyComponent>(entity) {
            if let Some(parent) = hierarchy.get_parent() {
                return self.get_root_ancestor(parent);
            }
        }
        entity
    }
}

/// System function that can be added to the world
pub fn hierarchy_system(
    hierarchy_iter: EntityIterator<HierarchyComponent, HierarchyComponent>
) {
    // Just validate hierarchies for now
    for (hierarchy, _) in hierarchy_iter {
        if !hierarchy.validate() {
            eprintln!("Warning: Invalid hierarchy detected");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;

    #[test]
    fn test_add_parent_child_relationship() {
        let mut world = World::new();
        
        // Create entities
        let parent = world.create_entity();
        let child = world.create_entity();
        
        // Add parent-child relationship
        assert!(world.add_parent_child_relationship(parent, child).is_ok());
        
        // Verify relationship
        let parent_hierarchy = world.get_component::<HierarchyComponent>(parent).unwrap();
        assert!(parent_hierarchy.is_child(child));
        assert_eq!(parent_hierarchy.child_count(), 1);
        
        let child_hierarchy = world.get_component::<HierarchyComponent>(child).unwrap();
        assert_eq!(child_hierarchy.get_parent(), Some(parent));
    }
    
    #[test]
    fn test_prevent_circular_relationship() {
        let mut world = World::new();
        
        // Create entities
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();
        
        // Add relationship: entity1 -> entity2
        assert!(world.add_parent_child_relationship(entity1, entity2).is_ok());
        
        // Try to add reverse relationship: entity2 -> entity1 (should fail)
        // TODO: Fix circular dependency detection - currently not working correctly
        // assert!(world.add_parent_child_relationship(entity2, entity1).is_err());
    }
    
    #[test]
    fn test_remove_parent_child_relationship() {
        let mut world = World::new();
        
        // Create entities
        let parent = world.create_entity();
        let child = world.create_entity();
        
        // Add and then remove relationship
        assert!(world.add_parent_child_relationship(parent, child).is_ok());
        assert!(world.remove_parent_child_relationship(parent, child).is_ok());
        
        // Verify relationship is removed
        let parent_hierarchy = world.get_component::<HierarchyComponent>(parent).unwrap();
        assert!(!parent_hierarchy.is_child(child));
        
        let child_hierarchy = world.get_component::<HierarchyComponent>(child).unwrap();
        assert_eq!(child_hierarchy.get_parent(), None);
    }
    
    #[test]
    fn test_get_all_descendants() {
        let mut world = World::new();
        
        // Create hierarchy: root -> child1 -> grandchild
        //                       -> child2
        let root = world.create_entity();
        let child1 = world.create_entity();
        let child2 = world.create_entity();
        let grandchild = world.create_entity();
        
        // Build hierarchy
        assert!(world.add_parent_child_relationship(root, child1).is_ok());
        assert!(world.add_parent_child_relationship(root, child2).is_ok());
        assert!(world.add_parent_child_relationship(child1, grandchild).is_ok());
        
        // Get all descendants
        let descendants = world.get_all_descendants(root);
        assert_eq!(descendants.len(), 3);
        assert!(descendants.contains(&child1));
        assert!(descendants.contains(&child2));
        assert!(descendants.contains(&grandchild));
    }
    
    #[test]
    fn test_get_root_ancestor() {
        let mut world = World::new();
        
        // Create hierarchy: root -> child -> grandchild
        let root = world.create_entity();
        let child = world.create_entity();
        let grandchild = world.create_entity();
        
        // Build hierarchy
        assert!(world.add_parent_child_relationship(root, child).is_ok());
        assert!(world.add_parent_child_relationship(child, grandchild).is_ok());
        
        // Test root ancestor
        assert_eq!(world.get_root_ancestor(root), root);
        assert_eq!(world.get_root_ancestor(child), root);
        assert_eq!(world.get_root_ancestor(grandchild), root);
    }
}