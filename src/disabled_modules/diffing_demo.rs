use crate::ecs::{World, Mut};
use crate::examples::{Position, Velocity, Health};
use std::any::TypeId;
use ron::ser::{to_string_pretty, PrettyConfig};

/// Demonstration of the ECS diffing system
pub fn demonstrate_diffing_system() {
    println!("=== ECS Component State Diffing Demonstration ===\n");
    
    // Create a world with debug tracking enabled
    let mut world = World::new();
    world.enable_debug_tracking();
    world.next_frame();
    
    println!("Creating entities with components...");
    
    // Create some entities with components
    let entity1 = world.create_entity();
    world.add_component(entity1, Position::new(10.0, 20.0));
    world.add_component(entity1, Velocity::new(1.5, -0.5));
    world.add_component(entity1, Health::new(100));
    
    let entity2 = world.create_entity();
    world.add_component(entity2, Position::new(5.0, 15.0));
    world.add_component(entity2, Velocity::new(-1.0, 2.0));
    world.add_component(entity2, Health::new(75));
    
    let entity3 = world.create_entity();
    world.add_component(entity3, Position::new(-2.0, 8.0));
    world.add_component(entity3, Health::new(50));
    
    println!("Entities created. Now running systems with change tracking...\n");
    
    // Run movement system that modifies Velocity (mutable)
    println!("--- Running Movement System ---");
    world.run_system_with_debug(
        "movement_system",
        |world| {
            let ent_it = world.iter_entities::<Position, Mut<Velocity>>();
            for (position, mut velocity) in ent_it {
                println!("  Entity at ({}, {}) with velocity ({}, {})", 
                         position.x, position.y, velocity.dx, velocity.dy);
                
                // Apply damping to velocity (this should be tracked as a change)
                velocity.dx *= 0.9;
                velocity.dy *= 0.9;
                
                println!("    Applied damping, new velocity: ({}, {})", velocity.dx, velocity.dy);
            }
        },
        &[TypeId::of::<Velocity>()]
    );
    
    println!("\n--- Running Health Damage System ---");
    world.run_system_with_debug(
        "health_damage_system", 
        |world| {
            let ent_it = world.iter_entities::<Position, Mut<Health>>();
            for (position, mut health) in ent_it {
                // Entities take damage based on their position
                let damage = if position.x < 0.0 || position.y < 0.0 { 15 } else { 5 };
                
                println!("  Entity at ({}, {}) taking {} damage", position.x, position.y, damage);
                let old_health = health.current;
                health.damage(damage);
                
                println!("    Health: {} -> {} (max: {})", old_health, health.current, health.max);
            }
        },
        &[TypeId::of::<Health>()]
    );
    
    println!("\n--- Running Position Update System ---");
    world.run_system_with_debug(
        "position_update_system",
        |world| {
            let ent_it = world.iter_entities::<Mut<Position>, Velocity>();
            for (mut position, velocity) in ent_it {
                println!("  Updating position from ({}, {}) with velocity ({}, {})",
                         position.x, position.y, velocity.dx, velocity.dy);
                
                // Update position based on velocity
                position.x += velocity.dx;
                position.y += velocity.dy;
                
                println!("    New position: ({}, {})", position.x, position.y);
            }
        },
        &[TypeId::of::<Position>()]
    );
    
    // Move to next frame and run another system
    world.next_frame();
    
    println!("\n--- Running Combined System (Frame 2) ---");
    world.run_system_with_debug(
        "combined_system",
        |world| {
            let ent_it = world.iter_entities::<Mut<Health>, Mut<Velocity>>();
            for (mut health, mut velocity) in ent_it {
                // Heal entities and boost velocity if they're healthy
                if health.current > 50 {
                    health.heal(10);
                    velocity.dx *= 1.1;
                    velocity.dy *= 1.1;
                    println!("  Healthy entity: healed and boosted velocity");
                } else {
                    velocity.dx *= 0.8;
                    velocity.dy *= 0.8;
                    println!("  Injured entity: velocity reduced");
                }
            }
        },
        &[TypeId::of::<Health>(), TypeId::of::<Velocity>()]
    );
    
    // Display the complete diff history
    println!("\n=== COMPLETE DIFF HISTORY ===");
    let history = world.get_debug_history();
    if history.is_empty() {
        println!("No changes were tracked (debug tracking might be disabled or no mutable components changed)");
    } else {
        println!("{}", history);
    }
    
    // Demonstrate RON serialization of a diff record
    println!("=== RON Serialization Example ===");
    if let Some(last_record) = world.debug_tracker.diff_history.last() {
        match to_string_pretty(last_record, PrettyConfig::default()) {
            Ok(ron_output) => {
                println!("Last system diff record in RON format:");
                println!("{}", ron_output);
            }
            Err(e) => {
                println!("Failed to serialize to RON: {}", e);
            }
        }
    }
    
    println!("\n=== Diffing System Demonstration Complete ===");
}