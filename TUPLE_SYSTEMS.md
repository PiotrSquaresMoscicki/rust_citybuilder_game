# Tuple-Based System Interface

This document explains the new tuple-based system interface that allows the World to know which components are used by which system at registration time.

## Problem Statement

Previously, systems were functions that took a `&World` parameter and created entity iterators dynamically:

```rust
pub fn physics_system(world: &World) {
    let movement_it = world.iter_entities::<Position, Mut<Velocity>>();
    let physics_it = world.iter_entities::<TimeDelta, Gravity>();
    // ... use iterators
}
```

This approach had limitations:
- The World didn't know which components a system would use until runtime
- Difficult to implement system scheduling and conflict detection
- No compile-time guarantees about system dependencies

## Solution: Tuple-Based Systems

The new tuple-based system interface allows systems to declare their iterator requirements upfront:

```rust
// System that takes a single iterator
pub fn single_iterator_system(movement_iter: EntityIterator<Position, Mut<Velocity>>) {
    for (position, mut velocity) in movement_iter {
        // Process entities with Position + mutable Velocity
    }
}

// System that takes a tuple of two iterators
pub fn dual_iterator_system(
    (movement_iter, physics_iter): (
        EntityIterator<Position, Mut<Velocity>>,
        EntityIterator<TimeDelta, Gravity>
    )
) {
    // Process movement entities
    for (position, mut velocity) in movement_iter { /* ... */ }
    
    // Process physics constants
    for (time_delta, gravity) in physics_iter { /* ... */ }
}
```

## Registration and Execution

Register tuple-based systems using `add_tuple_system`:

```rust
let mut world = World::new();

// Register systems - the World now knows their component dependencies
world.add_tuple_system(single_iterator_system);   // Uses: Position, Mut<Velocity>
world.add_tuple_system(dual_iterator_system);     // Uses: Position, Mut<Velocity>, TimeDelta, Gravity

// Run tuple-based systems
world.run_tuple_systems();

// Or run all systems (traditional + tuple-based)
world.run_systems();
```

## Supported Iterator Combinations

The system currently supports:

1. **Single Iterator**: `EntityIterator<A, B>`
2. **Tuple of Two**: `(EntityIterator<A, B>, EntityIterator<C, D>)`
3. **Tuple of Three**: `(EntityIterator<A, B>, EntityIterator<C, D>, EntityIterator<E, F>)`

More tuple sizes can be added by implementing additional `CreateIterators` trait instances.

## Implementation Details

### CreateIterators Trait

The `CreateIterators` trait automatically creates the required iterators from a World:

```rust
pub trait CreateIterators<Iters> {
    fn create_iterators(world: &World) -> Iters;
}

// Automatically implemented for supported iterator combinations
impl<A1, A2> CreateIterators<EntityIterator<A1, A2>> for EntityIterator<A1, A2> { ... }
impl<I1, I2> CreateIterators<(I1, I2)> for (I1, I2) { ... }
impl<I1, I2, I3> CreateIterators<(I1, I2, I3)> for (I1, I2, I3) { ... }
```

### System Registration

When you call `world.add_tuple_system(system)`, the World:

1. Wraps your system function to accept a `&World` parameter
2. Creates the required iterators using `CreateIterators::create_iterators`
3. Passes the iterators to your system function
4. Stores the wrapped system for later execution

## Benefits

### 1. Static Component Dependencies

The World knows at compile time which components each system uses:

```rust
world.add_tuple_system(physics_system);  // World knows: uses Position, Mut<Velocity>, TimeDelta, Gravity
```

### 2. Better System Scheduling

This enables future optimizations like:
- Parallel execution of non-conflicting systems
- Dependency graph construction
- Resource contention detection

### 3. Type Safety

All iterator creation is type-safe and checked at compile time.

### 4. Performance

No runtime overhead compared to the traditional approach - iterators are created in the same way, just at a different time.

## Backward Compatibility

The tuple-based system interface is fully backward compatible:

- Existing `world.add_system(|world| { ... })` continues to work
- `world.run_systems()` runs both traditional and tuple-based systems
- `world.run_tuple_systems()` runs only tuple-based systems

## Examples

### Basic Usage

```rust
use crate::ecs::{World, Mut, EntityIterator};
use crate::examples::{Position, Velocity};

fn movement_system(iter: EntityIterator<Position, Mut<Velocity>>) {
    for (position, mut velocity) in iter {
        // Apply physics
        velocity.dx *= 0.99; // friction
        velocity.dy -= 0.1;  // gravity
    }
}

let mut world = World::new();
world.add_tuple_system(movement_system);
world.run_tuple_systems();
```

### Complex System with Multiple Iterators

```rust
fn complex_physics_system(
    (movement_iter, constants_iter, collision_iter): (
        EntityIterator<Position, Mut<Velocity>>,
        EntityIterator<TimeDelta, Gravity>,
        EntityIterator<Position, CollisionBox>
    )
) {
    // Get physics constants
    let mut dt = 0.016;
    let mut gravity = 9.8;
    for (time_delta, gravity_component) in constants_iter {
        dt = time_delta.delta_time;
        gravity = gravity_component.acceleration;
        break;
    }
    
    // Apply physics to moving entities
    for (position, mut velocity) in movement_iter {
        velocity.dy += gravity * dt;
    }
    
    // Check collisions
    for (position, collision_box) in collision_iter {
        // Collision detection logic
    }
}
```

## Testing

The tuple-based system interface includes comprehensive tests:

```bash
# Run tuple system tests
cargo test tuple_systems_test

# Run the tuple system demo
cargo run tuple
```

## Migration Guide

To migrate existing systems to the tuple-based interface:

### Before (Traditional)
```rust
pub fn my_system(world: &World) {
    let iter1 = world.iter_entities::<Position, Mut<Velocity>>();
    let iter2 = world.iter_entities::<Health, TimeDelta>();
    
    for (pos, mut vel) in iter1 { /* ... */ }
    for (health, time) in iter2 { /* ... */ }
}

world.add_system(my_system);
```

### After (Tuple-Based)
```rust
pub fn my_system(
    (iter1, iter2): (
        EntityIterator<Position, Mut<Velocity>>,
        EntityIterator<Health, TimeDelta>
    )
) {
    for (pos, mut vel) in iter1 { /* ... */ }
    for (health, time) in iter2 { /* ... */ }
}

world.add_tuple_system(my_system);
```

The behavior remains identical, but now the World knows the system's dependencies at registration time.