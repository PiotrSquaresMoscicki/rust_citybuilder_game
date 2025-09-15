use std::any::Any;
use std::collections::HashMap;
use crate::ecs::Component;
use super::super::input::{Key, MouseButton, InputEvent};
use crate::core::math::Vector2d;
use serde::{Deserialize, Serialize};

/// Represents different types of input actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputAction {
    /// Key or button press event (discrete)
    ButtonPress { key_or_button: String },
    /// Key or button release event (discrete)
    ButtonRelease { key_or_button: String },
    /// Key or button click event (press followed by release, discrete)
    ButtonClick { key_or_button: String },
    /// Key or button is being held down (continuous)
    ButtonHold { key_or_button: String },
    /// Mouse movement event
    MouseMove { position: Vector2d, delta: Vector2d },
    /// Mouse wheel scroll event
    MouseWheel { delta: f32, position: Vector2d },
}

/// Tracks the state of input buttons and keys
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ButtonState {
    /// Button was just pressed this frame
    JustPressed,
    /// Button is currently being held down
    Held,
    /// Button was just released this frame
    JustReleased,
    /// Button is not pressed
    Released,
}

impl ButtonState {
    /// Check if the button is currently down (just pressed or held)
    pub fn is_down(&self) -> bool {
        matches!(self, ButtonState::JustPressed | ButtonState::Held)
    }

    /// Check if the button was just pressed this frame
    pub fn is_just_pressed(&self) -> bool {
        matches!(self, ButtonState::JustPressed)
    }

    /// Check if the button was just released this frame
    pub fn is_just_released(&self) -> bool {
        matches!(self, ButtonState::JustReleased)
    }

    /// Check if the button is not pressed
    pub fn is_released(&self) -> bool {
        matches!(self, ButtonState::Released)
    }
}

/// Component that stores the current input state for systems to access
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputComponent {
    /// Current state of keyboard keys
    pub key_states: HashMap<Key, ButtonState>,
    /// Current state of mouse buttons
    pub mouse_button_states: HashMap<MouseButton, ButtonState>,
    /// Current mouse position
    pub mouse_position: Vector2d,
    /// Mouse movement delta this frame
    pub mouse_delta: Vector2d,
    /// Mouse wheel delta this frame
    pub mouse_wheel_delta: f32,
    /// Actions that occurred this frame (discrete events)
    pub frame_actions: Vec<InputAction>,
    /// Actions that are currently active (continuous)
    pub active_actions: Vec<InputAction>,
}

impl InputComponent {
    /// Create a new input component
    pub fn new() -> Self {
        Self {
            key_states: HashMap::new(),
            mouse_button_states: HashMap::new(),
            mouse_position: Vector2d::new(0.0, 0.0),
            mouse_delta: Vector2d::new(0.0, 0.0),
            mouse_wheel_delta: 0.0,
            frame_actions: Vec::new(),
            active_actions: Vec::new(),
        }
    }

    /// Check if a key is currently pressed (continuous input)
    pub fn is_key_pressed(&self, key: &Key) -> bool {
        self.key_states.get(key).map_or(false, |state| state.is_down())
    }

    /// Check if a key was just pressed this frame (discrete input)
    pub fn is_key_just_pressed(&self, key: &Key) -> bool {
        self.key_states.get(key).map_or(false, |state| state.is_just_pressed())
    }

    /// Check if a key was just released this frame (discrete input)
    pub fn is_key_just_released(&self, key: &Key) -> bool {
        self.key_states.get(key).map_or(false, |state| state.is_just_released())
    }

    /// Check if a mouse button is currently pressed (continuous input)
    pub fn is_mouse_button_pressed(&self, button: &MouseButton) -> bool {
        self.mouse_button_states.get(button).map_or(false, |state| state.is_down())
    }

    /// Check if a mouse button was just pressed this frame (discrete input)
    pub fn is_mouse_button_just_pressed(&self, button: &MouseButton) -> bool {
        self.mouse_button_states.get(button).map_or(false, |state| state.is_just_pressed())
    }

    /// Check if a mouse button was just released this frame (discrete input)
    pub fn is_mouse_button_just_released(&self, button: &MouseButton) -> bool {
        self.mouse_button_states.get(button).map_or(false, |state| state.is_just_released())
    }

    /// Get the current mouse position
    pub fn get_mouse_position(&self) -> Vector2d {
        self.mouse_position
    }

    /// Get the mouse movement delta for this frame
    pub fn get_mouse_delta(&self) -> Vector2d {
        self.mouse_delta
    }

    /// Get the mouse wheel delta for this frame
    pub fn get_mouse_wheel_delta(&self) -> f32 {
        self.mouse_wheel_delta
    }

    /// Get all discrete actions that occurred this frame
    pub fn get_frame_actions(&self) -> &[InputAction] {
        &self.frame_actions
    }

    /// Get all currently active continuous actions
    pub fn get_active_actions(&self) -> &[InputAction] {
        &self.active_actions
    }

    /// Update the input component from a list of input events
    pub fn update_from_events(&mut self, events: &[InputEvent]) {
        // Clear frame-specific data
        self.frame_actions.clear();
        self.mouse_delta = Vector2d::new(0.0, 0.0);
        self.mouse_wheel_delta = 0.0;

        // Update key and button states to handle transitions
        self.update_button_states();

        // Process each input event
        for event in events {
            self.process_event(event);
        }

        // Update active actions based on current button states
        self.update_active_actions();
    }

    /// Process a single input event
    fn process_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::KeyPress { key } => {
                let current_state = self.key_states.get(key).unwrap_or(&ButtonState::Released);
                if current_state.is_released() {
                    self.key_states.insert(key.clone(), ButtonState::JustPressed);
                    self.frame_actions.push(InputAction::ButtonPress {
                        key_or_button: key.to_string(),
                    });
                }
            }
            InputEvent::KeyRelease { key } => {
                let current_state = self.key_states.get(key).copied().unwrap_or(ButtonState::Released);
                if current_state.is_down() {
                    self.key_states.insert(key.clone(), ButtonState::JustReleased);
                    self.frame_actions.push(InputAction::ButtonRelease {
                        key_or_button: key.to_string(),
                    });
                    // Add a click action if the key was just pressed
                    if current_state.is_just_pressed() {
                        self.frame_actions.push(InputAction::ButtonClick {
                            key_or_button: key.to_string(),
                        });
                    }
                }
            }
            InputEvent::MousePress { button, position } => {
                let current_state = self.mouse_button_states.get(button).unwrap_or(&ButtonState::Released);
                if current_state.is_released() {
                    self.mouse_button_states.insert(button.clone(), ButtonState::JustPressed);
                    self.mouse_position = *position;
                    self.frame_actions.push(InputAction::ButtonPress {
                        key_or_button: format!("Mouse{:?}", button),
                    });
                }
            }
            InputEvent::MouseRelease { button, position } => {
                let current_state = self.mouse_button_states.get(button).copied().unwrap_or(ButtonState::Released);
                if current_state.is_down() {
                    self.mouse_button_states.insert(button.clone(), ButtonState::JustReleased);
                    self.mouse_position = *position;
                    self.frame_actions.push(InputAction::ButtonRelease {
                        key_or_button: format!("Mouse{:?}", button),
                    });
                    // Add a click action if the button was just pressed
                    if current_state.is_just_pressed() {
                        self.frame_actions.push(InputAction::ButtonClick {
                            key_or_button: format!("Mouse{:?}", button),
                        });
                    }
                }
            }
            InputEvent::MouseMove { position, delta } => {
                self.mouse_position = *position;
                self.mouse_delta = self.mouse_delta + *delta;
                self.frame_actions.push(InputAction::MouseMove {
                    position: *position,
                    delta: *delta,
                });
            }
            InputEvent::MouseWheel { delta, position } => {
                self.mouse_position = *position;
                self.mouse_wheel_delta += delta;
                self.frame_actions.push(InputAction::MouseWheel {
                    delta: *delta,
                    position: *position,
                });
            }
            _ => {
                // Handle other input events if needed in the future
            }
        }
    }

    /// Update button states for transitions (JustPressed -> Held, JustReleased -> Released)
    fn update_button_states(&mut self) {
        for state in self.key_states.values_mut() {
            *state = match *state {
                ButtonState::JustPressed => ButtonState::Held,
                ButtonState::JustReleased => ButtonState::Released,
                other => other,
            };
        }

        for state in self.mouse_button_states.values_mut() {
            *state = match *state {
                ButtonState::JustPressed => ButtonState::Held,
                ButtonState::JustReleased => ButtonState::Released,
                other => other,
            };
        }
    }

    /// Update the list of active continuous actions
    fn update_active_actions(&mut self) {
        self.active_actions.clear();

        // Add held keys and buttons as active actions
        for (key, state) in &self.key_states {
            if state.is_down() {
                self.active_actions.push(InputAction::ButtonHold {
                    key_or_button: key.to_string(),
                });
            }
        }

        for (button, state) in &self.mouse_button_states {
            if state.is_down() {
                self.active_actions.push(InputAction::ButtonHold {
                    key_or_button: format!("Mouse{:?}", button),
                });
            }
        }
    }

    /// Clear all input state (useful for resetting or cleaning up)
    pub fn clear(&mut self) {
        self.key_states.clear();
        self.mouse_button_states.clear();
        self.mouse_position = Vector2d::new(0.0, 0.0);
        self.mouse_delta = Vector2d::new(0.0, 0.0);
        self.mouse_wheel_delta = 0.0;
        self.frame_actions.clear();
        self.active_actions.clear();
    }
}

impl Default for InputComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for InputComponent {
    fn validate(&self) -> bool {
        // Input component is always valid - no validation constraints
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_component_creation() {
        let input_comp = InputComponent::new();
        assert!(input_comp.key_states.is_empty());
        assert!(input_comp.mouse_button_states.is_empty());
        assert_eq!(input_comp.mouse_position, Vector2d::new(0.0, 0.0));
        assert_eq!(input_comp.mouse_delta, Vector2d::new(0.0, 0.0));
        assert_eq!(input_comp.mouse_wheel_delta, 0.0);
        assert!(input_comp.frame_actions.is_empty());
        assert!(input_comp.active_actions.is_empty());
        assert!(input_comp.validate());
    }

    #[test]
    fn test_button_state_checks() {
        assert!(ButtonState::JustPressed.is_down());
        assert!(ButtonState::JustPressed.is_just_pressed());
        assert!(!ButtonState::JustPressed.is_just_released());
        assert!(!ButtonState::JustPressed.is_released());

        assert!(ButtonState::Held.is_down());
        assert!(!ButtonState::Held.is_just_pressed());
        assert!(!ButtonState::Held.is_just_released());
        assert!(!ButtonState::Held.is_released());

        assert!(!ButtonState::JustReleased.is_down());
        assert!(!ButtonState::JustReleased.is_just_pressed());
        assert!(ButtonState::JustReleased.is_just_released());
        assert!(!ButtonState::JustReleased.is_released());

        assert!(!ButtonState::Released.is_down());
        assert!(!ButtonState::Released.is_just_pressed());
        assert!(!ButtonState::Released.is_just_released());
        assert!(ButtonState::Released.is_released());
    }

    #[test]
    fn test_key_press_and_release() {
        let mut input_comp = InputComponent::new();
        
        let events = vec![
            InputEvent::KeyPress { key: Key::A },
        ];
        
        input_comp.update_from_events(&events);
        
        // Key should be just pressed
        assert!(input_comp.is_key_just_pressed(&Key::A));
        assert!(input_comp.is_key_pressed(&Key::A));
        assert!(!input_comp.is_key_just_released(&Key::A));
        
        // Check frame actions
        assert_eq!(input_comp.frame_actions.len(), 1);
        assert!(matches!(input_comp.frame_actions[0], InputAction::ButtonPress { .. }));
        
        // Check active actions
        assert_eq!(input_comp.active_actions.len(), 1);
        assert!(matches!(input_comp.active_actions[0], InputAction::ButtonHold { .. }));
        
        // Next frame without events - key should be held
        input_comp.update_from_events(&[]);
        assert!(!input_comp.is_key_just_pressed(&Key::A));
        assert!(input_comp.is_key_pressed(&Key::A));
        assert!(!input_comp.is_key_just_released(&Key::A));
        
        // Release the key
        let events = vec![
            InputEvent::KeyRelease { key: Key::A },
        ];
        
        input_comp.update_from_events(&events);
        
        // Key should be just released
        assert!(!input_comp.is_key_just_pressed(&Key::A));
        assert!(!input_comp.is_key_pressed(&Key::A));
        assert!(input_comp.is_key_just_released(&Key::A));
        
        // Should have only release action, no click since key was held
        assert_eq!(input_comp.frame_actions.len(), 1);
        assert!(input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::ButtonRelease { .. })));
        assert!(!input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::ButtonClick { .. })));
    }

    #[test]
    fn test_mouse_events() {
        let mut input_comp = InputComponent::new();
        
        let events = vec![
            InputEvent::MousePress { 
                button: MouseButton::Left, 
                position: Vector2d::new(10.0, 20.0) 
            },
            InputEvent::MouseMove { 
                position: Vector2d::new(15.0, 25.0), 
                delta: Vector2d::new(5.0, 5.0) 
            },
            InputEvent::MouseWheel { 
                delta: 1.0, 
                position: Vector2d::new(15.0, 25.0) 
            },
        ];
        
        input_comp.update_from_events(&events);
        
        // Check mouse button state
        assert!(input_comp.is_mouse_button_just_pressed(&MouseButton::Left));
        assert!(input_comp.is_mouse_button_pressed(&MouseButton::Left));
        
        // Check mouse position and deltas
        assert_eq!(input_comp.get_mouse_position(), Vector2d::new(15.0, 25.0));
        assert_eq!(input_comp.get_mouse_delta(), Vector2d::new(5.0, 5.0));
        assert_eq!(input_comp.get_mouse_wheel_delta(), 1.0);
        
        // Check actions
        assert_eq!(input_comp.frame_actions.len(), 3);
        assert!(input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::ButtonPress { .. })));
        assert!(input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::MouseMove { .. })));
        assert!(input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::MouseWheel { .. })));
    }

    #[test]
    fn test_quick_click() {
        let mut input_comp = InputComponent::new();
        
        // Press and release in the same frame
        let events = vec![
            InputEvent::KeyPress { key: Key::Space },
            InputEvent::KeyRelease { key: Key::Space },
        ];
        
        input_comp.update_from_events(&events);
        
        // Should have press, release, and click actions
        assert_eq!(input_comp.frame_actions.len(), 3);
        assert!(input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::ButtonPress { .. })));
        assert!(input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::ButtonRelease { .. })));
        assert!(input_comp.frame_actions.iter().any(|a| matches!(a, InputAction::ButtonClick { .. })));
        
        // Key should be released
        assert!(!input_comp.is_key_pressed(&Key::Space));
        assert!(input_comp.is_key_just_released(&Key::Space));
    }

    #[test]
    fn test_clear() {
        let mut input_comp = InputComponent::new();
        
        let events = vec![
            InputEvent::KeyPress { key: Key::A },
            InputEvent::MousePress { 
                button: MouseButton::Left, 
                position: Vector2d::new(10.0, 20.0) 
            },
        ];
        
        input_comp.update_from_events(&events);
        assert!(!input_comp.key_states.is_empty());
        assert!(!input_comp.frame_actions.is_empty());
        
        input_comp.clear();
        assert!(input_comp.key_states.is_empty());
        assert!(input_comp.mouse_button_states.is_empty());
        assert!(input_comp.frame_actions.is_empty());
        assert!(input_comp.active_actions.is_empty());
        assert_eq!(input_comp.mouse_position, Vector2d::new(0.0, 0.0));
    }
}