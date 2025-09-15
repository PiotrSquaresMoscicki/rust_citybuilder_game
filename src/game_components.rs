use crate::ecs::{Component};
use crate::core::math::{Vector2d};
use std::any::Any;

/// Component for the player character
#[derive(Clone, Debug)]
pub struct PlayerComponent {
    pub movement_speed: f32,
    pub grid_position: Vector2d,
}

impl PlayerComponent {
    pub fn new(grid_x: i32, grid_y: i32, movement_speed: f32) -> Self {
        Self {
            movement_speed,
            grid_position: Vector2d::new(grid_x as f32, grid_y as f32),
        }
    }
    
    pub fn get_grid_position(&self) -> (i32, i32) {
        (self.grid_position.x as i32, self.grid_position.y as i32)
    }
    
    pub fn set_grid_position(&mut self, x: i32, y: i32) {
        self.grid_position.x = x as f32;
        self.grid_position.y = y as f32;
    }
}

impl Component for PlayerComponent {
    fn validate(&self) -> bool {
        self.movement_speed > 0.0
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

/// Component for the game grid
#[derive(Clone, Debug)]
pub struct GridComponent {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
}

impl GridComponent {
    pub fn new(width: u32, height: u32, cell_size: f32) -> Self {
        Self {
            width,
            height,
            cell_size,
        }
    }
    
    pub fn is_within_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }
}

impl Component for GridComponent {
    fn validate(&self) -> bool {
        self.width > 0 && self.height > 0 && self.cell_size > 0.0
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

/// Component for obstacles that block movement
#[derive(Clone, Debug)]
pub struct ObstacleComponent {
    pub grid_position: Vector2d,
    pub is_blocking: bool,
}

impl ObstacleComponent {
    pub fn new(grid_x: i32, grid_y: i32) -> Self {
        Self {
            grid_position: Vector2d::new(grid_x as f32, grid_y as f32),
            is_blocking: true,
        }
    }
    
    pub fn get_grid_position(&self) -> (i32, i32) {
        (self.grid_position.x as i32, self.grid_position.y as i32)
    }
}

impl Component for ObstacleComponent {
    fn validate(&self) -> bool {
        true // Always valid
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

/// Component to mark entities as renderable in the game grid
#[derive(Clone, Debug)]
pub struct GridRenderableComponent {
    pub character: char,
    pub color: String,
}

impl GridRenderableComponent {
    pub fn new(character: char, color: &str) -> Self {
        Self {
            character,
            color: color.to_string(),
        }
    }
}

impl Component for GridRenderableComponent {
    fn validate(&self) -> bool {
        !self.color.is_empty()
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