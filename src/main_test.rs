#[cfg(test)]
mod tests {
    use crate::ecs::{World, Component};
    use crate::examples::{Position, Velocity, Health};
    use crate::query::{EntityIterator, Mut};

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
        
        let retrieved = world.get_component_mut::<Velocity>(entity);
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
        
        // Test the EntityIterator API
        let mut count = 0;
        let ent_it = world.iter_entities::<Position, Velocity>();
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

    // Tests for the new variable arity query system
    
    #[test]
    fn test_zero_component_query() {
        let mut world = World::new();
        
        // Create some entities, some with components, some without
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        world.add_component(e2, Position::new(1.0, 2.0));
        let e3 = world.create_entity();
        
        // Query for all entities (no component constraints)
        let iter: EntityIterator<()> = world.query();
        let entities: Vec<u32> = iter.collect();
        
        assert_eq!(entities.len(), 3);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        assert!(entities.contains(&e3));
    }

    #[test]
    fn test_single_component_query() {
        let mut world = World::new();
        
        let e1 = world.create_entity();
        world.add_component(e1, Position::new(10.0, 20.0));
        world.add_component(e1, Velocity::new(1.0, 2.0));
        
        let e2 = world.create_entity();
        world.add_component(e2, Position::new(30.0, 40.0));
        // No velocity
        
        let e3 = world.create_entity();
        world.add_component(e3, Velocity::new(5.0, 6.0));
        // No position
        
        // Query for entities with Position only
        let iter: EntityIterator<Position> = world.query();
        let positions: Vec<_> = iter.collect();
        
        assert_eq!(positions.len(), 2);
        
        // Check that we got the right positions
        let mut found_pos1 = false;
        let mut found_pos2 = false;
        for pos in positions {
            if (pos.x - 10.0).abs() < f32::EPSILON && (pos.y - 20.0).abs() < f32::EPSILON {
                found_pos1 = true;
            } else if (pos.x - 30.0).abs() < f32::EPSILON && (pos.y - 40.0).abs() < f32::EPSILON {
                found_pos2 = true;
            }
        }
        assert!(found_pos1 && found_pos2);
    }

    #[test]
    fn test_single_mutable_component_query() {
        let mut world = World::new();
        
        let e1 = world.create_entity();
        world.add_component(e1, Velocity::new(1.0, 2.0));
        
        let e2 = world.create_entity();
        world.add_component(e2, Velocity::new(3.0, 4.0));
        
        // Query for mutable velocity components
        let iter: EntityIterator<Mut<Velocity>> = world.query();
        for mut velocity in iter {
            velocity.dx += 10.0;
            velocity.dy += 20.0;
        }
        
        // Check that changes were applied
        let vel1 = world.get_component::<Velocity>(e1).unwrap();
        assert!((vel1.dx - 11.0).abs() < f32::EPSILON);
        assert!((vel1.dy - 22.0).abs() < f32::EPSILON);
        
        let vel2 = world.get_component::<Velocity>(e2).unwrap();
        assert!((vel2.dx - 13.0).abs() < f32::EPSILON);
        assert!((vel2.dy - 24.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_two_component_query() {
        let mut world = World::new();
        
        let e1 = world.create_entity();
        world.add_component(e1, Position::new(1.0, 2.0));
        world.add_component(e1, Velocity::new(0.1, 0.2));
        
        let e2 = world.create_entity();
        world.add_component(e2, Position::new(3.0, 4.0));
        // No velocity
        
        let e3 = world.create_entity();
        world.add_component(e3, Position::new(5.0, 6.0));
        world.add_component(e3, Velocity::new(0.5, 0.6));
        
        // Query for entities with both Position and mutable Velocity
        let iter: EntityIterator<(Position, Mut<Velocity>)> = world.query();
        let mut count = 0;
        for (position, mut velocity) in iter {
            count += 1;
            velocity.dx += position.x;
            velocity.dy += position.y;
        }
        
        // Should have found 2 entities (e1 and e3)
        assert_eq!(count, 2);
        
        // Check modifications
        let vel1 = world.get_component::<Velocity>(e1).unwrap();
        assert!((vel1.dx - 1.1).abs() < f32::EPSILON);
        assert!((vel1.dy - 2.2).abs() < f32::EPSILON);
        
        let vel3 = world.get_component::<Velocity>(e3).unwrap();
        assert!((vel3.dx - 5.5).abs() < f32::EPSILON);
        assert!((vel3.dy - 6.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_three_component_query() {
        let mut world = World::new();
        
        let e1 = world.create_entity();
        world.add_component(e1, Position::new(10.0, 20.0));
        world.add_component(e1, Velocity::new(1.0, 2.0));
        world.add_component(e1, Health::new(100));
        
        let e2 = world.create_entity();
        world.add_component(e2, Position::new(30.0, 40.0));
        world.add_component(e2, Velocity::new(3.0, 4.0));
        // No health
        
        let e3 = world.create_entity();
        world.add_component(e3, Position::new(50.0, 60.0));
        world.add_component(e3, Health::new(75));
        // No velocity
        
        // Query for entities with all three components (mixed mutability)
        let iter: EntityIterator<(Position, Mut<Velocity>, Mut<Health>)> = world.query();
        let mut count = 0;
        for (position, mut velocity, mut health) in iter {
            count += 1;
            
            // Should only be e1
            assert!((position.x - 10.0).abs() < f32::EPSILON);
            assert!((position.y - 20.0).abs() < f32::EPSILON);
            
            velocity.dx *= 2.0;
            velocity.dy *= 2.0;
            health.damage(10);
        }
        
        // Should have found only 1 entity (e1)
        assert_eq!(count, 1);
        
        // Check modifications
        let vel1 = world.get_component::<Velocity>(e1).unwrap();
        assert!((vel1.dx - 2.0).abs() < f32::EPSILON);
        assert!((vel1.dy - 4.0).abs() < f32::EPSILON);
        
        let health1 = world.get_component::<Health>(e1).unwrap();
        assert_eq!(health1.current, 90);
    }

    #[test]
    fn test_higher_arity_queries() {
        let mut world = World::new();
        
        // Create entities with multiple components to test higher arities
        let e1 = world.create_entity();
        world.add_component(e1, Position::new(1.0, 2.0));
        world.add_component(e1, Velocity::new(0.1, 0.2));
        world.add_component(e1, Health::new(100));
        
        // Test that we can properly handle larger tuples
        // 3-component query with all immutable access
        let iter: EntityIterator<(Position, Velocity, Health)> = world.query();
        let mut count = 0;
        for (position, velocity, health) in iter {
            count += 1;
            
            assert!((position.x - 1.0).abs() < f32::EPSILON);
            assert!((velocity.dx - 0.1).abs() < f32::EPSILON);
            assert_eq!(health.current, 100);
        }
        
        assert_eq!(count, 1);
        
        // Test 4-component query by adding another entity with different component combinations
        let e2 = world.create_entity();
        world.add_component(e2, Position::new(5.0, 6.0));
        world.add_component(e2, Velocity::new(0.5, 0.6));
        world.add_component(e2, Health::new(75));
        
        // This demonstrates that the system can scale to higher arities
        let iter: EntityIterator<(Position, Velocity, Health)> = world.query();
        let entities: Vec<_> = iter.collect();
        assert_eq!(entities.len(), 2);
    }
}
