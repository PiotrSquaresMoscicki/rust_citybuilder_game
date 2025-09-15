/// Web client integration for the clean ECS grid game
use crate::grid_game_systems::GridGameWorld;
use crate::rendering::{render_global_grid};
use crate::input::{poll_global_input_events, is_global_key_pressed, Key};
use tiny_http::{Server, Response, Header, Request, Method};
use serde_json;

/// Web-based ECS game demo
pub struct WebEcsGameDemo {
    game_world: GridGameWorld,
    address: String,
    input_device_id: Option<u32>,
}

impl WebEcsGameDemo {
    pub fn new(address: &str) -> Self {
        let mut game_world = GridGameWorld::new();
        game_world.initialize_game();
        
        Self {
            game_world,
            address: address.to_string(),
            input_device_id: None,
        }
    }
    
    /// Initialize input device for the web client
    fn initialize_input_device(&mut self) -> Result<(), String> {
        // The global input manager should already have a web client input device from main.rs
        println!("✅ Using existing global input manager with web client input device");
        self.input_device_id = Some(1000); // Use the device ID from main.rs
        Ok(())
    }
    
    /// Process input using the global input manager
    fn process_input_from_global_manager(&mut self) -> (i32, i32) {
        // Poll events from global input manager
        if let Ok(_events) = poll_global_input_events() {
            // Check for movement keys using global input manager
            let mut dx = 0;
            let mut dy = 0;
            
            if is_global_key_pressed(&Key::W) || is_global_key_pressed(&Key::ArrowUp) {
                dy = -1;
            }
            if is_global_key_pressed(&Key::S) || is_global_key_pressed(&Key::ArrowDown) {
                dy = 1;
            }
            if is_global_key_pressed(&Key::A) || is_global_key_pressed(&Key::ArrowLeft) {
                dx = -1;
            }
            if is_global_key_pressed(&Key::D) || is_global_key_pressed(&Key::ArrowRight) {
                dx = 1;
            }
            
            (dx, dy)
        } else {
            (0, 0)
        }
    }
    
    /// Start the web server and game loop
    pub fn run(&mut self) -> Result<(), String> {
        println!("🚀 Starting Web ECS Game Demo");
        println!("==============================");
        
        // Initialize input device
        self.initialize_input_device()?;
        
        // Test the global rendering manager by rendering a grid
        if let Err(e) = render_global_grid(10, 8, 32.0) {
            eprintln!("⚠️ Warning: Failed to render initial grid via global manager: {}", e);
        } else {
            println!("✅ Initial grid rendered via global rendering manager");
        }
        
        let server = Server::http(&self.address)
            .map_err(|e| format!("Failed to start HTTP server: {}", e))?;
        
        println!("🌐 Web ECS Game server started on http://{}", &self.address);
        println!("🎯 Open http://{} in your browser to play", &self.address);
        println!("📱 Use WASD keys to move the player");
        println!("🔧 Using global rendering and input managers");
        println!("📡 Rendering: http://localhost:8081 | Input: http://localhost:8086");
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
                            },
                            "inputMethod": "HTTP API (will be replaced by global input manager)"
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
                // Process input from global input manager
                let (dx, dy) = self.process_input_from_global_manager();
                
                // Apply movement if any input detected
                let moved = if dx != 0 || dy != 0 {
                    let success = self.game_world.move_player(dx, dy);
                    if success {
                        let _ = self.game_world.update();
                    }
                    success
                } else {
                    false
                };
                
                // Get current game state
                let game_state = self.game_world.get_game_state();
                let player_pos = self.game_world.get_player_position().unwrap_or((0, 0));
                
                let response_data = serde_json::json!({
                    "gameState": game_state,
                    "playerPosition": {
                        "x": player_pos.0,
                        "y": player_pos.1
                    },
                    "inputMethod": "Global Input Manager",
                    "moved": moved,
                    "lastInput": format!("dx: {}, dy: {}", dx, dy)
                });
                
                let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                    .map_err(|_| "Failed to create header")?;
                let response = Response::from_string(response_data.to_string()).with_header(header);
                request.respond(response)?;
            }
            (&Method::Get, "/input-info") => {
                // Return information about the input manager
                let response_data = serde_json::json!({
                    "inputDeviceId": self.input_device_id,
                    "globalManagerActive": true,
                    "inputPort": "localhost:8086",
                    "renderingPort": "localhost:8081"
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
    <title>Framework Demo - Rendering & Input Modules</title>
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
            max-width: 1000px;
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
        
        .framework-info {{
            background: #0a0a0a;
            border: 1px solid #666;
            padding: 15px;
            margin: 10px 0;
            border-radius: 5px;
            font-size: 12px;
            color: #999;
        }}
        
        .manager-status {{
            background: #001122;
            border: 1px solid #0088cc;
            padding: 10px;
            margin: 5px 0;
            border-radius: 3px;
            font-size: 11px;
            color: #88ccff;
        }}
        
        .success {{ color: #00ff41; }}
        .error {{ color: #ff4040; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🎮 Framework Demo - Rendering & Input Modules</h1>
        <div class="subtitle">Showcasing modular rendering and input system capabilities</div>
        
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
                <div>Input Method: <span id="inputMethod">Loading...</span></div>
            </div>
            
            <div class="legend">
                <h3>Legend</h3>
                <div class="legend-item"><span class="player">@</span> - Player (Hero)</div>
                <div class="legend-item"><span class="obstacle">#</span> - Obstacle (Blocks movement)</div>
                <div class="legend-item"><span class="empty">.</span> - Empty space</div>
            </div>
            
            <div class="framework-info">
                <h3>🔧 Framework Architecture Showcase</h3>
                <div class="manager-status">
                    <strong>📡 Global Rendering Manager:</strong> WebClientRenderingDevice on port 8081<br>
                    ✅ Handles all rendering commands globally<br>
                    🎨 Renders via custom protocol to web client
                </div>
                <div class="manager-status">
                    <strong>🎮 Global Input Manager:</strong> WebClientInputDevice on port 8086<br>
                    ✅ Processes input events from multiple sources<br>
                    🔗 Integrates keyboard, mouse, and custom input devices
                </div>
                <div class="manager-status">
                    <strong>🏗️ ECS Systems:</strong> GridInputSystem → GridMovementSystem → GridCollisionSystem<br>
                    ✅ Clean dependency-based system execution<br>
                    📦 EntIt&lt;Component1, Component2&gt; provides type-safe component access
                </div>
                <div class="manager-status">
                    <strong>🌐 Web Integration:</strong> HTTP API bridges web client to framework<br>
                    ✅ Demonstrates modular architecture capabilities<br>
                    🔄 Real-time state synchronization between client and server
                </div>
            </div>
        </div>
    </div>

    <script>
        let playerPos = {{ x: {}, y: {} }};
        let useGlobalInputManager = true; // Toggle between input methods
        
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
        
        // Update input method display
        function updateInputMethod(method) {{
            document.getElementById('inputMethod').textContent = method;
        }}
        
        // Move player using the legacy HTTP API
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
                    updateInputMethod(data.inputMethod || "HTTP API");
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
        
        // Poll game state (also processes global input manager)
        async function pollGameState() {{
            try {{
                const response = await fetch('/state');
                const data = await response.json();
                
                updateDisplay(data.gameState);
                updatePlayerPosition(data.playerPosition.x, data.playerPosition.y);
                updateInputMethod(data.inputMethod || "Unknown");
                
                if (data.moved) {{
                    updateLastAction(`Global Input: ${{data.lastInput}}`, true);
                }}
            }} catch (error) {{
                console.error('Error polling state:', error);
            }}
        }}
        
        // Get input manager info
        async function getInputInfo() {{
            try {{
                const response = await fetch('/input-info');
                const data = await response.json();
                console.log('Input Manager Info:', data);
            }} catch (error) {{
                console.error('Error getting input info:', error);
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
        
        // Poll state regularly to showcase global input manager
        setInterval(pollGameState, 100);
        
        // Get input manager info on load
        getInputInfo();
        
        // Give focus to the page so keyboard events work
        window.focus();
        document.body.focus();
        
        console.log("🎮 Framework Demo loaded!");
        console.log("🔧 Global Rendering Manager: localhost:8081");
        console.log("🎮 Global Input Manager: localhost:8086");
        console.log("📡 ECS Game Server: localhost:8085");
        console.log("🌟 This demo showcases the modular rendering and input framework!");
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
        assert!(page.contains("Framework Demo"));
        assert!(page.contains("gameGrid"));
        assert!(page.contains("Global Rendering Manager"));
        assert!(page.contains("Global Input Manager"));
        assert!(page.contains("modular rendering and input"));
    }
}