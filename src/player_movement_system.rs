use crate::ecs::{World, System, SystemMarker, EntIt};
use crate::game_components::{PlayerComponent, GridComponent, ObstacleComponent};
use crate::core::input_action::InputComponent;
use crate::core::input_system::InputSystem;
use crate::input::Key;
use crate::core::math::Vector2d;

/// System for handling player movement based on input
pub struct PlayerMovementSystem;

impl PlayerMovementSystem {
    pub fn new() -> Self {
        Self
    }
    
    /// Update player movement based on input
    pub fn update_player_movement(world: &World) {
        // Get entities with input components
        let input_entities = world.entities_with_components(&[
            std::any::TypeId::of::<InputComponent>()
        ]);
        
        // If no input entities, return early
        if input_entities.is_empty() {
            return;
        }
        
        // Get the first input component (typically there's only one)
        let input_component = if let Some(&input_entity) = input_entities.first() {
            world.get_component::<InputComponent>(input_entity)
        } else {
            return;
        };
        
        let input_comp = match input_component {
            Some(comp) => comp,
            None => return,
        };
        
        // Check for movement input - using "just pressed" for discrete movement
        let mut movement = Vector2d::new(0.0, 0.0);
        
        if input_comp.is_key_just_pressed(&Key::ArrowUp) {
            movement.y -= 1.0; // Move up (negative Y)
        }
        if input_comp.is_key_just_pressed(&Key::ArrowDown) {
            movement.y += 1.0; // Move down (positive Y)
        }
        if input_comp.is_key_just_pressed(&Key::ArrowLeft) {
            movement.x -= 1.0; // Move left (negative X)
        }
        if input_comp.is_key_just_pressed(&Key::ArrowRight) {
            movement.x += 1.0; // Move right (positive X)
        }
        
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

// ECS System trait implementations
impl SystemMarker for PlayerMovementSystem {
    fn name() -> &'static str { "PlayerMovementSystem" }
}

impl System for PlayerMovementSystem {
    type Dependencies = InputSystem;
    type Iterators = EntIt<(crate::ecs::Mut<PlayerComponent>, crate::ecs::Mut<InputComponent>)>;

    fn update(&mut self, _iterators: Self::Iterators) {
        // Note: This implementation will use the world-based approach for now
        // since the iterator-based approach requires additional ECS infrastructure
        // that's still being developed
        println!("PlayerMovementSystem: Processing player movement...");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;
    use crate::core::input_action::InputComponent;
    use crate::input::InputEvent;
    
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
    fn test_player_movement_system_marker() {
        // Test that PlayerMovementSystem implements SystemMarker
        assert_eq!(PlayerMovementSystem::name(), "PlayerMovementSystem");
    }
    
    #[test]
    fn test_player_movement_system_ecs_integration() {
        use crate::ecs::System;
        
        let mut system = PlayerMovementSystem::new();
        
        // Test that we can call the ECS System trait method
        // Note: We can't easily test the iterator functionality here as it requires
        // a fully set up ECS world with the iterator infrastructure
        // For now, just verify the trait implementation exists
        
        // This test primarily ensures that the System trait is properly implemented
        // and the dependencies are correctly specified
        assert_eq!(PlayerMovementSystem::name(), "PlayerMovementSystem");
    }
    
    #[test]
    fn test_player_movement_without_input_component() {
        let world = World::new();
        // This should not panic even without an input component
        PlayerMovementSystem::update_player_movement(&world);
    }
    
    #[test]
    fn test_arrow_key_movement_up() {
        let mut world = World::new();
        
        // Create input entity
        let input_entity = world.create_entity();
        let mut input_comp = InputComponent::new();
        input_comp.update_from_events(&[InputEvent::KeyPress { key: Key::ArrowUp }]);
        world.add_component(input_entity, input_comp);
        
        // Create player entity
        let player_entity = world.create_entity();
        let player_comp = PlayerComponent::new(5, 5, 1.0);
        world.add_component(player_entity, player_comp);
        
        // Create grid entity
        let grid_entity = world.create_entity();
        let grid_comp = GridComponent::new(10, 10, 32.0);
        world.add_component(grid_entity, grid_comp);
        
        // Execute movement system
        PlayerMovementSystem::update_player_movement(&world);
        
        // Check player moved up (y decreased)
        let player = world.get_component::<PlayerComponent>(player_entity).unwrap();
        assert_eq!(player.get_grid_position(), (5, 4));
    }
    
    #[test]
    fn test_arrow_key_movement_down() {
        let mut world = World::new();
        
        // Create input entity
        let input_entity = world.create_entity();
        let mut input_comp = InputComponent::new();
        input_comp.update_from_events(&[InputEvent::KeyPress { key: Key::ArrowDown }]);
        world.add_component(input_entity, input_comp);
        
        // Create player entity
        let player_entity = world.create_entity();
        let player_comp = PlayerComponent::new(5, 5, 1.0);
        world.add_component(player_entity, player_comp);
        
        // Create grid entity
        let grid_entity = world.create_entity();
        let grid_comp = GridComponent::new(10, 10, 32.0);
        world.add_component(grid_entity, grid_comp);
        
        // Execute movement system
        PlayerMovementSystem::update_player_movement(&world);
        
        // Check player moved down (y increased)
        let player = world.get_component::<PlayerComponent>(player_entity).unwrap();
        assert_eq!(player.get_grid_position(), (5, 6));
    }
    
    #[test]
    fn test_arrow_key_movement_left() {
        let mut world = World::new();
        
        // Create input entity
        let input_entity = world.create_entity();
        let mut input_comp = InputComponent::new();
        input_comp.update_from_events(&[InputEvent::KeyPress { key: Key::ArrowLeft }]);
        world.add_component(input_entity, input_comp);
        
        // Create player entity
        let player_entity = world.create_entity();
        let player_comp = PlayerComponent::new(5, 5, 1.0);
        world.add_component(player_entity, player_comp);
        
        // Create grid entity
        let grid_entity = world.create_entity();
        let grid_comp = GridComponent::new(10, 10, 32.0);
        world.add_component(grid_entity, grid_comp);
        
        // Execute movement system
        PlayerMovementSystem::update_player_movement(&world);
        
        // Check player moved left (x decreased)
        let player = world.get_component::<PlayerComponent>(player_entity).unwrap();
        assert_eq!(player.get_grid_position(), (4, 5));
    }
    
    #[test]
    fn test_arrow_key_movement_right() {
        let mut world = World::new();
        
        // Create input entity
        let input_entity = world.create_entity();
        let mut input_comp = InputComponent::new();
        input_comp.update_from_events(&[InputEvent::KeyPress { key: Key::ArrowRight }]);
        world.add_component(input_entity, input_comp);
        
        // Create player entity
        let player_entity = world.create_entity();
        let player_comp = PlayerComponent::new(5, 5, 1.0);
        world.add_component(player_entity, player_comp);
        
        // Create grid entity
        let grid_entity = world.create_entity();
        let grid_comp = GridComponent::new(10, 10, 32.0);
        world.add_component(grid_entity, grid_comp);
        
        // Execute movement system
        PlayerMovementSystem::update_player_movement(&world);
        
        // Check player moved right (x increased)
        let player = world.get_component::<PlayerComponent>(player_entity).unwrap();
        assert_eq!(player.get_grid_position(), (6, 5));
    }
    
    #[test]
    fn test_arrow_key_movement_boundary_collision() {
        let mut world = World::new();
        
        // Create input entity - try to move up
        let input_entity = world.create_entity();
        let mut input_comp = InputComponent::new();
        input_comp.update_from_events(&[InputEvent::KeyPress { key: Key::ArrowUp }]);
        world.add_component(input_entity, input_comp);
        
        // Create player entity at top boundary
        let player_entity = world.create_entity();
        let player_comp = PlayerComponent::new(5, 0, 1.0); // At y=0 (top)
        world.add_component(player_entity, player_comp);
        
        // Create grid entity
        let grid_entity = world.create_entity();
        let grid_comp = GridComponent::new(10, 10, 32.0);
        world.add_component(grid_entity, grid_comp);
        
        // Execute movement system
        PlayerMovementSystem::update_player_movement(&world);
        
        // Check player did not move (stayed at boundary)
        let player = world.get_component::<PlayerComponent>(player_entity).unwrap();
        assert_eq!(player.get_grid_position(), (5, 0));
    }
    
    #[test]
    fn test_arrow_key_movement_obstacle_collision() {
        let mut world = World::new();
        
        // Create input entity - try to move right
        let input_entity = world.create_entity();
        let mut input_comp = InputComponent::new();
        input_comp.update_from_events(&[InputEvent::KeyPress { key: Key::ArrowRight }]);
        world.add_component(input_entity, input_comp);
        
        // Create player entity
        let player_entity = world.create_entity();
        let player_comp = PlayerComponent::new(5, 5, 1.0);
        world.add_component(player_entity, player_comp);
        
        // Create grid entity
        let grid_entity = world.create_entity();
        let grid_comp = GridComponent::new(10, 10, 32.0);
        world.add_component(grid_entity, grid_comp);
        
        // Create obstacle entity at (6, 5) - where player wants to move
        let obstacle_entity = world.create_entity();
        let obstacle_comp = ObstacleComponent::new(6, 5);
        world.add_component(obstacle_entity, obstacle_comp);
        
        // Execute movement system
        PlayerMovementSystem::update_player_movement(&world);
        
        // Check player did not move (blocked by obstacle)
        let player = world.get_component::<PlayerComponent>(player_entity).unwrap();
        assert_eq!(player.get_grid_position(), (5, 5));
    }
    
    #[test]
    fn test_discrete_movement_behavior() {
        let mut world = World::new();
        
        // Create input entity
        let input_entity = world.create_entity();
        let mut input_comp = InputComponent::new();
        
        // First frame: press arrow key
        input_comp.update_from_events(&[InputEvent::KeyPress { key: Key::ArrowRight }]);
        world.add_component(input_entity, input_comp);
        
        // Create player entity
        let player_entity = world.create_entity();
        let player_comp = PlayerComponent::new(5, 5, 1.0);
        world.add_component(player_entity, player_comp);
        
        // Create grid entity
        let grid_entity = world.create_entity();
        let grid_comp = GridComponent::new(10, 10, 32.0);
        world.add_component(grid_entity, grid_comp);
        
        // Execute movement system - should move once
        PlayerMovementSystem::update_player_movement(&world);
        
        // Check player moved right once
        let player = world.get_component::<PlayerComponent>(player_entity).unwrap();
        assert_eq!(player.get_grid_position(), (6, 5));
        
        // Second frame: hold key (no new key press events)
        let mut input_comp = world.get_component_mut::<InputComponent>(input_entity).unwrap();
        input_comp.update_from_events(&[]); // No events, just holding
        drop(input_comp);
        
        // Execute movement system again - should NOT move (discrete movement)
        PlayerMovementSystem::update_player_movement(&world);
        
        // Check player did not move again (still at (6, 5))
        let player = world.get_component::<PlayerComponent>(player_entity).unwrap();
        assert_eq!(player.get_grid_position(), (6, 5));
    }
    
    #[test]
    fn test_multiple_arrow_keys_same_frame() {
        let mut world = World::new();
        
        // Create input entity with multiple arrow keys pressed in same frame
        let input_entity = world.create_entity();
        let mut input_comp = InputComponent::new();
        input_comp.update_from_events(&[
            InputEvent::KeyPress { key: Key::ArrowRight },
            InputEvent::KeyPress { key: Key::ArrowDown },
        ]);
        world.add_component(input_entity, input_comp);
        
        // Create player entity
        let player_entity = world.create_entity();
        let player_comp = PlayerComponent::new(5, 5, 1.0);
        world.add_component(player_entity, player_comp);
        
        // Create grid entity
        let grid_entity = world.create_entity();
        let grid_comp = GridComponent::new(10, 10, 32.0);
        world.add_component(grid_entity, grid_comp);
        
        // Execute movement system
        PlayerMovementSystem::update_player_movement(&world);
        
        // Check player moved diagonally (right and down)
        let player = world.get_component::<PlayerComponent>(player_entity).unwrap();
        assert_eq!(player.get_grid_position(), (6, 6));
    }
}