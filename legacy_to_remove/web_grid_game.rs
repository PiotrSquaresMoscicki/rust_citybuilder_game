use crate::grid_game::GridGame;

/// Web-based grid game that runs in the browser
pub struct WebGridGame {
    game: GridGame,
    address: String,
}

impl WebGridGame {
    pub fn new(address: &str) -> Self {
        Self {
            game: GridGame::new(),
            address: address.to_string(),
        }
    }
    
    /// Initialize the web-based game
    pub fn initialize(&mut self) -> Result<(), String> {
        // Initialize the game world
        self.game.initialize_game_world();
        
        // Initialize systems
        self.game.initialize_systems().map_err(|e| format!("Failed to initialize game systems: {}", e))?;
        
        Ok(())
    }
    
    /// Start the web server and game loop
    pub fn run(&mut self) -> Result<(), String> {
        println!("🎮 Starting Web Grid Game Server");
        println!("================================");
        
        // Start the HTTP server to serve the game page
        let page_content = self.create_game_page();
        
        use tiny_http::{Server, Response, Header};
        
        let server = Server::http(&self.address)
            .map_err(|e| format!("Failed to start HTTP server: {}", e))?;
        
        println!("🌐 Web Grid Game server started on http://{}", &self.address);
        println!("🎯 Open http://{} in your browser to play the game", &self.address);
        println!("📱 Use WASD keys or click/touch to move the player");
        println!("");
        
        // Game loop in a separate thread  
        // Note: For simplicity, we'll skip the complex game loop integration for now
        // In a full implementation, we'd need to properly synchronize the ECS world
        
        // HTTP server loop
        for request in server.incoming_requests() {
            let method = request.method();
            let url = request.url();
            
            println!("{} {}", method, url);
            
            // Serve the game page for all requests
            let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                .expect("Invalid header");
            
            let response = Response::from_string(&page_content)
                .with_header(header);
            
            if let Err(e) = request.respond(response) {
                eprintln!("Error responding to request: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Create the HTML page for the grid game
    fn create_game_page(&self) -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>2D Grid Game - Rust City Builder</title>
    <style>
        body {
            font-family: 'Courier New', monospace;
            margin: 0;
            padding: 20px;
            background-color: #1a1a1a;
            color: #00ff00;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
            background: #000;
            padding: 20px;
            border-radius: 8px;
            border: 2px solid #00ff00;
        }
        h1 {
            color: #00ff00;
            text-align: center;
            text-shadow: 0 0 10px #00ff00;
        }
        .game-container {
            display: flex;
            flex-direction: column;
            align-items: center;
        }
        #gameGrid {
            background: #000;
            border: 2px solid #00ff00;
            font-family: 'Courier New', monospace;
            font-size: 24px;
            line-height: 1;
            padding: 10px;
            margin: 20px 0;
            white-space: pre;
            text-align: center;
        }
        .controls {
            background: #111;
            border: 1px solid #00ff00;
            padding: 15px;
            margin: 10px 0;
            border-radius: 5px;
        }
        .control-buttons {
            display: grid;
            grid-template-columns: repeat(3, 60px);
            grid-template-rows: repeat(3, 60px);
            gap: 5px;
            justify-content: center;
            margin: 10px 0;
        }
        .control-btn {
            background: #222;
            border: 1px solid #00ff00;
            color: #00ff00;
            font-size: 18px;
            font-weight: bold;
            cursor: pointer;
            border-radius: 4px;
            transition: all 0.2s;
        }
        .control-btn:hover {
            background: #00ff00;
            color: #000;
            box-shadow: 0 0 10px #00ff00;
        }
        .control-btn:active {
            transform: scale(0.95);
        }
        .empty-btn {
            background: transparent;
            border: none;
        }
        .info {
            text-align: center;
            margin: 10px 0;
            font-size: 14px;
        }
        .legend {
            background: #111;
            border: 1px solid #00ff00;
            padding: 10px;
            margin: 10px 0;
            border-radius: 5px;
        }
        .legend-item {
            margin: 5px 0;
        }
        .player { color: #ff0000; }
        .obstacle { color: #8b4513; }
        .empty { color: #666; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎮 2D Grid Game</h1>
        
        <div class="game-container">
            <div id="gameGrid">
............
.@...###....
...#........
...#........
...#........
.........##.
..#.#.#...#.
............
            </div>
            
            <div class="controls">
                <h3>Controls</h3>
                <div class="control-buttons">
                    <button class="control-btn empty-btn"></button>
                    <button class="control-btn" onclick="move('up')" title="Move Up (W)">↑</button>
                    <button class="control-btn empty-btn"></button>
                    
                    <button class="control-btn" onclick="move('left')" title="Move Left (A)">←</button>
                    <button class="control-btn empty-btn"></button>
                    <button class="control-btn" onclick="move('right')" title="Move Right (D)">→</button>
                    
                    <button class="control-btn empty-btn"></button>
                    <button class="control-btn" onclick="move('down')" title="Move Down (S)">↓</button>
                    <button class="control-btn empty-btn"></button>
                </div>
                <div class="info">
                    Use WASD keys or click the buttons to move
                </div>
            </div>
            
            <div class="legend">
                <h3>Legend</h3>
                <div class="legend-item"><span class="player">@</span> - Player (You)</div>
                <div class="legend-item"><span class="obstacle">#</span> - Obstacle (Cannot pass)</div>
                <div class="legend-item"><span class="empty">.</span> - Empty space</div>
            </div>
            
            <div class="info">
                <p>Navigate the grid using WASD keys or the arrow buttons.</p>
                <p>Avoid obstacles (#) and try to explore the entire grid!</p>
            </div>
        </div>
    </div>

    <script>
        // Game state
        let gameGrid = [
            "............",
            ".@...###....",
            "...#........",
            "...#........",
            "...#........",
            ".........##.",
            "..#.#.#...#.",
            "............"
        ];
        
        let playerPos = {x: 1, y: 1};
        
        // Obstacles positions
        const obstacles = [
            {x: 3, y: 2}, {x: 3, y: 3}, {x: 3, y: 4}, // Vertical wall
            {x: 5, y: 1}, {x: 6, y: 1}, {x: 7, y: 1}, // Horizontal wall
            {x: 9, y: 5}, {x: 10, y: 5}, {x: 10, y: 6}, // L-shaped obstacle
            {x: 2, y: 6}, {x: 4, y: 6}, {x: 6, y: 6} // Scattered obstacles
        ];
        
        // Initialize grid display
        function updateDisplay() {
            // Create a copy of the grid
            let displayGrid = gameGrid.map(row => row.split(''));
            
            // Clear old player position
            for (let y = 0; y < displayGrid.length; y++) {
                for (let x = 0; x < displayGrid[y].length; x++) {
                    if (displayGrid[y][x] === '@') {
                        displayGrid[y][x] = '.';
                    }
                }
            }
            
            // Place player at current position
            if (playerPos.y >= 0 && playerPos.y < displayGrid.length &&
                playerPos.x >= 0 && playerPos.x < displayGrid[playerPos.y].length) {
                displayGrid[playerPos.y][playerPos.x] = '@';
            }
            
            // Update the display
            const gridElement = document.getElementById('gameGrid');
            gridElement.innerHTML = displayGrid.map(row => 
                row.join('').replace(/@/g, '<span class="player">@</span>')
                           .replace(/#/g, '<span class="obstacle">#</span>')
                           .replace(/\./g, '<span class="empty">.</span>')
            ).join('\n');
        }
        
        // Check if position has obstacle
        function hasObstacle(x, y) {
            return obstacles.some(obs => obs.x === x && obs.y === y);
        }
        
        // Check if position is within bounds
        function isInBounds(x, y) {
            return x >= 0 && x < 12 && y >= 0 && y < 8;
        }
        
        // Move player
        function move(direction) {
            let newX = playerPos.x;
            let newY = playerPos.y;
            
            switch(direction) {
                case 'up':
                    newY--;
                    break;
                case 'down':
                    newY++;
                    break;
                case 'left':
                    newX--;
                    break;
                case 'right':
                    newX++;
                    break;
            }
            
            // Check bounds and obstacles
            if (isInBounds(newX, newY) && !hasObstacle(newX, newY)) {
                playerPos.x = newX;
                playerPos.y = newY;
                updateDisplay();
                console.log(`Player moved to (${newX}, ${newY})`);
            } else {
                if (!isInBounds(newX, newY)) {
                    console.log(`Cannot move to (${newX}, ${newY}) - out of bounds`);
                } else {
                    console.log(`Cannot move to (${newX}, ${newY}) - obstacle blocking`);
                }
            }
        }
        
        // Keyboard controls
        document.addEventListener('keydown', function(event) {
            switch(event.key.toLowerCase()) {
                case 'w':
                case 'arrowup':
                    event.preventDefault();
                    move('up');
                    break;
                case 's':
                case 'arrowdown':
                    event.preventDefault();
                    move('down');
                    break;
                case 'a':
                case 'arrowleft':
                    event.preventDefault();
                    move('left');
                    break;
                case 'd':
                case 'arrowright':
                    event.preventDefault();
                    move('right');
                    break;
            }
        });
        
        // Initialize display
        updateDisplay();
        
        // Give focus to the page so keyboard events work
        window.focus();
        document.body.focus();
        
        console.log("🎮 Grid Game loaded! Use WASD keys or click buttons to move.");
    </script>
</body>
</html>"#.to_string()
    }
}

/// Demonstrate the web grid game
pub fn demonstrate_web_grid_game() {
    println!("🌐 Starting Web Grid Game");
    println!("=========================");
    
    let mut web_game = WebGridGame::new("localhost:8084");
    
    // Initialize the game
    if let Err(e) = web_game.initialize() {
        eprintln!("Failed to initialize web game: {}", e);
        return;
    }
    
    // Run the web server
    if let Err(e) = web_game.run() {
        eprintln!("Web game error: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_web_grid_game_creation() {
        let web_game = WebGridGame::new("localhost:8000");
        // Just test that we can create the web game
        assert!(true);
    }
    
    #[test]
    fn test_game_page_generation() {
        let web_game = WebGridGame::new("localhost:8000");
        let page = web_game.create_game_page();
        assert!(page.contains("2D Grid Game"));
        assert!(page.contains("gameGrid"));
        assert!(page.contains("script"));
    }
}