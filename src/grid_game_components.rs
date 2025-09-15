/// Game components for the 2D grid game using the clean ECS implementation
use crate::ecs::*;
use std::any::Any;

/// Position component for entities in the 2D grid
#[derive(Clone, Debug)]
pub struct GridPositionComponent {
    pub x: i32,
    pub y: i32,
}

impl Component for GridPositionComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
    
    fn validate(&self) -> bool {
        // Ensure position is within reasonable bounds
        self.x >= 0 && self.y >= 0 && self.x < 1000 && self.y < 1000
    }
}

/// Player component to mark the player entity
#[derive(Clone, Debug)]
pub struct PlayerComponent {
    pub name: String,
}

impl Component for PlayerComponent {
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

/// Obstacle component for blocking movement
#[derive(Clone, Debug)]
pub struct ObstacleComponent {
    pub block_movement: bool,
}

impl Component for ObstacleComponent {
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

/// Input component for handling user input
#[derive(Clone, Debug)]
pub struct InputComponent {
    pub move_up: bool,
    pub move_down: bool,
    pub move_left: bool,
    pub move_right: bool,
}

impl Component for InputComponent {
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

impl InputComponent {
    pub fn new() -> Self {
        Self {
            move_up: false,
            move_down: false,
            move_left: false,
            move_right: false,
        }
    }
    
    pub fn clear(&mut self) {
        self.move_up = false;
        self.move_down = false;
        self.move_left = false;
        self.move_right = false;
    }
}

/// Render component for visual representation
#[derive(Clone, Debug)]
pub struct RenderComponent {
    pub symbol: char,
    pub color: String,
}

impl Component for RenderComponent {
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
    fn test_grid_position_component() {
        let pos = GridPositionComponent { x: 5, y: 10 };
        assert!(pos.validate());
        
        let invalid_pos = GridPositionComponent { x: -1, y: 5 };
        assert!(!invalid_pos.validate());
    }
    
    #[test]
    fn test_player_component() {
        let player = PlayerComponent { name: "Hero".to_string() };
        let cloned = player.clone_box();
        
        // Test that cloning works
        let cloned_player = cloned.as_any().downcast_ref::<PlayerComponent>().unwrap();
        assert_eq!(cloned_player.name, "Hero");
    }
    
    #[test]
    fn test_input_component() {
        let mut input = InputComponent::new();
        assert!(!input.move_up);
        
        input.move_up = true;
        input.move_right = true;
        
        input.clear();
        assert!(!input.move_up);
        assert!(!input.move_right);
    }
}