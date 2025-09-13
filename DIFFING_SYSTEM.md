# ECS Component State Diffing System

This implementation provides comprehensive component state change tracking for debugging ECS systems, as requested in the problem statement.

## Features

✅ **Component State Tracking**: Records changes in component state when executing systems  
✅ **Diffable Trait**: Components can be diffed to show only changed properties  
✅ **Diffable Macro**: Automatic implementation of the diffable trait for structs  
✅ **RON Serialization**: Complete diff records serialized in Rust Object Notation  
✅ **Frame Tracking**: Records frame number, system name, entity ID, and component type  
✅ **Human-Readable Output**: Formatted diff history for easy debugging  
✅ **Basic Type Support**: Implementations for integers, floats, strings, vectors, and hashmaps  

## Usage

### 1. Enable Debug Tracking

```rust
let mut world = World::new();
world.enable_debug_tracking();
world.next_frame(); // Advance to frame 1
```

### 2. Make Components Diffable

```rust
// Define your component
#[derive(Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

// Implement Component trait
impl Component for Position {
    fn validate(&self) -> bool { true }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_box(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

// Make it diffable using the macro
crate::diffable!(Position { x, y });
```

### 3. Run Systems with Debug Tracking

```rust
use std::any::TypeId;

// Run a system that modifies Velocity components
world.run_system_with_debug(
    "movement_system",
    |world| {
        let ent_it = world.iter_entities::<Position, Mut<Velocity>>();
        for (position, mut velocity) in ent_it {
            // Apply damping
            velocity.dx *= 0.9;
            velocity.dy *= 0.9;
        }
    },
    &[TypeId::of::<Velocity>()] // Specify which component types are mutable
);
```

### 4. View Diff History

```rust
// Get human-readable diff history
let history = world.get_debug_history();
println!("{}", history);
```

**Output Example:**
```
Frame 1: System 'movement_system'
  Entity 0: Velocity changed
    dx -> 1.3499999
    dy -> -0.45
  Entity 1: Velocity changed
    dx -> -0.9
    dy -> 1.8

Frame 1: System 'health_damage_system'
  Entity 0: Health changed
    current -> 95
  Entity 1: Health changed
    current -> 70
```

### 5. RON Serialization

```rust
// Access the raw diff records for RON serialization
if let Some(last_record) = world.debug_tracker.diff_history.last() {
    let ron_output = ron::ser::to_string_pretty(last_record, ron::ser::PrettyConfig::default())?;
    println!("{}", ron_output);
}
```

**RON Output Example:**
```ron
(
    frame_number: 1,
    system_name: "movement_system",
    component_diffs: [
        (
            entity_id: 0,
            component_type: "Velocity",
            changes: [
                (property_name: "dx", new_value: "1.3499999"),
                (property_name: "dy", new_value: "-0.45"),
            ],
        ),
    ],
)
```

## API Reference

### World Methods

- `enable_debug_tracking()` - Enable component state change tracking
- `disable_debug_tracking()` - Disable tracking
- `next_frame()` - Advance to the next frame for tracking
- `run_system_with_debug(name, system, mutable_types)` - Run a system with change tracking
- `get_debug_history()` - Get formatted diff history
- `clear_debug_history()` - Clear all recorded diffs

### Diffable Trait

```rust
pub trait Diffable {
    /// Create a diff representing changes from self to other
    /// Returns None if there are no changes
    fn diff(&self, other: &Self) -> Option<Vec<PropertyDiff>>;
    
    /// Get the type name for debugging purposes
    fn type_name() -> &'static str where Self: Sized;
}
```

### Diffable Macro

```rust
// Usage: list all fields that should be diffed
crate::diffable!(ComponentName { field1, field2, field3 });
```

### Data Structures

```rust
pub struct PropertyDiff {
    pub property_name: String,
    pub new_value: String, // RON serialized value
}

pub struct ComponentDiff {
    pub entity_id: Entity,
    pub component_type: String,
    pub changes: Vec<PropertyDiff>,
}

pub struct SystemDiffRecord {
    pub frame_number: u64,
    pub system_name: String,
    pub component_diffs: Vec<ComponentDiff>,
}
```

## Implementation Details

- **Only Changed Properties**: Diffs only include properties that actually changed
- **Type Safety**: Uses TypeId to safely cast components for diffing
- **Memory Efficient**: Takes snapshots only when debug tracking is enabled
- **No Performance Impact**: When disabled, systems run with no overhead
- **Extensible**: Easy to add diffing support for new component types

## Debugging Chain Reactions

This system is particularly useful for debugging ECS issues where:
- A crash in one system is caused by changes made several frames earlier
- Component state changes cascade through multiple systems
- You need to trace exactly when and how component values changed

The frame-by-frame tracking with system names and entity IDs makes it easy to identify the source of problematic state changes.