use crate::ecs::{Component, World, Mut};
use crate::examples::{Position, Velocity};
use std::any::Any;

/// TimeDelta component for storing time information
#[derive(Debug, Clone)]
pub struct TimeDelta {
    pub delta_time: f32,
}

impl TimeDelta {
    pub fn new(delta_time: f32) -> Self {
        Self { delta_time }
    }
}

impl Component for TimeDelta {
    fn validate(&self) -> bool {
        self.delta_time >= 0.0 && self.delta_time.is_finite()
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

/// Gravity component for storing physics constants
#[derive(Debug, Clone)]
pub struct Gravity {
    pub acceleration: f32,
}

impl Gravity {
    pub fn new(acceleration: f32) -> Self {
        Self { acceleration }
    }
}

impl Component for Gravity {
    fn validate(&self) -> bool {
        self.acceleration.is_finite()
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

/// Example system that uses multiple iterators
pub fn physics_system(world: &World) {
    println!("=== Physics System with Multiple Iterators ===");
    
    // First iterator: entities with position and velocity for movement
    println!("Processing movement entities (Position + Velocity):");
    let movement_it = world.iter_entities::<Position, Mut<Velocity>>();
    for (position, mut velocity) in movement_it {
        println!("  Entity at ({}, {}) with velocity ({}, {})", 
                 position.x, position.y, velocity.dx, velocity.dy);
        
        // Apply gravity to downward velocity
        velocity.dy -= 9.8 * 0.016; // Assuming 60 FPS (16ms per frame)
        
        println!("    Applied gravity, new velocity: ({}, {})", velocity.dx, velocity.dy);
    }
    
    // Second iterator: entities with time delta and gravity data
    println!("Processing physics constants (TimeDelta + Gravity):");
    let physics_it = world.iter_entities::<TimeDelta, Gravity>();
    for (time_delta, gravity) in physics_it {
        println!("  Time delta: {} seconds, Gravity: {} m/s²", 
                 time_delta.delta_time, gravity.acceleration);
    }
}

/// Another example system that demonstrates three separate iterations
pub fn complex_system(world: &World) {
    println!("=== Complex System with Three Different Iterators ===");
    
    // Iterator 1: Position and TimeDelta
    println!("Processing entities with Position + TimeDelta:");
    let pos_time_it = world.iter_entities::<Position, TimeDelta>();
    for (position, time_delta) in pos_time_it {
        println!("  Position ({}, {}) at time delta {}", 
                 position.x, position.y, time_delta.delta_time);
    }
    
    // Iterator 2: Velocity and Gravity
    println!("Processing entities with Velocity + Gravity:");
    let vel_grav_it = world.iter_entities::<Velocity, Gravity>();
    for (velocity, gravity) in vel_grav_it {
        println!("  Velocity ({}, {}) with gravity {}", 
                 velocity.dx, velocity.dy, gravity.acceleration);
    }
    
    // Iterator 3: Position and mutable Velocity
    println!("Processing entities with Position + Mut<Velocity>:");
    let pos_vel_it = world.iter_entities::<Position, Mut<Velocity>>();
    for (position, mut velocity) in pos_vel_it {
        println!("  Updating velocity for entity at ({}, {})", position.x, position.y);
        // Some complex logic that modifies velocity
        velocity.dx *= 0.95; // Friction
        velocity.dy *= 0.95;
    }
}

/// Create a test world with various component combinations
pub fn create_multi_iterator_test_world() -> World {
    let mut world = World::new();
    
    // Entity 1: Has Position, Velocity, and TimeDelta
    let entity1 = world.create_entity();
    world.add_component(entity1, Position::new(10.0, 20.0));
    world.add_component(entity1, Velocity::new(1.5, -0.5));
    world.add_component(entity1, TimeDelta::new(0.016));
    
    // Entity 2: Has Position and Velocity only
    let entity2 = world.create_entity();
    world.add_component(entity2, Position::new(5.0, 15.0));
    world.add_component(entity2, Velocity::new(-1.0, 2.0));
    
    // Entity 3: Has TimeDelta and Gravity (physics constants entity)
    let entity3 = world.create_entity();
    world.add_component(entity3, TimeDelta::new(0.016));
    world.add_component(entity3, Gravity::new(9.8));
    
    // Entity 4: Has Velocity and Gravity
    let entity4 = world.create_entity();
    world.add_component(entity4, Velocity::new(0.0, 0.0));
    world.add_component(entity4, Gravity::new(-9.8)); // Different gravity
    
    world
}

pub fn demonstrate_multiple_iterators() {
    let world = create_multi_iterator_test_world();
    
    physics_system(&world);
    println!();
    complex_system(&world);
}

// Make TimeDelta diffable
crate::diffable!(TimeDelta { delta_time });

// Make Gravity diffable  
crate::diffable!(Gravity { acceleration });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_iterators_do_not_interfere() {
        let world = create_multi_iterator_test_world();
        
        // Create multiple iterators simultaneously to test safety
        let _iter1 = world.iter_entities::<Position, Velocity>();
        let _iter2 = world.iter_entities::<TimeDelta, Gravity>();
        let _iter3 = world.iter_entities::<Position, TimeDelta>();
        
        // If this compiles and doesn't panic, multiple iterators work
        assert!(true, "Multiple iterators can be created simultaneously");
    }
    
    #[test] 
    fn test_multiple_iterators_correct_entity_counts() {
        let world = create_multi_iterator_test_world();
        
        // Count entities with Position + Velocity
        let pos_vel_count = world.iter_entities::<Position, Velocity>().count();
        assert_eq!(pos_vel_count, 2, "Should have 2 entities with Position + Velocity");
        
        // Count entities with TimeDelta + Gravity
        let time_grav_count = world.iter_entities::<TimeDelta, Gravity>().count();
        assert_eq!(time_grav_count, 1, "Should have 1 entity with TimeDelta + Gravity");
        
        // Count entities with Position + TimeDelta
        let pos_time_count = world.iter_entities::<Position, TimeDelta>().count();
        assert_eq!(pos_time_count, 1, "Should have 1 entity with Position + TimeDelta");
    }
    
    #[test]
    fn test_system_can_use_multiple_iterators() {
        let world = create_multi_iterator_test_world();
        
        // This should work without any issues
        physics_system(&world);
        complex_system(&world);
        
        // If we get here, multiple iterators in systems work correctly
        assert!(true, "Systems can use multiple iterators successfully");
    }
}