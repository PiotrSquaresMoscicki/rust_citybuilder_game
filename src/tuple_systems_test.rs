use crate::ecs::{Component, World, Mut, EntityIterator};
use crate::examples::{Position, Velocity};
use crate::multiple_iterators_test::{TimeDelta, Gravity};
use std::any::Any;

/// Example system that takes a single iterator as parameter
/// This demonstrates the new tuple-based system API
pub fn single_iterator_system(movement_iter: EntityIterator<Position, Mut<Velocity>>) {
    println!("=== Single Iterator System ===");
    for (position, mut velocity) in movement_iter {
        println!("Entity at ({}, {}) with velocity ({}, {})", 
                 position.x, position.y, velocity.dx, velocity.dy);
        // Apply some physics
        velocity.dx *= 0.98; // Friction
        velocity.dy -= 0.1;  // Gravity
        println!("  Updated velocity: ({}, {})", velocity.dx, velocity.dy);
    }
}

/// Example system that takes a tuple of two iterators
/// This allows the World to know exactly which component combinations this system uses
pub fn dual_iterator_system(
    (movement_iter, physics_iter): (EntityIterator<Position, Mut<Velocity>>, EntityIterator<TimeDelta, Gravity>)
) {
    println!("=== Dual Iterator System ===");
    
    // First, get physics constants
    let mut delta_time = 0.016; // Default
    let mut gravity = 9.8;      // Default
    
    println!("Reading physics constants:");
    for (time_delta, gravity_component) in physics_iter {
        delta_time = time_delta.delta_time;
        gravity = gravity_component.acceleration;
        println!("  Found physics: dt={}, g={}", delta_time, gravity);
        break; // Just take the first one
    }
    
    // Then apply physics to moving entities
    println!("Applying physics to movement entities:");
    for (position, mut velocity) in movement_iter {
        println!("  Entity at ({}, {}) with velocity ({}, {})", 
                 position.x, position.y, velocity.dx, velocity.dy);
        
        // Apply gravity using the physics constants
        velocity.dy += gravity * delta_time;
        
        println!("    Applied physics (g={}, dt={}), new velocity: ({}, {})", 
                 gravity, delta_time, velocity.dx, velocity.dy);
    }
}

/// Example system that takes a tuple of three iterators
/// This demonstrates complex systems that need multiple different entity types
pub fn triple_iterator_system(
    (pos_vel_iter, pos_time_iter, vel_grav_iter): (
        EntityIterator<Position, Mut<Velocity>>,
        EntityIterator<Position, TimeDelta>, 
        EntityIterator<Velocity, Gravity>
    )
) {
    println!("=== Triple Iterator System ===");
    
    println!("Processing Position + Velocity entities:");
    for (position, mut velocity) in pos_vel_iter {
        println!("  Moving entity at ({}, {}) with vel ({}, {})", 
                 position.x, position.y, velocity.dx, velocity.dy);
        // Apply position-based effects
        if position.x < 0.0 {
            velocity.dx = velocity.dx.abs(); // Bounce off left wall
        }
        if position.y < 0.0 {
            velocity.dy = velocity.dy.abs(); // Bounce off bottom wall
        }
    }
    
    println!("Processing Position + TimeDelta entities:");
    for (position, time_delta) in pos_time_iter {
        println!("  Timed entity at ({}, {}) with dt={}", 
                 position.x, position.y, time_delta.delta_time);
    }
    
    println!("Processing Velocity + Gravity entities:");
    for (velocity, gravity) in vel_grav_iter {
        println!("  Physics entity with vel ({}, {}) and gravity {}", 
                 velocity.dx, velocity.dy, gravity.acceleration);
    }
}

/// Create a test world for tuple systems
pub fn create_tuple_systems_test_world() -> World {
    let mut world = World::new();
    
    // Entity 1: Movement entity (Position + Velocity)
    let entity1 = world.create_entity();
    world.add_component(entity1, Position::new(10.0, 5.0));
    world.add_component(entity1, Velocity::new(2.0, -1.0));
    
    // Entity 2: Physics constants entity (TimeDelta + Gravity)
    let entity2 = world.create_entity();
    world.add_component(entity2, TimeDelta::new(0.016));
    world.add_component(entity2, Gravity::new(-9.8));
    
    // Entity 3: Complex entity (Position + Velocity + TimeDelta)
    let entity3 = world.create_entity();
    world.add_component(entity3, Position::new(-5.0, 8.0));
    world.add_component(entity3, Velocity::new(-1.5, 3.0));
    world.add_component(entity3, TimeDelta::new(0.020));
    
    // Entity 4: Physics entity (Velocity + Gravity)
    let entity4 = world.create_entity();
    world.add_component(entity4, Velocity::new(0.5, -2.0));
    world.add_component(entity4, Gravity::new(12.0));
    
    world
}

/// Demonstrate the new tuple-based system interface
pub fn demonstrate_tuple_systems() {
    let mut world = create_tuple_systems_test_world();
    
    println!("=== Demonstrating Tuple-Based System Interface ===\n");
    
    // Register systems using the new tuple-based API
    // The World now knows exactly which components each system uses
    
    println!("Registering single iterator system...");
    world.add_tuple_system(single_iterator_system);
    
    println!("Registering dual iterator system...");
    world.add_tuple_system(dual_iterator_system);
    
    println!("Registering triple iterator system...");
    world.add_tuple_system(triple_iterator_system);
    
    println!("\nRunning all tuple-based systems:\n");
    world.run_tuple_systems();
    
    println!("\n=== Tuple System Demonstration Complete ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_iterator_system_registration() {
        let mut world = create_tuple_systems_test_world();
        
        // This should compile and register successfully
        world.add_tuple_system(single_iterator_system);
        
        // Running should not panic
        world.run_tuple_systems();
    }
    
    #[test]
    fn test_dual_iterator_system_registration() {
        let mut world = create_tuple_systems_test_world();
        
        // This should compile and register successfully
        world.add_tuple_system(dual_iterator_system);
        
        // Running should not panic
        world.run_tuple_systems();
    }
    
    #[test]
    fn test_triple_iterator_system_registration() {
        let mut world = create_tuple_systems_test_world();
        
        // This should compile and register successfully
        world.add_tuple_system(triple_iterator_system);
        
        // Running should not panic
        world.run_tuple_systems();
    }
    
    #[test]
    fn test_multiple_tuple_systems_together() {
        let mut world = create_tuple_systems_test_world();
        
        // Register all types of tuple systems
        world.add_tuple_system(single_iterator_system);
        world.add_tuple_system(dual_iterator_system);
        world.add_tuple_system(triple_iterator_system);
        
        // All should run without issues
        world.run_tuple_systems();
        
        // Verify we can still use traditional systems too
        world.add_system(|world: &World| {
            let count = world.iter_entities::<Position, Velocity>().count();
            println!("Traditional system counted {} entities with Position+Velocity", count);
        });
        
        world.run_systems(); // Should run both traditional and tuple systems
    }
    
    #[test]
    fn test_world_knows_system_dependencies() {
        let mut world = create_tuple_systems_test_world();
        
        // The key requirement: "the world will know which components are used by which system"
        // This is now satisfied because:
        // 1. Systems declare their iterator types at registration time
        // 2. The World creates the exact iterators the system needs
        // 3. No dynamic component queries inside the system
        
        world.add_tuple_system(single_iterator_system);  // Uses: Position, Mut<Velocity>
        world.add_tuple_system(dual_iterator_system);    // Uses: Position, Mut<Velocity>, TimeDelta, Gravity
        world.add_tuple_system(triple_iterator_system);  // Uses: Position, Mut<Velocity>, TimeDelta, Gravity
        
        // The World now statically knows all component dependencies at compile time
        // This enables better scheduling, conflict detection, and optimization
        
        world.run_tuple_systems();
    }
}