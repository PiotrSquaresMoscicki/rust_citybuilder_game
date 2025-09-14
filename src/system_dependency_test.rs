#[cfg(test)]
mod tests {
    use crate::ecs::*;
    use crate::core::time::TimeComponent;
    use std::sync::{Arc, Mutex};
    use std::cell::RefCell;

    // Test components for dependency testing
    #[derive(Clone, Debug)]
    struct TestComponent {
        pub value: i32,
    }

    impl Component for TestComponent {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn clone_box(&self) -> Box<dyn Component> {
            Box::new(self.clone())
        }
    }

    #[derive(Clone, Debug)]
    struct VelocityComponent {
        pub x: f32,
        pub y: f32,
    }

    impl Component for VelocityComponent {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn clone_box(&self) -> Box<dyn Component> {
            Box::new(self.clone())
        }
    }

    #[derive(Clone, Debug)]
    struct PositionComponent {
        pub x: f32,
        pub y: f32,
    }

    impl Component for PositionComponent {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn clone_box(&self) -> Box<dyn Component> {
            Box::new(self.clone())
        }
    }

    // Test execution order tracking
    thread_local! {
        static EXECUTION_ORDER: RefCell<Vec<String>> = RefCell::new(Vec::new());
    }

    fn record_execution(system_name: &str) {
        EXECUTION_ORDER.with(|order| {
            order.borrow_mut().push(system_name.to_string());
        });
    }

    fn get_execution_order() -> Vec<String> {
        EXECUTION_ORDER.with(|order| order.borrow().clone())
    }

    fn clear_execution_order() {
        EXECUTION_ORDER.with(|order| order.borrow_mut().clear());
    }

    #[test]
    fn test_no_dependencies_execution() {
        let mut world = World::new();
        clear_execution_order();

        // Add system with no dependencies
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("test_system");
            },
            "test_system",
            "test_system",
            vec![]
        ).unwrap();

        world.run_systems();
        
        let order = get_execution_order();
        assert_eq!(order, vec!["test_system"]);
    }

    #[test]
    fn test_single_dependency() {
        let mut world = World::new();
        clear_execution_order();

        // Add time system (dependency)
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TimeComponent, TimeComponent>| {
                record_execution("time_system");
            },
            "time_system",
            "time_system",
            vec![]
        ).unwrap();

        // Add movement system that depends on time system
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<PositionComponent, VelocityComponent>| {
                record_execution("movement_system");
            },
            "movement_system",
            "movement_system",
            vec!["time_system"]
        ).unwrap();

        world.run_systems();
        
        let order = get_execution_order();
        assert_eq!(order, vec!["time_system", "movement_system"]);
    }

    #[test]
    fn test_multiple_dependencies_chain() {
        let mut world = World::new();
        clear_execution_order();

        // System A (no dependencies)
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("system_a");
            },
            "system_a",
            "system_a",
            vec![]
        ).unwrap();

        // System B depends on A
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("system_b");
            },
            "system_b",
            "system_b",
            vec!["system_a"]
        ).unwrap();

        // System C depends on B
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("system_c");
            },
            "system_c",
            "system_c",
            vec!["system_b"]
        ).unwrap();

        world.run_systems();
        
        let order = get_execution_order();
        assert_eq!(order, vec!["system_a", "system_b", "system_c"]);
    }

    #[test]
    fn test_diamond_dependency() {
        let mut world = World::new();
        clear_execution_order();

        // System A (root)
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("system_a");
            },
            "system_a",
            "system_a",
            vec![]
        ).unwrap();

        // System B depends on A
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("system_b");
            },
            "system_b",
            "system_b",
            vec!["system_a"]
        ).unwrap();

        // System C depends on A
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("system_c");
            },
            "system_c",
            "system_c",
            vec!["system_a"]
        ).unwrap();

        // System D depends on both B and C
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("system_d");
            },
            "system_d",
            "system_d",
            vec!["system_b", "system_c"]
        ).unwrap();

        world.run_systems();
        
        let order = get_execution_order();
        // A should be first, D should be last, B and C can be in either order
        assert_eq!(order[0], "system_a");
        assert_eq!(order[3], "system_d");
        assert!(order.contains(&"system_b".to_string()));
        assert!(order.contains(&"system_c".to_string()));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut world = World::new();

        // System A depends on B
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {},
            "system_a",
            "system_a",
            vec!["system_b"]
        ).unwrap();

        // System B depends on A (circular) - this should detect the circular dependency
        let result = world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {},
            "system_b",
            "system_b",
            vec!["system_a"]
        );

        assert!(result.is_err());
        if let Err(DependencyError::CircularDependency(cycle)) = result {
            assert!(cycle.contains(&"system_a"));
            assert!(cycle.contains(&"system_b"));
        } else {
            panic!("Expected circular dependency error, got: {:?}", result);
        }
    }

    #[test]
    fn test_unknown_dependency() {
        let mut world = World::new();

        // System depends on unknown system
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {},
            "system_a",
            "system_a",
            vec!["unknown_system"]
        ).unwrap(); // This should succeed due to forward references

        // Now try to finalize - this should fail due to unknown dependency
        let result = world.finalize_systems();

        assert!(result.is_err());
        if let Err(DependencyError::UnknownSystemDependency(dep)) = result {
            assert_eq!(dep, "unknown_system");
        } else {
            panic!("Expected unknown dependency error, got: {:?}", result);
        }
    }

    #[test]
    fn test_time_system_dependency_example() {
        let mut world = World::new();
        clear_execution_order();

        // Create entities with components
        let time_entity = world.create_entity();
        world.add_component(time_entity, TimeComponent::new());

        let moving_entity = world.create_entity();
        world.add_component(moving_entity, PositionComponent { x: 0.0, y: 0.0 });
        world.add_component(moving_entity, VelocityComponent { x: 1.0, y: 1.0 });

        // Add time system
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<Mut<TimeComponent>, Mut<TimeComponent>>| {
                record_execution("time_system");
            },
            "time_system",
            "time_system",
            vec![]
        ).unwrap();

        // Add movement system that depends on time system
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<Mut<PositionComponent>, VelocityComponent>| {
                record_execution("movement_system");
            },
            "movement_system",
            "movement_system",
            vec!["time_system"]
        ).unwrap();

        world.run_systems();
        
        let order = get_execution_order();
        assert_eq!(order, vec!["time_system", "movement_system"]);
        
        // Verify execution order via API
        let execution_order = world.get_system_execution_order();
        assert_eq!(execution_order, vec!["time_system", "movement_system"]);
    }

    #[test]
    fn test_mixed_systems_with_and_without_dependencies() {
        let mut world = World::new();
        clear_execution_order();

        // System without dependencies (should be added via old method)
        world.add_single_iterator_system(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("legacy_system");
            },
            "legacy_system"
        );

        // System with dependencies
        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("dependent_system");
            },
            "dependent_system",
            "dependent_system",
            vec![]
        ).unwrap();

        world.run_systems();
        
        let order = get_execution_order();
        // Both systems should execute (order may vary for systems without dependencies)
        assert_eq!(order.len(), 2);
        assert!(order.contains(&"legacy_system".to_string()));
        assert!(order.contains(&"dependent_system".to_string()));
    }

    #[test] 
    fn test_system_dependencies_query() {
        let mut world = World::new();

        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {},
            "time_system",
            "time_system",
            vec![]
        ).unwrap();

        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {},
            "input_system", 
            "input_system",
            vec![]
        ).unwrap();

        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {},
            "dependent_system",
            "dependent_system",
            vec!["time_system", "input_system"]
        ).unwrap();

        let deps = world.get_system_dependencies("dependent_system");
        assert!(deps.is_some());
        let deps = deps.unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"time_system"));
        assert!(deps.contains(&"input_system"));
    }

    #[test]
    fn test_complex_dependency_graph() {
        let mut world = World::new();
        clear_execution_order();

        // Build a complex dependency graph:
        // time_system (root)
        // input_system (root)
        // physics_system depends on time_system
        // movement_system depends on physics_system and time_system
        // rendering_system depends on movement_system
        // audio_system depends on input_system

        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("time_system");
            },
            "time_system",
            "time_system",
            vec![]
        ).unwrap();

        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("input_system");
            },
            "input_system",
            "input_system", 
            vec![]
        ).unwrap();

        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("physics_system");
            },
            "physics_system",
            "physics_system",
            vec!["time_system"]
        ).unwrap();

        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("movement_system");
            },
            "movement_system",
            "movement_system",
            vec!["physics_system", "time_system"]
        ).unwrap();

        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("rendering_system");
            },
            "rendering_system",
            "rendering_system",
            vec!["movement_system"]
        ).unwrap();

        world.add_single_iterator_system_with_dependencies(
            |_iter: EntityIterator<TestComponent, TestComponent>| {
                record_execution("audio_system");
            },
            "audio_system",
            "audio_system",
            vec!["input_system"]
        ).unwrap();

        world.run_systems();
        
        let order = get_execution_order();
        
        // Verify dependencies are respected
        let time_idx = order.iter().position(|s| s == "time_system").unwrap();
        let input_idx = order.iter().position(|s| s == "input_system").unwrap();
        let physics_idx = order.iter().position(|s| s == "physics_system").unwrap();
        let movement_idx = order.iter().position(|s| s == "movement_system").unwrap();
        let rendering_idx = order.iter().position(|s| s == "rendering_system").unwrap();
        let audio_idx = order.iter().position(|s| s == "audio_system").unwrap();

        // Check dependency constraints
        assert!(time_idx < physics_idx);
        assert!(physics_idx < movement_idx);
        assert!(time_idx < movement_idx);
        assert!(movement_idx < rendering_idx);
        assert!(input_idx < audio_idx);
    }
}