#[cfg(test)]
mod tests {
    use crate::ecs::{World, Component, Mut};
    use crate::core::time::TimeComponent;
    use std::any::Any;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone)]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {
        fn validate(&self) -> bool {
            self.x.is_finite() && self.y.is_finite()
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

    #[derive(Debug, Clone)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    impl Component for Velocity {
        fn validate(&self) -> bool {
            self.dx.is_finite() && self.dy.is_finite()
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

    // Test structure to track system execution order
    lazy_static::lazy_static! {
        static ref EXECUTION_ORDER: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    }

    fn clear_execution_order() {
        EXECUTION_ORDER.lock().unwrap().clear();
    }

    fn record_execution(system_name: &str) {
        EXECUTION_ORDER.lock().unwrap().push(system_name.to_string());
    }

    fn get_execution_order() -> Vec<String> {
        EXECUTION_ORDER.lock().unwrap().clone()
    }

    #[test]
    fn test_system_dependencies_basic_ordering() {
        let mut world = World::new();
        clear_execution_order();

        // Create entities for testing (not used in systems, just for structure)
        let entity = world.create_entity();
        world.add_component(entity, Position { x: 0.0, y: 0.0 });

        // Time system (no dependencies)
        world.add_single_iterator_system(
            |_pos_iter: crate::ecs::EntityIterator<Position, Position>| {
                record_execution("time_system");
            },
            "time_system"
        );

        // Movement system (depends on time_system)
        world.add_single_iterator_system_with_dependencies(
            |_pos_iter: crate::ecs::EntityIterator<Position, Position>| {
                record_execution("movement_system");
            },
            "movement_system",
            vec!["time_system"]
        );

        // Run systems
        world.run_systems();

        // Check execution order
        let execution_order = get_execution_order();
        assert_eq!(execution_order.len(), 2);
        assert_eq!(execution_order[0], "time_system");
        assert_eq!(execution_order[1], "movement_system");
    }

    #[test]
    fn test_system_dependencies_complex_chain() {
        let mut world = World::new();
        clear_execution_order();

        // Create entities
        let time_entity = world.create_entity();
        world.add_component(time_entity, TimeComponent::new());

        let entity = world.create_entity();
        world.add_component(entity, Position { x: 0.0, y: 0.0 });
        world.add_component(entity, Velocity { dx: 1.0, dy: 1.0 });

        // System A (no dependencies)
        world.add_single_iterator_system(
            |_time_iter: crate::ecs::EntityIterator<TimeComponent, TimeComponent>| {
                record_execution("system_a");
            },
            "system_a"
        );

        // System B (depends on A)
        world.add_single_iterator_system_with_dependencies(
            |_time_iter: crate::ecs::EntityIterator<TimeComponent, TimeComponent>| {
                record_execution("system_b");
            },
            "system_b",
            vec!["system_a"]
        );

        // System C (depends on B)
        world.add_single_iterator_system_with_dependencies(
            |_time_iter: crate::ecs::EntityIterator<TimeComponent, TimeComponent>| {
                record_execution("system_c");
            },
            "system_c",
            vec!["system_b"]
        );

        // System D (depends on A and C)
        world.add_single_iterator_system_with_dependencies(
            |_time_iter: crate::ecs::EntityIterator<TimeComponent, TimeComponent>| {
                record_execution("system_d");
            },
            "system_d",
            vec!["system_a", "system_c"]
        );

        // Run systems
        world.run_systems();

        // Check execution order
        let execution_order = get_execution_order();
        assert_eq!(execution_order.len(), 4, "Expected 4 systems but got: {:?}", execution_order);
        
        // A must come before B, C, and D
        let a_index = execution_order.iter().position(|x| x == "system_a").unwrap();
        let b_index = execution_order.iter().position(|x| x == "system_b").unwrap();
        let c_index = execution_order.iter().position(|x| x == "system_c").unwrap();
        let d_index = execution_order.iter().position(|x| x == "system_d").unwrap();
        
        assert!(a_index < b_index);
        assert!(a_index < c_index);
        assert!(a_index < d_index);
        
        // B must come before C
        assert!(b_index < c_index);
        
        // C must come before D
        assert!(c_index < d_index);
    }

    #[test]
    #[should_panic(expected = "Circular dependency detected")]
    fn test_circular_dependency_detection() {
        let mut world = World::new();

        // System A depends on B
        world.add_single_iterator_system_with_dependencies(
            |_iter: crate::ecs::EntityIterator<TimeComponent, TimeComponent>| {},
            "system_a",
            vec!["system_b"]
        );

        // System B depends on A (circular dependency)
        world.add_single_iterator_system_with_dependencies(
            |_iter: crate::ecs::EntityIterator<TimeComponent, TimeComponent>| {},
            "system_b",
            vec!["system_a"]
        );

        // This should panic due to circular dependency
        world.run_systems();
    }

    #[test]
    #[should_panic(expected = "depends on 'nonexistent_system' which does not exist")]
    fn test_missing_dependency_detection() {
        let mut world = World::new();

        // System depends on a non-existent system
        world.add_single_iterator_system_with_dependencies(
            |_iter: crate::ecs::EntityIterator<TimeComponent, TimeComponent>| {},
            "system_a",
            vec!["nonexistent_system"]
        );

        // This should panic due to missing dependency
        world.run_systems();
    }

    #[test]
    fn test_systems_without_dependencies_run_normally() {
        let mut world = World::new();
        clear_execution_order();

        // Create entity
        let entity = world.create_entity();
        world.add_component(entity, Position { x: 0.0, y: 0.0 });

        // System without dependencies
        world.add_single_iterator_system(
            |_iter: crate::ecs::EntityIterator<Position, Position>| {
                record_execution("independent_system");
            },
            "independent_system"
        );

        // Run systems
        world.run_systems();

        // Check that system ran
        let execution_order = get_execution_order();
        assert_eq!(execution_order.len(), 1);
        assert_eq!(execution_order[0], "independent_system");
    }
}