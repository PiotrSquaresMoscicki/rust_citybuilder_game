# Multiple Iterators in ECS Systems

This document explains how to use multiple iterators in ECS systems to operate on different entity types within a single system function.

## Problem Statement

Sometimes a system needs to operate on multiple different entity types. For example:
- A physics system might need to update entities with Position+Velocity components AND access entities containing physics constants like TimeDelta+Gravity
- A rendering system might process drawable entities AND access configuration entities
- A collision system might process collidable entities AND spatial partitioning entities

## Solution: Multiple Iterators in System Functions

The ECS library supports creating multiple iterators within a single system function by passing a reference to the World and creating different iterators for different component combinations.

## Basic Example

```rust
pub fn physics_system(world: &World) {
    // First iterator: entities with position and velocity for movement
    let movement_it = world.iter_entities::<Position, Mut<Velocity>>();
    for (position, mut velocity) in movement_it {
        // Update velocity based on position
        velocity.dy -= 9.8 * 0.016; // Apply gravity
    }
    
    // Second iterator: entities with time delta and gravity data
    let physics_it = world.iter_entities::<TimeDelta, Gravity>();
    for (time_delta, gravity) in physics_it {
        // Use physics constants for calculations
        println!("Physics constants: {} seconds, {} m/s²", 
                 time_delta.delta_time, gravity.acceleration);
    }
}
```

## Advanced Example with Three Iterators

```rust
pub fn complex_system(world: &World) {
    // Iterator 1: Position and TimeDelta
    let pos_time_it = world.iter_entities::<Position, TimeDelta>();
    for (position, time_delta) in pos_time_it {
        // Process entities that have timing information
    }
    
    // Iterator 2: Velocity and Gravity
    let vel_grav_it = world.iter_entities::<Velocity, Gravity>();
    for (velocity, gravity) in vel_grav_it {
        // Process velocity with gravity constants
    }
    
    // Iterator 3: Position and mutable Velocity
    let pos_vel_it = world.iter_entities::<Position, Mut<Velocity>>();
    for (position, mut velocity) in pos_vel_it {
        // Update velocity based on position
        velocity.dx *= 0.95; // Apply friction
    }
}
```

## Safety and Performance Considerations

### Multiple Iterator Safety
- Multiple iterators can be created simultaneously from the same World
- Each iterator maintains its own reference to the World
- Rust's borrow checker ensures safe access patterns
- Mutable access through `Mut<T>` follows standard RefCell rules

### Iteration Order
- Each iterator processes entities independently
- No guaranteed order between different iterators
- Entities are processed in the order they were added to component pools

### Performance Notes
- Creating multiple iterators has minimal overhead
- Each iterator only processes entities that have the required components
- Memory usage scales with the number of entities, not the number of iterators

## Entity Setup Examples

```rust
fn create_multi_iterator_world() -> World {
    let mut world = World::new();
    
    // Entity 1: Movement entity (Position + Velocity + TimeDelta)
    let movement_entity = world.create_entity();
    world.add_component(movement_entity, Position::new(10.0, 20.0));
    world.add_component(movement_entity, Velocity::new(1.5, -0.5));
    world.add_component(movement_entity, TimeDelta::new(0.016));
    
    // Entity 2: Physics constants entity (TimeDelta + Gravity)
    let physics_entity = world.create_entity();
    world.add_component(physics_entity, TimeDelta::new(0.016));
    world.add_component(physics_entity, Gravity::new(9.8));
    
    // Entity 3: Simple movement entity (Position + Velocity only)
    let simple_entity = world.create_entity();
    world.add_component(simple_entity, Position::new(5.0, 15.0));
    world.add_component(simple_entity, Velocity::new(-1.0, 2.0));
    
    world
}
```

## Component Type Requirements

All the standard component type rules apply:
- Components must implement the `Component` trait
- Use plain types (e.g., `Position`) for immutable access
- Use `Mut<T>` wrapper (e.g., `Mut<Velocity>`) for mutable access
- Components can be any combination of types across different iterators

## Testing Multiple Iterators

The library includes comprehensive tests for multiple iterator functionality:

```rust
#[test]
fn test_multiple_iterators_do_not_interfere() {
    let world = create_test_world();
    
    // Create multiple iterators simultaneously
    let _iter1 = world.iter_entities::<Position, Velocity>();
    let _iter2 = world.iter_entities::<TimeDelta, Gravity>();
    let _iter3 = world.iter_entities::<Position, TimeDelta>();
    
    // If this compiles and runs, multiple iterators work safely
}
```

## Running the Demo

To see multiple iterators in action:

```bash
# Run the full ECS demo (includes multiple iterators)
cargo run ecs

# Run only the multiple iterators demo
cargo run multi
```

## Implementation Details

The multiple iterator support works through:
1. World stores component pools indexed by TypeId
2. Each `iter_entities()` call creates a new EntityIterator with its own entity list
3. EntityIterator holds a raw pointer to World for safe access during iteration
4. RefCell provides interior mutability for component access
5. Rust's type system ensures compile-time safety for access patterns

This design allows systems to be flexible and powerful while maintaining safety and performance.