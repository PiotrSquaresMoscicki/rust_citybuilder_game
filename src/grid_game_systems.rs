/// Game systems for the 2D grid game using the clean ECS implementation
use crate::ecs::*;
use crate::grid_game_components::*;

/// Input System - handles input processing (no dependencies)
pub struct GridInputSystem;

impl SystemMarker for GridInputSystem {
    fn name() -> &'static str { "GridInputSystem" }
}

impl System for GridInputSystem {
    type Dependencies = ();
    type Iterators = EntIt<(Mut<InputComponent>, ())>;

    fn update(&mut self, _iterators: Self::Iterators) {
        // In a real implementation, this would read from web client input
        // For now, just print that input system is running
        println!("GridInputSystem: Processing input...");
    }
}

/// Movement System - handles player movement (depends on input)
pub struct GridMovementSystem;

impl SystemMarker for GridMovementSystem {
    fn name() -> &'static str { "GridMovementSystem" }
}

impl System for GridMovementSystem {
    type Dependencies = GridInputSystem;
    type Iterators = EntIt<(Mut<GridPositionComponent>, PlayerComponent)>;

    fn update(&mut self, iterators: Self::Iterators) {
        // Since our iterators return entities for now, we can't directly access components in the loop
        // In a full implementation, this would iterate over the actual component tuples
        println!("GridMovementSystem: Processing movement...");
        
        // Count how many player entities we have
        let mut player_count = 0;
        for _entity in iterators {
            player_count += 1;
        }
        println!("Found {} player entities to move", player_count);
    }
}

/// Collision System - handles collision detection with obstacles
pub struct GridCollisionSystem;

impl SystemMarker for GridCollisionSystem {
    fn name() -> &'static str { "GridCollisionSystem" }
}

impl System for GridCollisionSystem {
    type Dependencies = GridMovementSystem;
    type Iterators = EntIt<(GridPositionComponent, ObstacleComponent)>;

    fn update(&mut self, iterators: Self::Iterators) {
        println!("GridCollisionSystem: Checking collisions...");
        
        let mut obstacle_count = 0;
        for _entity in iterators {
            obstacle_count += 1;
        }
        println!("Found {} obstacles for collision detection", obstacle_count);
    }
}

/// Render System - handles rendering to web client (depends on movement and collision)
pub struct GridRenderSystem;

impl SystemMarker for GridRenderSystem {
    fn name() -> &'static str { "GridRenderSystem" }
}

impl System for GridRenderSystem {
    type Dependencies = (GridMovementSystem, GridCollisionSystem);
    type Iterators = EntIt<(GridPositionComponent, RenderComponent)>;

    fn update(&mut self, iterators: Self::Iterators) {
        println!("GridRenderSystem: Rendering entities...");
        
        let mut render_count = 0;
        for _entity in iterators {
            render_count += 1;
        }
        println!("Rendered {} entities", render_count);
    }
}

/// Game world for the 2D grid game
pub struct GridGameWorld {
    pub world: World,
    pub system_registry: SystemRegistry,
}

impl GridGameWorld {
    pub fn new() -> Self {
        let world = World::new();
        let mut system_registry = SystemRegistry::new();
        
        // Register systems in dependency order
        system_registry.register_system("GridInputSystem", GridInputSystem);
        system_registry.register_system("GridMovementSystem", GridMovementSystem);
        system_registry.register_system("GridCollisionSystem", GridCollisionSystem);
        system_registry.register_system("GridRenderSystem", GridRenderSystem);
        
        Self {
            world,
            system_registry,
        }
    }
    
    /// Initialize the game world with entities
    pub fn initialize_game(&mut self) {
        // Create the player entity
        let player = self.world.create_entity();
        self.world.add_component(player, GridPositionComponent { x: 1, y: 1 });
        self.world.add_component(player, PlayerComponent { name: "Hero".to_string() });
        self.world.add_component(player, InputComponent::new());
        self.world.add_component(player, RenderComponent { symbol: '@', color: "red".to_string() });
        
        // Create some obstacles
        let obstacles = vec![
            (3, 1), (4, 1), (5, 1), // Horizontal wall
            (3, 2), (3, 3), (3, 4), // Vertical wall
            (7, 2), (8, 2), (9, 2), // Another horizontal wall
            (1, 5), (2, 5), (3, 5), // Bottom wall
        ];
        
        let obstacle_count = obstacles.len();
        for (x, y) in &obstacles {
            let obstacle = self.world.create_entity();
            self.world.add_component(obstacle, GridPositionComponent { x: *x, y: *y });
            self.world.add_component(obstacle, ObstacleComponent { block_movement: true });
            self.world.add_component(obstacle, RenderComponent { symbol: '#', color: "brown".to_string() });
        }
        
        println!("🎮 Grid game world initialized!");
        println!("   Player at (1, 1)");
        println!("   {} obstacles created", obstacle_count);
    }
    
    /// Run one game update cycle
    pub fn update(&mut self) -> Result<(), String> {
        self.system_registry.execute_systems(&self.world)
    }
    
    /// Get the current player position
    pub fn get_player_position(&self) -> Option<(i32, i32)> {
        // Find the player entity and get its position
        for entity in &self.world.entities {
            if self.world.has_component::<PlayerComponent>(*entity) {
                if let Some(pos) = self.world.get_component::<GridPositionComponent>(*entity) {
                    return Some((pos.x, pos.y));
                }
            }
        }
        None
    }
    
    /// Move the player in a direction (if possible)
    pub fn move_player(&mut self, dx: i32, dy: i32) -> bool {
        // Find the player entity
        let mut player_entity = None;
        for entity in &self.world.entities {
            if self.world.has_component::<PlayerComponent>(*entity) {
                player_entity = Some(*entity);
                break;
            }
        }
        
        let player_entity = match player_entity {
            Some(e) => e,
            None => return false,
        };
        
        // Get current position
        let current_pos = {
            match self.world.get_component::<GridPositionComponent>(player_entity) {
                Some(pos) => (pos.x, pos.y),
                None => return false,
            }
        };
        
        let new_x = current_pos.0 + dx;
        let new_y = current_pos.1 + dy;
        
        // Check bounds (simple 10x8 grid for now)
        if new_x < 0 || new_x >= 10 || new_y < 0 || new_y >= 8 {
            return false;
        }
        
        // Check for obstacles at the new position
        for entity in &self.world.entities {
            if self.world.has_component::<ObstacleComponent>(*entity) {
                if let Some(pos) = self.world.get_component::<GridPositionComponent>(*entity) {
                    if pos.x == new_x && pos.y == new_y {
                        println!("Movement blocked by obstacle at ({}, {})", new_x, new_y);
                        return false;
                    }
                }
            }
        }
        
        // Move the player
        if let Some(mut pos) = self.world.get_component_mut::<GridPositionComponent>(player_entity) {
            pos.x = new_x;
            pos.y = new_y;
            println!("Player moved to ({}, {})", new_x, new_y);
            return true;
        }
        
        false
    }
    
    /// Get the game state as a string representation
    pub fn get_game_state(&self) -> String {
        let mut grid = vec![vec!['.'; 10]; 8];
        
        // Place obstacles
        for entity in &self.world.entities {
            if self.world.has_component::<ObstacleComponent>(*entity) {
                if let (Some(pos), Some(render)) = (
                    self.world.get_component::<GridPositionComponent>(*entity),
                    self.world.get_component::<RenderComponent>(*entity)
                ) {
                    if pos.x >= 0 && pos.x < 10 && pos.y >= 0 && pos.y < 8 {
                        grid[pos.y as usize][pos.x as usize] = render.symbol;
                    }
                }
            }
        }
        
        // Place player
        for entity in &self.world.entities {
            if self.world.has_component::<PlayerComponent>(*entity) {
                if let (Some(pos), Some(render)) = (
                    self.world.get_component::<GridPositionComponent>(*entity),
                    self.world.get_component::<RenderComponent>(*entity)
                ) {
                    if pos.x >= 0 && pos.x < 10 && pos.y >= 0 && pos.y < 8 {
                        grid[pos.y as usize][pos.x as usize] = render.symbol;
                    }
                }
            }
        }
        
        // Convert grid to string
        grid.iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_game_world_creation() {
        let mut game = GridGameWorld::new();
        game.initialize_game();
        
        // Test that player was created
        assert!(game.get_player_position().is_some());
        
        // Test initial position
        let pos = game.get_player_position().unwrap();
        assert_eq!(pos, (1, 1));
    }
    
    #[test]
    fn test_player_movement() {
        let mut game = GridGameWorld::new();
        game.initialize_game();
        
        // Test valid movement
        assert!(game.move_player(1, 0)); // Move right
        let pos = game.get_player_position().unwrap();
        assert_eq!(pos, (2, 1));
        
        // Test movement into obstacle (should fail)
        assert!(!game.move_player(1, 0)); // Try to move into obstacle at (3, 1)
        let pos = game.get_player_position().unwrap();
        assert_eq!(pos, (2, 1)); // Should still be at (2, 1)
    }
    
    #[test]
    fn test_system_execution() {
        let mut game = GridGameWorld::new();
        game.initialize_game();
        
        // Test that systems can be executed without errors
        assert!(game.update().is_ok());
    }
    
    #[test]
    fn test_game_state_rendering() {
        let mut game = GridGameWorld::new();
        game.initialize_game();
        
        let state = game.get_game_state();
        assert!(state.contains('@')); // Player symbol
        assert!(state.contains('#')); // Obstacle symbol
        
        // Test that the string has the right number of lines
        let lines: Vec<&str> = state.split('\n').collect();
        assert_eq!(lines.len(), 8); // 8 rows
        assert_eq!(lines[0].len(), 10); // 10 columns
    }
}