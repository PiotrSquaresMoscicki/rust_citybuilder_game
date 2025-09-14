/// Web client integration for the clean ECS grid game
use crate::grid_game_systems::GridGameWorld;
use tiny_http::{Server, Response, Header, Request, Method};
use std::io::Read;
use serde_json;

/// Web-based ECS game demo
pub struct WebEcsGameDemo {
    game_world: GridGameWorld,
    address: String,
}

impl WebEcsGameDemo {
    pub fn new(address: &str) -> Self {
        let mut game_world = GridGameWorld::new();
        game_world.initialize_game();
        
        Self {
            game_world,
            address: address.to_string(),
        }
    }
    
    /// Start the web server and game loop
    pub fn run(&mut self) -> Result<(), String> {
        println!("🚀 Starting Web ECS Game Demo");
        println!("==============================");
        
        let server = Server::http(&self.address)
            .map_err(|e| format!("Failed to start HTTP server: {}", e))?;
        
        println!("🌐 Web ECS Game server started on http://{}", &self.address);
        println!("🎯 Open http://{} in your browser to play", &self.address);
        println!("📱 Use WASD keys to move the player");
        println!("🔧 Using clean ECS implementation with system dependencies");
        println!("");
        
        // HTTP server loop
        for request in server.incoming_requests() {
            if let Err(e) = self.handle_request(request) {
                eprintln!("Error handling request: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Handle HTTP requests
    fn handle_request(&mut self, mut request: Request) -> Result<(), Box<dyn std::error::Error>> {
        let method = request.method();
        let url = request.url();
        
        println!("{} {}", method, url);
        
        match (method, url) {
            (&Method::Get, "/") => {
                let html = self.create_game_page();
                let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                    .map_err(|_| "Failed to create header")?;
                let response = Response::from_string(html).with_header(header);
                request.respond(response)?;
            }
            (&Method::Post, "/move") => {
                // Read the JSON body
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body)?;
                
                // Parse the movement command
                if let Ok(move_data) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(direction) = move_data["direction"].as_str() {
                        let (dx, dy) = match direction {
                            "up" => (0, -1),
                            "down" => (0, 1),
                            "left" => (-1, 0),
                            "right" => (1, 0),
                            _ => (0, 0),
                        };
                        
                        let moved = self.game_world.move_player(dx, dy);
                        
                        // Update the game systems after movement
                        let _ = self.game_world.update();
                        
                        // Send back the game state
                        let game_state = self.game_world.get_game_state();
                        let player_pos = self.game_world.get_player_position().unwrap_or((0, 0));
                        
                        let response_data = serde_json::json!({
                            "success": moved,
                            "gameState": game_state,
                            "playerPosition": {
                                "x": player_pos.0,
                                "y": player_pos.1
                            }
                        });
                        
                        let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                            .map_err(|_| "Failed to create header")?;
                        let response = Response::from_string(response_data.to_string()).with_header(header);
                        request.respond(response)?;
                    }
                } else {
                    let error_response = serde_json::json!({"error": "Invalid request"});
                    let response = Response::from_string(error_response.to_string());
                    request.respond(response)?;
                }
            }
            (&Method::Get, "/state") => {
                // Get current game state
                let game_state = self.game_world.get_game_state();
                let player_pos = self.game_world.get_player_position().unwrap_or((0, 0));
                
                let response_data = serde_json::json!({
                    "gameState": game_state,
                    "playerPosition": {
                        "x": player_pos.0,
                        "y": player_pos.1
                    }
                });
                
                let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                    .map_err(|_| "Failed to create header")?;
                let response = Response::from_string(response_data.to_string()).with_header(header);
                request.respond(response)?;
            }
            _ => {
                // Default to serving the game page
                let html = self.create_game_page();
                let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                    .map_err(|_| "Failed to create header")?;
                let response = Response::from_string(html).with_header(header);
                request.respond(response)?;
            }
        }
        
        Ok(())
    }
    
    /// Create the HTML page for the ECS grid game
    fn create_game_page(&self) -> String {
        let initial_state = self.game_world.get_game_state();
        let player_pos = self.game_world.get_player_position().unwrap_or((1, 1));
        
        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ECS Grid Game Demo - Clean Architecture</title>
    <style>
        body {{
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #0c1445 0%, #1a1a2e 50%, #16213e 100%);
            color: #00ff41;
            min-height: 100vh;
        }}
        .container {{
            max-width: 900px;
            margin: 0 auto;
            background: #000;
            padding: 30px;
            border-radius: 12px;
            border: 2px solid #00ff41;
            box-shadow: 0 0 30px rgba(0, 255, 65, 0.3);
        }}
        h1 {{
            color: #00ff41;
            text-align: center;
            text-shadow: 0 0 15px #00ff41;
            margin-bottom: 10px;
        }}
        .subtitle {{
            text-align: center;
            color: #888;
            margin-bottom: 30px;
            font-size: 14px;
        }}
        .game-container {{
            display: flex;
            flex-direction: column;
            align-items: center;
        }}
        #gameGrid {{
            background: #111;
            border: 3px solid #00ff41;
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            font-size: 24px;
            line-height: 1.2;
            padding: 20px;
            margin: 20px 0;
            white-space: pre;
            text-align: left;
            box-shadow: inset 0 0 20px rgba(0, 255, 65, 0.2);
        }}
        .controls {{
            background: #111;
            border: 2px solid #00ff41;
            padding: 20px;
            margin: 15px 0;
            border-radius: 8px;
            box-shadow: 0 0 15px rgba(0, 255, 65, 0.2);
        }}
        .control-buttons {{
            display: grid;
            grid-template-columns: repeat(3, 70px);
            grid-template-rows: repeat(3, 70px);
            gap: 8px;
            justify-content: center;
            margin: 15px 0;
        }}
        .control-btn {{
            background: #222;
            border: 2px solid #00ff41;
            color: #00ff41;
            font-size: 20px;
            font-weight: bold;
            cursor: pointer;
            border-radius: 6px;
            transition: all 0.3s;
            font-family: 'Monaco', monospace;
        }}
        .control-btn:hover {{
            background: #00ff41;
            color: #000;
            box-shadow: 0 0 15px #00ff41;
            transform: translateY(-2px);
        }}
        .control-btn:active {{
            transform: scale(0.95);
        }}
        .empty-btn {{
            background: transparent;
            border: none;
            cursor: default;
        }}
        .empty-btn:hover {{
            background: transparent;
            color: transparent;
            box-shadow: none;
            transform: none;
        }}
        .info {{
            text-align: center;
            margin: 15px 0;
            font-size: 14px;
            color: #888;
        }}
        .legend {{
            background: #111;
            border: 2px solid #00ff41;
            padding: 15px;
            margin: 15px 0;
            border-radius: 8px;
            box-shadow: 0 0 15px rgba(0, 255, 65, 0.2);
        }}
        .legend-item {{
            margin: 8px 0;
            font-size: 16px;
        }}
        .player {{ color: #ff0040; text-shadow: 0 0 10px #ff0040; }}
        .obstacle {{ color: #8b4513; text-shadow: 0 0 5px #8b4513; }}
        .empty {{ color: #333; }}
        
        .status {{
            background: #111;
            border: 2px solid #00ff41;
            padding: 15px;
            margin: 15px 0;
            border-radius: 8px;
            box-shadow: 0 0 15px rgba(0, 255, 65, 0.2);
        }}
        
        .ecs-info {{
            background: #0a0a0a;
            border: 1px solid #666;
            padding: 10px;
            margin: 10px 0;
            border-radius: 5px;
            font-size: 12px;
            color: #999;
        }}
        
        .success {{ color: #00ff41; }}
        .error {{ color: #ff4040; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🎮 ECS Grid Game Demo</h1>
        <div class="subtitle">Clean Entity Component System Implementation</div>
        
        <div class="game-container">
            <div id="gameGrid">{}</div>
            
            <div class="controls">
                <h3>Movement Controls</h3>
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
            
            <div class="status">
                <h3>Game Status</h3>
                <div>Player Position: <span id="playerPos">({}, {})</span></div>
                <div>Last Action: <span id="lastAction">Game Started</span></div>
            </div>
            
            <div class="legend">
                <h3>Legend</h3>
                <div class="legend-item"><span class="player">@</span> - Player (Hero)</div>
                <div class="legend-item"><span class="obstacle">#</span> - Obstacle (Blocks movement)</div>
                <div class="legend-item"><span class="empty">.</span> - Empty space</div>
            </div>
            
            <div class="ecs-info">
                <h3>🔧 ECS Architecture</h3>
                <div><strong>Systems:</strong> GridInputSystem → GridMovementSystem → GridCollisionSystem → GridRenderSystem</div>
                <div><strong>Components:</strong> GridPositionComponent, PlayerComponent, ObstacleComponent, RenderComponent</div>
                <div><strong>Dependencies:</strong> Systems execute in order based on declared dependencies</div>
                <div><strong>Iterators:</strong> EntIt&lt;Component1, Component2&gt; provides clean component access</div>
            </div>
        </div>
    </div>

    <script>
        let playerPos = {{ x: {}, y: {} }};
        
        // Update the display with current game state
        function updateDisplay(gameState) {{
            const gridElement = document.getElementById('gameGrid');
            gridElement.innerHTML = gameState.replace(/@/g, '<span class="player">@</span>')
                                              .replace(/#/g, '<span class="obstacle">#</span>')
                                              .replace(/\./g, '<span class="empty">.</span>');
        }}
        
        // Update player position display
        function updatePlayerPosition(x, y) {{
            playerPos.x = x;
            playerPos.y = y;
            document.getElementById('playerPos').textContent = `(${{x}}, ${{y}})`;
        }}
        
        // Update last action display
        function updateLastAction(message, isSuccess = true) {{
            const element = document.getElementById('lastAction');
            element.textContent = message;
            element.className = isSuccess ? 'success' : 'error';
        }}
        
        // Move player
        async function move(direction) {{
            try {{
                const response = await fetch('/move', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json',
                    }},
                    body: JSON.stringify({{ direction: direction }})
                }});
                
                const data = await response.json();
                
                if (data.success) {{
                    updateDisplay(data.gameState);
                    updatePlayerPosition(data.playerPosition.x, data.playerPosition.y);
                    updateLastAction(`Moved ${{direction}} to (${{data.playerPosition.x}}, ${{data.playerPosition.y}})`, true);
                    console.log(`Player moved ${{direction}} to (${{data.playerPosition.x}}, ${{data.playerPosition.y}})`);
                }} else {{
                    updateLastAction(`Cannot move ${{direction}} - blocked or out of bounds`, false);
                    console.log(`Movement ${{direction}} blocked`);
                }}
            }} catch (error) {{
                console.error('Error moving player:', error);
                updateLastAction('Error communicating with server', false);
            }}
        }}
        
        // Keyboard controls
        document.addEventListener('keydown', function(event) {{
            switch(event.key.toLowerCase()) {{
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
            }}
        }});
        
        // Initialize display
        updateDisplay(`{}`);
        updatePlayerPosition({}, {});
        
        // Give focus to the page so keyboard events work
        window.focus();
        document.body.focus();
        
        console.log("🎮 ECS Grid Game loaded!");
        console.log("🔧 Using clean System trait with Dependencies and Iterators");
        console.log("📡 Web client sends input to server, server runs ECS systems");
    </script>
</body>
</html>"#, 
        initial_state.replace("@", "<span class=\"player\">@</span>")
                    .replace("#", "<span class=\"obstacle\">#</span>")
                    .replace(".", "<span class=\"empty\">.</span>"),
        player_pos.0, player_pos.1,
        player_pos.0, player_pos.1,
        initial_state,
        player_pos.0, player_pos.1)
    }
}

/// Demonstrate the web ECS game
pub fn demonstrate_web_ecs_game() {
    println!("🚀 Starting Web ECS Game Demo");
    println!("=============================");
    
    let mut web_game = WebEcsGameDemo::new("localhost:8085");
    
    if let Err(e) = web_game.run() {
        eprintln!("Web ECS game error: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_web_ecs_game_creation() {
        let _web_game = WebEcsGameDemo::new("localhost:8000");
        // Just test that we can create the web game
        assert!(true);
    }
    
    #[test]
    fn test_game_page_generation() {
        let web_game = WebEcsGameDemo::new("localhost:8000");
        let page = web_game.create_game_page();
        assert!(page.contains("ECS Grid Game Demo"));
        assert!(page.contains("gameGrid"));
        assert!(page.contains("System trait"));
        assert!(page.contains("Dependencies"));
    }
}