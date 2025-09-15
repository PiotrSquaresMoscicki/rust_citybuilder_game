use crate::ecs::{World, System, Mut};
use crate::core::input_action::{InputComponent, InputAction};
use crate::core::input_system::{InputSystem, create_input_entity};
use crate::input::{InputEvent, Key, MouseButton, initialize_global_input_manager};
use crate::core::math::Vector2d;

/// Comprehensive test demonstrating the input system functionality
#[test]
fn test_comprehensive_input_system() {
    // Initialize the global input manager
    if let Err(_) = initialize_global_input_manager() {
        println!("Global input manager already initialized");
    }

    // Create a world and an input entity
    let mut world = World::new();
    let input_entity = create_input_entity(&mut world);

    // Simulate a series of input events
    println!("=== Testing Input System Comprehensive Functionality ===");

    // Test 1: Key press and hold
    println!("\n1. Testing key press and hold...");
    simulate_key_press_and_hold(&world, input_entity);

    // Test 2: Quick key click
    println!("\n2. Testing quick key click...");
    simulate_quick_key_click(&world, input_entity);

    // Test 3: Mouse interaction
    println!("\n3. Testing mouse interaction...");
    simulate_mouse_interaction(&world, input_entity);

    // Test 4: Multiple inputs simultaneously
    println!("\n4. Testing multiple simultaneous inputs...");
    simulate_multiple_inputs(&world, input_entity);

    println!("\n=== Input System Test Complete ===");
}

fn simulate_key_press_and_hold(world: &World, entity: crate::ecs::Entity) {
    // Simulate key press
    {
        let mut input_comp = world.get_component_mut::<InputComponent>(entity).unwrap();
        let events = vec![InputEvent::KeyPress { key: Key::W }];
        input_comp.update_from_events(&events);

        // Check key is just pressed
        assert!(input_comp.is_key_just_pressed(&Key::W));
        assert!(input_comp.is_key_pressed(&Key::W));
        assert_eq!(input_comp.frame_actions.len(), 1);
        assert!(matches!(input_comp.frame_actions[0], InputAction::ButtonPress { .. }));
        assert_eq!(input_comp.active_actions.len(), 1);
        assert!(matches!(input_comp.active_actions[0], InputAction::ButtonHold { .. }));

        println!("   ✓ Key W pressed - detected as just pressed and held");
    }

    // Next frame - no events, key should be held
    {
        let mut input_comp = world.get_component_mut::<InputComponent>(entity).unwrap();
        input_comp.update_from_events(&[]);
        assert!(!input_comp.is_key_just_pressed(&Key::W));
        assert!(input_comp.is_key_pressed(&Key::W)); // Still pressed
        assert!(input_comp.frame_actions.is_empty()); // No new events
        assert_eq!(input_comp.active_actions.len(), 1); // Still held

        println!("   ✓ Key W held - no longer just pressed but still pressed");
    }

    // Release key
    {
        let mut input_comp = world.get_component_mut::<InputComponent>(entity).unwrap();
        let events = vec![InputEvent::KeyRelease { key: Key::W }];
        input_comp.update_from_events(&events);
        assert!(!input_comp.is_key_pressed(&Key::W));
        assert!(input_comp.is_key_just_released(&Key::W));
        assert_eq!(input_comp.frame_actions.len(), 1); // Only release, no click (was held)
        assert!(matches!(input_comp.frame_actions[0], InputAction::ButtonRelease { .. }));
        assert!(input_comp.active_actions.is_empty()); // No longer held

        println!("   ✓ Key W released - detected as just released, no click (was held)");
    }
}

fn simulate_quick_key_click(world: &World, entity: crate::ecs::Entity) {
    let mut input_comp = world.get_component_mut::<InputComponent>(entity).unwrap();

    // Simulate quick press and release in same frame
    let events = vec![
        InputEvent::KeyPress { key: Key::Space },
        InputEvent::KeyRelease { key: Key::Space },
    ];
    input_comp.update_from_events(&events);

    // Should have press, release, and click actions
    assert_eq!(input_comp.frame_actions.len(), 3);
    let has_press = input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::ButtonPress { .. }));
    let has_release = input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::ButtonRelease { .. }));
    let has_click = input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::ButtonClick { .. }));

    assert!(has_press);
    assert!(has_release);
    assert!(has_click);

    println!("   ✓ Space key quick click - detected press, release, and click actions");

    // Key should be released
    assert!(!input_comp.is_key_pressed(&Key::Space));
    assert!(input_comp.is_key_just_released(&Key::Space));
}

fn simulate_mouse_interaction(world: &World, entity: crate::ecs::Entity) {
    {
        let mut input_comp = world.get_component_mut::<InputComponent>(entity).unwrap();

        // Simulate mouse events
        let events = vec![
            InputEvent::MouseMove { 
                position: Vector2d::new(100.0, 150.0), 
                delta: Vector2d::new(10.0, 5.0) 
            },
            InputEvent::MousePress { 
                button: MouseButton::Left, 
                position: Vector2d::new(100.0, 150.0) 
            },
            InputEvent::MouseWheel { 
                delta: 2.0, 
                position: Vector2d::new(100.0, 150.0) 
            },
        ];
        input_comp.update_from_events(&events);

        // Check mouse state
        assert_eq!(input_comp.get_mouse_position(), Vector2d::new(100.0, 150.0));
        assert_eq!(input_comp.get_mouse_delta(), Vector2d::new(10.0, 5.0));
        assert_eq!(input_comp.get_mouse_wheel_delta(), 2.0);
        assert!(input_comp.is_mouse_button_pressed(&MouseButton::Left));
        assert!(input_comp.is_mouse_button_just_pressed(&MouseButton::Left));

        // Check actions
        assert_eq!(input_comp.frame_actions.len(), 3);
        let has_move = input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::MouseMove { .. }));
        let has_press = input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::ButtonPress { .. }));
        let has_wheel = input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::MouseWheel { .. }));

        assert!(has_move);
        assert!(has_press);
        assert!(has_wheel);

        println!("   ✓ Mouse interaction - move, click, and wheel events detected");
    }

    // Release mouse button
    {
        let mut input_comp = world.get_component_mut::<InputComponent>(entity).unwrap();
        let events = vec![
            InputEvent::MouseRelease { 
                button: MouseButton::Left, 
                position: Vector2d::new(105.0, 155.0) 
            },
        ];
        input_comp.update_from_events(&events);

        assert!(!input_comp.is_mouse_button_pressed(&MouseButton::Left));
        assert!(input_comp.is_mouse_button_just_released(&MouseButton::Left));

        println!("   ✓ Mouse button released - state correctly updated");
    }
}

fn simulate_multiple_inputs(world: &World, entity: crate::ecs::Entity) {
    {
        let mut input_comp = world.get_component_mut::<InputComponent>(entity).unwrap();

        // Simulate multiple keys and mouse pressed simultaneously
        let events = vec![
            InputEvent::KeyPress { key: Key::W },
            InputEvent::KeyPress { key: Key::A },
            InputEvent::KeyPress { key: Key::S },
            InputEvent::MousePress { 
                button: MouseButton::Right, 
                position: Vector2d::new(200.0, 300.0) 
            },
        ];
        input_comp.update_from_events(&events);

        // Check all inputs are detected
        assert!(input_comp.is_key_pressed(&Key::W));
        assert!(input_comp.is_key_pressed(&Key::A));
        assert!(input_comp.is_key_pressed(&Key::S));
        assert!(input_comp.is_mouse_button_pressed(&MouseButton::Right));

        // Should have 4 press actions and 4 active hold actions
        assert_eq!(input_comp.frame_actions.len(), 4);
        assert_eq!(input_comp.active_actions.len(), 4);

        println!("   ✓ Multiple simultaneous inputs - all detected correctly");
    }

    // Next frame - all should be held
    {
        let mut input_comp = world.get_component_mut::<InputComponent>(entity).unwrap();
        input_comp.update_from_events(&[]);
        assert!(input_comp.is_key_pressed(&Key::W));
        assert!(input_comp.is_key_pressed(&Key::A));
        assert!(input_comp.is_key_pressed(&Key::S));
        assert!(input_comp.is_mouse_button_pressed(&MouseButton::Right));

        // Should have no frame actions but still have active actions
        assert!(input_comp.frame_actions.is_empty());
        assert_eq!(input_comp.active_actions.len(), 4);

        println!("   ✓ Multiple inputs held - state transitions correctly");
    }

    // Release some inputs
    {
        let mut input_comp = world.get_component_mut::<InputComponent>(entity).unwrap();
        let events = vec![
            InputEvent::KeyRelease { key: Key::A },
            InputEvent::MouseRelease { 
                button: MouseButton::Right, 
                position: Vector2d::new(200.0, 300.0) 
            },
        ];
        input_comp.update_from_events(&events);

        // Check partial release
        assert!(input_comp.is_key_pressed(&Key::W)); // Still pressed
        assert!(!input_comp.is_key_pressed(&Key::A)); // Released
        assert!(input_comp.is_key_pressed(&Key::S)); // Still pressed
        assert!(!input_comp.is_mouse_button_pressed(&MouseButton::Right)); // Released

        // Should have 2 release actions and 2 remaining active actions
        assert_eq!(input_comp.frame_actions.len(), 2);
        assert_eq!(input_comp.active_actions.len(), 2);

        println!("   ✓ Partial input release - remaining inputs still active");
    }
}

/// Example demonstrating how to use the input system in practice
#[test]
fn test_input_system_usage_example() {
    println!("=== Input System Usage Example ===");

    // Initialize global input manager
    if let Err(_) = initialize_global_input_manager() {
        println!("Global input manager already initialized");
    }

    // Create world and input system
    let mut world = World::new();
    let input_entity = create_input_entity(&mut world);
    let mut input_system = InputSystem::new();

    // Add input system to world (in a real game, this would be done in the main loop)
    println!("Input system created");
    println!("System dependencies: []"); // No dependencies for input system

    // Simulate game loop frame using ECS iterators
    let iter = world.iter_entities_1::<crate::ecs::Mut<InputComponent>>();
    input_system.update(iter);

    // Read input state for game logic
    if let Some(input_comp) = world.get_component::<InputComponent>(input_entity) {
        println!("Input component ready, mouse position: {:?}", input_comp.get_mouse_position());

        // Example: Check for player movement input
        let movement_direction = get_movement_direction(&*input_comp);
        if movement_direction != Vector2d::new(0.0, 0.0) {
            println!("Player movement: {:?}", movement_direction);
        }

        // Example: Check for action inputs
        if input_comp.is_key_just_pressed(&Key::Space) {
            println!("Player jumped!");
        }

        if input_comp.is_mouse_button_just_pressed(&MouseButton::Left) {
            println!("Player attacked at position: {:?}", input_comp.get_mouse_position());
        }

        // Example: Check continuous input
        if input_comp.is_key_pressed(&Key::Shift) {
            println!("Player is running (continuous input)");
        }
    }

    println!("=== Usage Example Complete ===");
}

/// Helper function demonstrating how to extract movement input
fn get_movement_direction(input: &InputComponent) -> Vector2d {
    let mut direction = Vector2d::new(0.0, 0.0);

    if input.is_key_pressed(&Key::W) { direction.y += 1.0; }
    if input.is_key_pressed(&Key::S) { direction.y -= 1.0; }
    if input.is_key_pressed(&Key::A) { direction.x -= 1.0; }
    if input.is_key_pressed(&Key::D) { direction.x += 1.0; }

    // Normalize diagonal movement
    if direction.x != 0.0 && direction.y != 0.0 {
        let magnitude = (direction.x * direction.x + direction.y * direction.y).sqrt();
        direction.x /= magnitude;
        direction.y /= magnitude;
    }

    direction
}

/// Test the distinction between continuous and discrete input
#[test]
fn test_continuous_vs_discrete_input() {
    println!("=== Testing Continuous vs Discrete Input ===");

    let mut input_comp = InputComponent::new();

    // Frame 1: Press key
    let events = vec![InputEvent::KeyPress { key: Key::F }];
    input_comp.update_from_events(&events);

    println!("Frame 1 - Key pressed:");
    println!("  Discrete input (just pressed): {}", input_comp.is_key_just_pressed(&Key::F));
    println!("  Continuous input (pressed): {}", input_comp.is_key_pressed(&Key::F));
    println!("  Frame actions: {}", input_comp.frame_actions.len());
    println!("  Active actions: {}", input_comp.active_actions.len());

    // Frame 2: Hold key (no events)
    input_comp.update_from_events(&[]);

    println!("Frame 2 - Key held:");
    println!("  Discrete input (just pressed): {}", input_comp.is_key_just_pressed(&Key::F));
    println!("  Continuous input (pressed): {}", input_comp.is_key_pressed(&Key::F));
    println!("  Frame actions: {}", input_comp.frame_actions.len());
    println!("  Active actions: {}", input_comp.active_actions.len());

    // Frame 3: Release key
    let events = vec![InputEvent::KeyRelease { key: Key::F }];
    input_comp.update_from_events(&events);

    println!("Frame 3 - Key released:");
    println!("  Discrete input (just released): {}", input_comp.is_key_just_released(&Key::F));
    println!("  Continuous input (pressed): {}", input_comp.is_key_pressed(&Key::F));
    println!("  Frame actions: {}", input_comp.frame_actions.len());
    println!("  Active actions: {}", input_comp.active_actions.len());

    println!("=== Continuous vs Discrete Test Complete ===");
}