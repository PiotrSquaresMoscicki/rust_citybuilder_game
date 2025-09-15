use crate::ecs::{World, System, SystemMarker, EntIt};
use crate::core::input_action::InputComponent;
use super::super::input::{poll_global_input_events, get_global_input_manager, Key, MouseButton};
use std::error::Error;

/// Input system that processes input events from the global input manager 
/// and updates input components in the ECS world
pub struct InputSystem {
    name: String,
}

impl InputSystem {
    /// Create a new input system
    pub fn new() -> Self {
        Self {
            name: "InputSystem".to_string(),
        }
    }

    /// Helper function to update input components directly in the world
    /// This is useful when the ECS doesn't have the dual-component iterator requirement
    pub fn update_input_components_in_world(world: &World) -> Result<(), Box<dyn Error>> {
        // Poll events from the global input manager
        let events = match poll_global_input_events() {
            Ok(events) => events,
            Err(e) => {
                eprintln!("Failed to poll input events: {}", e);
                return Err(e);
            }
        };

        // Get all entities with InputComponent
        let entities_with_input = world.entities_with_components(&[
            std::any::TypeId::of::<InputComponent>()
        ]);

        for entity in entities_with_input {
            if let Some(mut input_comp) = world.get_component_mut::<InputComponent>(entity) {
                input_comp.update_from_events(&events);
            }
        }

        Ok(())
    }

    /// Update the input system
    pub fn update(&self, world: &World) {
        if let Err(e) = Self::update_input_components_in_world(world) {
            eprintln!("Input system update failed: {}", e);
        }
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Function-based input system for compatibility with function-based ECS
pub fn input_system(world: &World) {
    if let Err(e) = InputSystem::update_input_components_in_world(world) {
        eprintln!("Input system function failed: {}", e);
    }
}

/// Helper function to create and return an input entity with InputComponent
pub fn create_input_entity(world: &mut World) -> crate::ecs::Entity {
    let entity = world.create_entity();
    world.add_component(entity, InputComponent::new());
    entity
}

/// Check if the global input system is ready to process events
pub fn is_input_system_ready() -> bool {
    if let Ok(manager_arc) = get_global_input_manager() {
        if let Ok(manager) = manager_arc.lock() {
            manager.is_ready()
        } else {
            false
        }
    } else {
        false
    }
}

/// Get input component from an entity for reading input state
pub fn get_input_state(world: &World, entity: crate::ecs::Entity) -> Option<impl std::ops::Deref<Target = InputComponent> + '_> {
    world.get_component::<InputComponent>(entity)
}

/// Helper functions for common input queries
pub fn is_key_pressed_in_world(world: &World, key: &Key) -> bool {
    let entities_with_input = world.entities_with_components(&[
        std::any::TypeId::of::<InputComponent>()
    ]);
    for entity in entities_with_input {
        if let Some(input_comp) = world.get_component::<InputComponent>(entity) {
            if input_comp.is_key_pressed(key) {
                return true;
            }
        }
    }
    false
}

pub fn is_key_just_pressed_in_world(world: &World, key: &Key) -> bool {
    let entities_with_input = world.entities_with_components(&[
        std::any::TypeId::of::<InputComponent>()
    ]);
    for entity in entities_with_input {
        if let Some(input_comp) = world.get_component::<InputComponent>(entity) {
            if input_comp.is_key_just_pressed(key) {
                return true;
            }
        }
    }
    false
}

pub fn is_mouse_button_pressed_in_world(world: &World, button: &MouseButton) -> bool {
    let entities_with_input = world.entities_with_components(&[
        std::any::TypeId::of::<InputComponent>()
    ]);
    for entity in entities_with_input {
        if let Some(input_comp) = world.get_component::<InputComponent>(entity) {
            if input_comp.is_mouse_button_pressed(button) {
                return true;
            }
        }
    }
    false
}

pub fn get_mouse_position_from_world(world: &World) -> Option<crate::core::math::Vector2d> {
    let entities_with_input = world.entities_with_components(&[
        std::any::TypeId::of::<InputComponent>()
    ]);
    for entity in entities_with_input {
        if let Some(input_comp) = world.get_component::<InputComponent>(entity) {
            return Some(input_comp.get_mouse_position());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{World, Component};
    use crate::input::{initialize_global_input_manager, InputEvent, Key, MouseButton};
    use crate::core::math::Vector2d;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_input_system_creation() {
        let input_system = InputSystem::new();
        assert_eq!(input_system.name, "InputSystem");
        // Just basic tests since we removed complex system interface
    }

    #[test]
    fn test_create_input_entity() {
        let mut world = World::new();
        let entity = create_input_entity(&mut world);
        
        // Check that the entity has an input component
        assert!(world.get_component::<InputComponent>(entity).is_some());
        
        let input_comp = world.get_component::<InputComponent>(entity).unwrap();
        assert!(input_comp.validate());
        assert!(input_comp.frame_actions.is_empty());
        assert!(input_comp.active_actions.is_empty());
    }

    #[test]
    fn test_input_system_without_global_manager() {
        let mut world = World::new();
        let entity = create_input_entity(&mut world);
        
        // Try to update input - this might succeed or fail depending on global state
        let result = InputSystem::update_input_components_in_world(&world);
        
        // Should either succeed or fail gracefully
        if result.is_err() {
            println!("Input system failed gracefully as expected without global manager");
        } else {
            println!("Input system succeeded - global manager was already initialized");
        }
        
        // Input component should still exist and be valid regardless
        let input_comp = world.get_component::<InputComponent>(entity).unwrap();
        assert!(input_comp.validate());
    }

    #[test]
    fn test_input_system_with_global_manager() {
        // Initialize global input manager
        if let Err(_) = initialize_global_input_manager() {
            // Manager might already be initialized from other tests
            println!("Global input manager already initialized");
        }
        
        let mut world = World::new();
        let entity = create_input_entity(&mut world);
        
        // Update input system
        let result = InputSystem::update_input_components_in_world(&world);
        assert!(result.is_ok());
        
        // Input component should still be valid
        let input_comp = world.get_component::<InputComponent>(entity).unwrap();
        assert!(input_comp.validate());
    }

    #[test]
    fn test_input_helper_functions() {
        let mut world = World::new();
        let entity = create_input_entity(&mut world);
        
        // Test helper functions with no input
        assert!(!is_key_pressed_in_world(&world, &Key::A));
        assert!(!is_key_just_pressed_in_world(&world, &Key::A));
        assert!(!is_mouse_button_pressed_in_world(&world, &MouseButton::Left));
        
        // Mouse position should be available but at origin
        let mouse_pos = get_mouse_position_from_world(&world);
        assert!(mouse_pos.is_some());
        assert_eq!(mouse_pos.unwrap(), Vector2d::new(0.0, 0.0));
    }

    #[test]
    fn test_input_system_object_interface() {
        let input_system = InputSystem::new();
        let mut world = World::new();
        let _entity = create_input_entity(&mut world);
        
        // Test the system object interface
        input_system.update(&world);
        
        // Should complete without panicking
        assert_eq!(input_system.name, "InputSystem");
    }

    #[test]
    fn test_multiple_input_entities() {
        let mut world = World::new();
        let entity1 = create_input_entity(&mut world);
        let entity2 = create_input_entity(&mut world);
        
        // Both entities should have input components
        assert!(world.get_component::<InputComponent>(entity1).is_some());
        assert!(world.get_component::<InputComponent>(entity2).is_some());
        
        // Helper functions should work with multiple entities
        assert!(!is_key_pressed_in_world(&world, &Key::A));
        assert!(!is_mouse_button_pressed_in_world(&world, &MouseButton::Left));
        
        // Should get mouse position from any entity
        let mouse_pos = get_mouse_position_from_world(&world);
        assert!(mouse_pos.is_some());
    }

    #[test]
    fn test_is_input_system_ready() {
        // Should work even if manager is not initialized
        let ready_status = is_input_system_ready();
        // Just test that it doesn't panic - actual result depends on global state
        println!("Input system ready: {}", ready_status);
    }
}

// ECS System trait implementations
impl SystemMarker for InputSystem {
    fn name() -> &'static str { "InputSystem" }
}

impl System for InputSystem {
    type Dependencies = ();
    type Iterators = EntIt<(crate::ecs::Mut<InputComponent>, ())>;

    fn update(&mut self, _iterators: Self::Iterators) {
        // Note: This implementation uses the world-based approach for now
        // since the iterator-based approach requires additional ECS infrastructure
        println!("InputSystem: Processing input events...");
    }
}