# Multiple Iterator System Parameters

This document explains the new API for creating systems that accept multiple entity iterators as parameters.

## Overview

Previously, systems in the ECS library had to accept a `&World` parameter and create iterators inside the system function:

```rust
// Old API
fn physics_system(world: &World) {
    let movement_it = world.iter_entities::<Position, Mut<Velocity>>();
    let physics_it = world.iter_entities::<TimeDelta, Gravity>();
    // ... use iterators
}
```

The new API allows systems to accept entity iterators directly as parameters:

```rust
// New API
fn physics_system(
    movement_entities: EntityIterator<Position, Mut<Velocity>>,
    physics_entities: EntityIterator<TimeDelta, Gravity>
) {
    // ... use iterators directly
}
```

## Benefits

1. **Clear Interface**: System signatures explicitly show which component combinations the system uses
2. **Type Safety**: The World knows exactly which components each system accesses
3. **Better Debugging**: Automatic tracking of mutable component types for debug purposes
4. **Performance**: Iterators are created once by the World and passed to systems

## System Types

The ECS library supports three types of iterator-based systems:

### Single Iterator Systems

Systems that take one entity iterator:

```rust
fn movement_system(entities: EntityIterator<Position, Mut<Velocity>>) {
    for (position, mut velocity) in entities {
        // Apply movement logic
        velocity.dx *= 0.98; // friction
        velocity.dy -= 0.1;  // gravity
    }
}

// Register the system
world.add_single_iterator_system(movement_system, "MovementSystem");
```

### Dual Iterator Systems

Systems that take two entity iterators:

```rust
fn physics_system(
    movement_entities: EntityIterator<Position, Mut<Velocity>>,
    physics_entities: EntityIterator<TimeDelta, Gravity>
) {
    // First, gather physics constants
    let mut time_delta = 0.016;
    let mut gravity = -9.8;
    
    for (delta, grav) in physics_entities {
        time_delta = delta.delta_time;
        gravity = grav.acceleration;
        break; // Use first one found
    }
    
    // Then apply to movement entities
    for (position, mut velocity) in movement_entities {
        velocity.dy += gravity * time_delta;
    }
}

// Register the system
world.add_dual_iterator_system(physics_system, "PhysicsSystem");
```

### Triple Iterator Systems

Systems that take three entity iterators:

```rust
fn complex_system(
    pos_vel_entities: EntityIterator<Position, Velocity>,
    time_grav_entities: EntityIterator<TimeDelta, Gravity>,
    pos_time_entities: EntityIterator<Position, TimeDelta>
) {
    // Process different entity combinations
    let pos_vel_count = pos_vel_entities.count();
    let time_grav_count = time_grav_entities.count();
    let pos_time_count = pos_time_entities.count();
    
    println!("Entity counts: {}, {}, {}", pos_vel_count, time_grav_count, pos_time_count);
}

// Register the system
world.add_triple_iterator_system(complex_system, "ComplexSystem");
```

## Component Access Patterns

All the standard component access rules apply:

- **Immutable access**: Use plain component types (e.g., `Position`)
- **Mutable access**: Use `Mut<T>` wrapper (e.g., `Mut<Velocity>`)
- **Mixed access**: Different parameters can have different access patterns

```rust
fn example_system(
    read_only: EntityIterator<Position, Velocity>,        // Both immutable
    read_write: EntityIterator<Position, Mut<Velocity>>,  // Mixed access
    write_only: EntityIterator<Mut<Health>, Mut<Energy>>  // Both mutable
) {
    // ... system logic
}
```

## System Registration and Execution

### Registration

```rust
let mut world = World::new();

// Add different types of systems
world.add_single_iterator_system(movement_system, "Movement");
world.add_dual_iterator_system(physics_system, "Physics");
world.add_triple_iterator_system(complex_system, "Complex");
```

### Execution

```rust
// Run all systems (both new and legacy)
world.run_systems();

// Run only new iterator-based systems with debug tracking
world.enable_debug_tracking();
world.run_iterator_systems_with_debug();
```

## Debug Tracking

The new API automatically tracks which component types each system modifies:

```rust
world.enable_debug_tracking();
world.run_iterator_systems_with_debug();

let debug_history = world.get_debug_history();
println!("Debug history:\n{}", debug_history);
```

Output example:
```
Frame 0: System 'MovementSystem'
  Entity 0: Velocity changed
    dx -> 1.4406
    dy -> -0.8568001

Frame 0: System 'PhysicsSystem'  
  Entity 0: Velocity changed
    dy -> -1.0136001
```

## Backward Compatibility

The new API maintains full backward compatibility with existing systems:

```rust
// Legacy systems still work
world.add_system(|world| {
    let entities = world.iter_entities::<Position, Mut<Velocity>>();
    for (pos, mut vel) in entities {
        // ... legacy system logic
    }
});

// Both old and new systems run together
world.run_systems();
```

## Implementation Details

The new API is implemented using:

1. **SystemCall trait**: Abstracts different system parameter patterns
2. **Typed system structs**: `SingleIteratorSystem`, `DualIteratorSystem`, `TripleIteratorSystem`
3. **Automatic type tracking**: Systems report which component types they access mutably
4. **Iterator creation**: World creates appropriate iterators and passes them to systems

## Usage Examples

### Basic Movement System

```rust
fn basic_movement(entities: EntityIterator<Position, Mut<Velocity>>) {
    for (position, mut velocity) in entities {
        println!("Moving entity at ({}, {})", position.x, position.y);
        // Apply movement logic here
    }
}

world.add_single_iterator_system(basic_movement, "BasicMovement");
```

### Physics Integration

```rust
fn apply_physics(
    entities: EntityIterator<Position, Mut<Velocity>>,
    constants: EntityIterator<Gravity, TimeDelta>
) {
    // Extract physics constants
    let (gravity, time_delta) = constants
        .next()
        .map(|(g, t)| (g.acceleration, t.delta_time))
        .unwrap_or((-9.8, 0.016));
    
    // Apply to entities
    for (position, mut velocity) in entities {
        velocity.dy += gravity * time_delta;
    }
}

world.add_dual_iterator_system(apply_physics, "PhysicsIntegration");
```

## Running the Demo

To see the new API in action:

```bash
cargo run multi-systems
```

This will demonstrate all three system types working together with debug tracking enabled.