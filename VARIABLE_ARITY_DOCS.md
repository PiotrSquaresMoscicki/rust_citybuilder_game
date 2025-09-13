# Variable Arity Entity Iterator Documentation

This document explains the new variable arity entity iterator system that supports any number of components from zero to 64.

## Overview

The ECS library now supports querying entities with any number of components using a flexible `Query` trait system. This allows for more expressive and efficient entity iteration patterns.

## API Examples

### Zero Components - Iterate over all entities
```rust
let iter: EntityIterator<()> = world.query();
for entity in iter {
    println!("Entity: {}", entity);
}
```

### Single Component - Immutable access
```rust
let iter: EntityIterator<Position> = world.query();
for position in iter {
    println!("Position: ({}, {})", position.x, position.y);
}
```

### Single Component - Mutable access
```rust
let iter: EntityIterator<Mut<Velocity>> = world.query();
for mut velocity in iter {
    velocity.dx *= 0.9; // Apply damping
    velocity.dy *= 0.9;
}
```

### Two Components - Mixed mutability
```rust
let iter: EntityIterator<(Position, Mut<Velocity>)> = world.query();
for (position, mut velocity) in iter {
    // position is immutable, velocity is mutable
    velocity.dx += position.x * 0.01;
    velocity.dy += position.y * 0.01;
}
```

### Three Components - All combinations
```rust
// All immutable
let iter: EntityIterator<(Position, Velocity, Health)> = world.query();
for (pos, vel, health) in iter {
    // Read-only access to all components
}

// Mixed mutability
let iter: EntityIterator<(Position, Mut<Velocity>, Mut<Health>)> = world.query();
for (pos, mut vel, mut health) in iter {
    // position immutable, velocity and health mutable
}
```

### Higher Arities
The system supports up to 64 components in a single query:
```rust
let iter: EntityIterator<(C1, C2, C3, C4, C5, C6, C7, C8)> = world.query();
```

For extreme cases with many components:
```rust
// Example with 10 components
let iter: EntityIterator<(
    Position, Mut<Velocity>, Health, Mut<Energy>, 
    Armor, Mut<Experience>, Level, Mut<Inventory>,
    Skills, Mut<Stats>
)> = world.query();
```

## System Functions

System functions can use the new query API for cleaner code:

```rust
fn movement_system(world: &World) {
    let iter: EntityIterator<(Position, Mut<Velocity>)> = world.query();
    for (position, mut velocity) in iter {
        // Apply physics updates
        velocity.dx *= 0.95; // Damping
        velocity.dy *= 0.95;
    }
}

fn health_regen_system(world: &World) {
    let iter: EntityIterator<Mut<Health>> = world.query();
    for mut health in iter {
        health.heal(1); // Regenerate 1 HP
    }
}
```

## Backward Compatibility

The original 2-component iterator API is still supported:
```rust
let iter = world.iter_entities::<Position, Velocity>();
for (position, mut velocity) in iter {
    // Original API still works
}
```

## Implementation Details

### Query Trait
The `Query` trait defines how to access components:
```rust
pub trait Query {
    type Item<'a>;
    fn type_ids() -> Vec<TypeId>;
    fn fetch<'a>(world: &'a World, entity: Entity) -> Option<Self::Item<'a>>;
}
```

### Mut<T> Wrapper
Use `Mut<T>` to request mutable access to a component:
```rust
use crate::query::Mut;

let iter: EntityIterator<(Position, Mut<Velocity>)> = world.query();
```

### Type Safety
The system prevents:
- Multiple mutable references to the same component type
- Accessing non-existent components
- Invalid entity references

## Performance Characteristics

- **Zero-cost abstractions**: No runtime overhead compared to manual iteration
- **Efficient filtering**: Only iterates over entities with all required components  
- **Cache-friendly**: Sequential access patterns where possible
- **Compile-time validation**: Type errors caught at compile time

## Limitations

1. **Maximum arity**: Supports up to 64 components per query
2. **No nested queries**: Complex query compositions require separate iterations
3. **Static typing**: All component types must be known at compile time
4. **Compilation time**: Large tuple implementations may increase compile times

## Migration Guide

Existing code using the 2-component API continues to work unchanged. To use the new variable arity system:

1. Replace `world.iter_entities::<T1, T2>()` with `world.query::<(T1, Mut<T2>)>()`
2. Add explicit type annotations for the iterator: `EntityIterator<QueryType>`
3. Use `Mut<T>` for mutable component access
4. Expand queries to include more components as needed

This system provides a powerful and flexible foundation for entity iteration in the ECS library while maintaining type safety and performance.