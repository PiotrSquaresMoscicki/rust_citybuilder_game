use crate::ecs::{System, SystemMarker, EntIt, Mut, TupleIter1};
use crate::game_components::PlayerComponent;
use crate::core::input_action::InputComponent;
use crate::core::input_system::InputSystem;
use crate::input::Key;
use crate::core::math::Vector2d;

/// System for handling player movement based on input
pub struct PlayerMovementSystem;

/// Direction enumeration for movement
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_vector(&self) -> Vector2d {
        match self {
            Direction::Up => Vector2d::new(0.0, -1.0),
            Direction::Down => Vector2d::new(0.0, 1.0),
            Direction::Left => Vector2d::new(-1.0, 0.0),
            Direction::Right => Vector2d::new(1.0, 0.0),
        }
    }
}

// ECS System trait implementations
impl SystemMarker for PlayerMovementSystem {
    fn name() -> &'static str { "PlayerMovementSystem" }
}

impl System for PlayerMovementSystem {
    type Dependencies = InputSystem;
    type Iterators = TupleIter1<EntIt<(Mut<PlayerComponent>, InputComponent)>>;

    fn update(&mut self, iterators: Self::Iterators) {
        let (player_input_iter,) = iterators;
        
        // Process each entity that has both PlayerComponent and InputComponent
        for (mut player_ref, input_ref) in player_input_iter {
            // Get component references
            let player = match player_ref.get_mut() {
                Some(p) => p,
                None => continue,
            };
            let input = input_ref.get();
            
            // Check for movement input - using "just pressed" for discrete movement
            let mut movement = Vector2d::new(0.0, 0.0);
            
            if input.is_key_just_pressed(&Key::ArrowUp) {
                movement.y -= 1.0; // Move up (negative Y)
            }
            if input.is_key_just_pressed(&Key::ArrowDown) {
                movement.y += 1.0; // Move down (positive Y)
            }
            if input.is_key_just_pressed(&Key::ArrowLeft) {
                movement.x -= 1.0; // Move left (negative X)
            }
            if input.is_key_just_pressed(&Key::ArrowRight) {
                movement.x += 1.0; // Move right (positive X)
            }
            
            // If no movement input, continue to next entity
            if movement.x == 0.0 && movement.y == 0.0 {
                continue;
            }
            
            // Calculate new position
            let current_pos = player.get_grid_position();
            let new_x = current_pos.0 + movement.x as i32;
            let new_y = current_pos.1 + movement.y as i32;
            
            // TODO: Add collision detection with grid boundaries and obstacles
            // For now, just move without collision checking to keep it simple
            player.set_grid_position(new_x, new_y);
            println!("Player moved to ({}, {})", new_x, new_y);
        }
    }
}

impl PlayerMovementSystem {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_direction_vectors() {
        assert_eq!(Direction::Up.to_vector(), Vector2d::new(0.0, -1.0));
        assert_eq!(Direction::Down.to_vector(), Vector2d::new(0.0, 1.0));
        assert_eq!(Direction::Left.to_vector(), Vector2d::new(-1.0, 0.0));
        assert_eq!(Direction::Right.to_vector(), Vector2d::new(1.0, 0.0));
    }
    
    #[test]
    fn test_player_movement_system_creation() {
        let _system = PlayerMovementSystem::new();
        // Just test that we can create the system
        assert!(true);
    }
    
    #[test]
    fn test_player_movement_system_marker() {
        // Test that PlayerMovementSystem implements SystemMarker
        assert_eq!(PlayerMovementSystem::name(), "PlayerMovementSystem");
    }
}