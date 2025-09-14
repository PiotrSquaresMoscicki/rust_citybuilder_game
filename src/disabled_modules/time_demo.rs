use crate::ecs::{World, EntityIterator, Mut};
use crate::core::time::{TimeComponent, initialize_time_manager, update_global_time_manager};
use crate::core::time_system::update_time_components_in_world;
use crate::core::math::{Vector2d, Transform2dComponent};

/// Velocity component for 2D movement
#[derive(Clone, Debug)]
pub struct Velocity {
    pub velocity: Vector2d,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            velocity: Vector2d::new(x, y),
        }
    }
}

impl crate::ecs::Component for Velocity {
    fn validate(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn crate::ecs::Component> {
        Box::new(self.clone())
    }
}

// Make Velocity diffable
crate::diffable!(Velocity { velocity });

/// Movement system that moves entities with velocity based on delta time
/// This demonstrates how systems should use time components to get delta time
pub fn movement_system(
    entity_iter: EntityIterator<Mut<Transform2dComponent>, Mut<Velocity>>,
    time_iter: EntityIterator<TimeComponent, TimeComponent>
) {
    // Get delta time from time component (expects only one time entity)
    let delta_time = if let Some((time_component, _)) = time_iter.into_iter().next() {
        time_component.scaled_delta_time() as f32 // Convert to f32 for math operations
    } else {
        0.0 // No time component available
    };

    // Move all entities with transform and velocity
    for (mut transform, velocity) in entity_iter {
        let movement = velocity.velocity * delta_time;
        let current_translation = transform.translation();
        transform.set_translation(current_translation + movement);
    }
}

/// Example showing how to set up and run a game loop with time management
pub fn run_time_demo() {
    // Initialize the global time manager
    initialize_time_manager();
    
    // Create a world
    let mut world = World::new();
    
    // Create a time entity (usually only one per world)
    let time_entity = world.create_entity();
    world.add_component(time_entity, TimeComponent::new());
    
    // Create some entities with transform and velocity
    for i in 0..3 {
        let entity = world.create_entity();
        let position = Vector2d::new(i as f32 * 10.0, i as f32 * 10.0);
        world.add_component(entity, Transform2dComponent::from_translation(position));
        world.add_component(entity, Velocity::new(50.0, 25.0)); // 50 units/sec right, 25 units/sec up
    }
    
    println!("Starting time demo with {} entities", world.entity_count());
    
    // Simulate several frames
    for frame in 0..5 {
        // Update global time manager (this would be called once per frame in your game loop)
        update_global_time_manager();
        
        // Update time components with current delta time
        update_time_components_in_world(&world);
        
        // Run movement system
        let transform_velocity_iter = world.iter_entities::<Mut<Transform2dComponent>, Mut<Velocity>>();
        let time_iter = world.iter_entities::<TimeComponent, TimeComponent>();
        movement_system(transform_velocity_iter, time_iter);
        
        // Print current state
        if let Some(time_comp) = world.get_component::<TimeComponent>(time_entity) {
            println!("Frame {}: delta_time={:.6}s, total_time={:.6}s, fps={:.1}", 
                     frame, time_comp.delta_time, time_comp.total_time, time_comp.fps());
        }
        
        // Print entity positions
        let entities_with_transform = world.entities_with_component::<Transform2dComponent>();
        for (i, entity) in entities_with_transform.into_iter().enumerate() {
            if let Some(transform) = world.get_component::<Transform2dComponent>(entity) {
                let pos = transform.translation();
                println!("  Entity {}: position=({:.2}, {:.2})", i, pos.x, pos.y);
            }
        }
        
        // Small delay to simulate frame time
        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_movement_system_with_time() {
        initialize_time_manager();
        
        let mut world = World::new();
        
        // Create time entity
        let time_entity = world.create_entity();
        world.add_component(time_entity, TimeComponent::new());
        
        // Create moving entity
        let entity = world.create_entity();
        world.add_component(entity, Transform2dComponent::from_translation(Vector2d::new(0.0, 0.0)));
        world.add_component(entity, Velocity::new(100.0, 50.0)); // 100 units/sec right, 50 units/sec up
        
        // Simulate time passing
        sleep(Duration::from_millis(10)); // Small but measurable time
        update_global_time_manager();
        update_time_components_in_world(&world);
        
        // Get initial position
        let initial_position = {
            let transform = world.get_component::<Transform2dComponent>(entity).unwrap();
            transform.translation()
        };
        
        // Run movement system
        let transform_velocity_iter = world.iter_entities::<Mut<Transform2dComponent>, Mut<Velocity>>();
        let time_iter = world.iter_entities::<TimeComponent, TimeComponent>();
        movement_system(transform_velocity_iter, time_iter);
        
        // Check that entity moved
        let final_position = {
            let transform = world.get_component::<Transform2dComponent>(entity).unwrap();
            transform.translation()
        };
        
        // Position should have changed
        assert!(final_position.x > initial_position.x);
        assert!(final_position.y > initial_position.y);
        
        // Movement should be proportional to delta time
        let delta_time = {
            let time_comp = world.get_component::<TimeComponent>(time_entity).unwrap();
            time_comp.delta_time as f32
        };
        
        let expected_x = initial_position.x + 100.0 * delta_time;
        let expected_y = initial_position.y + 50.0 * delta_time;
        
        assert!((final_position.x - expected_x).abs() < 0.001);
        assert!((final_position.y - expected_y).abs() < 0.001);
    }

    #[test]
    fn test_movement_system_with_paused_time() {
        initialize_time_manager();
        
        let mut world = World::new();
        
        // Create paused time entity
        let time_entity = world.create_entity();
        let mut time_comp = TimeComponent::new();
        time_comp.pause();
        world.add_component(time_entity, time_comp);
        
        // Create moving entity
        let entity = world.create_entity();
        world.add_component(entity, Transform2dComponent::from_translation(Vector2d::new(0.0, 0.0)));
        world.add_component(entity, Velocity::new(100.0, 50.0));
        
        sleep(Duration::from_millis(10));
        update_global_time_manager();
        update_time_components_in_world(&world);
        
        let initial_position = {
            let transform = world.get_component::<Transform2dComponent>(entity).unwrap();
            transform.translation()
        };
        
        // Run movement system
        let transform_velocity_iter = world.iter_entities::<Mut<Transform2dComponent>, Mut<Velocity>>();
        let time_iter = world.iter_entities::<TimeComponent, TimeComponent>();
        movement_system(transform_velocity_iter, time_iter);
        
        let final_position = {
            let transform = world.get_component::<Transform2dComponent>(entity).unwrap();
            transform.translation()
        };
        
        // Position should not have changed when time is paused
        assert_eq!(final_position.x, initial_position.x);
        assert_eq!(final_position.y, initial_position.y);
    }

    #[test]
    fn test_movement_system_with_time_scale() {
        initialize_time_manager();
        
        let mut world = World::new();
        
        // Create time entity with 2x time scale
        let time_entity = world.create_entity();
        let mut time_comp = TimeComponent::new();
        time_comp.set_time_scale(2.0);
        world.add_component(time_entity, time_comp);
        
        // Create moving entity
        let entity = world.create_entity();
        world.add_component(entity, Transform2dComponent::from_translation(Vector2d::new(0.0, 0.0)));
        world.add_component(entity, Velocity::new(100.0, 50.0));
        
        sleep(Duration::from_millis(10));
        update_global_time_manager();
        update_time_components_in_world(&world);
        
        let initial_position = {
            let transform = world.get_component::<Transform2dComponent>(entity).unwrap();
            transform.translation()
        };
        
        // Run movement system
        let transform_velocity_iter = world.iter_entities::<Mut<Transform2dComponent>, Mut<Velocity>>();
        let time_iter = world.iter_entities::<TimeComponent, TimeComponent>();
        movement_system(transform_velocity_iter, time_iter);
        
        let final_position = {
            let transform = world.get_component::<Transform2dComponent>(entity).unwrap();
            transform.translation()
        };
        
        // Movement should be 2x faster due to time scale
        let delta_time = {
            let time_comp = world.get_component::<TimeComponent>(time_entity).unwrap();
            time_comp.delta_time as f32
        };
        
        let expected_x = initial_position.x + 100.0 * delta_time * 2.0; // 2x time scale
        let expected_y = initial_position.y + 50.0 * delta_time * 2.0;
        
        assert!((final_position.x - expected_x).abs() < 0.001);
        assert!((final_position.y - expected_y).abs() < 0.001);
    }
}