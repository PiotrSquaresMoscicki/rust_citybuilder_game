use crate::ecs::{World, Component};
use crate::core::{HierarchyComponent, Transform2dComponent, Vector2d, Angle2d};

/// Example usage and tests for the hierarchy system
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_basic_usage() {
        let mut world = World::new();

        // Create a parent entity (like a vehicle)
        let vehicle = world.create_entity();
        world.add_component(vehicle, HierarchyComponent::new());
        world.add_component(vehicle, Transform2dComponent::from_translation(Vector2d::new(100.0, 100.0)));

        // Create child entities (like wheels)
        let wheel1 = world.create_entity();
        let wheel2 = world.create_entity();
        
        world.add_component(wheel1, HierarchyComponent::with_parent(vehicle));
        world.add_component(wheel1, Transform2dComponent::from_translation(Vector2d::new(10.0, 5.0))); // Local position relative to vehicle
        
        world.add_component(wheel2, HierarchyComponent::with_parent(vehicle));
        world.add_component(wheel2, Transform2dComponent::from_translation(Vector2d::new(-10.0, 5.0))); // Local position relative to vehicle

        // Update parent hierarchy to include children
        let mut vehicle_hierarchy = world.get_component_mut::<HierarchyComponent>(vehicle).unwrap();
        vehicle_hierarchy.add_child(wheel1);
        vehicle_hierarchy.add_child(wheel2);

        // Verify hierarchy structure using dual iterator since single iterator doesn't exist
        let hierarchy_iter = world.iter_entities::<HierarchyComponent, Transform2dComponent>();
        for (hierarchy, _transform) in hierarchy_iter {
            if hierarchy.has_children() {
                // This is the parent
                assert_eq!(hierarchy.child_count(), 2);
                assert!(hierarchy.is_child(wheel1));
                assert!(hierarchy.is_child(wheel2));
            } else {
                // These are children
                assert!(hierarchy.has_parent());
                assert_eq!(hierarchy.parent(), Some(vehicle));
            }
        }
    }

    #[test]
    fn test_hierarchy_component_functionality() {
        let mut hierarchy = HierarchyComponent::new();
        
        // Initially empty
        assert!(!hierarchy.has_parent());
        assert!(!hierarchy.has_children());
        assert_eq!(hierarchy.child_count(), 0);
        
        // Add children
        hierarchy.add_child(10);
        hierarchy.add_child(20);
        hierarchy.add_child(30);
        
        assert!(hierarchy.has_children());
        assert_eq!(hierarchy.child_count(), 3);
        assert!(hierarchy.is_child(10));
        assert!(hierarchy.is_child(20));
        assert!(hierarchy.is_child(30));
        assert!(!hierarchy.is_child(999));
        
        // Set parent
        hierarchy.set_parent(Some(5));
        assert!(hierarchy.has_parent());
        assert_eq!(hierarchy.parent(), Some(5));
        assert!(hierarchy.is_parent(5));
        
        // Remove child
        assert!(hierarchy.remove_child(20));
        assert_eq!(hierarchy.child_count(), 2);
        assert!(!hierarchy.is_child(20));
        
        // Clear all children
        hierarchy.clear_children();
        assert!(!hierarchy.has_children());
        assert_eq!(hierarchy.child_count(), 0);
    }

    #[test]
    fn test_hierarchy_validation() {
        // Valid hierarchy - no circular reference
        let mut hierarchy = HierarchyComponent::new();
        hierarchy.set_parent(Some(100));
        hierarchy.add_child(200);
        hierarchy.add_child(300);
        assert!(hierarchy.validate());

        // Invalid hierarchy - parent is also a child
        let mut invalid_hierarchy = HierarchyComponent::new();
        invalid_hierarchy.set_parent(Some(100));
        invalid_hierarchy.add_child(100); // Circular reference!
        assert!(!invalid_hierarchy.validate());
    }

    #[test]
    fn test_multi_level_hierarchy() {
        let mut world = World::new();

        // Create a 3-level hierarchy: Grandparent -> Parent -> Child
        let grandparent = world.create_entity();
        let parent = world.create_entity();
        let child = world.create_entity();

        // Grandparent at origin
        world.add_component(grandparent, HierarchyComponent::new());
        world.add_component(grandparent, Transform2dComponent::from_translation(Vector2d::new(0.0, 0.0)));

        // Parent offset from grandparent
        let mut parent_hierarchy = HierarchyComponent::with_parent(grandparent);
        parent_hierarchy.add_child(child);
        world.add_component(parent, parent_hierarchy);
        world.add_component(parent, Transform2dComponent::from_translation(Vector2d::new(50.0, 50.0)));

        // Child offset from parent
        world.add_component(child, HierarchyComponent::with_parent(parent));
        world.add_component(child, Transform2dComponent::from_translation(Vector2d::new(25.0, 25.0)));

        // Update grandparent to include parent as child
        let mut grandparent_hierarchy = world.get_component_mut::<HierarchyComponent>(grandparent).unwrap();
        grandparent_hierarchy.add_child(parent);

        // Verify the structure
        let grandparent_hier = world.get_component::<HierarchyComponent>(grandparent).unwrap();
        assert!(!grandparent_hier.has_parent());
        assert!(grandparent_hier.has_children());
        assert_eq!(grandparent_hier.child_count(), 1);
        assert!(grandparent_hier.is_child(parent));

        let parent_hier = world.get_component::<HierarchyComponent>(parent).unwrap();
        assert!(parent_hier.has_parent());
        assert!(parent_hier.has_children());
        assert_eq!(parent_hier.parent(), Some(grandparent));
        assert!(parent_hier.is_child(child));

        let child_hier = world.get_component::<HierarchyComponent>(child).unwrap();
        assert!(child_hier.has_parent());
        assert!(!child_hier.has_children());
        assert_eq!(child_hier.parent(), Some(parent));
    }

    #[test]
    fn test_hierarchy_with_transform_propagation_calculation() {
        // Test the transform calculation logic without full system integration
        use crate::core::hierarchy_system::HierarchySystem;
        use crate::core::math::transform2d::Transform2d;

        // Parent transform: translate by (100, 100) and rotate by 90 degrees
        let parent_transform = Transform2d::from_trs(
            Vector2d::new(100.0, 100.0),
            Angle2d::from_degrees(90.0),
            1.0
        );

        // Child local transform: translate by (10, 0) relative to parent
        let child_local_transform = Transform2d::translation(Vector2d::new(10.0, 0.0));

        // Calculate world transform
        let child_world_transform = HierarchySystem::calculate_world_transform(
            child_local_transform,
            Some(parent_transform)
        );

        // After rotation and translation, the child should be at approximately (100, 110)
        let world_position = child_world_transform.get_translation();
        assert!((world_position.x - 100.0).abs() < 0.1, "Expected X ~= 100, got {}", world_position.x);
        assert!((world_position.y - 110.0).abs() < 0.1, "Expected Y ~= 110, got {}", world_position.y);
    }

    #[test]
    fn test_hierarchy_component_cloning() {
        let mut original = HierarchyComponent::new();
        original.set_parent(Some(42));
        original.add_child(1);
        original.add_child(2);
        original.add_child(3);

        let cloned = original.clone();
        
        assert_eq!(original.parent(), cloned.parent());
        assert_eq!(original.children(), cloned.children());
        assert_eq!(original.child_count(), cloned.child_count());
    }
}