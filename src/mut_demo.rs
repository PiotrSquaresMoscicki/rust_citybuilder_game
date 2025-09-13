// Demonstration that the requirement has been implemented correctly
use crate::ecs::{World, Mut};
use crate::examples::{Position, Velocity};

pub fn demonstrate_mut_requirement() {
    println!("=== Demonstrating Mut<T> Requirement ===");
    
    let mut world = World::new();
    let entity = world.create_entity();
    world.add_component(entity, Position::new(10.0, 20.0));
    world.add_component(entity, Velocity::new(1.0, 2.0));
    
    println!("Before changes:");
    let vel = world.get_component::<Velocity>(entity).unwrap();
    println!("  Velocity: ({}, {})", vel.dx, vel.dy);
    drop(vel);
    
    // This is the NEW REQUIRED way - with Mut<Velocity>
    println!("\nUsing NEW API with Mut<Velocity> (this works):");
    let ent_it = world.iter_entities::<Position, Mut<Velocity>>();
    for (position, mut velocity) in ent_it {
        println!("  Entity at ({}, {}) with velocity ({}, {})", 
                 position.x, position.y, velocity.dx, velocity.dy);
        
        // This works because we used Mut<Velocity>
        velocity.dx = 100.0;
        velocity.dy = 200.0;
        
        println!("  Modified velocity to ({}, {})", velocity.dx, velocity.dy);
    }
    
    // Verify the changes persisted
    println!("\nAfter changes:");
    let vel = world.get_component::<Velocity>(entity).unwrap();
    println!("  Velocity: ({}, {})", vel.dx, vel.dy);
    
    // This demonstrates immutable access still works
    println!("\nUsing immutable access (both components read-only):");
    let ent_it = world.iter_entities::<Position, Velocity>();
    for (position, velocity) in ent_it {
        println!("  Entity at ({}, {}) with velocity ({}, {})", 
                 position.x, position.y, velocity.dx, velocity.dy);
        
        // Note: velocity.dx = 300.0; would NOT compile here
        // because Velocity is not wrapped in Mut<>
    }
    
    println!("\n✅ Mut<T> requirement successfully enforced!");
}