use crate::ecs::*;
use crate::core::time::TimeComponent;

/// Example demonstrating the system dependency mechanism where a movement system
/// depends on a time system to ensure delta time is always up to date
/// 
/// This example shows:
/// 1. How to create systems with dependencies using system type IDs
/// 2. How the World ensures correct execution order based on dependencies
/// 3. How this prevents issues where delta time might be stale

#[derive(Clone, Debug)]
struct Position {
    pub x: f32,
    pub y: f32,
}

impl Component for Position {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug)]
struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Component for Velocity {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

/// Time system that updates all time components
/// This system has no dependencies - it runs first
fn time_system(_time_iter: EntityIterator<Mut<TimeComponent>, Mut<TimeComponent>>) {
    println!("Time system executed - delta time updated");
    // In a real implementation, this would update delta time from the global time manager
}

/// Movement system that applies velocity to position using delta time
/// This system depends on the time system to ensure delta time is current
fn movement_system(
    movement_iter: EntityIterator<Mut<Position>, Velocity>,
    time_iter: EntityIterator<TimeComponent, TimeComponent>
) {
    // Get delta time from time components
    let delta_time = if let Some((time_comp, _)) = time_iter.into_iter().next() {
        time_comp.scaled_delta_time() as f32
    } else {
        0.016 // fallback to 60fps if no time component
    };

    println!("Movement system executed with delta_time: {}", delta_time);

    // Apply velocity to position
    for (mut position, velocity) in movement_iter {
        position.x += velocity.x * delta_time;
        position.y += velocity.y * delta_time;
    }
}

/// Physics system that updates velocities (e.g., applies gravity, friction)
/// This system depends on the time system but not on movement
fn physics_system(
    physics_iter: EntityIterator<Mut<Velocity>, Mut<Velocity>>,
    time_iter: EntityIterator<TimeComponent, TimeComponent>
) {
    let delta_time = if let Some((time_comp, _)) = time_iter.into_iter().next() {
        time_comp.scaled_delta_time() as f32
    } else {
        0.016
    };

    println!("Physics system executed with delta_time: {}", delta_time);

    // Apply physics (gravity in this example)
    for (mut velocity, _) in physics_iter {
        velocity.y -= 9.8 * delta_time; // gravity
    }
}

/// Rendering system that depends on movement (needs updated positions)
fn rendering_system(render_iter: EntityIterator<Position, Position>) {
    println!("Rendering system executed - drawing entities at updated positions");
    
    for (position, _) in render_iter {
        println!("  Drawing entity at ({}, {})", position.x, position.y);
    }
}

pub fn demonstrate_system_dependencies() {
    let mut world = World::new();

    // Create entities with components
    let time_entity = world.create_entity();
    world.add_component(time_entity, TimeComponent::new());

    let player_entity = world.create_entity();
    world.add_component(player_entity, Position { x: 0.0, y: 100.0 });
    world.add_component(player_entity, Velocity { x: 5.0, y: 0.0 });

    let ball_entity = world.create_entity();
    world.add_component(ball_entity, Position { x: 50.0, y: 200.0 });
    world.add_component(ball_entity, Velocity { x: -2.0, y: 0.0 });

    println!("Setting up systems with dependencies...\n");

    // Add time system (no dependencies)
    world.add_single_iterator_system_with_dependencies(
        time_system,
        "time_system",
        "time_system",
        vec![] // No dependencies
    ).unwrap();

    // Add physics system (depends on time system)
    world.add_dual_iterator_system_with_dependencies(
        physics_system,
        "physics_system", 
        "physics_system",
        vec!["time_system"] // Depends on time system
    ).unwrap();

    // Add movement system (depends on time system and physics system)
    world.add_dual_iterator_system_with_dependencies(
        movement_system,
        "movement_system",
        "movement_system", 
        vec!["time_system", "physics_system"] // Depends on both time and physics
    ).unwrap();

    // Add rendering system (depends on movement system)
    world.add_single_iterator_system_with_dependencies(
        rendering_system,
        "rendering_system",
        "rendering_system",
        vec!["movement_system"] // Depends on movement system
    ).unwrap();

    println!("System execution order:");
    let execution_order = world.get_system_execution_order();
    for (i, system_name) in execution_order.iter().enumerate() {
        println!("{}. {}", i + 1, system_name);
    }

    println!("\nRunning systems for one frame:");
    world.run_systems();

    println!("\nDemonstration complete!");
    println!("\nNote: The dependency system ensures that:");
    println!("1. Time system runs first to update delta time");
    println!("2. Physics system runs after time (has access to current delta time)");
    println!("3. Movement system runs after both time and physics");
    println!("4. Rendering system runs last (after positions are updated)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_example_execution_order() {
        let mut world = World::new();

        // Add systems in different order than they should execute
        world.add_single_iterator_system_with_dependencies(
            rendering_system,
            "rendering_system",
            "rendering_system",
            vec!["movement_system"]
        ).unwrap();

        world.add_dual_iterator_system_with_dependencies(
            movement_system,
            "movement_system", 
            "movement_system",
            vec!["time_system", "physics_system"]
        ).unwrap();

        world.add_dual_iterator_system_with_dependencies(
            physics_system,
            "physics_system",
            "physics_system", 
            vec!["time_system"]
        ).unwrap();

        world.add_single_iterator_system_with_dependencies(
            time_system,
            "time_system",
            "time_system",
            vec![]
        ).unwrap();

        // Verify execution order is correct despite registration order
        let execution_order = world.get_system_execution_order();
        
        // Find indices
        let time_idx = execution_order.iter().position(|s| *s == "time_system").unwrap();
        let physics_idx = execution_order.iter().position(|s| *s == "physics_system").unwrap();
        let movement_idx = execution_order.iter().position(|s| *s == "movement_system").unwrap();
        let rendering_idx = execution_order.iter().position(|s| *s == "rendering_system").unwrap();

        // Verify dependencies are respected
        assert!(time_idx < physics_idx, "Time should run before physics");
        assert!(time_idx < movement_idx, "Time should run before movement");
        assert!(physics_idx < movement_idx, "Physics should run before movement");
        assert!(movement_idx < rendering_idx, "Movement should run before rendering");
    }

    #[test]
    fn test_circular_dependency_prevention() {
        let mut world = World::new();

        // Add first system
        world.add_single_iterator_system_with_dependencies(
            |_: EntityIterator<Position, Position>| {},
            "system_a",
            "system_a",
            vec!["system_b"]
        ).unwrap();

        // Try to add system that creates circular dependency
        let result = world.add_single_iterator_system_with_dependencies(
            |_: EntityIterator<Position, Position>| {},
            "system_b",
            "system_b",
            vec!["system_a"]
        );

        assert!(result.is_err());
    }
}