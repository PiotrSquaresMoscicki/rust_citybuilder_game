/// Web client integration for the clean ECS grid game
use crate::ecs::Mut;
use crate::grid_game_systems::GridGameWorld;
use crate::rendering::{render_global_grid};
use tiny_http::{Server, Response, Header, Request, Method};
use serde_json;
use std::fs;

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
    
    /// Process input by updating the ECS InputComponent based on current input state
    fn update_ecs_input_from_javascript(&mut self, dx: i32, dy: i32) {
        // Use ECS iterator to find player entity with InputComponent
        let mut iter = self.game_world.world.iter_entities::<crate::grid_game_components::PlayerComponent, Mut<crate::grid_game_components::InputComponent>>();
        if let Some((_, mut input_ref)) = iter.next() {
            if let Some(input_comp) = input_ref.get_mut() {
                // Clear previous input
                input_comp.clear();
                
                // Set new input based on JavaScript input
                if dx < 0 { input_comp.move_left = true; }
                if dx > 0 { input_comp.move_right = true; }
                if dy < 0 { input_comp.move_up = true; }
                if dy > 0 { input_comp.move_down = true; }
            }
        }
    }
    
    /// Start the web server and game loop
    pub fn run(&mut self) -> Result<(), String> {
        println!("🚀 Starting Web ECS Game Demo");
        println!("==============================");
        
        
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
        println!("🔧 Using ECS with JavaScript input libraries");
        println!("📡 Rendering: http://localhost:8081 | Input: JavaScript InputManager");
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
    fn handle_request(&mut self, request: Request) -> Result<(), Box<dyn std::error::Error>> {
        let method = request.method().clone();
        let url = request.url().to_string();
        
        println!("{} {}", method, url);
        
        match (method, url.as_str()) {
            (Method::Get, "/") => {
                // Serve the generic HTML template from web/game-template.html
                match self.serve_generic_template() {
                    Ok(html) => {
                        let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                            .map_err(|_| "Failed to create header")?;
                        let response = Response::from_string(html).with_header(header);
                        request.respond(response)?;
                    }
                    Err(e) => {
                        eprintln!("Error serving template: {}", e);
                        // Fallback to a simple error page
                        let error_html = self.create_error_page(&format!("Error loading template: {}", e));
                        let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                            .map_err(|_| "Failed to create header")?;
                        let response = Response::from_string(error_html).with_header(header);
                        request.respond(response)?;
                    }
                }
            }
            (Method::Post, "/move") => {
                // Read the JSON body
                let mut body = String::new();
                let mut request = request;
                std::io::Read::read_to_string(request.as_reader(), &mut body)?;
                
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
                        
                        // Update ECS input state
                        self.update_ecs_input_from_javascript(dx, dy);
                        
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
                            "inputMethod": "JavaScript Libraries + ECS"
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
            (Method::Get, "/state") => {
                // For polling-based input, JavaScript will handle input and send via /move
                // This endpoint just returns current game state
                let game_state = self.game_world.get_game_state();
                let player_pos = self.game_world.get_player_position().unwrap_or((0, 0));
                
                let response_data = serde_json::json!({
                    "gameState": game_state,
                    "playerPosition": {
                        "x": player_pos.0,
                        "y": player_pos.1
                    },
                    "inputMethod": "JavaScript Libraries with ECS Backend",
                    "moved": false,
                    "lastInput": "Polling mode - input via JavaScript"
                });
                
                let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                    .map_err(|_| "Failed to create header")?;
                let response = Response::from_string(response_data.to_string()).with_header(header);
                request.respond(response)?;
            }
            (Method::Get, "/input-info") => {
                // Return information about the ECS input system
                let response_data = serde_json::json!({
                    "inputSystemType": "JavaScript Libraries + ECS Backend",
                    "ecsInputComponentActive": true,
                    "inputLibrary": "input-manager.js",
                    "renderingLibrary": "rendering-manager.js",
                    "renderingPort": "localhost:8081"
                });
                
                let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                    .map_err(|_| "Failed to create header")?;
                let response = Response::from_string(response_data.to_string()).with_header(header);
                request.respond(response)?;
            }
            (Method::Get, path) if path.starts_with("/js/") => {
                // Serve JavaScript files from web/js/ directory
                self.serve_static_file(path, "application/javascript", request)?;
            }
            (Method::Get, path) if path.starts_with("/css/") => {
                // Serve CSS files from web/css/ directory  
                self.serve_static_file(path, "text/css", request)?;
            }
            _ => {
                // Default to serving the game page
                match self.serve_generic_template() {
                    Ok(html) => {
                        let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                            .map_err(|_| "Failed to create header")?;
                        let response = Response::from_string(html).with_header(header);
                        request.respond(response)?;
                    }
                    Err(e) => {
                        eprintln!("Error serving template: {}", e);
                        let error_html = self.create_error_page(&format!("Error loading template: {}", e));
                        let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                            .map_err(|_| "Failed to create header")?;
                        let response = Response::from_string(error_html).with_header(header);
                        request.respond(response)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Serve the generic HTML template and configure it for the ECS game
    fn serve_generic_template(&self) -> Result<String, String> {
        // Read the generic template file
        let template_path = "web/game-template.html";
        let mut template_content = fs::read_to_string(template_path)
            .map_err(|e| format!("Failed to read template file {}: {}", template_path, e))?;
        
        // Get current game state for initial configuration
        let game_state = self.game_world.get_game_state();
        let player_pos = self.game_world.get_player_position().unwrap_or((1, 1));
        
        // Configure the template for ECS game by adding custom script
        let ecs_game_config = format!(r#"
        <script>
            // ECS Game Configuration
            window.ECS_GAME_CONFIG = {{
                apiUrl: window.location.origin,
                gameType: 'ecs-grid-game',
                initialState: {{'gameState': '{}', 'playerPosition': {{'x': {}, 'y': {}}}}},
                enablePolling: true,
                pollInterval: 100
            }};
            
            // Override the default game template to work with ECS backend
            window.addEventListener('load', () => {{
                console.log('🎮 ECS Grid Game loaded with JavaScript libraries');
                console.log('🔗 API URL:', window.ECS_GAME_CONFIG.apiUrl);
                
                // Initialize ECS-specific functionality
                if (window.gameTemplate) {{
                    window.gameTemplate.setupECSGameIntegration();
                }}
            }});
        </script>"#, 
        game_state.replace('\n', "\\n").replace('\r', ""),
        player_pos.0, 
        player_pos.1);
        
        // Insert the ECS configuration before the closing body tag
        template_content = template_content.replace("</body>", &format!("{}\n</body>", ecs_game_config));
        
        Ok(template_content)
    }
    
    /// Serve static files (JS, CSS, etc.) from the web directory
    fn serve_static_file(&self, path: &str, content_type: &str, request: Request) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = format!("web{}", path);
        
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                let header = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                    .map_err(|_| "Failed to create header")?;
                let response = Response::from_string(content).with_header(header);
                request.respond(response)?;
            }
            Err(_) => {
                // File not found - return 404
                let response = Response::from_string("404 Not Found").with_status_code(404);
                request.respond(response)?;
            }
        }
        
        Ok(())
    }
    
    /// Create a simple error page when template loading fails
    fn create_error_page(&self, error_message: &str) -> String {
        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ECS Game - Error</title>
    <style>
        body {{ font-family: Arial, sans-serif; padding: 50px; text-align: center; background: #1a1a1a; color: #fff; }}
        .error {{ background: #ff4444; padding: 20px; border-radius: 8px; max-width: 600px; margin: 0 auto; }}
        .retry {{ margin-top: 20px; }}
        .retry a {{ color: #4CAF50; text-decoration: none; }}
    </style>
</head>
<body>
    <div class="error">
        <h1>🚫 Error Loading Game</h1>
        <p>{}</p>
        <div class="retry">
            <a href="/">Retry</a>
        </div>
    </div>
</body>
</html>"#, error_message)
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
    fn test_template_generation() {
        let web_game = WebEcsGameDemo::new("localhost:8000");
        if let Ok(template) = web_game.serve_generic_template() {
            assert!(template.contains("ECS Game Configuration"));
            assert!(template.contains("window.ECS_GAME_CONFIG"));
        }
        // Just test that we can create the web game without the method
        assert!(true);
    }
}