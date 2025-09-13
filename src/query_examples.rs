use crate::ecs::World;
use crate::query::{EntityIterator, Mut};
use crate::examples::{Position, Velocity, Health};

/// Test the new query system with different component arities
pub fn test_variable_arity_queries() {
    println!("=== Testing Variable Arity Entity Queries ===\n");
    
    let world = create_test_world();
    
    // Test 0 components (iterate over all entities)
    println!("0 Components - All entities:");
    let iter: EntityIterator<()> = world.query();
    for entity in iter {
        println!("  Entity: {}", entity);
    }
    
    println!();
    
    // Test 1 component (immutable)
    println!("1 Component (immutable) - Position:");
    let iter: EntityIterator<Position> = world.query();
    for position in iter {
        println!("  Position: ({}, {})", position.x, position.y);
    }
    
    println!();
    
    // Test 1 component (mutable)
    println!("1 Component (mutable) - Velocity (applying damping):");
    let iter: EntityIterator<Mut<Velocity>> = world.query();
    for mut velocity in iter {
        println!("  Before: ({}, {})", velocity.dx, velocity.dy);
        velocity.dx *= 0.9;
        velocity.dy *= 0.9;
        println!("  After:  ({}, {})", velocity.dx, velocity.dy);
    }
    
    println!();
    
    // Test 2 components (mixed mutability)
    println!("2 Components (Position immutable, Health mutable):");
    let iter: EntityIterator<(Position, Mut<Health>)> = world.query();
    for (position, mut health) in iter {
        println!("  Entity at ({}, {}) with health {}/{}", 
                 position.x, position.y, health.current, health.max);
        
        // Apply position-based damage
        let damage = if position.x < 0.0 || position.y < 0.0 { 10 } else { 2 };
        health.damage(damage);
        println!("    After damage: {}/{}", health.current, health.max);
    }
    
    println!();
    
    // Test 3 components (all immutable)
    println!("3 Components (all immutable) - Position, Velocity, Health:");
    let iter: EntityIterator<(Position, Velocity, Health)> = world.query();
    for (position, velocity, health) in iter {
        println!("  Entity: pos=({}, {}), vel=({}, {}), hp={}/{}", 
                 position.x, position.y, velocity.dx, velocity.dy,
                 health.current, health.max);
    }
    
    println!();
    
    // Test 3 components (mixed mutability)
    println!("3 Components (mixed) - Position immutable, Velocity mutable, Health mutable:");
    let iter: EntityIterator<(Position, Mut<Velocity>, Mut<Health>)> = world.query();
    for (position, mut velocity, mut health) in iter {
        println!("  Processing entity at ({}, {})", position.x, position.y);
        
        // Apply physics-based changes
        velocity.dx += position.x * 0.01; // Gravity-like effect
        velocity.dy += position.y * 0.01;
        
        // Health regeneration
        health.heal(1);
        
        println!("    New velocity: ({}, {}), health: {}/{}",
                 velocity.dx, velocity.dy, health.current, health.max);
    }
}

/// Create a test world with entities having different component combinations
pub fn create_test_world() -> World {
    let mut world = World::new();
    
    // Entity 1: All components
    let e1 = world.create_entity();
    world.add_component(e1, Position::new(10.0, 15.0));
    world.add_component(e1, Velocity::new(1.0, -0.5));
    world.add_component(e1, Health::new(100));
    
    // Entity 2: Position and Velocity only
    let e2 = world.create_entity();
    world.add_component(e2, Position::new(5.0, 8.0));
    world.add_component(e2, Velocity::new(-2.0, 1.5));
    
    // Entity 3: Position and Health only
    let e3 = world.create_entity();
    world.add_component(e3, Position::new(-3.0, 12.0));
    world.add_component(e3, Health::new(75));
    
    // Entity 4: Only Position
    let e4 = world.create_entity();
    world.add_component(e4, Position::new(0.0, 0.0));
    
    // Entity 5: Only Health
    let e5 = world.create_entity();
    world.add_component(e5, Health::new(50));
    
    // Entity 6: No components (just an entity ID)
    let _e6 = world.create_entity();
    
    world
}

/// Demonstrate system functions using the new query API
pub fn system_with_variable_queries(world: &World) {
    println!("=== System Functions with Variable Queries ===\n");
    
    // System 1: Movement system (Position immutable, Velocity mutable)
    println!("Movement System:");
    movement_system_v2(world);
    
    println!();
    
    // System 2: Health regeneration system (Health mutable only)
    println!("Health Regeneration System:");
    health_regen_system(world);
    
    println!();
    
    // System 3: Cleanup system (all components immutable)
    println!("Entity Cleanup Check:");
    cleanup_check_system(world);
}

/// Movement system using the new query API
fn movement_system_v2(world: &World) {
    let iter: EntityIterator<(Position, Mut<Velocity>)> = world.query();
    for (position, mut velocity) in iter {
        println!("  Entity at ({}, {}) with velocity ({}, {})", 
                 position.x, position.y, velocity.dx, velocity.dy);
        
        // Apply damping and boundary effects
        velocity.dx *= 0.95;
        velocity.dy *= 0.95;
        
        // Boundary bouncing
        if position.x > 20.0 || position.x < -20.0 {
            velocity.dx *= -0.8;
        }
        if position.y > 20.0 || position.y < -20.0 {
            velocity.dy *= -0.8;
        }
        
        println!("    New velocity: ({}, {})", velocity.dx, velocity.dy);
    }
}

/// Health regeneration system (only Health component needed)
fn health_regen_system(world: &World) {
    let iter: EntityIterator<Mut<Health>> = world.query();
    for mut health in iter {
        let old_health = health.current;
        health.heal(2); // Regenerate 2 HP
        if old_health != health.current {
            println!("  Health regenerated: {} -> {}/{}", 
                     old_health, health.current, health.max);
        }
    }
}

/// Cleanup check system (checks entities that should be removed)
fn cleanup_check_system(world: &World) {
    let iter: EntityIterator<(Position, Velocity, Health)> = world.query();
    for (position, velocity, health) in iter {
        let should_cleanup = !health.is_alive() || 
                           (velocity.dx.abs() < 0.01 && velocity.dy.abs() < 0.01);
        
        if should_cleanup {
            println!("  Entity at ({}, {}) marked for cleanup (dead or static)", 
                     position.x, position.y);
        }
    }
}