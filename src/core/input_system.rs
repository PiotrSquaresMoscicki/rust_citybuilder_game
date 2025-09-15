use crate::ecs::{System, SystemMarker, EntIt, Mut};
use crate::core::input_action::InputComponent;
use super::super::input::InputEvent;

/// Input system that processes input events and updates input components in the ECS world
pub struct InputSystem {
    /// Cached input events to be processed each frame
    cached_events: Vec<InputEvent>,
}

impl InputSystem {
    /// Create a new input system
    pub fn new() -> Self {
        Self {
            cached_events: Vec::new(),
        }
    }
    
    /// Add input events to be processed on the next update
    pub fn add_events(&mut self, events: Vec<InputEvent>) {
        self.cached_events.extend(events);
    }
    
    /// Clear any pending events
    pub fn clear_events(&mut self) {
        self.cached_events.clear();
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create and return an input entity with InputComponent
pub fn create_input_entity(world: &mut crate::ecs::World) -> crate::ecs::Entity {
    let entity = world.create_entity();
    world.add_component(entity, InputComponent::new());
    entity
}

// ECS System trait implementations
impl SystemMarker for InputSystem {
    fn name() -> &'static str { "InputSystem" }
}

impl System for InputSystem {
    type Dependencies = ();
    type Iterators = EntIt<Mut<InputComponent>>;

    fn update(&mut self, iterators: Self::Iterators) {
        // Process each input component in the ECS world
        for mut input_ref in iterators {
            if let Some(input) = input_ref.get_mut() {
                // Update the input component with cached events
                input.update_from_events(&self.cached_events);
            }
        }
        
        // Clear events after processing
        self.cached_events.clear();
    }
}