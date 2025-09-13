#[cfg(test)]
mod tests {
    use crate::diffing::{Diffable, PropertyDiff, DebugTracker};
    use crate::examples::{Position, Velocity, Health};
    use crate::ecs::{World, Mut};

    #[test]
    fn test_basic_diffable_implementations() {
        // Test i32 diffing
        let a = 5i32;
        let b = 10i32;
        let diff = a.diff(&b);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].property_name, "value");
        assert_eq!(diff[0].new_value, "10");

        // Test no change
        let diff = a.diff(&a);
        assert!(diff.is_none());

        // Test f32 diffing
        let a = 1.5f32;
        let b = 2.7f32;
        let diff = a.diff(&b);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff[0].property_name, "value");
        assert_eq!(diff[0].new_value, "2.7");

        // Test String diffing
        let a = "hello".to_string();
        let b = "world".to_string();
        let diff = a.diff(&b);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff[0].property_name, "value");
        assert_eq!(diff[0].new_value, "\"world\"");
    }

    #[test]
    fn test_component_diffing() {
        // Test Position diffing
        let pos1 = Position::new(1.0, 2.0);
        let pos2 = Position::new(3.0, 2.0);
        let diff = pos1.diff(&pos2);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff.len(), 1); // Only x changed
        assert_eq!(diff[0].property_name, "x");
        assert_eq!(diff[0].new_value, "3.0");

        // Test Velocity diffing
        let vel1 = Velocity::new(1.0, 2.0);
        let vel2 = Velocity::new(1.0, 5.0);
        let diff = vel1.diff(&vel2);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff.len(), 1); // Only dy changed
        assert_eq!(diff[0].property_name, "dy");
        assert_eq!(diff[0].new_value, "5.0");

        // Test Health diffing
        let health1 = Health::new(100);
        let mut health2 = Health::new(100);
        health2.damage(30);
        let diff = health1.diff(&health2);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff.len(), 1); // Only current changed
        assert_eq!(diff[0].property_name, "current");
        assert_eq!(diff[0].new_value, "70");
    }

    #[test]
    fn test_vec_diffing() {
        let vec1 = vec![1i32, 2i32, 3i32];
        let vec2 = vec![1i32, 5i32, 3i32];
        let diff = vec1.diff(&vec2);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff.len(), 1); // Only index 1 changed
        assert_eq!(diff[0].property_name, "[1].value");
        assert_eq!(diff[0].new_value, "5");

        // Test size change
        let vec3 = vec![1i32, 2i32];
        let diff = vec1.diff(&vec3);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].property_name, "value");
        assert!(diff[0].new_value.contains("Vec with 2 elements"));
    }

    #[test]
    fn test_debug_tracker_basic() {
        let mut tracker = DebugTracker::new();
        assert!(!tracker.enabled);
        
        tracker.enable();
        assert!(tracker.enabled);
        
        tracker.disable();
        assert!(!tracker.enabled);
        
        tracker.enable();
        tracker.next_frame();
        assert_eq!(tracker.frame_number, 1);
    }

    #[test]
    fn test_world_debug_integration() {
        let mut world = World::new();
        
        // Test debug methods exist and work
        world.enable_debug_tracking();
        world.next_frame();
        
        let history = world.get_debug_history();
        assert!(history.is_empty()); // No systems run yet
        
        world.clear_debug_history();
        world.disable_debug_tracking();
    }

    #[test] 
    fn test_component_snapshot() {
        let mut world = World::new();
        let entity = world.create_entity();
        let position = Position::new(10.0, 20.0);
        world.add_component(entity, position);
        
        // Test that we can get a component snapshot
        let type_id = std::any::TypeId::of::<Position>();
        let snapshot = world.get_component_snapshot(entity, type_id);
        assert!(snapshot.is_some());
    }

    #[test]
    fn test_diffable_macro_with_no_changes() {
        let pos1 = Position::new(5.0, 10.0);
        let pos2 = Position::new(5.0, 10.0);
        let diff = pos1.diff(&pos2);
        assert!(diff.is_none());
    }

    #[test]
    fn test_diffable_macro_with_multiple_changes() {
        let pos1 = Position::new(1.0, 2.0);
        let pos2 = Position::new(3.0, 4.0);
        let diff = pos1.diff(&pos2);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff.len(), 2); // Both x and y changed
        
        // Check that both properties are in the diff
        let prop_names: Vec<&String> = diff.iter().map(|d| &d.property_name).collect();
        assert!(prop_names.contains(&&"x".to_string()));
        assert!(prop_names.contains(&&"y".to_string()));
    }

    #[test]
    fn test_type_names() {
        assert_eq!(Position::type_name(), "Position");
        assert_eq!(Velocity::type_name(), "Velocity");
        assert_eq!(Health::type_name(), "Health");
        assert_eq!(i32::type_name(), "i32");
        assert_eq!(f32::type_name(), "f32");
        assert_eq!(String::type_name(), "String");
    }
}