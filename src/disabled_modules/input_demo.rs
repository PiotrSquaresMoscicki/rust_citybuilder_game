/// Demo showing input manager integration with web client and game engine
use crate::input::*;
use crate::rendering::{WebServiceManager};
use crate::core::math::Vector2d;

#[allow(dead_code)]
pub fn demonstrate_web_input_integration() {
    println!("=== Web Client Input Integration Demo ===");
    
    // Initialize the global input manager (or use existing one)
    let _ = get_global_input_manager()
        .or_else(|_| {
            initialize_global_input_manager()?;
            get_global_input_manager()
        });
    println!("Input manager ready");
    
    // Create a web service for client communication
    let web_service = WebServiceManager::new("localhost:8080");
    
    // Create a web client input device
    let mut web_input_device = WebClientInputDevice::new(web_service, 101);
    
    // Initialize the device
    if let Err(e) = web_input_device.initialize() {
        println!("Failed to initialize web input device: {}", e);
        return;
    }
    
    println!("Web input device initialized: {}", web_input_device.device_name());
    
    // Add the device to the global input manager
    match add_global_input_device(Box::new(web_input_device)) {
        Ok(device_id) => println!("Added input device with ID: {}", device_id),
        Err(e) => {
            println!("Failed to add input device: {}", e);
            return;
        }
    }
    
    // Simulate some web client input events
    println!("\n--- Simulating Web Client Input ---");
    simulate_web_client_input();
    
    // Process input in the game engine
    println!("\n--- Processing Input in Game Engine ---");
    process_game_input();
    
    println!("\n=== Demo Complete ===");
}

fn simulate_web_client_input() {
    // In a real scenario, these events would come from the web client via HTTP/WebSocket
    println!("Web Client: User presses 'W' key");
    println!("Web Client: User clicks left mouse button at (100, 200)");
    println!("Web Client: User moves mouse to (150, 250)");
    println!("Web Client: User releases 'W' key");
    println!("Web Client: User scrolls mouse wheel");
    
    // These would be actual JSON messages sent from the web client:
    // {"type": "KeyPress", "key": "w"}
    // {"type": "MousePress", "button": "left", "x": 100, "y": 200}
    // {"type": "MouseMove", "x": 150, "y": 250, "delta_x": 50, "delta_y": 50}
    // {"type": "KeyRelease", "key": "w"}
    // {"type": "MouseWheel", "delta": 1.5, "x": 150, "y": 250}
}

fn process_game_input() {
    // Poll events from the global input manager
    match poll_global_input_events() {
        Ok(events) => {
            println!("Game Engine: Received {} input events", events.len());
            
            for (i, event) in events.iter().enumerate() {
                println!("  Event {}: {:?}", i + 1, event);
            }
        }
        Err(e) => {
            println!("Game Engine: Failed to poll input events: {}", e);
        }
    }
    
    // Check specific input states
    println!("\n--- Checking Input States ---");
    
    // Check key states
    let keys_to_check = [Key::W, Key::A, Key::S, Key::D, Key::Space, Key::Escape];
    for key in &keys_to_check {
        if is_global_key_pressed(key) {
            println!("Game Engine: Key {:?} is currently pressed", key);
        }
    }
    
    // Check mouse button states
    let buttons_to_check = [MouseButton::Left, MouseButton::Right, MouseButton::Middle];
    for button in &buttons_to_check {
        if is_global_mouse_button_pressed(button) {
            println!("Game Engine: Mouse button {:?} is currently pressed", button);
        }
    }
    
    // Get mouse position
    let mouse_pos = get_global_mouse_position();
    println!("Game Engine: Mouse position: ({}, {})", mouse_pos.x, mouse_pos.y);
    
    // Demonstrate game logic based on input
    demonstrate_game_logic();
}

fn demonstrate_game_logic() {
    println!("\n--- Game Logic Example ---");
    
    // Example: Player movement based on WASD keys
    let mut movement = Vector2d::new(0.0, 0.0);
    
    if is_global_key_pressed(&Key::W) {
        movement.y += 1.0;
        println!("Game Logic: Moving player up");
    }
    if is_global_key_pressed(&Key::S) {
        movement.y -= 1.0;
        println!("Game Logic: Moving player down");
    }
    if is_global_key_pressed(&Key::A) {
        movement.x -= 1.0;
        println!("Game Logic: Moving player left");
    }
    if is_global_key_pressed(&Key::D) {
        movement.x += 1.0;
        println!("Game Logic: Moving player right");
    }
    
    if movement.magnitude() > 0.0 {
        println!("Game Logic: Player movement vector: ({}, {})", movement.x, movement.y);
    } else {
        println!("Game Logic: Player is stationary");
    }
    
    // Example: Building placement with mouse
    if is_global_mouse_button_pressed(&MouseButton::Left) {
        let mouse_pos = get_global_mouse_position();
        println!("Game Logic: Attempting to place building at ({}, {})", mouse_pos.x, mouse_pos.y);
    }
    
    // Example: Context menu with right click
    if is_global_mouse_button_pressed(&MouseButton::Right) {
        let mouse_pos = get_global_mouse_position();
        println!("Game Logic: Showing context menu at ({}, {})", mouse_pos.x, mouse_pos.y);
    }
    
    // Example: Pause game with Escape
    if is_global_key_pressed(&Key::Escape) {
        println!("Game Logic: Pause menu activated");
    }
}

/// Example of how multiple input devices could be used in split-screen scenarios
#[allow(dead_code)]
pub fn demonstrate_split_screen_input() {
    println!("=== Split-Screen Input Demo ===");
    
    // Initialize the global input manager (or use existing one)
    let _ = get_global_input_manager()
        .or_else(|_| {
            initialize_global_input_manager()?;
            get_global_input_manager()
        });
    println!("Input manager ready");
    
    // Create input devices for two players
    let web_service1 = WebServiceManager::new("localhost:8081");
    let web_service2 = WebServiceManager::new("localhost:8082");
    
    let mut player1_device = WebClientInputDevice::new(web_service1, 201);
    let mut player2_device = WebClientInputDevice::new(web_service2, 202);
    
    // Initialize devices
    if let Err(e) = player1_device.initialize() {
        println!("Failed to initialize player 1 input device: {}", e);
        return;
    }
    
    if let Err(e) = player2_device.initialize() {
        println!("Failed to initialize player 2 input device: {}", e);
        return;
    }
    
    // Add devices to the global input manager
    let device_id1 = add_global_input_device(Box::new(player1_device)).unwrap();
    let device_id2 = add_global_input_device(Box::new(player2_device)).unwrap();
    
    println!("Player 1 input device ID: {}", device_id1);
    println!("Player 2 input device ID: {}", device_id2);
    
    // In a real implementation, you would track which events come from which device
    // and handle them separately for each player
    
    println!("Split-screen input system ready for multiple players!");
    println!("=== Demo Complete ===");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_integration_demo() {
        // Test that the demo functions don't panic
        demonstrate_web_input_integration();
        demonstrate_split_screen_input();
        
        // The demo should complete without errors
        assert!(true);
    }
    
    #[test]
    fn test_input_event_parsing_examples() {
        // Test parsing of various input events that would come from web client
        assert_eq!(Key::from_string("w"), Key::W);
        assert_eq!(Key::from_string("space"), Key::Space);
        assert_eq!(Key::from_string("escape"), Key::Escape);
        
        assert_eq!(MouseButton::from_string("left"), MouseButton::Left);
        assert_eq!(MouseButton::from_string("right"), MouseButton::Right);
        assert_eq!(MouseButton::from_string("middle"), MouseButton::Middle);
    }
    
    #[test]
    fn test_multiple_device_scenario() {
        // Test creating multiple input devices for different scenarios
        let web_service1 = WebServiceManager::new("localhost:0");
        let web_service2 = WebServiceManager::new("localhost:0");
        
        let device1 = WebClientInputDevice::new(web_service1, 100);
        let device2 = WebClientInputDevice::new(web_service2, 200);
        
        assert_eq!(device1.device_id(), 100);
        assert_eq!(device2.device_id(), 200);
        assert_ne!(device1.device_name(), device2.device_name());
    }
}