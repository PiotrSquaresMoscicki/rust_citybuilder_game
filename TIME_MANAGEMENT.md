# Time Management System

This document describes the time management system implemented for the ECS library. The system provides delta time information to systems that need it, such as movement or animation systems.

## Overview

The time management system consists of three main components:

1. **TimeManager** - A global service that provides current time information
2. **TimeComponent** - An ECS component that stores delta time and other timing data
3. **Time System** - An ECS system that updates TimeComponent instances with data from TimeManager

## Architecture

### TimeManager (Global Service)

The TimeManager is a singleton service that tracks:
- Start time of the application
- Last frame time
- Current delta time since last frame
- Total elapsed time since start

```rust
use crate::core::time::{initialize_time_manager, update_global_time_manager};

// Initialize at program start
initialize_time_manager();

// Update once per frame in your game loop
update_global_time_manager();
```

### TimeComponent (ECS Component)

TimeComponent stores frame-based timing information:

```rust
use crate::core::time::TimeComponent;

let mut time_comp = TimeComponent::new();
time_comp.set_time_scale(2.0); // 2x speed
time_comp.pause(); // Pause time
time_comp.resume(); // Resume time
```

Properties:
- `delta_time`: Time since last frame in seconds
- `total_time`: Total elapsed time since start
- `frame_count`: Number of frames since start
- `time_scale`: Speed multiplier (1.0 = normal, 2.0 = double speed)
- `is_paused`: Whether time advancement is paused

### Time System

The time system updates TimeComponent instances with current delta time:

```rust
use crate::core::time_system::update_time_components_in_world;

// Update all time components in a world
update_time_components_in_world(&world);
```

## Usage Pattern

### Setup (Once per application)

```rust
use crate::core::time::{initialize_time_manager, TimeComponent};
use crate::ecs::World;

// 1. Initialize the global time manager
initialize_time_manager();

// 2. Create a world and add a time entity (usually only one per world)
let mut world = World::new();
let time_entity = world.create_entity();
world.add_component(time_entity, TimeComponent::new());
```

### Game Loop

```rust
use crate::core::time::{update_global_time_manager};
use crate::core::time_system::update_time_components_in_world;

loop {
    // 1. Update global time manager (gets current time)
    update_global_time_manager();
    
    // 2. Update time components (propagates delta time to ECS)
    update_time_components_in_world(&world);
    
    // 3. Run your game systems (they can access time via TimeComponent)
    run_movement_system(&world);
    run_animation_system(&world);
    // ... other systems
    
    // 4. Sleep or present frame
    std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
}
```

### Using Time in Systems

Systems that need timing information should iterate over TimeComponent:

```rust
use crate::ecs::{EntityIterator, Mut};
use crate::core::time::TimeComponent;

pub fn movement_system(
    entity_iter: EntityIterator<Mut<Transform>, Mut<Velocity>>,
    time_iter: EntityIterator<TimeComponent, TimeComponent>
) {
    // Get delta time from time component (expects only one time entity)
    let delta_time = if let Some((time_component, _)) = time_iter.into_iter().next() {
        time_component.scaled_delta_time() as f32 // Apply time scale and pause
    } else {
        0.0 // No time component available
    };

    // Move all entities based on velocity and delta time
    for (mut transform, velocity) in entity_iter {
        let movement = velocity.velocity * delta_time;
        let current_pos = transform.translation();
        transform.set_translation(current_pos + movement);
    }
}
```

## Example: Complete Movement System

```rust
use crate::ecs::{World, EntityIterator, Mut, Component};
use crate::core::time::{TimeComponent, initialize_time_manager, update_global_time_manager};
use crate::core::time_system::update_time_components_in_world;
use crate::core::math::{Vector2d, Transform2dComponent};

#[derive(Clone, Debug)]
pub struct Velocity {
    pub velocity: Vector2d,
}

impl Component for Velocity {
    fn validate(&self) -> bool { true }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_box(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

pub fn movement_system(
    entity_iter: EntityIterator<Mut<Transform2dComponent>, Mut<Velocity>>,
    time_iter: EntityIterator<TimeComponent, TimeComponent>
) {
    let delta_time = if let Some((time_component, _)) = time_iter.into_iter().next() {
        time_component.scaled_delta_time() as f32
    } else {
        0.0
    };

    for (mut transform, velocity) in entity_iter {
        let movement = velocity.velocity * delta_time;
        let current_translation = transform.translation();
        transform.set_translation(current_translation + movement);
    }
}

fn main() {
    // Initialize time management
    initialize_time_manager();
    
    let mut world = World::new();
    
    // Create time entity
    let time_entity = world.create_entity();
    world.add_component(time_entity, TimeComponent::new());
    
    // Create moving entity
    let entity = world.create_entity();
    world.add_component(entity, Transform2dComponent::from_translation(Vector2d::new(0.0, 0.0)));
    world.add_component(entity, Velocity { velocity: Vector2d::new(100.0, 50.0) });
    
    // Game loop
    for _frame in 0..60 {
        update_global_time_manager();
        update_time_components_in_world(&world);
        
        // Run movement system
        let transform_velocity_iter = world.iter_entities::<Mut<Transform2dComponent>, Mut<Velocity>>();
        let time_iter = world.iter_entities::<TimeComponent, TimeComponent>();
        movement_system(transform_velocity_iter, time_iter);
        
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
```

## Time Control Features

### Time Scale

Control game speed without changing system logic:

```rust
let mut time_comp = world.get_component_mut::<TimeComponent>(time_entity).unwrap();
time_comp.set_time_scale(0.5); // Half speed
time_comp.set_time_scale(2.0); // Double speed
```

### Pause/Resume

Pause game time while keeping the application running:

```rust
let mut time_comp = world.get_component_mut::<TimeComponent>(time_entity).unwrap();
time_comp.pause();     // Pause time
time_comp.resume();    // Resume time
time_comp.toggle_pause(); // Toggle pause state
```

### FPS Monitoring

Get current frames per second:

```rust
let time_comp = world.get_component::<TimeComponent>(time_entity).unwrap();
println!("FPS: {:.1}", time_comp.fps());
```

## Best Practices

1. **Single Time Entity**: Usually have only one TimeComponent per world
2. **Update Order**: Always update global time manager before time components
3. **Delta Time Usage**: Always multiply velocities and rates by delta time
4. **Time Scale**: Use `scaled_delta_time()` instead of `delta_time` to respect pause and scale
5. **System Design**: Systems should handle zero delta time gracefully (when paused)

## Testing

The time system includes comprehensive tests demonstrating:
- Basic time component functionality
- Time manager behavior
- Integration with ECS systems
- Pause and time scale features
- Movement system examples

Run tests with:
```bash
cargo test time
```

Run the interactive demo with:
```bash
cargo run time
```