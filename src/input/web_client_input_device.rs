use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use super::{InputDevice, InputEvent, Key, MouseButton};
use crate::core::math::Vector2d;
use crate::rendering::web_service_manager::WebServiceManager;
use serde::{Serialize, Deserialize};

/// Message types for input communication with web client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputMessage {
    /// Key press event from web client
    KeyPress { key: String },
    /// Key release event from web client  
    KeyRelease { key: String },
    /// Mouse button press event from web client
    MousePress { button: String, x: f32, y: f32 },
    /// Mouse button release event from web client
    MouseRelease { button: String, x: f32, y: f32 },
    /// Mouse movement event from web client
    MouseMove { x: f32, y: f32, delta_x: f32, delta_y: f32 },
    /// Mouse wheel scroll event from web client
    MouseWheel { delta: f32, x: f32, y: f32 },
}

/// Web client input device that receives input from a web client
/// via the WebServiceManager, similar to how WebClientRenderingDevice works
pub struct WebClientInputDevice {
    web_service: Arc<Mutex<WebServiceManager>>,
    device_name: String,
    device_id: u32,
    is_initialized: bool,
    
    // Input state tracking
    key_states: HashMap<Key, bool>,
    mouse_button_states: HashMap<MouseButton, bool>,
    mouse_position: Vector2d,
    
    // Event buffer for polling
    event_buffer: Vec<InputEvent>,
}

impl WebClientInputDevice {
    /// Create a new web client input device
    pub fn new(web_service_manager: WebServiceManager, device_id: u32) -> Self {
        Self {
            web_service: Arc::new(Mutex::new(web_service_manager)),
            device_name: format!("WebClientInputDevice_{}", device_id),
            device_id,
            is_initialized: false,
            key_states: HashMap::new(),
            mouse_button_states: HashMap::new(),
            mouse_position: Vector2d::new(0.0, 0.0),
            event_buffer: Vec::new(),
        }
    }
    
    /// Create a new web client input device with shared web service
    pub fn new_shared(web_service: Arc<Mutex<WebServiceManager>>, device_id: u32) -> Self {
        Self {
            web_service,
            device_name: format!("WebClientInputDevice_{}", device_id),
            device_id,
            is_initialized: false,
            key_states: HashMap::new(),
            mouse_button_states: HashMap::new(),
            mouse_position: Vector2d::new(0.0, 0.0),
            event_buffer: Vec::new(),
        }
    }
    
    /// Get the web service manager for external access
    pub fn get_web_service(&self) -> Arc<Mutex<WebServiceManager>> {
        self.web_service.clone()
    }
    
    /// Check if there are connected clients
    pub fn has_connected_clients(&self) -> bool {
        if let Ok(service) = self.web_service.lock() {
            service.client_count() > 0
        } else {
            false
        }
    }
    
    /// Get the number of connected clients
    pub fn client_count(&self) -> usize {
        if let Ok(service) = self.web_service.lock() {
            service.client_count()
        } else {
            0
        }
    }
    
    /// Process incoming messages from web clients
    fn process_web_messages(&mut self) -> Result<(), Box<dyn Error>> {
        // Collect messages first to avoid borrowing conflicts
        let messages = {
            let service = self.web_service.lock()
                .map_err(|e| format!("Failed to lock web service: {}", e))?;
            
            let mut collected_messages = Vec::new();
            
            // Process any messages in the queue
            while let Some(client_message) = service.receive_client_message() {
                // Parse the message as input if it's formatted correctly
                if let Ok(input_message) = serde_json::from_str::<InputMessage>(&format!("{:?}", client_message)) {
                    collected_messages.push(input_message);
                }
            }
            
            collected_messages
        };
        
        // Process messages without holding the service lock
        for input_message in messages {
            self.process_input_message(input_message)?;
        }
        
        Ok(())
    }
    
    /// Process an individual input message and update state
    fn process_input_message(&mut self, message: InputMessage) -> Result<(), Box<dyn Error>> {
        let event = match message {
            InputMessage::KeyPress { key } => {
                let key_enum = Key::from_string(&key);
                self.key_states.insert(key_enum.clone(), true);
                InputEvent::KeyPress { key: key_enum }
            }
            InputMessage::KeyRelease { key } => {
                let key_enum = Key::from_string(&key);
                self.key_states.insert(key_enum.clone(), false);
                InputEvent::KeyRelease { key: key_enum }
            }
            InputMessage::MousePress { button, x, y } => {
                let button_enum = MouseButton::from_string(&button);
                let position = Vector2d::new(x, y);
                self.mouse_button_states.insert(button_enum.clone(), true);
                self.mouse_position = position;
                InputEvent::MousePress { button: button_enum, position }
            }
            InputMessage::MouseRelease { button, x, y } => {
                let button_enum = MouseButton::from_string(&button);
                let position = Vector2d::new(x, y);
                self.mouse_button_states.insert(button_enum.clone(), false);
                self.mouse_position = position;
                InputEvent::MouseRelease { button: button_enum, position }
            }
            InputMessage::MouseMove { x, y, delta_x, delta_y } => {
                let position = Vector2d::new(x, y);
                let delta = Vector2d::new(delta_x, delta_y);
                self.mouse_position = position;
                InputEvent::MouseMove { position, delta }
            }
            InputMessage::MouseWheel { delta, x, y } => {
                let position = Vector2d::new(x, y);
                self.mouse_position = position;
                InputEvent::MouseWheel { delta, position }
            }
        };
        
        self.event_buffer.push(event);
        Ok(())
    }
    
    /// Simulate input events for testing
    pub fn simulate_key_press(&mut self, key: Key) {
        self.key_states.insert(key.clone(), true);
        self.event_buffer.push(InputEvent::KeyPress { key });
    }
    
    /// Simulate key release for testing
    pub fn simulate_key_release(&mut self, key: Key) {
        self.key_states.insert(key.clone(), false);
        self.event_buffer.push(InputEvent::KeyRelease { key });
    }
    
    /// Simulate mouse press for testing
    pub fn simulate_mouse_press(&mut self, button: MouseButton, position: Vector2d) {
        self.mouse_button_states.insert(button.clone(), true);
        self.mouse_position = position;
        self.event_buffer.push(InputEvent::MousePress { button, position });
    }
    
    /// Simulate mouse release for testing
    pub fn simulate_mouse_release(&mut self, button: MouseButton, position: Vector2d) {
        self.mouse_button_states.insert(button.clone(), false);
        self.mouse_position = position;
        self.event_buffer.push(InputEvent::MouseRelease { button, position });
    }
    
    /// Simulate mouse movement for testing
    pub fn simulate_mouse_move(&mut self, position: Vector2d, delta: Vector2d) {
        self.mouse_position = position;
        self.event_buffer.push(InputEvent::MouseMove { position, delta });
    }
}

impl InputDevice for WebClientInputDevice {
    fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        if self.is_initialized {
            return Ok(());
        }
        
        let mut service = self.web_service.lock()
            .map_err(|e| format!("Failed to lock web service: {}", e))?;
        
        // Start the web service if it's not already running
        if !service.is_running() {
            service.start()?;
        }
        
        self.is_initialized = true;
        
        println!("WebClientInputDevice {} initialized successfully", self.device_id);
        Ok(())
    }
    
    fn poll_events(&mut self) -> Result<Vec<InputEvent>, Box<dyn Error>> {
        if !self.is_initialized {
            return Ok(Vec::new());
        }
        
        // Process any incoming web messages
        self.process_web_messages()?;
        
        // Return and clear the event buffer
        let events = self.event_buffer.clone();
        self.event_buffer.clear();
        
        Ok(events)
    }
    
    fn is_key_pressed(&self, key: &Key) -> bool {
        self.key_states.get(key).copied().unwrap_or(false)
    }
    
    fn is_mouse_button_pressed(&self, button: &MouseButton) -> bool {
        self.mouse_button_states.get(button).copied().unwrap_or(false)
    }
    
    fn get_mouse_position(&self) -> Vector2d {
        self.mouse_position
    }
    
    fn is_ready(&self) -> bool {
        if !self.is_initialized {
            return false;
        }
        
        if let Ok(service) = self.web_service.lock() {
            service.is_running() && service.client_count() > 0
        } else {
            false
        }
    }
    
    fn device_name(&self) -> &str {
        &self.device_name
    }
    
    fn device_id(&self) -> u32 {
        self.device_id
    }
    
    fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.is_initialized {
            return Ok(());
        }
        
        // Clear all state
        self.key_states.clear();
        self.mouse_button_states.clear();
        self.event_buffer.clear();
        self.mouse_position = Vector2d::new(0.0, 0.0);
        self.is_initialized = false;
        
        println!("WebClientInputDevice {} shut down successfully", self.device_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::WebServiceManager;
    
    #[test]
    fn test_web_client_input_device_creation() {
        let web_service = WebServiceManager::new("localhost:0");
        let device = WebClientInputDevice::new(web_service, 1);
        
        assert_eq!(device.device_name(), "WebClientInputDevice_1");
        assert_eq!(device.device_id(), 1);
        assert!(!device.is_ready());
        assert_eq!(device.client_count(), 0);
    }
    
    #[test]
    fn test_device_initialization() {
        let web_service = WebServiceManager::new("localhost:0");
        let mut device = WebClientInputDevice::new(web_service, 2);
        
        // Device should not be ready before initialization
        assert!(!device.is_ready());
        
        // Initialize should succeed
        assert!(device.initialize().is_ok());
        
        // Should be able to shutdown after initialization
        assert!(device.shutdown().is_ok());
    }
    
    #[test]
    fn test_input_event_simulation() {
        let web_service = WebServiceManager::new("localhost:0");
        let mut device = WebClientInputDevice::new(web_service, 3);
        
        assert!(device.initialize().is_ok());
        
        // Simulate key press
        device.simulate_key_press(Key::A);
        assert!(device.is_key_pressed(&Key::A));
        assert!(!device.is_key_pressed(&Key::B));
        
        // Simulate mouse press
        let mouse_pos = Vector2d::new(100.0, 200.0);
        device.simulate_mouse_press(MouseButton::Left, mouse_pos);
        assert!(device.is_mouse_button_pressed(&MouseButton::Left));
        assert!(!device.is_mouse_button_pressed(&MouseButton::Right));
        assert_eq!(device.get_mouse_position(), mouse_pos);
        
        // Poll events should return the simulated events
        let events = device.poll_events().unwrap();
        assert_eq!(events.len(), 2);
        
        // Events should be cleared after polling
        let events2 = device.poll_events().unwrap();
        assert_eq!(events2.len(), 0);
    }
    
    #[test]
    fn test_key_state_management() {
        let web_service = WebServiceManager::new("localhost:0");
        let mut device = WebClientInputDevice::new(web_service, 4);
        
        assert!(device.initialize().is_ok());
        
        // Initially no keys pressed
        assert!(!device.is_key_pressed(&Key::Space));
        
        // Press key
        device.simulate_key_press(Key::Space);
        assert!(device.is_key_pressed(&Key::Space));
        
        // Release key
        device.simulate_key_release(Key::Space);
        assert!(!device.is_key_pressed(&Key::Space));
    }
    
    #[test]
    fn test_mouse_state_management() {
        let web_service = WebServiceManager::new("localhost:0");
        let mut device = WebClientInputDevice::new(web_service, 5);
        
        assert!(device.initialize().is_ok());
        
        // Initially no buttons pressed
        assert!(!device.is_mouse_button_pressed(&MouseButton::Left));
        
        let pos1 = Vector2d::new(50.0, 75.0);
        let pos2 = Vector2d::new(150.0, 175.0);
        
        // Press button
        device.simulate_mouse_press(MouseButton::Left, pos1);
        assert!(device.is_mouse_button_pressed(&MouseButton::Left));
        assert_eq!(device.get_mouse_position(), pos1);
        
        // Move mouse
        device.simulate_mouse_move(pos2, Vector2d::new(100.0, 100.0));
        assert_eq!(device.get_mouse_position(), pos2);
        
        // Release button
        device.simulate_mouse_release(MouseButton::Left, pos2);
        assert!(!device.is_mouse_button_pressed(&MouseButton::Left));
    }
}