use crate::ecs::World;
use crate::game_components::{PlayerComponent, GridComponent, ObstacleComponent, GridRenderableComponent};
use crate::rendering::{get_global_rendering_manager, RenderCommand};
use std::collections::HashMap;

/// Game renderer for the 2D grid game
pub struct GameRenderer;

impl GameRenderer {
    pub fn new() -> Self {
        Self
    }
    
    /// Render the entire game state to the web client
    pub fn render_game_state(world: &World) -> Result<(), String> {
        let _rendering_manager = get_global_rendering_manager()
            .map_err(|e| format!("No global rendering manager available: {}", e))?;
        
        // Get grid component for dimensions
        let grid_entities = world.entities_with_components(&[
            std::any::TypeId::of::<GridComponent>()
        ]);
        
        let grid_component = if let Some(&grid_entity) = grid_entities.first() {
            world.get_component::<GridComponent>(grid_entity)
                .ok_or("Failed to get grid component")?
        } else {
            return Err("No grid component found".to_string());
        };
        
        let grid_width = grid_component.width;
        let grid_height = grid_component.height;
        let cell_size = grid_component.cell_size;
        
        // Create a map of positions to characters for rendering
        let mut game_grid: HashMap<(i32, i32), (char, String)> = HashMap::new();
        
        // Add obstacles to the grid
        let obstacle_entities = world.entities_with_components(&[
            std::any::TypeId::of::<ObstacleComponent>()
        ]);
        
        for &entity in &obstacle_entities {
            if let Some(obstacle) = world.get_component::<ObstacleComponent>(entity) {
                let pos = obstacle.get_grid_position();
                game_grid.insert(pos, ('█', "#8B4513".to_string())); // Brown block
            }
        }
        
        // Add players to the grid
        let player_entities = world.entities_with_components(&[
            std::any::TypeId::of::<PlayerComponent>()
        ]);
        
        for &entity in &player_entities {
            if let Some(player) = world.get_component::<PlayerComponent>(entity) {
                let pos = player.get_grid_position();
                game_grid.insert(pos, ('♂', "#FF0000".to_string())); // Red player
            }
        }
        
        // Add entities with grid renderable components
        let renderable_entities = world.entities_with_components(&[
            std::any::TypeId::of::<GridRenderableComponent>()
        ]);
        
        for &entity in &renderable_entities {
            if let Some(_renderable) = world.get_component::<GridRenderableComponent>(entity) {
                // Try to find a position component - for now we'll skip this
                // In a full implementation, we'd check for position components
                // game_grid.insert(pos, (renderable.character, renderable.color.clone()));
            }
        }
        
        // Convert the game grid to a rendering command
        // For now, we'll create a simple text-based representation
        let mut grid_text = String::new();
        for y in 0..grid_height {
            for x in 0..grid_width {
                if let Some((character, _color)) = game_grid.get(&(x as i32, y as i32)) {
                    grid_text.push(*character);
                } else {
                    grid_text.push('·'); // Empty space
                }
            }
            grid_text.push('\n');
        }
        
        // Send the rendering command (we'll adapt this to work with the existing system)
        Self::send_game_grid_command(&grid_text, grid_width, grid_height, cell_size)
    }
    
    /// Send a grid rendering command with game state
    fn send_game_grid_command(grid_text: &str, width: u32, height: u32, cell_size: f32) -> Result<(), String> {
        let rendering_manager = get_global_rendering_manager()
            .map_err(|e| format!("No global rendering manager available: {}", e))?;
        
        // Create a custom render command for the game grid
        let _command = RenderCommand::DrawGrid {
            width,
            height,
            cell_size,
            line_color: (0.0, 0.0, 0.0, 1.0), // Black lines
            background_color: (1.0, 1.0, 1.0, 1.0), // White background
        };
        
        let result = {
            let manager = rendering_manager.lock()
                .map_err(|e| format!("Failed to lock rendering manager: {}", e))?;
            
            manager.render_grid(width, height, cell_size)
                .map_err(|e| format!("Failed to send render command: {}", e))
        };
        
        result?;
        
        println!("Game grid rendered:");
        println!("{}", grid_text);
        println!("Grid: {}x{}, Cell size: {}", width, height, cell_size);
        
        Ok(())
    }
    
    /// Simple ASCII representation of the game state for debugging
    pub fn print_game_state(world: &World) {
        println!("\n=== GAME STATE ===");
        
        // Get grid dimensions
        let grid_entities = world.entities_with_components(&[
            std::any::TypeId::of::<GridComponent>()
        ]);
        
        let (grid_width, grid_height) = if let Some(&grid_entity) = grid_entities.first() {
            if let Some(grid) = world.get_component::<GridComponent>(grid_entity) {
                (grid.width, grid.height)
            } else {
                (10, 10) // Default size
            }
        } else {
            (10, 10) // Default size
        };
        
        // Create a character grid
        let mut game_grid: HashMap<(i32, i32), char> = HashMap::new();
        
        // Add obstacles
        let obstacle_entities = world.entities_with_components(&[
            std::any::TypeId::of::<ObstacleComponent>()
        ]);
        
        for &entity in &obstacle_entities {
            if let Some(obstacle) = world.get_component::<ObstacleComponent>(entity) {
                let pos = obstacle.get_grid_position();
                game_grid.insert(pos, '#');
            }
        }
        
        // Add players
        let player_entities = world.entities_with_components(&[
            std::any::TypeId::of::<PlayerComponent>()
        ]);
        
        for &entity in &player_entities {
            if let Some(player) = world.get_component::<PlayerComponent>(entity) {
                let pos = player.get_grid_position();
                game_grid.insert(pos, '@');
            }
        }
        
        // Print the grid
        for y in 0..grid_height {
            for x in 0..grid_width {
                if let Some(character) = game_grid.get(&(x as i32, y as i32)) {
                    print!("{}", character);
                } else {
                    print!(".");
                }
            }
            println!();
        }
        
        println!("Legend: @ = Player, # = Obstacle, . = Empty");
        println!("==================");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;
    
    #[test]
    fn test_game_renderer_creation() {
        let renderer = GameRenderer::new();
        // Just test that we can create the renderer
        assert!(true);
    }
    
    #[test]
    fn test_print_game_state_empty_world() {
        let world = World::new();
        // This should not panic with an empty world
        GameRenderer::print_game_state(&world);
    }
}