#[cfg(test)]
mod tests {
    use crate::ecs::{World, Component, Mut};
    use crate::examples::{Position, Velocity, Health};

    #[test]
    fn dummy_test() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_entity_creation() {
        let mut world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();
        
        assert_eq!(entity1, 0);
        assert_eq!(entity2, 1);
    }

    #[test]
    fn test_component_addition_and_retrieval() {
        let mut world = World::new();
        let entity = world.create_entity();
        
        let position = Position::new(10.0, 20.0);
        world.add_component(entity, position);
        
        let retrieved = world.get_component::<Position>(entity);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.x, 10.0);
        assert_eq!(retrieved.y, 20.0);
    }

    #[test]
    fn test_mutable_component_access() {
        let mut world = World::new();
        let entity = world.create_entity();
        
        let velocity = Velocity::new(1.0, 2.0);
        world.add_component(entity, velocity);
        
        let mut retrieved = world.get_component_mut::<Velocity>(entity);
        assert!(retrieved.is_some());
        let mut retrieved = retrieved.unwrap();
        retrieved.dx = 5.0;
        retrieved.dy = 10.0;
        
        // Drop the mutable reference and check the change persisted
        drop(retrieved);
        let retrieved = world.get_component::<Velocity>(entity);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.dx, 5.0);
        assert_eq!(retrieved.dy, 10.0);
    }

    #[test]
    fn test_entity_iterator_api() {
        let mut world = World::new();
        
        // Create entities with different component combinations
        let entity1 = world.create_entity();
        world.add_component(entity1, Position::new(1.0, 2.0));
        world.add_component(entity1, Velocity::new(0.1, 0.2));
        
        let entity2 = world.create_entity();
        world.add_component(entity2, Position::new(3.0, 4.0));
        world.add_component(entity2, Velocity::new(0.3, 0.4));
        
        let entity3 = world.create_entity();
        world.add_component(entity3, Position::new(5.0, 6.0));
        // No velocity for entity3
        
        // Test the EntityIterator API with Mut<Velocity> for mutable access
        let mut count = 0;
        let ent_it = world.iter_entities::<Position, Mut<Velocity>>();
        for (position, mut velocity) in ent_it {
            count += 1;
            // Should only iterate over entities that have both components
            assert!(position.x > 0.0);
            assert!(position.y > 0.0);
            
            // Test mutable access
            velocity.dx += 1.0;
            velocity.dy += 1.0;
        }
        
        // Should have iterated over 2 entities (entity1 and entity2)
        assert_eq!(count, 2);
        
        // Verify the changes were applied
        let velocity1 = world.get_component::<Velocity>(entity1).unwrap();
        assert!((velocity1.dx - 1.1).abs() < f32::EPSILON);
        assert!((velocity1.dy - 1.2).abs() < f32::EPSILON);
    }

    #[test] 
    fn test_immutable_only_iterator() {
        let mut world = World::new();
        
        let entity1 = world.create_entity();
        world.add_component(entity1, Position::new(1.0, 2.0));
        world.add_component(entity1, Velocity::new(0.1, 0.2));
        
        // Test iteration with both components immutable
        let ent_it = world.iter_entities::<Position, Velocity>();
        for (position, velocity) in ent_it {
            // We can read from both components
            assert!(position.x > 0.0);
            assert!(velocity.dx > 0.0);
            
            // But we cannot modify velocity because it's not Mut<Velocity>
            // This would cause a compilation error:
            // velocity.dx += 1.0; // <- This would fail to compile
        }
    }

    #[test]
    fn test_mixed_mutability() {
        let mut world = World::new();
        
        let entity1 = world.create_entity();
        world.add_component(entity1, Position::new(1.0, 2.0));
        world.add_component(entity1, Velocity::new(0.1, 0.2));
        
        // Test iteration with Position immutable and Velocity mutable
        let ent_it = world.iter_entities::<Position, Mut<Velocity>>();
        for (position, mut velocity) in ent_it {
            // Can read from position (immutable)
            assert!(position.x > 0.0);
            
            // Can modify velocity (mutable)
            velocity.dx = 99.0;
            velocity.dy = 88.0;
        }
        
        // Verify the changes
        let velocity = world.get_component::<Velocity>(entity1).unwrap();
        assert_eq!(velocity.dx, 99.0);
        assert_eq!(velocity.dy, 88.0);
    }

    #[test]
    fn test_component_validation() {
        let valid_position = Position::new(10.0, 20.0);
        assert!(valid_position.validate());
        
        let invalid_position = Position::new(f32::NAN, 20.0);
        assert!(!invalid_position.validate());
        
        let valid_health = Health::new(100);
        assert!(valid_health.validate());
        assert_eq!(valid_health.current, 100);
        assert_eq!(valid_health.max, 100);
    }

    #[test]
    fn test_health_component_methods() {
        let mut health = Health::new(100);
        
        assert!(health.is_alive());
        assert_eq!(health.health_percentage(), 1.0);
        
        health.damage(30);
        assert_eq!(health.current, 70);
        assert_eq!(health.health_percentage(), 0.7);
        
        health.heal(20);
        assert_eq!(health.current, 90);
        
        health.damage(200); // Over-damage
        assert_eq!(health.current, 0);
        assert!(!health.is_alive());
    }
}
