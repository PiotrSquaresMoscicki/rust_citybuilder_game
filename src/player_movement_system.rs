use crate::ecs::World;
use crate::game_components::{PlayerComponent, GridComponent, ObstacleComponent};
use crate::input::{get_global_input_manager, Key};
use crate::core::math::Vector2d;

/// System for handling player movement based on input
pub struct PlayerMovementSystem;

impl PlayerMovementSystem {
    pub fn new() -> Self {
        Self
    }
    
    /// Update player movement based on input
    pub fn update_player_movement(world: &World) {
        // Get input manager
        let input_manager = match get_global_input_manager() {
            Ok(manager) => manager,
            Err(_) => return, // No input manager available
        };
        
        // Check for movement input
        let mut movement = Vector2d::new(0.0, 0.0);
        
        let manager_lock = match input_manager.lock() {
            Ok(lock) => lock,
            Err(_) => return,
        };
        
        if manager_lock.is_key_pressed(&Key::W) || manager_lock.is_key_pressed(&Key::ArrowUp) {
            movement.y -= 1.0; // Move up (negative Y)
        }
        if manager_lock.is_key_pressed(&Key::S) || manager_lock.is_key_pressed(&Key::ArrowDown) {
            movement.y += 1.0; // Move down (positive Y)
        }
        if manager_lock.is_key_pressed(&Key::A) || manager_lock.is_key_pressed(&Key::ArrowLeft) {
            movement.x -= 1.0; // Move left (negative X)
        }
        if manager_lock.is_key_pressed(&Key::D) || manager_lock.is_key_pressed(&Key::ArrowRight) {
            movement.x += 1.0; // Move right (positive X)
        }
        
        drop(manager_lock); // Release the lock
        
        // If no movement input, return early
        if movement.x == 0.0 && movement.y == 0.0 {
            return;
        }
        
        // Get all entities with player components
        let player_entities = world.entities_with_components(&[
            std::any::TypeId::of::<PlayerComponent>()
        ]);
        
        // Get grid component for boundary checking
        let grid_entities = world.entities_with_components(&[
            std::any::TypeId::of::<GridComponent>()
        ]);
        
        let grid_component = if let Some(&grid_entity) = grid_entities.first() {
            world.get_component::<GridComponent>(grid_entity)
        } else {
            return; // No grid component found
        };
        
        // Get all obstacle positions for collision detection
        let obstacle_entities = world.entities_with_components(&[
            std::any::TypeId::of::<ObstacleComponent>()
        ]);
        
        let obstacle_positions: Vec<(i32, i32)> = obstacle_entities.iter()
            .filter_map(|&entity| {
                world.get_component::<ObstacleComponent>(entity)
                    .map(|obstacle| obstacle.get_grid_position())
            })
            .collect();
        
        // Update each player entity
        for &player_entity in &player_entities {
            if let Some(mut player) = world.get_component_mut::<PlayerComponent>(player_entity) {
                let current_pos = player.get_grid_position();
                let new_x = current_pos.0 + movement.x as i32;
                let new_y = current_pos.1 + movement.y as i32;
                
                // Check grid boundaries
                let within_bounds = if let Some(grid) = &grid_component {
                    grid.is_within_bounds(new_x, new_y)
                } else {
                    true // If no grid, assume no bounds
                };
                
                // Check collision with obstacles
                let collides_with_obstacle = obstacle_positions.contains(&(new_x, new_y));
                
                // Move only if within bounds and not colliding
                if within_bounds && !collides_with_obstacle {
                    player.set_grid_position(new_x, new_y);
                    println!("Player moved to ({}, {})", new_x, new_y);
                } else {
                    if !within_bounds {
                        println!("Cannot move to ({}, {}) - out of bounds", new_x, new_y);
                    }
                    if collides_with_obstacle {
                        println!("Cannot move to ({}, {}) - obstacle blocking", new_x, new_y);
                    }
                }
            }
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;
    
    #[test]
    fn test_direction_vectors() {
        assert_eq!(Direction::Up.to_vector(), Vector2d::new(0.0, -1.0));
        assert_eq!(Direction::Down.to_vector(), Vector2d::new(0.0, 1.0));
        assert_eq!(Direction::Left.to_vector(), Vector2d::new(-1.0, 0.0));
        assert_eq!(Direction::Right.to_vector(), Vector2d::new(1.0, 0.0));
    }
    
    #[test]
    fn test_player_movement_system_creation() {
        let system = PlayerMovementSystem::new();
        // Just test that we can create the system
        assert!(true);
    }
    
    #[test]
    fn test_player_movement_without_input_manager() {
        let world = World::new();
        // This should not panic even without an input manager
        PlayerMovementSystem::update_player_movement(&world);
    }
}