use crate::ecs::{System, SystemTypeId, World, EntityIterator, Mut, Component};
use crate::examples::{Position, Velocity};
use std::any::{Any, TypeId};

/// Example system using the new object-based approach
pub struct MovementSystem {
    name: String,
}

impl MovementSystem {
    pub fn new() -> Self {
        Self {
            name: "MovementSystem".to_string(),
        }
    }
}

impl System for MovementSystem {
    fn system_type_id(&self) -> SystemTypeId {
        "movement_system"
    }

    fn get_dependencies(&self) -> Vec<SystemTypeId> {
        vec![] // No dependencies
    }

    fn update(&self, world: &World) {
        let iter = world.iter_entities::<Position, Mut<Velocity>>();
        for (position, mut velocity) in iter {
            println!("Entity at ({}, {}) with velocity ({}, {})", 
                     position.x, position.y, velocity.dx, velocity.dy);
            
            // Demonstrate mutable access - apply some damping to velocity
            velocity.dx *= 0.99;
            velocity.dy *= 0.99;
            
            println!("  Applied damping, new velocity: ({}, {})", velocity.dx, velocity.dy);
        }
    }

    fn get_system_name(&self) -> &str {
        &self.name
    }

    fn get_mutable_component_types(&self) -> Vec<TypeId> {
        vec![TypeId::of::<Velocity>()]
    }
}

/// Example system with dependencies
pub struct PhysicsSystem {
    name: String,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            name: "PhysicsSystem".to_string(),
        }
    }
}

impl System for PhysicsSystem {
    fn system_type_id(&self) -> SystemTypeId {
        "physics_system"
    }

    fn get_dependencies(&self) -> Vec<SystemTypeId> {
        vec!["movement_system"] // Depends on movement system
    }

    fn update(&self, world: &World) {
        let iter = world.iter_entities::<Mut<Position>, Velocity>();
        for (mut position, velocity) in iter {
            println!("Updating position: ({}, {}) + velocity ({}, {})", 
                     position.x, position.y, velocity.dx, velocity.dy);
            
            // Apply velocity to position
            position.x += velocity.dx;
            position.y += velocity.dy;
            
            println!("  New position: ({}, {})", position.x, position.y);
        }
    }

    fn get_system_name(&self) -> &str {
        &self.name
    }

    fn get_mutable_component_types(&self) -> Vec<TypeId> {
        vec![TypeId::of::<Position>()]
    }
}

/// Demonstration function for the new system approach
pub fn demonstrate_system_objects() {
    use crate::ecs::World;
    use crate::examples::{Position, Velocity};

    println!("🎯 System Objects Demonstration");
    println!("===============================");

    let mut world = World::new();
    
    // Create entities with components
    let entity1 = world.create_entity();
    world.add_component(entity1, Position::new(10.0, 20.0));
    world.add_component(entity1, Velocity::new(1.5, -0.5));
    
    let entity2 = world.create_entity();
    world.add_component(entity2, Position::new(5.0, 8.0));
    world.add_component(entity2, Velocity::new(-2.0, 1.0));
    
    // Add systems using the new object-based approach
    println!("\nAdding MovementSystem (no dependencies)...");
    if let Err(e) = world.add_system_object(MovementSystem::new()) {
        println!("Error adding MovementSystem: {:?}", e);
    }
    
    println!("Adding PhysicsSystem (depends on MovementSystem)...");
    if let Err(e) = world.add_system_object(PhysicsSystem::new()) {
        println!("Error adding PhysicsSystem: {:?}", e);
    }
    
    // Finalize dependency resolution
    if let Err(e) = world.finalize_systems() {
        println!("Error finalizing systems: {:?}", e);
    } else {
        println!("✅ Systems finalized successfully");
    }
    
    // Show execution order
    let execution_order = world.get_system_execution_order();
    println!("\n📋 System execution order: {:?}", execution_order);
    
    // Run systems
    println!("\n🔄 Running systems...");
    world.run_systems();
    
    println!("\nSystem objects demonstration complete!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;
    use crate::examples::{Position, Velocity};

    #[test]
    fn test_system_object_basic_functionality() {
        let mut world = World::new();
        
        // Create an entity with components
        let entity = world.create_entity();
        world.add_component(entity, Position::new(0.0, 0.0));
        world.add_component(entity, Velocity::new(1.0, 1.0));
        
        // Add movement system
        let movement_system = MovementSystem::new();
        assert_eq!(movement_system.system_type_id(), "movement_system");
        assert_eq!(movement_system.get_system_name(), "MovementSystem");
        assert!(movement_system.get_dependencies().is_empty());
        
        world.add_system_object(movement_system).unwrap();
        assert_eq!(world.system_count(), 1);
        
        // Run the system
        world.run_systems();
        
        // The system should have modified the velocity (damping)
        let velocity = world.get_component::<Velocity>(entity).unwrap();
        assert!(velocity.dx < 1.0); // Should be damped
        assert!(velocity.dy < 1.0);
    }

    #[test]
    fn test_system_dependencies() {
        let mut world = World::new();
        
        // Add systems with dependencies
        world.add_system_object(MovementSystem::new()).unwrap();
        world.add_system_object(PhysicsSystem::new()).unwrap();
        
        // Check dependencies
        let physics_deps = world.get_system_dependencies("PhysicsSystem");
        assert!(physics_deps.is_some());
        assert_eq!(physics_deps.unwrap(), vec!["movement_system"]);
        
        let movement_deps = world.get_system_dependencies("MovementSystem");
        assert!(movement_deps.is_some());
        assert!(movement_deps.unwrap().is_empty());
        
        // Check execution order
        world.finalize_systems().unwrap();
        let execution_order = world.get_system_execution_order();
        
        // MovementSystem should come before PhysicsSystem
        let movement_pos = execution_order.iter().position(|&name| name == "MovementSystem");
        let physics_pos = execution_order.iter().position(|&name| name == "PhysicsSystem");
        
        assert!(movement_pos.is_some());
        assert!(physics_pos.is_some());
        assert!(movement_pos.unwrap() < physics_pos.unwrap());
    }

    #[test]
    fn test_mixed_system_types() {
        let mut world = World::new();
        
        // Add both object-based and legacy systems
        world.add_system_object(MovementSystem::new()).unwrap();
        
        // Add a legacy function system
        world.add_system(|world: &World| {
            println!("Legacy system executing");
        });
        
        assert_eq!(world.system_count(), 1); // Object-based systems
        assert_eq!(world.legacy_system_count(), 1); // Legacy function systems
        
        // Both should run without errors
        world.run_systems();
    }
}