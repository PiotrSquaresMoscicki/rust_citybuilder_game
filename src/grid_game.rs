use crate::ecs_clean::World;
use crate::game_components::{PlayerComponent, GridComponent, ObstacleComponent};
use crate::player_movement_system::PlayerMovementSystem;
use crate::game_renderer::GameRenderer;
use crate::input::{initialize_global_input_manager, add_global_input_device, poll_global_input_events, WebClientInputDevice};
use crate::rendering::WebServiceManager;
use std::time::{Duration, Instant};
use std::thread;

/// Main game struct that manages the 2D grid game
pub struct GridGame {
    world: World,
    is_running: bool,
    target_fps: u32,
}

impl GridGame {
    /// Create a new grid game
    pub fn new() -> Self {
        Self {
            world: World::new(),
            is_running: false,
            target_fps: 10, // Lower FPS for grid-based game
        }
    }
    
    /// Initialize the game world with player, grid, and obstacles
    pub fn initialize_game_world(&mut self) {
        println!("Initializing game world...");
        
        // Create grid entity
        let grid_entity = self.world.create_entity();
        let grid_component = GridComponent::new(12, 8, 40.0);
        self.world.add_component(grid_entity, grid_component);
        println!("Created grid: 12x8 cells");
        
        // Create player entity at starting position
        let player_entity = self.world.create_entity();
        let player_component = PlayerComponent::new(1, 1, 1.0);
        self.world.add_component(player_entity, player_component);
        println!("Created player at position (1, 1)");
        
        // Create some obstacles
        let obstacle_positions = vec![
            (3, 2), (3, 3), (3, 4), // Vertical wall
            (5, 1), (6, 1), (7, 1), // Horizontal wall
            (9, 5), (10, 5), (10, 6), // L-shaped obstacle
            (2, 6), (4, 6), (6, 6), // Scattered obstacles
        ];
        
        for (x, y) in obstacle_positions {
            let obstacle_entity = self.world.create_entity();
            let obstacle_component = ObstacleComponent::new(x, y);
            self.world.add_component(obstacle_entity, obstacle_component);
        }
        
        println!("Created {} obstacles", 10);
        println!("Game world initialized!");
    }
    
    /// Initialize rendering and input systems
    pub fn initialize_systems(&mut self) -> Result<(), String> {
        println!("Initializing game systems...");
        
        // Initialize input manager
        if initialize_global_input_manager().is_err() {
            println!("Warning: Failed to initialize input manager");
        }
        
        // Initialize web client input device
        let web_service_input = WebServiceManager::new("localhost:8083");
        let input_device = WebClientInputDevice::new(web_service_input, 1);
        if let Err(e) = add_global_input_device(Box::new(input_device)) {
            println!("Warning: Failed to add web client input device: {}", e);
        }
        
        // Note: Rendering manager should already be initialized by main.rs
        println!("Game systems initialized!");
        Ok(())
    }
    
    /// Start the game loop
    pub fn run(&mut self) -> Result<(), String> {
        println!("Starting 2D Grid Game!");
        println!("Use WASD or arrow keys to move the player (@)");
        println!("Avoid obstacles (#)");
        println!("");
        
        self.is_running = true;
        let frame_duration = Duration::from_millis(1000 / self.target_fps as u64);
        
        // Initial render
        GameRenderer::print_game_state(&self.world);
        
        while self.is_running {
            let frame_start = Instant::now();
            
            // Update game logic
            self.update();
            
            // Render game state
            self.render();
            
            // Frame rate limiting
            let frame_time = frame_start.elapsed();
            if frame_time < frame_duration {
                thread::sleep(frame_duration - frame_time);
            }
            
            // Simple exit condition for demo (in a real game, this would be input-driven)
            // For now, we'll run for a limited time
            static mut FRAME_COUNT: u32 = 0;
            unsafe {
                FRAME_COUNT += 1;
                if FRAME_COUNT > 1000 { // Run for ~100 seconds at 10 FPS
                    self.is_running = false;
                }
            }
        }
        
        println!("Game stopped.");
        Ok(())
    }
    
    /// Update game logic
    fn update(&mut self) {
        // Poll input events
        if let Err(_e) = poll_global_input_events() {
            // Input polling failed, but continue
        }
        
        // Update player movement
        PlayerMovementSystem::update_player_movement(&self.world);
        
        // Add more game logic here as needed
    }
    
    /// Render the game state
    fn render(&mut self) {
        // Print ASCII representation to console
        GameRenderer::print_game_state(&self.world);
        
        // Try to render to web client
        if let Err(_e) = GameRenderer::render_game_state(&self.world) {
            // Web rendering failed, but continue with ASCII
        }
    }
    
    /// Stop the game
    pub fn stop(&mut self) {
        self.is_running = false;
    }
    
    /// Get access to the world for testing
    pub fn get_world(&self) -> &World {
        &self.world
    }
    
    /// Get mutable access to the world for testing
    pub fn get_world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

/// Demonstrate the grid game
pub fn demonstrate_grid_game() {
    println!("🎮 2D Grid Game Demonstration");
    println!("=============================");
    
    let mut game = GridGame::new();
    
    // Initialize the game world
    game.initialize_game_world();
    
    // Initialize systems
    if let Err(e) = game.initialize_systems() {
        println!("Warning: System initialization failed: {}", e);
    }
    
    // Run the game
    if let Err(e) = game.run() {
        eprintln!("Game error: {}", e);
    }
}

/// Run a simple interactive demo of the grid game
pub fn run_interactive_grid_game() {
    println!("🎮 Interactive 2D Grid Game");
    println!("===========================");
    println!("Controls:");
    println!("  WASD or Arrow Keys - Move player");
    println!("  Player: @");
    println!("  Obstacles: #");
    println!("  Empty: .");
    println!("");
    
    let mut game = GridGame::new();
    game.initialize_game_world();
    
    // Show initial state
    println!("Initial game state:");
    GameRenderer::print_game_state(game.get_world());
    
    // For demonstration, simulate some moves
    println!("\nSimulating player movement...");
    
    // Simulate moving right a few times
    for i in 0..3 {
        println!("\nMove {} - Moving right:", i + 1);
        
        // Get player and try to move right
        let player_entities = game.get_world().entities_with_components(&[
            std::any::TypeId::of::<PlayerComponent>()
        ]);
        
        if let Some(&player_entity) = player_entities.first() {
            if let Some(mut player) = game.get_world().get_component_mut::<PlayerComponent>(player_entity) {
                let current_pos = player.get_grid_position();
                let new_x = current_pos.0 + 1;
                let new_y = current_pos.1;
                
                // Check if move is valid (simplified collision check)
                if new_x < 12 { // Within grid bounds
                    player.set_grid_position(new_x, new_y);
                    println!("Player moved to ({}, {})", new_x, new_y);
                } else {
                    println!("Cannot move - out of bounds");
                }
            }
        }
        
        GameRenderer::print_game_state(game.get_world());
        thread::sleep(Duration::from_millis(1000));
    }
    
    // Try to move into an obstacle
    println!("\nTrying to move into obstacle...");
    if let Some(&player_entity) = game.get_world().entities_with_components(&[
        std::any::TypeId::of::<PlayerComponent>()
    ]).first() {
        if let Some(mut player) = game.get_world().get_component_mut::<PlayerComponent>(player_entity) {
            let current_pos = player.get_grid_position();
            let new_x = current_pos.0;
            let new_y = current_pos.1 + 1; // Move down
            
            // Check for obstacle collision
            let obstacle_entities = game.get_world().entities_with_components(&[
                std::any::TypeId::of::<ObstacleComponent>()
            ]);
            
            let mut blocked = false;
            for &entity in &obstacle_entities {
                if let Some(obstacle) = game.get_world().get_component::<ObstacleComponent>(entity) {
                    let obstacle_pos = obstacle.get_grid_position();
                    if obstacle_pos == (new_x, new_y) {
                        blocked = true;
                        break;
                    }
                }
            }
            
            if !blocked {
                player.set_grid_position(new_x, new_y);
                println!("Player moved to ({}, {})", new_x, new_y);
            } else {
                println!("Cannot move to ({}, {}) - obstacle blocking!", new_x, new_y);
            }
        }
    }
    
    GameRenderer::print_game_state(game.get_world());
    
    println!("\nDemo complete! In a full game, you would use keyboard input to control the player.");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_grid_game_creation() {
        let game = GridGame::new();
        assert!(!game.is_running);
        assert_eq!(game.target_fps, 10);
    }
    
    #[test]
    fn test_game_world_initialization() {
        let mut game = GridGame::new();
        game.initialize_game_world();
        
        // Check that entities were created
        let grid_entities = game.world.entities_with_components(&[
            std::any::TypeId::of::<GridComponent>()
        ]);
        assert_eq!(grid_entities.len(), 1);
        
        let player_entities = game.world.entities_with_components(&[
            std::any::TypeId::of::<PlayerComponent>()
        ]);
        assert_eq!(player_entities.len(), 1);
        
        let obstacle_entities = game.world.entities_with_components(&[
            std::any::TypeId::of::<ObstacleComponent>()
        ]);
        assert!(obstacle_entities.len() > 0);
    }
    
    #[test]
    fn test_player_initial_position() {
        let mut game = GridGame::new();
        game.initialize_game_world();
        
        let player_entities = game.world.entities_with_components(&[
            std::any::TypeId::of::<PlayerComponent>()
        ]);
        
        if let Some(&player_entity) = player_entities.first() {
            if let Some(player) = game.world.get_component::<PlayerComponent>(player_entity) {
                assert_eq!(player.get_grid_position(), (1, 1));
            }
        }
    }
}