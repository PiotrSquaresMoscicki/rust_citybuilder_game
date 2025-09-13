use crate::ecs::{World, Mut};
use crate::examples::{Position, Velocity};
use crate::multiple_iterators_test::{TimeDelta, Gravity};

/// Test system that takes a single iterator
fn movement_system(entities: crate::ecs::EntityIterator<Position, Mut<Velocity>>) {
    println!("=== Movement System with Single Iterator ===");
    for (position, mut velocity) in entities {
        println!("  Updating entity at ({}, {}) with velocity ({}, {})", 
                 position.x, position.y, velocity.dx, velocity.dy);
        // Apply some movement logic
        velocity.dx *= 0.98; // Apply friction
        velocity.dy -= 0.1; // Apply gravity
    }
}

/// Test system that takes two iterators
fn physics_dual_system(
    movement_entities: crate::ecs::EntityIterator<Position, Mut<Velocity>>,
    physics_entities: crate::ecs::EntityIterator<TimeDelta, Gravity>
) {
    println!("=== Physics Dual System with Two Iterators ===");
    
    // First, gather physics constants
    let mut time_delta = 0.016; // Default
    let mut gravity = -9.8; // Default
    
    println!("Processing physics constants:");
    for (delta, grav) in physics_entities {
        println!("  Found physics constants: time={}, gravity={}", delta.delta_time, grav.acceleration);
        time_delta = delta.delta_time;
        gravity = grav.acceleration;
        break; // Use the first one found
    }
    
    // Then apply to movement entities
    println!("Applying physics to movement entities:");
    for (position, mut velocity) in movement_entities {
        println!("  Entity at ({}, {}) - applying physics", position.x, position.y);
        velocity.dy += gravity * time_delta;
        println!("    New velocity: ({}, {})", velocity.dx, velocity.dy);
    }
}

/// Test system that takes three iterators
fn complex_system(
    pos_vel_entities: crate::ecs::EntityIterator<Position, Velocity>,
    time_grav_entities: crate::ecs::EntityIterator<TimeDelta, Gravity>,
    pos_time_entities: crate::ecs::EntityIterator<Position, TimeDelta>
) {
    println!("=== Complex System with Three Iterators ===");
    
    println!("Processing Position+Velocity entities:");
    let pos_vel_count = pos_vel_entities.count();
    println!("  Found {} entities with Position+Velocity", pos_vel_count);
    
    println!("Processing TimeDelta+Gravity entities:");
    let time_grav_count = time_grav_entities.count();
    println!("  Found {} entities with TimeDelta+Gravity", time_grav_count);
    
    println!("Processing Position+TimeDelta entities:");
    let pos_time_count = pos_time_entities.count();
    println!("  Found {} entities with Position+TimeDelta", pos_time_count);
}

/// Create a test world for the new iterator system functionality
pub fn create_test_world() -> World {
    let mut world = World::new();
    
    // Entity 1: Movement entity (Position + Velocity)
    let entity1 = world.create_entity();
    world.add_component(entity1, Position::new(10.0, 20.0));
    world.add_component(entity1, Velocity::new(1.5, -0.5));
    
    // Entity 2: Movement entity with timing (Position + Velocity + TimeDelta)
    let entity2 = world.create_entity();
    world.add_component(entity2, Position::new(5.0, 15.0));
    world.add_component(entity2, Velocity::new(-1.0, 2.0));
    world.add_component(entity2, TimeDelta::new(0.016));
    
    // Entity 3: Physics constants (TimeDelta + Gravity)
    let entity3 = world.create_entity();
    world.add_component(entity3, TimeDelta::new(0.016));
    world.add_component(entity3, Gravity::new(-9.8));
    
    world
}

/// Demonstrate the new multiple iterator system functionality
pub fn demonstrate_multiple_iterator_systems() {
    let mut world = create_test_world();
    
    println!("=== Multiple Iterator Systems Demo ===\n");
    
    // Add systems using the new API
    world.add_single_iterator_system(movement_system, "MovementSystem");
    world.add_dual_iterator_system(physics_dual_system, "PhysicsDualSystem");
    world.add_triple_iterator_system(complex_system, "ComplexSystem");
    
    // Run all systems
    println!("Running all registered systems:\n");
    world.run_systems();
    
    println!("\n=== Running with Debug Tracking ===");
    world.enable_debug_tracking();
    world.run_iterator_systems_with_debug();
    
    let debug_history = world.get_debug_history();
    println!("Debug history:\n{}", debug_history);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_iterator_system() {
        let mut world = create_test_world();
        world.add_single_iterator_system(movement_system, "TestMovementSystem");
        
        // This should run without errors
        world.run_systems();
        
        // Check that the system was added
        assert_eq!(world.system_count(), 1);
    }
    
    #[test]
    fn test_dual_iterator_system() {
        let mut world = create_test_world();
        world.add_dual_iterator_system(physics_dual_system, "TestPhysicsSystem");
        
        // This should run without errors
        world.run_systems();
        
        // Check that the system was added
        assert_eq!(world.system_count(), 1);
    }
    
    #[test]
    fn test_triple_iterator_system() {
        let mut world = create_test_world();
        world.add_triple_iterator_system(complex_system, "TestComplexSystem");
        
        // This should run without errors
        world.run_systems();
        
        // Check that the system was added
        assert_eq!(world.system_count(), 1);
    }
    
    #[test]
    fn test_mixed_system_types() {
        let mut world = create_test_world();
        
        // Add different types of systems
        world.add_single_iterator_system(movement_system, "Movement");
        world.add_dual_iterator_system(physics_dual_system, "Physics");
        world.add_triple_iterator_system(complex_system, "Complex");
        
        // Also add a legacy system
        world.add_system(|world| {
            println!("Legacy system running");
        });
        
        // Should have 3 new systems and 1 legacy system
        assert_eq!(world.system_count(), 3);
        assert_eq!(world.legacy_system_count(), 1);
        
        // All should run without errors
        world.run_systems();
    }
    
    #[test]
    fn test_system_component_type_tracking() {
        let mut world = create_test_world();
        world.add_dual_iterator_system(physics_dual_system, "PhysicsSystem");
        
        // Enable debug tracking to test the component type detection
        world.enable_debug_tracking();
        world.run_iterator_systems_with_debug();
        
        // Should not panic and should track changes if any were made
        let history = world.get_debug_history();
        println!("Debug history: {}", history);
    }
}