# Input System Implementation

This document describes the comprehensive input system implementation that transforms user input events into actions for other systems to react to.

## Overview

The input system provides a clean distinction between continuous input (like holding a button) and discrete input (like a button press). It's designed to work seamlessly with the existing ECS architecture and input device interfaces.

## Key Components

### InputAction Enum
Represents different types of input actions:
- `ButtonPress` - Discrete event when a key/button is initially pressed
- `ButtonRelease` - Discrete event when a key/button is released  
- `ButtonClick` - Discrete event for quick press-release (not for held buttons)
- `ButtonHold` - Continuous action while a key/button is held down
- `MouseMove` - Mouse movement with position and delta
- `MouseWheel` - Mouse wheel scroll events

### ButtonState Enum
Tracks the current state of keys and buttons:
- `JustPressed` - Button was pressed this frame
- `Held` - Button is being held down (was pressed in a previous frame)
- `JustReleased` - Button was released this frame
- `Released` - Button is not pressed

### InputComponent
ECS component that stores the current input state:
- Key and mouse button states with proper state tracking
- Mouse position, movement delta, and wheel delta
- Frame actions (discrete events that occurred this frame)
- Active actions (continuous inputs currently active)

### InputSystem
ECS system that processes input events:
- Polls events from the global input manager
- Updates all input components in the world
- Handles state transitions properly
- Supports both object-based and function-based system interfaces

## Key Features

### Continuous vs Discrete Input
- **Continuous**: `is_key_pressed()` returns true while a key is held down
- **Discrete**: `is_key_just_pressed()` returns true only on the frame when pressed

### Smart Click Detection
Click events are only generated for quick press-release sequences, not for buttons that are held down for multiple frames.

### Multiple Input Support
The system supports multiple input entities for scenarios like split-screen gaming or multiple input devices.

### Integration
Seamlessly integrates with:
- Existing ECS architecture
- Global input manager
- Input device interfaces (web client, keyboard, mouse, gamepad)
- Debug tracking and diffing system

## Usage Example

```rust
// Create input entity and system
let mut world = World::new();
let input_entity = create_input_entity(&mut world);
let input_system = InputSystem::new();

// In game loop
input_system.update(&world);

// Read input state for game logic
if let Some(input) = world.get_component::<InputComponent>(input_entity) {
    // Continuous input (movement)
    if input.is_key_pressed(&Key::W) { move_forward(); }
    
    // Discrete input (actions)
    if input.is_key_just_pressed(&Key::Space) { jump(); }
    if input.is_mouse_button_just_pressed(&MouseButton::Left) { 
        attack_at(input.get_mouse_position()); 
    }
    
    // Mouse interaction
    let mouse_delta = input.get_mouse_delta();
    let wheel_delta = input.get_mouse_wheel_delta();
}
```

## Testing

The implementation includes comprehensive tests:
- Unit tests for all input action functionality
- Integration tests with ECS world and system framework
- Demo tests showing practical usage patterns
- 37 total tests covering all aspects of the input system

## Files

- `src/core/input_action.rs` - Input action types and component
- `src/core/input_system.rs` - Input system implementation
- `src/core/input_system_demo.rs` - Comprehensive tests and examples
- `src/core/mod.rs` - Module exports
- `src/input/input_device.rs` - Updated with serialization support